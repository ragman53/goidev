use goidev_core::pdf_parser::{parse_pdf, BBox, TextLine};

#[test]
fn test_parse_pdf_mvp_happy_path() {
    // This test uses a simple, one-line PDF fixture.
    // The test runner executes from the crate root, so the path is relative to `goidev-core`.
    let path = "goidev-core/test.pdf";

    // The current MVP uses `extract_text`, which works, but returns dummy bbox/font_size.
    // We expect this test to FAIL because the bbox coordinates will be 0.0.
    // This is our "Red" test in "Red-Green-Refactor".
    let result = parse_pdf(path, 1, 1);

    assert!(result.is_ok());
    let lines = result.unwrap();
    assert!(!lines.is_empty(), "Should have extracted at least one line of text.");

    let first_line = &lines[0];
    assert_eq!(first_line.text, "Hello, World!");
    assert_ne!(first_line.bbox.x1, 0.0, "BBox x1 should not be the dummy value 0.0");
}

