//! Markdown serialization/deserialization for cached reflow data.
//!
//! # Format
//!
//! - YAML frontmatter with `source_hash` (SHA-256 of source PDF).
//! - Each block preceded by an HTML comment: `<!-- goidev:page=N bbox=x1,y1,x2,y2 -->`
//! - Headings rendered as `# ...`, paragraphs as plain text.
//!
//! # External Markdown Import (lenient)
//!
//! - `#` → Heading, else Paragraph
//! - Synthetic page_num=1, placeholder bbox.

use crate::pdf_parser::BBox;
use crate::reflow_engine::{Block, BlockRole};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{self, Read};
use std::path::Path;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default page margin (points) for synthetic bbox.
const DEFAULT_MARGIN: f32 = 72.0;
/// Default page width minus margins (points).
const DEFAULT_WIDTH: f32 = 540.0;
/// Starting Y position for synthetic layout (top of page).
const SYNTH_START_Y: f32 = 700.0;
/// Line height for headings (points).
const HEADING_HEIGHT: f32 = 20.0;
/// Line height for paragraphs (points).
const PARAGRAPH_HEIGHT: f32 = 14.0;
/// Vertical gap after heading.
const HEADING_GAP: f32 = 30.0;
/// Vertical gap after paragraph.
const PARAGRAPH_GAP: f32 = 20.0;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Metadata stored in the Markdown frontmatter.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MarkdownMeta {
    /// SHA-256 hex digest of the source PDF (if any).
    pub source_hash: Option<String>,
}

// ---------------------------------------------------------------------------
// Serialization: Blocks → Markdown
// ---------------------------------------------------------------------------

/// Serialize blocks to a Markdown string with goidev metadata.
pub fn blocks_to_markdown(blocks: &[Block], meta: &MarkdownMeta) -> String {
    let mut out = String::new();
    write_frontmatter(&mut out, meta);
    for block in blocks {
        write_block(&mut out, block);
    }
    out
}

fn write_frontmatter(out: &mut String, meta: &MarkdownMeta) {
    out.push_str("---\n");
    if let Some(ref hash) = meta.source_hash {
        out.push_str(&format!("source_hash: {}\n", hash));
    }
    out.push_str("---\n\n");
}

fn write_block(out: &mut String, block: &Block) {
    // Metadata comment with role
    let role_str = match &block.role {
        BlockRole::Paragraph => "paragraph",
        BlockRole::Heading { level } => match level {
            1 => "heading1",
            2 => "heading2",
            _ => "heading3",
        },
        BlockRole::PageNumber => "pagenumber",
        BlockRole::Header => "header",
        BlockRole::Footer => "footer",
        BlockRole::Footnote => "footnote",
        BlockRole::Caption => "caption",
        BlockRole::Citation => "citation",
        BlockRole::Author => "author",
        BlockRole::Abstract => "abstract",
        BlockRole::Reference => "reference",
    };
    out.push_str(&format!(
        "<!-- goidev:page={} bbox={:.1},{:.1},{:.1},{:.1} role={} -->\n",
        block.page_num, block.bbox.x1, block.bbox.y1, block.bbox.x2, block.bbox.y2, role_str
    ));
    // Content format based on role
    match &block.role {
        BlockRole::Heading { level: 1 } | BlockRole::Reference => {
            out.push_str(&format!("# {}\n\n", block.text.trim()))
        }
        BlockRole::Heading { level: 2 } => {
            out.push_str(&format!("## {}\n\n", block.text.trim()))
        }
        BlockRole::Heading { level: _ } => {
            out.push_str(&format!("### {}\n\n", block.text.trim()))
        }
        BlockRole::PageNumber | BlockRole::Header | BlockRole::Footer => {
            // Skip rendering these in output (they're metadata only)
            out.push_str(&format!("<!-- {} -->\n\n", block.text.trim()))
        }
        BlockRole::Footnote => {
            out.push_str(&format!("> [^note]: {}\n\n", block.text.trim()))
        }
        BlockRole::Caption => {
            out.push_str(&format!("*{}*\n\n", block.text.trim()))
        }
        BlockRole::Citation => {
            out.push_str(&format!("- {}\n\n", block.text.trim()))
        }
        BlockRole::Author => {
            out.push_str(&format!("**{}**\n\n", block.text.trim()))
        }
        BlockRole::Abstract => {
            out.push_str(&format!("> {}\n\n", block.text.trim()))
        }
        BlockRole::Paragraph => {
            out.push_str(&format!("{}\n\n", block.text.trim()))
        }
    }
}

/// Write blocks to a Markdown file (sidecar).
pub fn save_markdown<P: AsRef<Path>>(
    blocks: &[Block],
    meta: &MarkdownMeta,
    dest: P,
) -> io::Result<()> {
    fs::write(dest, blocks_to_markdown(blocks, meta))
}

