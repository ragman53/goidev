// In c:/Users/ragma/dev/goidev/goidev-core/tests/pdf_parser_test.rs

use goidev_core::pdf_parser::parse_pdf;
use std::path::PathBuf;

#[test]
fn test_extract_test_simple_happy_path() {
    // define path_str
    let mut pdf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/sample.pdf");
    let path_str = pdf_path.to_str().expect("Path should be valid UTF-8");
    
    // call parse_pdf
    let result = parse_pdf(path_str, 1, 1);

    assert!(result.is_ok(), "Parsing should succeed");
    let chunks = result.unwrap();
    assert!(!chunks.is_empty(), "Expected to find text chunks, but found none.");
}


