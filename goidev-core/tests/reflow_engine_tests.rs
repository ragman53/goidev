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
        },
    ];

    let blocks = ReflowEngine::process(lines);
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[0].text, "Chapter 1");
    assert_eq!(blocks[0].role, BlockRole::Heading);
    assert_eq!(blocks[1].text, "It was a dark night.");
    assert_eq!(blocks[1].role, BlockRole::Paragraph);
}
