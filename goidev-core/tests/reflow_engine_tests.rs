//! Unit tests for the reflow engine.
//!
//! Tests line grouping into paragraphs, heading detection based on font size,
//! and block role assignment.

use goidev_core::pdf_parser::{BBox, PageGeometry, TextLine};
use goidev_core::reflow_engine::{BlockRole, ReflowEngine};

/// Helper to create a default page geometry (US Letter).
fn default_geom() -> PageGeometry {
    PageGeometry {
        width: 612.0,
        height: 792.0,
        origin_x: 0.0,
        origin_y: 0.0,
    }
}

/// Helper to create a TextLine at a given Y position.
fn make_line(text: &str, y: f32, font_size: f32, page_num: u32) -> TextLine {
    TextLine {
        text: text.to_string(),
        bbox: BBox {
            x1: 72.0,
            y1: y,
            x2: 300.0,
            y2: y + font_size,
        },
        font_size,
        page_num,
        page_geometry: default_geom(),
    }
}

#[test]
fn test_group_lines_into_paragraph() {
    let lines = vec![
        TextLine {
            text: "Hello ".to_string(),
            bbox: BBox {
                x1: 10.0,
                y1: 400.0,  // Middle of page
                x2: 50.0,
                y2: 412.0,
            },
            font_size: 12.0,
            page_num: 1,
            page_geometry: default_geom(),
        },
        TextLine {
            text: "world.".to_string(),
            bbox: BBox {
                x1: 50.0,
                y1: 400.0,
                x2: 90.0,
                y2: 412.0,
            },
            font_size: 12.0,
            page_num: 1,
            page_geometry: default_geom(),
        },
    ];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].text, "Hello world.");
    assert_eq!(blocks[0].role, BlockRole::Paragraph);
}

#[test]
fn test_detect_heading() {
    let lines = vec![
        TextLine {
            text: "Chapter 1".to_string(),
            bbox: BBox {
                x1: 10.0,
                y1: 500.0,  // Middle of page
                x2: 100.0,
                y2: 524.0,
            },
            font_size: 24.0,
            page_num: 1,
            page_geometry: default_geom(),
        },
        TextLine {
            text: "It was a dark night.".to_string(),
            bbox: BBox {
                x1: 10.0,
                y1: 400.0,
                x2: 150.0,
                y2: 412.0,
            },
            font_size: 12.0,
            page_num: 1,
            page_geometry: default_geom(),
        },
    ];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].text, "Chapter 1");
    assert!(matches!(blocks[0].role, BlockRole::Heading { .. }));
    assert_eq!(blocks[1].text, "It was a dark night.");
    assert_eq!(blocks[1].role, BlockRole::Paragraph);
}

#[test]
fn test_no_merge_across_pages() {
    let lines = vec![
        TextLine {
            text: "Page 1 content ".to_string(),
            bbox: BBox {
                x1: 10.0,
                y1: 400.0,
                x2: 100.0,
                y2: 412.0,
            },
            font_size: 12.0,
            page_num: 1,
            page_geometry: default_geom(),
        },
        TextLine {
            text: "continues on Page 2.".to_string(),
            bbox: BBox {
                x1: 10.0,
                y1: 400.0,
                x2: 100.0,
                y2: 412.0,
            },
            font_size: 12.0,
            page_num: 2,
            page_geometry: default_geom(),
        },
    ];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 2, "Should not merge blocks across pages");
    assert_eq!(blocks[0].text, "Page 1 content ");
    assert_eq!(blocks[1].text, "continues on Page 2.");
    assert_eq!(blocks[0].page_num, 1);
    assert_eq!(blocks[1].page_num, 2);
}

// ============================================================================
// New tests for enhanced role detection
// ============================================================================

#[test]
fn test_detect_page_number_in_header() {
    // Page number in header zone (top 8% of 792 = ~730+)
    let lines = vec![make_line("42", 760.0, 10.0, 1)];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].role, BlockRole::PageNumber);
}

#[test]
fn test_detect_page_number_in_footer() {
    // Page number in footer zone (bottom 8% of 792 = ~63)
    let lines = vec![make_line("- 5 -", 30.0, 10.0, 1)];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].role, BlockRole::PageNumber);
}

#[test]
fn test_detect_header_content() {
    // Non-page-number content in header zone
    let lines = vec![make_line("My Document Title", 760.0, 10.0, 1)];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].role, BlockRole::Header);
}

#[test]
fn test_detect_footer_content() {
    // Non-page-number content in footer zone
    let lines = vec![make_line("Copyright 2024 Example Inc.", 30.0, 10.0, 1)];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].role, BlockRole::Footer);
}

#[test]
fn test_detect_footnote() {
    // Footnote marker in footer zone
    let lines = vec![make_line("1. This is a footnote.", 30.0, 9.0, 1)];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].role, BlockRole::Footnote);
}

#[test]
fn test_detect_figure_caption() {
    // Figure caption pattern
    let lines = vec![make_line("Figure 1: Architecture overview", 400.0, 10.0, 1)];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].role, BlockRole::Caption);
}

#[test]
fn test_detect_table_caption() {
    // Table caption pattern
    let lines = vec![make_line("Table 2. Performance metrics", 400.0, 10.0, 1)];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].role, BlockRole::Caption);
}

#[test]
fn test_detect_references_section() {
    let lines = vec![
        make_line("References", 500.0, 14.0, 5),
        make_line("[1] Smith, J. (2020). A paper.", 480.0, 10.0, 5),
        // Add a larger gap to prevent merging
        make_line("[2] Jones, A. (2021). Another paper.", 440.0, 10.0, 5),
    ];

    let blocks = ReflowEngine::process(lines);
    // "References" header triggers Reference role
    assert!(blocks.iter().any(|b| b.role == BlockRole::Reference && b.text.contains("References")));
    // Should have Citation entries (may be merged or separate)
    assert!(blocks.iter().any(|b| b.role == BlockRole::Citation));
}

#[test]
fn test_detect_bibliography_section() {
    let lines = vec![
        make_line("Bibliography", 500.0, 14.0, 5),
        make_line("1. Author, A. Title. 2020.", 480.0, 10.0, 5),
    ];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].role, BlockRole::Reference);
    assert_eq!(blocks[1].role, BlockRole::Citation);
}

#[test]
fn test_citation_pattern_in_body() {
    // Citation pattern should be detected even in body
    let lines = vec![make_line("[1] Author, A. Some title. 2020.", 400.0, 10.0, 1)];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].role, BlockRole::Citation);
}

#[test]
fn test_mixed_content_classification() {
    let lines = vec![
        make_line("42", 760.0, 10.0, 1),                             // Header page number
        make_line("Introduction", 700.0, 18.0, 1),                   // Heading
        make_line("This paper presents...", 650.0, 12.0, 1),         // Paragraph
        make_line("Figure 1: Overview", 400.0, 10.0, 1),             // Caption
        make_line("1. See appendix for details.", 50.0, 9.0, 1),     // Footnote
        make_line("Page 1 of 10", 30.0, 10.0, 1),                    // Footer page number
    ];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 6);
    assert_eq!(blocks[0].role, BlockRole::PageNumber);
    assert!(matches!(blocks[1].role, BlockRole::Heading { .. }));
    assert_eq!(blocks[2].role, BlockRole::Paragraph);
    assert_eq!(blocks[3].role, BlockRole::Caption);
    assert_eq!(blocks[4].role, BlockRole::Footnote);
    assert_eq!(blocks[5].role, BlockRole::PageNumber);
}
