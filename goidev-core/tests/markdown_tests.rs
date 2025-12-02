//! Tests for markdown serialization/deserialization.

use goidev_core::markdown::{blocks_to_markdown, markdown_to_blocks, MarkdownMeta};
use goidev_core::pdf_parser::BBox;
use goidev_core::reflow_engine::{Block, BlockRole};

fn sample_blocks() -> Vec<Block> {
    vec![
        Block {
            text: "Chapter One".to_string(),
            bbox: BBox {
                x1: 72.0,
                y1: 720.0,
                x2: 200.0,
                y2: 740.0,
            },
            role: BlockRole::Heading { level: 1 },
            page_num: 1,
            doc_page_num: None,
            starts_new_paragraph: false,
        },
        Block {
            text: "This is the first paragraph of the document.".to_string(),
            bbox: BBox {
                x1: 72.0,
                y1: 700.0,
                x2: 540.0,
                y2: 714.0,
            },
            role: BlockRole::Paragraph,
            page_num: 1,
            doc_page_num: None,
            starts_new_paragraph: false,
        },
        Block {
            text: "Another paragraph on page two.".to_string(),
            bbox: BBox {
                x1: 72.0,
                y1: 720.0,
                x2: 400.0,
                y2: 734.0,
            },
            role: BlockRole::Paragraph,
            page_num: 2,
            doc_page_num: None,
            starts_new_paragraph: false,
        },
    ]
}

#[test]
fn test_blocks_to_markdown_roundtrip() {
    let blocks = sample_blocks();
    let meta = MarkdownMeta {
        source_hash: Some("abc123".to_string()),
    };

    let md = blocks_to_markdown(&blocks, &meta);

    // Verify frontmatter
    assert!(md.contains("source_hash: abc123"));

    // Verify metadata comments (now includes role=)
    assert!(md.contains("<!-- goidev:page=1 bbox=72.0,720.0,200.0,740.0 role=heading1 -->"));
    assert!(md.contains("# Chapter One"));
    assert!(md.contains("<!-- goidev:page=2 bbox=72.0,720.0,400.0,734.0 role=paragraph -->"));

    // Roundtrip
    let (parsed_blocks, parsed_meta) = markdown_to_blocks(&md);

    assert_eq!(parsed_meta.source_hash, Some("abc123".to_string()));
    assert_eq!(parsed_blocks.len(), 3);

    assert_eq!(parsed_blocks[0].text, "Chapter One");
    assert_eq!(parsed_blocks[0].role, BlockRole::Heading { level: 1 });
    assert_eq!(parsed_blocks[0].page_num, 1);
    assert!((parsed_blocks[0].bbox.x1 - 72.0).abs() < 0.1);

    assert_eq!(
        parsed_blocks[1].text,
        "This is the first paragraph of the document."
    );
    assert_eq!(parsed_blocks[1].role, BlockRole::Paragraph);

    assert_eq!(parsed_blocks[2].page_num, 2);
}

#[test]
fn test_lenient_import_external_markdown() {
    // Markdown without goidev metadata (external tool)
    let external_md = r#"
# Welcome

This is a paragraph from an external tool.

## Subtitle

More content here.
"#;

    let (blocks, meta) = markdown_to_blocks(external_md);

    // No source hash expected
    assert!(meta.source_hash.is_none());

    // Should parse 4 blocks: heading, para, heading (H2 treated as Paragraph per current logic), para
    // Actually H2 is not H1 so it becomes Paragraph per current impl
    assert!(blocks.len() >= 3);

    // First block should be heading
    assert_eq!(blocks[0].text, "Welcome");
    assert!(matches!(blocks[0].role, BlockRole::Heading { .. }));
    assert_eq!(blocks[0].page_num, 1); // synthetic

    // Synthetic bbox should have reasonable defaults
    assert!(blocks[0].bbox.x1 > 0.0);
}

#[test]
fn test_empty_markdown() {
    let (blocks, meta) = markdown_to_blocks("");
    assert!(blocks.is_empty());
    assert!(meta.source_hash.is_none());
}

#[test]
fn test_frontmatter_only() {
    let md = "---\nsource_hash: deadbeef\n---\n";
    let (blocks, meta) = markdown_to_blocks(md);
    assert!(blocks.is_empty());
    assert_eq!(meta.source_hash, Some("deadbeef".to_string()));
}
