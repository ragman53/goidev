// In c:/Users/ragma/dev/goidev/goidev-core/tests/pdf_parser_test.rs

use goidev_core::pdf_parser::parse_pdf;
use std::path::PathBuf;

#[test]
fn test_parse_pdf_happy_path() {
    // Arrange: Set up the path to the test PDF.
    let mut pdf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/sample.pdf");
    let path_str = pdf_path.to_str().expect("Path should be valid UTF-8");
    
    // Act: Call parse_pdf for the first page.
    let result = parse_pdf(path_str, 1, 1);

    // Assert: Check that parsing succeeded and returned at least one line of text.
    // This test is currently RED because the implementation returns an empty Vec.
    assert!(result.is_ok(), "Parsing should succeed");
    let lines = result.unwrap();
    assert!(!lines.is_empty(), "Expected to find text lines, but found none.");
}

#[test]
fn test_parse_pdf_empty_page() {
    // Arrange: Set up the path to the test PDF.
    let mut pdf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/sample.pdf");
    let path_str = pdf_path.to_str().expect("Path should be valid UTF-8");

    // Act: Call parse_pdf for page 2, which is assumed to be empty.
    let result = parse_pdf(path_str, 2, 2);

    // Assert: Check that parsing succeeds and returns an empty Vec.
    // This test is currently GREEN because the stub returns an empty Vec,
    // but it will serve as a regression test once the parser is implemented.
    assert!(result.is_ok(), "Parsing an empty page should succeed");
    let lines = result.unwrap();
    assert!(lines.is_empty(), "Expected no text lines from an empty page, but found some.");
}

#[test]
fn test_parse_pdf_invalid_page_range() {
    // Arrange: Set up the path to the test PDF.
    let mut pdf_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/sample.pdf");
    let path_str = pdf_path.to_str().expect("Path should be valid UTF-8");

    // Act: Call parse_pdf for a page that does not exist.
    let result = parse_pdf(path_str, 99, 99);

    // Assert: Check that parsing returns an Err.
    // This test is currently RED because the implementation does not yet
    // validate page numbers and will return Ok(empty_vec).
    assert!(result.is_err(), "Expected an error for an invalid page number, but got Ok.");
}
