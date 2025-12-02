use crate::pdf_parser::{BBox, TextLine};
use serde::{Deserialize, Serialize};

/// Represents the semantic role of a text block in a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockRole {
    /// Main body text
    Paragraph,
    /// Section or document heading with level (1 = main title, 2 = section, 3 = subsection)
    Heading { level: u8 },
    /// Page number (typically in header/footer zone)
    PageNumber,
    /// Header content (top zone, not page number)
    Header,
    /// Footer content (bottom zone, not page number)
    Footer,
    /// Footnote text (bottom zone with marker)
    Footnote,
    /// Figure/table caption
    Caption,
    /// Citation or reference entry
    Citation,
    /// Author/affiliation text
    Author,
    /// Abstract section
    Abstract,
    /// Reference section header or entries
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub text: String,
    pub bbox: BBox,
    pub role: BlockRole,
    /// Physical PDF page number (1-indexed)
    pub page_num: u32,
    /// Logical document page number extracted from content (e.g., "214" from page header)
    pub doc_page_num: Option<String>,
    /// Whether this block starts a new paragraph (detected by indentation)
    pub starts_new_paragraph: bool,
}

/// Indentation threshold in points - lines indented more than this start new paragraphs
const INDENT_THRESHOLD: f32 = 15.0;

/// Font size thresholds for heading levels
const HEADING_L1_SIZE: f32 = 18.0;  // Main title
const HEADING_L2_SIZE: f32 = 14.0;  // Section heading

/// Extract the numeric page number from a page number string
fn extract_page_number(text: &str) -> Option<String> {
    patterns::PAGE_NUMBER_EXTRACT
        .captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

pub struct ReflowEngine {
    /// Track if we're in the References section
    in_references_section: bool,
    /// Track the left margin of the current page for indentation detection
    page_left_margin: f32,
    /// Detected logical page numbers by PDF page
    page_numbers: std::collections::HashMap<u32, String>,
}

/// Patterns for role detection
mod patterns {
    use regex::Regex;
    use std::sync::LazyLock;

    /// Matches page numbers: "1", "- 1 -", "Page 1", "1 of 10", etc.
    pub static PAGE_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[\s\-]*(?:Page\s*)?\d+(?:\s*(?:of|/)\s*\d+)?[\s\-]*$").unwrap()
    });
    
    /// Extracts just the number from a page number string
    pub static PAGE_NUMBER_EXTRACT: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(\d+)").unwrap()
    });

    /// Matches footnote markers: "1", "†", "*", "[1]", etc.
    pub static FOOTNOTE_MARKER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[\s]*[\*†‡§\d\[\]]+[\.\)\s]").unwrap()
    });

    /// Matches figure/table captions: "Figure 1:", "Table 2.", "Fig. 3:", etc.
    pub static CAPTION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^(?:Fig(?:ure)?|Table|Scheme|Chart)\s*\.?\s*\d").unwrap()
    });

    /// Matches citation entries: "[1] Author...", "1. Author...", etc.
    pub static CITATION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^\s*(?:\[\d+\]|\d+\.)\s+[A-Z]").unwrap()
    });

    /// Matches References/Bibliography section headers
    pub static REFERENCES_HEADER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^\s*(?:References|Bibliography|Works\s+Cited|Literature\s+Cited)\s*$").unwrap()
    });

    /// Matches Abstract section header
    pub static ABSTRACT_HEADER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"(?i)^\s*Abstract\s*$").unwrap()
    });
}

impl ReflowEngine {
    pub fn new() -> Self {
        Self {
            in_references_section: false,
            page_left_margin: f32::MAX,
            page_numbers: std::collections::HashMap::new(),
        }
    }

    pub fn process(lines: Vec<TextLine>) -> Vec<Block> {
        let mut engine = Self::new();
        
        // First pass: detect page numbers and left margins per page
        engine.analyze_pages(&lines);
        
        // Second pass: process lines into blocks
        engine.process_lines(lines)
    }

    /// First pass: analyze pages for margins and page numbers
    fn analyze_pages(&mut self, lines: &[TextLine]) {
        let mut page_margins: std::collections::HashMap<u32, f32> = std::collections::HashMap::new();
        
        for line in lines {
            let page = line.page_num;
            let x = line.bbox.x1;
            let text = line.text.trim();
            let geom = &line.page_geometry;
            let y = line.bbox.y1;
            
            // Track minimum X for body content (not headers/footers)
            if !geom.is_header_zone(y) && !geom.is_footer_zone(y) {
                let margin = page_margins.entry(page).or_insert(f32::MAX);
                if x < *margin && x > 0.0 {
                    *margin = x;
                }
            }
            
            // Detect page numbers from header/footer zones
            if (geom.is_header_zone(y) || geom.is_footer_zone(y)) 
                && patterns::PAGE_NUMBER.is_match(text) 
            {
                // Extract just the number
                if let Some(num) = extract_page_number(text) {
                    self.page_numbers.insert(page, num);
                }
            }
        }
        
        // Store margins (will be used per-page)
        // For now, use the most common margin as default
        if let Some(min_margin) = page_margins.values().copied().reduce(f32::min) {
            self.page_left_margin = min_margin;
        }
    }

    fn process_lines(&mut self, lines: Vec<TextLine>) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();

