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
/// For now, it's a placeholder that returns an empty Vec.
pub fn parse_pdf(_path: &str) -> Result<Vec<TextLine>, String> {
    // TODO: Implement the actual PDF parsing logic.
    // This will involve iterating through pages and their content streams.
    Ok(vec![])
}
