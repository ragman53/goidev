//! This module is responsible for parsing PDF files to extract text content
//! along with its properties like position and font size. It uses the `lopdf`
//! crate to process the low-level PDF structure.

use lopdf::Document;

/// Represents a bounding box for a text chunk.
/// Coordinates are in PDF points (1/72 inch), with the origin at the bottom-left of the page.
#[derive(Debug, Clone, PartialEq)]
pub struct BBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Represents a piece of text extracted from a PDF with its associated metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct TextChunk {
    pub text: String,
    pub bbox: BBox,
    pub font_size: f32,
}

/// Parses a range of pages from a PDF document and extracts text chunks.
pub fn parse_pdf(path: &str, start_page: u32, end_page: u32) -> Result<Vec<TextChunk>, String> {
    // Load the document
    let doc = Document::load(path).map_err(|e| format!("Failed to load PDF: {}", e))?;
    
    let mut all_chunks = Vec::new();

    // Page numbers in lopdf are 1-based, which matches our funtion signature.
    for page_num in start_page..=end_page {
        // Get the text from the page using a high-level helper.
        let text = doc
            .extract_text(&[page_num])
            .map_err(|e| format!("Failed to extract text from page {}: {}", page_num, e))?;

        if !text.is_empty() {
            all_chunks.push(TextChunk {
                text,
                bbox: BBox { x: 0.0, y: 0.0, w: 0.0, h: 0.0 }, //Dummy
                font_size: 0.0, //Dummy value
            });
        }
    }
    
    Ok(all_chunks)
}