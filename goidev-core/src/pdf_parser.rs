use lopdf::{Document, Object, ObjectId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::font_utils::{FontEncoding, parse_font_encoding};
use crate::pdf_state::PdfState;

/// Helper to convert a lopdf Object to f32, handling both Integer and Real types.
fn object_to_f32(obj: &Object) -> Option<f32> {
    match obj {
        Object::Real(f) => Some(*f),
        Object::Integer(i) => Some(*i as f32),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

/// Page geometry extracted from PDF MediaBox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PageGeometry {
    pub width: f32,
    pub height: f32,
    pub origin_x: f32,
    pub origin_y: f32,
}

impl PageGeometry {
    /// Returns the relative Y position (0.0 = bottom, 1.0 = top) for Y-up coordinate system.
    pub fn relative_y(&self, y: f32) -> f32 {
        if self.height > 0.0 {
            (y - self.origin_y) / self.height
        } else {
            0.5
        }
    }

    /// Check if Y position is in header zone (top 8% of page).
    pub fn is_header_zone(&self, y: f32) -> bool {
        self.relative_y(y) > 0.92
    }

    /// Check if Y position is in footer zone (bottom 8% of page).
    pub fn is_footer_zone(&self, y: f32) -> bool {
        self.relative_y(y) < 0.08
    }
}

/// Extract page geometry (MediaBox) from a PDF page.
/// Falls back to default Letter size (612x792) if MediaBox is not found.
fn extract_page_geometry(doc: &Document, page_id: ObjectId) -> PageGeometry {
    // Try to get the page dictionary
    let page_dict = match doc.get_object(page_id).and_then(|o| o.as_dict()) {
        Ok(dict) => dict,
        Err(_) => return PageGeometry::default(),
    };

    // Try MediaBox directly on page, then walk up parent chain
    let media_box = get_media_box(doc, page_dict);

    // MediaBox format: [origin_x, origin_y, width, height] or [x1, y1, x2, y2]
    if let Some(arr) = media_box {
        if arr.len() >= 4 {
            let values: Vec<f32> = arr
                .iter()
                .filter_map(|o| match o {
                    Object::Integer(i) => Some(*i as f32),
                    Object::Real(f) => Some(*f),
                    _ => None,
                })
                .collect();

            if values.len() >= 4 {
                return PageGeometry {
                    origin_x: values[0],
                    origin_y: values[1],
                    width: values[2] - values[0],
                    height: values[3] - values[1],
                };
            }
        }
    }

    // Default to US Letter size
    PageGeometry {
        origin_x: 0.0,
        origin_y: 0.0,
        width: 612.0,
        height: 792.0,
    }
}

/// Recursively search for MediaBox in page dictionary or parent.
fn get_media_box<'a>(doc: &'a Document, page_dict: &'a lopdf::Dictionary) -> Option<&'a Vec<Object>> {
    // Check for MediaBox on this page
    if let Ok(media_box) = page_dict.get(b"MediaBox") {
        match media_box {
            Object::Array(arr) => return Some(arr),
            Object::Reference(id) => {
                if let Ok(Object::Array(arr)) = doc.get_object(*id) {
                    return Some(arr);
                }
            }
            _ => {}
        }
    }

    // Walk up to parent
    if let Ok(parent_ref) = page_dict.get(b"Parent") {
        if let Object::Reference(parent_id) = parent_ref {
            if let Ok(Object::Dictionary(parent_dict)) = doc.get_object(*parent_id) {
                return get_media_box(doc, parent_dict);
            }
        }
    }

    None
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextLine {
    pub text: String,
    pub bbox: BBox,
    pub font_size: f32,
    pub page_num: u32,
    /// Page geometry for position-based classification.
    pub page_geometry: PageGeometry,
}

/// Parses a PDF file and extracts text lines with their positions.
pub fn parse_pdf(path: &str) -> Result<Vec<TextLine>, String> {
    let doc = Document::load(path).map_err(|e| format!("Failed to load PDF: {}", e))?;
    let mut text_lines = Vec::new();
    let mut font_map: HashMap<Vec<u8>, FontEncoding> = HashMap::new();
    let mut state = PdfState::new();

    for (page_num, page_id) in doc.get_pages() {
        // Reset state for each new page
        state.reset_for_page();
        
        // Extract page geometry from MediaBox
        let page_geometry = extract_page_geometry(&doc, page_id);

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
                        if let Some(f) = object_to_f32(size) {
                            current_font_size = f;
                        }
                    }
                }
                "Td" => {
                    if let (Some(x), Some(y)) =
                        (operation.operands.get(0), operation.operands.get(1))
                    {
                        if let (Some(dx), Some(dy)) = (object_to_f32(x), object_to_f32(y)) {
                            state.td(dx, dy);
                        }
                    }
                }
                "TD" => {
                    if let (Some(x), Some(y)) =
                        (operation.operands.get(0), operation.operands.get(1))
                    {
                        if let (Some(dx), Some(dy)) = (object_to_f32(x), object_to_f32(y)) {
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
                            .filter_map(|o| object_to_f32(o))
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
                            .filter_map(|o| object_to_f32(o))
                            .collect();
                        if ops.len() == 6 {
                            state.cm(ops[0], ops[1], ops[2], ops[3], ops[4], ops[5]);
                        }
                    }
                }
                // Graphics state operators
                "q" => state.save_graphics_state(),
                "Q" => state.restore_graphics_state(),
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
                        // Get effective font size (Tf size × text matrix scale)
                        let effective_font_size = current_font_size * state.text_scale();
                        // Simplified width calculation: char_count * font_size * 0.5 (approx aspect ratio)
                        let width = text_str.len() as f32 * effective_font_size * 0.5;
                        let height = effective_font_size;

                        text_lines.push(TextLine {
                            text: text_str,
                            bbox: BBox {
                                x1: x,
                                y1: y,
                                x2: x + width,
                                y2: y + height,
                            },
                            font_size: effective_font_size,
                            page_num,
                            page_geometry: page_geometry.clone(),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    Ok(text_lines)
}
