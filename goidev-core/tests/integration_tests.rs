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
    pdf_path.push("tests/resources/test-2.pdf");

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

#[test]
fn test_role_detection_works() {
    let mut pdf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/test-2.pdf");

    let lines = parse_pdf(pdf_path.to_str().unwrap()).expect("Failed to parse PDF.");
    let blocks = ReflowEngine::process(lines);

    // Count blocks by role
    let mut role_counts = std::collections::HashMap::new();
    for block in &blocks {
        *role_counts.entry(format!("{:?}", block.role)).or_insert(0) += 1;
    }

    println!("Role distribution:");
    for (role, count) in &role_counts {
        println!("  {}: {}", role, count);
    }

    // Should have some paragraphs (not all Footer!)
    let para_count = role_counts.get("Paragraph").unwrap_or(&0);
    let footer_count = role_counts.get("Footer").unwrap_or(&0);
    
    assert!(
        *para_count > 0,
        "Should detect some Paragraph blocks, but found none"
    );
    
    assert!(
        *para_count > *footer_count,
        "Should have more paragraphs ({}) than footers ({})",
        para_count, footer_count
    );
}