// ---------------------------------------------------------------------------
// Deserialization: Markdown → Blocks (lenient)
// ---------------------------------------------------------------------------

/// Parse a Markdown string into blocks. Lenient mode:
/// - If goidev metadata comments are present, use them.
/// - Otherwise, synthesize defaults (page=1, placeholder bbox).
pub fn markdown_to_blocks(md: &str) -> (Vec<Block>, MarkdownMeta) {
    let (body, meta) = parse_frontmatter(md);
    let blocks = parse_body(body);
    (blocks, meta)
}

fn parse_frontmatter(md: &str) -> (&str, MarkdownMeta) {
    let mut meta = MarkdownMeta::default();
    if !md.starts_with("---") {
        return (md, meta);
    }
    let Some(end) = md[3..].find("---") else {
        return (md, meta);
    };
    let fm = &md[3..3 + end];
    for line in fm.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("source_hash:") {
            meta.source_hash = Some(rest.trim().to_string());
        }
    }
    (&md[3 + end + 3..], meta)
}

/// Internal state for parsing markdown body.
struct ParseState {
    pending_page: Option<u32>,
    pending_bbox: Option<BBox>,
    pending_role: Option<BlockRole>,
    synth_y: f32,
    current_text: String,
    current_role: Option<BlockRole>,
    blocks: Vec<Block>,
}

impl ParseState {
    fn new() -> Self {
        Self {
            pending_page: None,
            pending_bbox: None,
            pending_role: None,
            synth_y: SYNTH_START_Y,
            current_text: String::new(),
            current_role: None,
            blocks: Vec::new(),
        }
    }

    fn handle_metadata_comment(&mut self, html: &str) {
        let s = html.trim();
        if !(s.starts_with("<!-- goidev:") && s.ends_with("-->")) {
            return;
        }
        let inner = s[12..s.len() - 3].trim();
        for part in inner.split_whitespace() {
            if let Some(val) = part.strip_prefix("page=") {
                self.pending_page = val.parse().ok();
            } else if let Some(val) = part.strip_prefix("bbox=") {
                self.pending_bbox = parse_bbox(val);
            } else if let Some(val) = part.strip_prefix("role=") {
                self.pending_role = parse_role(val);
            }
        }
    }

    fn start_heading(&mut self, level: HeadingLevel) {
        let lvl = match level {
            HeadingLevel::H1 => 1,
            HeadingLevel::H2 => 2,
            _ => 3,
        };
        self.current_role = Some(BlockRole::Heading { level: lvl });
    }

    fn end_heading(&mut self) {
        self.finish_block(BlockRole::Heading { level: 2 }, HEADING_HEIGHT, HEADING_GAP);
    }

    fn start_paragraph(&mut self) {
        self.current_role = Some(BlockRole::Paragraph);
    }

    fn end_paragraph(&mut self) {
        self.finish_block(BlockRole::Paragraph, PARAGRAPH_HEIGHT, PARAGRAPH_GAP);
    }

    fn start_blockquote(&mut self) {
        // Blockquotes are used for Abstract/Footnote; role comes from metadata
        self.current_role = Some(BlockRole::Abstract);
    }

    fn end_blockquote(&mut self) {
        self.finish_block(BlockRole::Abstract, PARAGRAPH_HEIGHT, PARAGRAPH_GAP);
    }

    fn start_list_item(&mut self) {
        // List items are used for Citations; role comes from metadata
        self.current_role = Some(BlockRole::Citation);
    }

    fn end_list_item(&mut self) {
        self.finish_block(BlockRole::Citation, PARAGRAPH_HEIGHT, PARAGRAPH_GAP);
    }

    fn append_text(&mut self, t: &str) {
        self.current_text.push_str(t);
    }

    fn append_space(&mut self) {
        self.current_text.push(' ');
    }

    fn finish_block(&mut self, default_role: BlockRole, height: f32, gap: f32) {
        if self.current_text.is_empty() {
            return;
        }
        let page = self.pending_page.take().unwrap_or(1);
        let bbox = self.pending_bbox.take().unwrap_or_else(|| {
            let b = BBox {
                x1: DEFAULT_MARGIN,
                y1: self.synth_y,
                x2: DEFAULT_WIDTH,
                y2: self.synth_y + height,
            };
            self.synth_y -= gap;
            b
        });
        // Priority: pending_role from metadata > current_role from element > default_role
        let role = self.pending_role.take()
            .or_else(|| self.current_role.take())
            .unwrap_or(default_role);
        self.blocks.push(Block {
            text: std::mem::take(&mut self.current_text),
            bbox,
            role,
            page_num: page,
            doc_page_num: None,
            starts_new_paragraph: false,
        });
    }
}

