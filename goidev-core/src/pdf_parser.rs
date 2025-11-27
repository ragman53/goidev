use lopdf::{Document, Object};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::font_utils::{FontEncoding, parse_font_encoding};
use crate::pdf_state::PdfState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLine {
    pub text: String,
    pub bbox: BBox,
    pub font_size: f32,
    pub page_num: u32,
}

/// Parses a PDF file and extracts text lines with their positions.
pub fn parse_pdf(path: &str) -> Result<Vec<TextLine>, String> {
    let doc = Document::load(path).map_err(|e| format!("Failed to load PDF: {}", e))?;
    let mut text_lines = Vec::new();
    let mut font_map: HashMap<Vec<u8>, FontEncoding> = HashMap::new();

    for (page_num, page_id) in doc.get_pages() {
        let content_data = doc
            .get_page_content(page_id)
            .map_err(|e| format!("Failed to get content for page {}: {}", page_num, e))?;
        let content = lopdf::content::Content::decode(&content_data)
            .map_err(|e| format!("Failed to decode content for page {}: {}", page_num, e))?;

        // Parse page fonts to build encoding maps
        if let Ok((Some(resources), _)) = doc.get_page_resources(page_id) {
            if let Ok(fonts) = resources.get(b"Font").and_then(|o| o.as_dict()) {
                for (name, obj) in fonts.iter() {
                    let font_dict = match obj {
                        Object::Reference(id) => doc.get_object(*id).and_then(|o| o.as_dict()).ok(),
                        Object::Dictionary(dict) => Some(dict),
                        _ => None,
                    };

                    if let Some(dict) = font_dict {
                        let mut encoding = parse_font_encoding(dict);

                        if let Ok(to_unicode) = dict.get(b"ToUnicode") {
                            let stream_obj = match to_unicode {
                                Object::Reference(id) => doc.get_object(*id).ok(),
                                Object::Stream(_) => Some(to_unicode),
                                _ => None,
                            };
                            if let Some(Object::Stream(stream)) = stream_obj {
                                if let Ok(content) = stream.decompressed_content() {
                                    encoding.apply_to_unicode(&content);
                                }
                            }
                        }

                        if encoding.map.is_empty() {
                            crate::font_utils::populate_win_ansi(&mut encoding.map);
                        }
                        font_map.insert(name.clone(), encoding);
                    }
                }
            }
        }

        let mut state = PdfState::new();
        let mut current_font_size = 12.0; // Default
        let mut current_font_name = Vec::new();
        let default_encoding = FontEncoding::new(); // Fallback

        for operation in content.operations.iter() {
            match operation.operator.as_str() {
                "BT" => state.bt(),
                "ET" => state.et(),
                "Tf" => {
                    if let Some(name_obj) = operation.operands.get(0) {
                        if let Ok(name) = name_obj.as_name() {
                            current_font_name = name.to_vec();
                        }
                    }
                    if let Some(size) = operation.operands.get(1) {
                        if let Ok(f) = size.as_f32() {
                            current_font_size = f;
                        }
                    }
                }
                "Td" => {
                    if let (Some(x), Some(y)) =
                        (operation.operands.get(0), operation.operands.get(1))
                    {
                        if let (Ok(dx), Ok(dy)) = (x.as_f32(), y.as_f32()) {
                            state.td(dx, dy);
                        }
                    }
                }
                "TD" => {
                    if let (Some(x), Some(y)) =
                        (operation.operands.get(0), operation.operands.get(1))
                    {
                        if let (Ok(dx), Ok(dy)) = (x.as_f32(), y.as_f32()) {
                            state.td_capital(dx, dy);
                        }
                    }
                }
                "Tm" => {
                    if operation.operands.len() >= 6 {
                        let ops: Vec<f32> = operation
                            .operands
                            .iter()
                            .take(6)
                            .filter_map(|o| o.as_f32().ok())
                            .collect();
                        if ops.len() == 6 {
                            state.tm(ops[0], ops[1], ops[2], ops[3], ops[4], ops[5]);
                        }
                    }
                }
                "T*" => state.t_star(),
                "cm" => {
                    if operation.operands.len() >= 6 {
                        let ops: Vec<f32> = operation
                            .operands
                            .iter()
                            .take(6)
                            .filter_map(|o| o.as_f32().ok())
                            .collect();
                        if ops.len() == 6 {
                            state.cm(ops[0], ops[1], ops[2], ops[3], ops[4], ops[5]);
                        }
                    }
                }
                "Tj" | "TJ" => {
                    let encoding = font_map
                        .get(&current_font_name)
                        .unwrap_or(&default_encoding);

                    // Extract text string
                    let text_str = if operation.operator == "Tj" {
                        operation
                            .operands
                            .get(0)
                            .and_then(|o| o.as_str().ok())
                            .map(|bytes| encoding.decode(bytes))
                            .unwrap_or_default()
                    } else {
                        // TJ is an array of strings and numbers (kerning)
                        operation
                            .operands
                            .get(0)
                            .and_then(|o| o.as_array().ok())
                            .map(|arr| {
                                let mut text = String::new();
                                for op in arr {
                                    match op {
                                        lopdf::Object::String(bytes, _) => {
                                            text.push_str(&encoding.decode(bytes));
                                        }
                                        lopdf::Object::Integer(i) => {
                                            if *i < -100 {
                                                text.push(' ');
                                            }
                                        }
                                        lopdf::Object::Real(f) => {
                                            if *f < -100.0 {
                                                text.push(' ');
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                                text
                            })
                            .unwrap_or_default()
                    };

                    if !text_str.trim().is_empty() {
                        let (x, y) = state.current_position();
                        // Simplified width calculation: char_count * font_size * 0.5 (approx aspect ratio)
                        let width = text_str.len() as f32 * current_font_size * 0.5;
                        let height = current_font_size;

                        text_lines.push(TextLine {
                            text: text_str,
                            bbox: BBox {
                                x1: x,
                                y1: y,
                                x2: x + width,
                                y2: y + height,
                            },
                            font_size: current_font_size,
                            page_num,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    Ok(text_lines)
}
