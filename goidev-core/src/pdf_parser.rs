use lopdf::Document;
use serde::{Deserialize, Serialize};

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
}

/// Parses a PDF file and extracts text lines with their positions.
pub fn parse_pdf(path: &str) -> Result<Vec<TextLine>, String> {
    let doc = Document::load(path).map_err(|e| format!("Failed to load PDF: {}", e))?;
    let mut text_lines = Vec::new();

    for (page_num, page_id) in doc.get_pages() {
        let content_data = doc
            .get_page_content(page_id)
            .map_err(|e| format!("Failed to get content for page {}: {}", page_num, e))?;
        let content = lopdf::content::Content::decode(&content_data)
            .map_err(|e| format!("Failed to decode content for page {}: {}", page_num, e))?;

        let mut current_font_size = 12.0; // Default
        let mut current_x = 0.0;
        let mut current_y = 0.0;

        for operation in content.operations.iter() {
            match operation.operator.as_str() {
                "Tf" => {
                    if let Some(size) = operation.operands.get(1) {
                        if let Ok(f) = size.as_f32() {
                            current_font_size = f;
                        }
                    }
                }
                "Td" | "TD" => {
                    if let (Some(x), Some(y)) =
                        (operation.operands.get(0), operation.operands.get(1))
                    {
                        if let (Ok(dx), Ok(dy)) = (x.as_f32(), y.as_f32()) {
                            current_x += dx;
                            current_y += dy;
                        }
                    }
                }
                "Tm" => {
                    if let (Some(x), Some(y)) =
                        (operation.operands.get(4), operation.operands.get(5))
                    {
                        if let (Ok(new_x), Ok(new_y)) = (x.as_f32(), y.as_f32()) {
                            current_x = new_x;
                            current_y = new_y;
                        }
                    }
                }
                "Tj" | "TJ" => {
                    // Extract text string
                    let text_str = if operation.operator == "Tj" {
                        operation
                            .operands
                            .get(0)
                            .and_then(|o| o.as_str().ok())
                            .map(|bytes| String::from_utf8_lossy(bytes).to_string())
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
                                            text.push_str(&String::from_utf8_lossy(bytes));
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
                        // Simplified width calculation: char_count * font_size * 0.5 (approx aspect ratio)
                        let width = text_str.len() as f32 * current_font_size * 0.5;
                        let height = current_font_size;

                        text_lines.push(TextLine {
                            text: text_str,
                            bbox: BBox {
                                x1: current_x,
                                y1: current_y,
                                x2: current_x + width,
                                y2: current_y + height,
                            },
                            font_size: current_font_size,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    Ok(text_lines)
}
