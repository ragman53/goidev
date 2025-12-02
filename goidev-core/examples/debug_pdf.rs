//! Debug tool to inspect PDF parsing coordinates

use goidev_core::pdf_parser::parse_pdf;
use std::env;

fn main() {
    // Use command-line arg or default to test.pdf
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        args[1].as_str()
    } else {
        "goidev-core/tests/resources/test.pdf"
    };
    
    // Check for --raw flag for raw operator debugging
    let raw_mode = args.iter().any(|a| a == "--raw");
    let state_mode = args.iter().any(|a| a == "--state");
    
    if raw_mode {
        debug_raw_pdf(path);
    } else if state_mode {
        debug_with_state(path);
    } else {
        debug_parsed_pdf(path);
    }
}

fn debug_parsed_pdf(path: &str) {
    println!("Parsing: {}", path);
    
    match parse_pdf(path) {
        Ok(lines) => {
            println!("\nFound {} text lines\n", lines.len());
            
            // Show all lines with their coordinates
            for (i, line) in lines.iter().take(50).enumerate() {
                let text_preview: String = line.text.chars().take(50).collect();
                println!(
                    "[{}] Page {} | Y={:.1} | bbox=({:.1},{:.1},{:.1},{:.1}) | size={:.1} | \"{}\"",
                    i,
                    line.page_num,
                    line.bbox.y1,
                    line.bbox.x1, line.bbox.y1, line.bbox.x2, line.bbox.y2,
                    line.font_size,
                    text_preview
                );
            }
            
            // Show page geometry
            if let Some(first) = lines.first() {
                println!("\nPage geometry: {:?}", first.page_geometry);
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}

fn debug_raw_pdf(path: &str) {
    use lopdf::Document;
    
    println!("Raw PDF operators for: {}\n", path);
    
    let doc = Document::load(path).expect("Failed to load PDF");
    let pages = doc.get_pages();
    
    // Just show first page
    if let Some((page_num, page_id)) = pages.into_iter().next() {
        println!("=== Page {} ===\n", page_num);
        
        // Show MediaBox
        if let Ok((Some(_resources), _)) = doc.get_page_resources(page_id) {
            println!("Resources found");
        }
        
        if let Ok(content_data) = doc.get_page_content(page_id) {
            if let Ok(content) = lopdf::content::Content::decode(&content_data) {
                // Show first 100 operators
                for (i, op) in content.operations.iter().take(100).enumerate() {
                    println!("[{:3}] {} {:?}", i, op.operator, op.operands);
                }
            }
        }
    }
}

/// Debug with state tracking
fn debug_with_state(path: &str) {
    use lopdf::Document;
    use goidev_core::pdf_state::PdfState;
    
    println!("State-tracked PDF parsing for: {}\n", path);
    
    let doc = Document::load(path).expect("Failed to load PDF");
    let pages = doc.get_pages();
    let mut state = PdfState::new();
    
    if let Some((page_num, page_id)) = pages.into_iter().next() {
        println!("=== Page {} ===\n", page_num);
        
        if let Ok(content_data) = doc.get_page_content(page_id) {
            if let Ok(content) = lopdf::content::Content::decode(&content_data) {
                let mut text_count = 0;
                for (i, op) in content.operations.iter().enumerate() {
                    let operator = op.operator.as_str();
                    match operator {
                        "BT" => {
                            state.bt();
                            println!("[{:3}] BT - text matrix reset", i);
                        }
                        "Tm" => {
                            println!("[{:3}] Found Tm with {} operands: {:?}", i, op.operands.len(), op.operands);
                            if op.operands.len() >= 6 {
                                let ops: Vec<f32> = op.operands.iter()
                                    .take(6)
                                    .filter_map(|o| {
                                        let result = o.as_f32();
                                        if result.is_err() {
                                            println!("  Failed to convert {:?} to f32", o);
                                        }
                                        result.ok()
                                    })
                                    .collect();
                                println!("  Converted {} operands: {:?}", ops.len(), ops);
                                if ops.len() == 6 {
                                    state.tm(ops[0], ops[1], ops[2], ops[3], ops[4], ops[5]);
                                    let (x, y) = state.current_position();
                                    println!("[{:3}] Tm [{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}] => position ({:.1}, {:.1})",
                                        i, ops[0], ops[1], ops[2], ops[3], ops[4], ops[5], x, y);
                                }
                            }
                        }
                        "Td" | "TD" => {
                            if let (Some(x), Some(y)) = (op.operands.get(0), op.operands.get(1)) {
                                if let (Ok(dx), Ok(dy)) = (x.as_f32(), y.as_f32()) {
                                    if op.operator == "Td" {
                                        state.td(dx, dy);
                                    } else {
                                        state.td_capital(dx, dy);
                                    }
                                    let (px, py) = state.current_position();
                                    println!("[{:3}] {} [{:.1},{:.1}] => position ({:.1}, {:.1})",
                                        i, op.operator, dx, dy, px, py);
                                }
                            }
                        }
                        "Tj" | "TJ" => {
                            let (x, y) = state.current_position();
                            text_count += 1;
                            if text_count <= 10 {
                                println!("[{:3}] {} at ({:.1}, {:.1})", i, op.operator, x, y);
                            }
                        }
                        "q" => {
                            state.save_graphics_state();
                            println!("[{:3}] q (save)", i);
                        }
                        "Q" => {
                            state.restore_graphics_state();
                            println!("[{:3}] Q (restore)", i);
                        }
                        "cm" => {
                            if op.operands.len() >= 6 {
                                let ops: Vec<f32> = op.operands.iter()
                                    .take(6)
                                    .filter_map(|o| o.as_f32().ok())
                                    .collect();
                                if ops.len() == 6 {
                                    state.cm(ops[0], ops[1], ops[2], ops[3], ops[4], ops[5]);
                                    println!("[{:3}] cm [{:.1},{:.1},{:.1},{:.1},{:.1},{:.1}]",
                                        i, ops[0], ops[1], ops[2], ops[3], ops[4], ops[5]);
                                }
                            }
                        }
                        _ => {}
                    }
                    
                    if text_count >= 10 {
                        println!("\n... (stopped after 10 text operators)");
                        break;
                    }
                }
            }
        }
    }
}
