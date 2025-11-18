use goidev_core::pdf_parser::{parse_pdf, BBox, TextLine};

#[test]
fn test_extract_text_with_position() {
    let mut pdf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/test.pdf");

    let lines = parse_pdf(pdf_path.to_str().unwrap()).expect("Failed to parse PDF.");

    assert_eq!(lines.len(), 1, "Should extract exactly one line of text.");

    let line = &lines[0];
    assert_eq!(line.text, "Hello, World!");
    assert_eq!(line.font_size, 12.0);

    // Note: These BBox coordinates are approximate and may need adjustment
    // once the parsing logic is implemented. They represent a plausible
    // position for a single line of text near the top-left of a page.
    let expected_bbox = BBox {
        x1: 50.0,
        y1: 770.0,
        x2: 120.0,
        y2: 790.0,
    };

    assert!(
        (line.bbox.x1 - expected_bbox.x1).abs() < 5.0 && (line.bbox.y1 - expected_bbox.y1).abs() < 5.0,
        "BBox position is not as expected."
    );
}