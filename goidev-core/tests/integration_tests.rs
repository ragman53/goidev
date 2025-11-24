//! Integration tests for the PDF parsing and reflow pipeline.
//!
//! These tests verify the end-to-end functionality of parsing PDF files
//! and reflowing their content into logical blocks.
//!
//! - `test_reflow_simple_pdf` - Tests with a simple Lorem Ipsum PDF
//! - `test_reflow_complex_pdf` - Tests with a complex academic PDF that has
//!   encoding challenges (custom ligatures, special quotes, etc.)

use goidev_core::pdf_parser::parse_pdf;
use goidev_core::reflow_engine::ReflowEngine;

#[test]
fn test_reflow_simple_pdf() {
    let mut pdf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/test.pdf");

    let lines = parse_pdf(pdf_path.to_str().unwrap()).expect("Failed to parse PDF.");
    let blocks = ReflowEngine::process(lines);

    println!("--- Reflowed Blocks (Simple) ---");
    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: Role={:?}", i, block.role);
        println!("  BBox: {:?}", block.bbox);
        println!("  Text: {}", block.text);
        println!("-----------------------");
    }
    println!("--- End of Blocks ---");

    assert!(!blocks.is_empty(), "Should produce blocks from real PDF.");
}

#[test]
fn test_reflow_complex_pdf() {
    let mut pdf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/test-1.pdf");

    let lines = parse_pdf(pdf_path.to_str().unwrap()).expect("Failed to parse PDF.");
    let blocks = ReflowEngine::process(lines);

    println!("--- Reflowed Blocks (Complex) ---");
    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: Role={:?}", i, block.role);
        println!("  BBox: {:?}", block.bbox);
        println!("  Text: {}", block.text);
        println!("-----------------------");
    }
    println!("--- End of Blocks ---");

    // Verification for encoding fix
    let full_text = blocks
        .iter()
        .map(|b| b.text.as_str())
        .collect::<Vec<_>>()
        .join(" ");

    // Verify encoding fix - should not contain garbled text
    assert!(
        !full_text.contains("・ｽ"),
        "Should not contain garbled encoding characters"
    );
}