        for line in lines {
            // Classify role first (may update internal state for References section)
            let role = self.classify_role(&line);
            
            // Get logical document page number for this PDF page
            let doc_page_num = self.page_numbers.get(&line.page_num).cloned();

            // Don't merge certain roles (page numbers, headers, footers)
            let can_merge = matches!(&role, 
                BlockRole::Paragraph | BlockRole::Heading { .. } | BlockRole::Citation | 
                BlockRole::Footnote | BlockRole::Abstract
            );

            // Try to merge with previous block
            if can_merge {
                if let Some(last_block) = blocks.last_mut() {
                    // Check if same line (Y overlap) - indentation doesn't matter for same-line fragments
                    let y_overlap = last_block.bbox.y1.max(line.bbox.y1) < last_block.bbox.y2.min(line.bbox.y2);
                    
                    // Only check indentation for new lines (not same-line fragments)
                    let block_starts_new_paragraph = if !y_overlap && last_block.page_num == line.page_num {
                        self.is_indented(&line)
                    } else {
                        false
                    };
                    
                    if !block_starts_new_paragraph && Self::should_merge(last_block, &line) && Self::roles_match(&last_block.role, &role) {
                        // Add space if needed (simple heuristic: if not ending with hyphen or whitespace)
                        if !last_block.text.trim_end().ends_with('-')
                            && !last_block.text.ends_with(char::is_whitespace)
                        {
                            last_block.text.push(' ');
                        }
                        last_block.text.push_str(&line.text);

                        // Update bbox to include new line
                        last_block.bbox.x1 = last_block.bbox.x1.min(line.bbox.x1);
                        last_block.bbox.y1 = last_block.bbox.y1.min(line.bbox.y1);
                        last_block.bbox.x2 = last_block.bbox.x2.max(line.bbox.x2);
                        last_block.bbox.y2 = last_block.bbox.y2.max(line.bbox.y2);
                        continue;
                    }
                }
            }
            
            // Check if this new block starts a new paragraph (first line in block that is indented)
            let starts_new_paragraph = self.is_indented(&line);

            blocks.push(Block {
                text: line.text,
                bbox: line.bbox,
                role,
                page_num: line.page_num,
                doc_page_num,
                starts_new_paragraph,
            });
        }

        blocks
    }
    
    /// Check if two roles are compatible for merging
    fn roles_match(role1: &BlockRole, role2: &BlockRole) -> bool {
        match (role1, role2) {
            (BlockRole::Heading { level: l1 }, BlockRole::Heading { level: l2 }) => l1 == l2,
            (r1, r2) => r1 == r2,
        }
    }
    
    /// Check if a line is indented relative to the page margin
    fn is_indented(&self, line: &TextLine) -> bool {
        let x = line.bbox.x1;
        // Consider it indented if it's INDENT_THRESHOLD points more than the margin
        x > self.page_left_margin + INDENT_THRESHOLD
    }

    /// Classify a text line into a BlockRole based on position and content patterns.
    fn classify_role(&mut self, line: &TextLine) -> BlockRole {
        let text = line.text.trim();
        let geom = &line.page_geometry;
        let y = line.bbox.y1;

        // Check for References section header
        if patterns::REFERENCES_HEADER.is_match(text) {
            self.in_references_section = true;
            return BlockRole::Reference;
        }

        // Check for Abstract header
        if patterns::ABSTRACT_HEADER.is_match(text) {
            return BlockRole::Abstract;
        }

        // If we're in References section, treat citations
        if self.in_references_section {
            // Reset on new major heading
            if line.font_size > 14.0 && !patterns::CITATION.is_match(text) {
                self.in_references_section = false;
            } else {
                return BlockRole::Citation;
            }
        }

        // Position-based classification
        if geom.is_header_zone(y) {
            if patterns::PAGE_NUMBER.is_match(text) {
                return BlockRole::PageNumber;
            }
            return BlockRole::Header;
        }

        if geom.is_footer_zone(y) {
            if patterns::PAGE_NUMBER.is_match(text) {
                return BlockRole::PageNumber;
            }
            if patterns::FOOTNOTE_MARKER.is_match(text) {
                return BlockRole::Footnote;
            }
            return BlockRole::Footer;
        }

        // Pattern-based classification for body content
        if patterns::CAPTION.is_match(text) {
            return BlockRole::Caption;
        }

        if patterns::CITATION.is_match(text) {
            return BlockRole::Citation;
        }

        // Font-size based heading detection with levels
        if line.font_size >= HEADING_L1_SIZE {
            return BlockRole::Heading { level: 1 };
        }
        if line.font_size >= HEADING_L2_SIZE {
            return BlockRole::Heading { level: 2 };
        }

        BlockRole::Paragraph
    }

    fn should_merge(block: &Block, line: &TextLine) -> bool {
        // 0. Check Page Number
        if block.page_num != line.page_num {
            return false;
        }

        // Role matching is now handled by the caller

        // Check Vertical/Horizontal proximity

        // Horizontal merge (same line)
        // Check if Y ranges overlap significantly
        let y_overlap = block.bbox.y1.max(line.bbox.y1) < block.bbox.y2.min(line.bbox.y2);
        if y_overlap {
            return true;
        }

        // Vertical merge (next line in paragraph)
        // Calculate gap based on coordinate system direction
        let vertical_gap = if block.bbox.y1 > line.bbox.y1 {
            // Y-up (Standard PDF): Block is above Line (higher Y)
            // Gap = Block Bottom (y1) - Line Top (y2)
            block.bbox.y1 - line.bbox.y2
        } else {
            // Y-down (Screen): Block is above Line (lower Y)
            // Gap = Line Top (y1) - Block Bottom (y2)
            line.bbox.y1 - block.bbox.y2
        };

        // Allow normal line spacing (up to 1.5x font size)
        // Negative gap means overlap, which we allow up to 5 units
        if vertical_gap >= -5.0 && vertical_gap < line.font_size * 1.5 {
            return true;
        }

        false
    }
}