fn parse_bbox(val: &str) -> Option<BBox> {
    let nums: Vec<f32> = val.split(',').filter_map(|n| n.parse().ok()).collect();
    if nums.len() == 4 {
        Some(BBox {
            x1: nums[0],
            y1: nums[1],
            x2: nums[2],
            y2: nums[3],
        })
    } else {
        None
    }
}

fn parse_role(val: &str) -> Option<BlockRole> {
    match val.to_lowercase().as_str() {
        "paragraph" => Some(BlockRole::Paragraph),
        "heading" | "heading1" => Some(BlockRole::Heading { level: 1 }),
        "heading2" => Some(BlockRole::Heading { level: 2 }),
        "heading3" => Some(BlockRole::Heading { level: 3 }),
        "pagenumber" => Some(BlockRole::PageNumber),
        "header" => Some(BlockRole::Header),
        "footer" => Some(BlockRole::Footer),
        "footnote" => Some(BlockRole::Footnote),
        "caption" => Some(BlockRole::Caption),
        "citation" => Some(BlockRole::Citation),
        "author" => Some(BlockRole::Author),
        "abstract" => Some(BlockRole::Abstract),
        "reference" => Some(BlockRole::Reference),
        _ => None,
    }
}

fn parse_body(body: &str) -> Vec<Block> {
    let mut state = ParseState::new();
    for event in Parser::new(body) {
        match event {
            Event::Html(html) => state.handle_metadata_comment(&html),
            Event::Start(Tag::Heading { level, .. }) => state.start_heading(level),
            Event::End(TagEnd::Heading(_)) => state.end_heading(),
            Event::Start(Tag::Paragraph) => state.start_paragraph(),
            Event::End(TagEnd::Paragraph) => state.end_paragraph(),
            // Handle blockquotes (Abstract, Footnote)
            Event::Start(Tag::BlockQuote(_)) => state.start_blockquote(),
            Event::End(TagEnd::BlockQuote(_)) => state.end_blockquote(),
            // Handle list items (Citation)
            Event::Start(Tag::Item) => state.start_list_item(),
            Event::End(TagEnd::Item) => state.end_list_item(),
            // Handle emphasis and strong (Caption, Author)
            Event::Start(Tag::Emphasis) | Event::End(TagEnd::Emphasis) => {}
            Event::Start(Tag::Strong) | Event::End(TagEnd::Strong) => {}
            Event::Text(t) => state.append_text(&t),
            Event::SoftBreak | Event::HardBreak => state.append_space(),
            _ => {}
        }
    }
    state.blocks
}

/// Load blocks from a Markdown file.
pub fn load_markdown<P: AsRef<Path>>(path: P) -> io::Result<(Vec<Block>, MarkdownMeta)> {
    let content = fs::read_to_string(path)?;
    Ok(markdown_to_blocks(&content))
}

// ---------------------------------------------------------------------------
// File Helpers
// ---------------------------------------------------------------------------

/// Compute SHA-256 hash of a file and return hex string.
pub fn hash_file<P: AsRef<Path>>(path: P) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Generate the sidecar Markdown path for a given source file.
/// Uses a cache directory in the user's local app data to avoid file watcher issues.
/// e.g., `C:\path\to\doc.pdf` → `{cache_dir}/goidev/{hash_of_path}.goidev.md`
pub fn sidecar_path<P: AsRef<Path>>(source: P) -> std::path::PathBuf {
    use sha2::{Digest, Sha256};
    
    // Get cache directory (platform-specific)
    let cache_dir = dirs::cache_dir()
        .or_else(|| dirs::data_local_dir())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    
    let goidev_cache = cache_dir.join("goidev").join("cache");
    
    // Create cache directory if it doesn't exist
    let _ = std::fs::create_dir_all(&goidev_cache);
    
    // Hash the source path to create a unique filename
    let source_path = source.as_ref().to_string_lossy();
    let mut hasher = Sha256::new();
    hasher.update(source_path.as_bytes());
    let path_hash = format!("{:x}", hasher.finalize());
    
    // Use first 16 chars of hash + original filename for readability
    let file_name = source.as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("document");
    
    let cache_name = format!("{}_{}.goidev.md", &path_hash[..16], file_name);
    goidev_cache.join(cache_name)
}

/// Check if the cached sidecar is stale compared to the source PDF.
/// Returns `true` if the cache is valid (hashes match), `false` otherwise.
pub fn is_cache_valid<P: AsRef<Path>>(source: P, sidecar: P) -> bool {
    let Ok(source_hash) = hash_file(&source) else {
        return false;
    };
    let Ok((_, meta)) = load_markdown(&sidecar) else {
        return false;
    };
    meta.source_hash.as_deref() == Some(source_hash.as_str())
}
