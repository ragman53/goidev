use goidev_core::pdf_parser::parse_pdf;
use goidev_core::reflow_engine::ReflowEngine;

#[test]
fn test_reflow_real_pdf() {
    let mut pdf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/test.pdf");

    let lines = parse_pdf(pdf_path.to_str().unwrap()).expect("Failed to parse PDF.");
    let blocks = ReflowEngine::process(lines);

    println!("--- Reflowed Blocks ---");
    for (i, block) in blocks.iter().enumerate() {
        println!("Block {}: Role={:?}", i, block.role);
        println!("  BBox: {:?}", block.bbox);
        println!("  Text: {}", block.text);
        println!("-----------------------");
    }
    println!("--- End of Blocks ---");

    assert!(!blocks.is_empty(), "Should produce blocks from real PDF.");
}
