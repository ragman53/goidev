use crate::pdf_parser::{BBox, TextLine};
use serde::{Deserialize, Serialize};

/// Represents the semantic role of a text block in a document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockRole {
    /// Main body text
    Paragraph,
    /// Section or document heading (large font)
    Heading,
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
    pub page_num: u32,
}

pub struct ReflowEngine {
    /// Track if we're in the References section
    in_references_section: bool,
}

/// Patterns for role detection
mod patterns {
    use regex::Regex;
    use std::sync::LazyLock;

    /// Matches page numbers: "1", "- 1 -", "Page 1", "1 of 10", etc.
    pub static PAGE_NUMBER: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[\s\-]*(?:Page\s*)?\d+(?:\s*(?:of|/)\s*\d+)?[\s\-]*$").unwrap()
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
        }
    }

    pub fn process(lines: Vec<TextLine>) -> Vec<Block> {
        let mut engine = Self::new();
        engine.process_lines(lines)
    }

    fn process_lines(&mut self, lines: Vec<TextLine>) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();

        for line in lines {
            // Classify role first (may update internal state for References section)
            let role = self.classify_role(&line);

            // Don't merge certain roles (page numbers, headers, footers)
            let can_merge = matches!(role, 
                BlockRole::Paragraph | BlockRole::Heading | BlockRole::Citation | 
                BlockRole::Footnote | BlockRole::Abstract
            );

            if can_merge {
                if let Some(last_block) = blocks.last_mut() {
                    if Self::should_merge(last_block, &line) && last_block.role == role {
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

            blocks.push(Block {
                text: line.text,
                bbox: line.bbox,
                role,
                page_num: line.page_num,
            });
        }

        blocks
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

        // Font-size based heading detection
        if line.font_size > 14.0 {
            return BlockRole::Heading;
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
