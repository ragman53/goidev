//! Unit tests for the reflow engine.
//!
//! Tests line grouping into paragraphs, heading detection based on font size,
//! and block role assignment.

use goidev_core::pdf_parser::{BBox, TextLine};
use goidev_core::reflow_engine::{BlockRole, ReflowEngine};

#[test]
fn test_group_lines_into_paragraph() {
    let lines = vec![
        TextLine {
            text: "Hello ".to_string(),
            bbox: BBox {
                x1: 10.0,
                y1: 100.0,
                x2: 50.0,
                y2: 112.0,
            },
            font_size: 12.0,
            page_num: 1,
        },
        TextLine {
            text: "world.".to_string(),
            bbox: BBox {
                x1: 50.0,
                y1: 100.0,
                x2: 90.0,
                y2: 112.0,
            },
            font_size: 12.0,
            page_num: 1,
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
                y1: 200.0,
                x2: 100.0,
                y2: 224.0,
            },
            font_size: 24.0,
            page_num: 1,
        },
        TextLine {
            text: "It was a dark night.".to_string(),
            bbox: BBox {
                x1: 10.0,
                y1: 100.0,
                x2: 150.0,
                y2: 112.0,
            },
            font_size: 12.0,
            page_num: 1,
        },
    ];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].text, "Chapter 1");
    assert_eq!(blocks[0].role, BlockRole::Heading);
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
                y1: 50.0,
                x2: 100.0,
                y2: 62.0,
            },
            font_size: 12.0,
            page_num: 1,
        },
        TextLine {
            text: "continues on Page 2.".to_string(),
            // Even if coordinates suggest proximity (e.g. at top of page 2 vs bottom of page 1 usually distinct, but here just to prove page_num check)
            bbox: BBox {
                x1: 10.0,
                y1: 40.0, // Close Y coordinate
                x2: 100.0,
                y2: 52.0,
            },
            font_size: 12.0,
            page_num: 2,
        },
    ];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 2, "Should not merge blocks across pages");
    assert_eq!(blocks[0].text, "Page 1 content ");
    assert_eq!(blocks[1].text, "continues on Page 2.");
    assert_eq!(blocks[0].page_num, 1);
    assert_eq!(blocks[1].page_num, 2);
}
