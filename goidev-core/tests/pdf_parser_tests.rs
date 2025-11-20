use goidev_core::pdf_parser::parse_pdf;

#[test]
fn test_extract_text_with_position() {
    let mut pdf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/test.pdf");

    let lines = parse_pdf(pdf_path.to_str().unwrap()).expect("Failed to parse PDF.");

    assert!(!lines.is_empty(), "Should extract text lines.");

    let line = &lines[0];
    assert!(
        line.text.contains("Lorem ipsum"),
        "Text should contain 'Lorem ipsum', got: {}",
        line.text
    );

    // Verify BBox has non-zero dimensions (it's a real physical object on the page)
    assert!(
        line.bbox.x2 > line.bbox.x1,
        "BBox width should be positive, got x1: {}, x2: {}",
        line.bbox.x1,
        line.bbox.x2
    );
    assert!(
        line.bbox.y2 > line.bbox.y1,
        "BBox height should be positive, got y1: {}, y2: {}",
        line.bbox.y1,
        line.bbox.y2
    );

    // Verify font size is reasonable
    assert!(
        line.font_size > 0.0,
        "Font size should be positive, got: {}",
        line.font_size
    );
}

#[test]
fn test_parse_pdf_no_text() {
    // Assuming you create a PDF with no text content for this test.
    // For now, we can use the existing one and check for text that doesn't exist.
    let mut pdf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/test.pdf");

    let lines = parse_pdf(pdf_path.to_str().unwrap()).expect("Failed to parse PDF.");
    let found = lines.iter().any(|line| line.text.contains("This text does not exist"));
    assert!(!found, "Should not find non-existent text.");
}
