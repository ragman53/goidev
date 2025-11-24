//! Debug utility for inspecting raw PDF byte sequences and their decoded output.
//!
//! This test is ignored by default. Run with:
//! `cargo test debug_raw_bytes -- --ignored --nocapture`

use goidev_core::pdf_parser::decode_pdf_str;
use lopdf::Document;

#[test]
#[ignore]
fn debug_raw_bytes() {
    let mut pdf_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pdf_path.push("tests/resources/test-1.pdf");
    let path_str = pdf_path.to_str().unwrap();

    let doc = Document::load(path_str).expect("Failed to load PDF");

    for (page_num, page_id) in doc.get_pages() {
        println!("Page {}", page_num);
        let content_bytes = doc
            .get_page_content(page_id)
            .expect("Failed to get content");
        let content =
            lopdf::content::Content::decode(&content_bytes).expect("Failed to decode content");

        let mut op_count = 0;
        for op in content.operations.iter() {
            if op.operator == "Tj" || op.operator == "TJ" {
                op_count += 1;
                if op_count > 50 {
                    break;
                }

                println!("Op {}: {:?}", op_count, op);
                for operand in &op.operands {
                    if let Ok(text_bytes) = operand.as_str() {
                        println!("  Bytes: {:02X?}", text_bytes);
                        println!("  Decoded: {}", decode_pdf_str(text_bytes));
                    } else if let Ok(arr) = operand.as_array() {
                        for item in arr {
                            if let Ok(text_bytes) = item.as_str() {
                                println!("  TJ Bytes: {:02X?}", text_bytes);
                                println!("  TJ Decoded: {}", decode_pdf_str(text_bytes));
                            }
                        }
                    }
                }
            }
        }
    }
}
