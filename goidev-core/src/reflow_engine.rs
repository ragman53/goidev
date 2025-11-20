use crate::pdf_parser::{BBox, TextLine};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BlockRole {
    Paragraph,
    Heading,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub text: String,
    pub bbox: BBox,
    pub role: BlockRole,
}

pub struct ReflowEngine;

impl ReflowEngine {
    pub fn process(lines: Vec<TextLine>) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();

        for line in lines {
            if let Some(last_block) = blocks.last_mut() {
                if Self::should_merge(last_block, &line) {
                    // Add space if needed (simple heuristic: if not ending with hyphen or whitespace)
                    if !last_block.text.trim_end().ends_with('-')
                        && !last_block.text.ends_with(char::is_whitespace)
                    {
                        last_block.text.push(' ');
                    }
                    last_block.text.push_str(&line.text);

                    // Update bbox to include new line
                    last_block.bbox.x1 = last_block.bbox.x1.min(line.bbox.x1);
                    last_block.bbox.y1 = last_block.bbox.y1.min(line.bbox.y1);
                    last_block.bbox.x2 = last_block.bbox.x2.max(line.bbox.x2);
                    last_block.bbox.y2 = last_block.bbox.y2.max(line.bbox.y2);
                    continue;
                }
            }

            let role = if line.font_size > 14.0 {
                BlockRole::Heading
            } else {
                BlockRole::Paragraph
            };

            blocks.push(Block {
                text: line.text,
                bbox: line.bbox,
                role,
            });
        }

        blocks
    }

    fn should_merge(block: &Block, line: &TextLine) -> bool {
        // 1. Check Role (Font Size)
        let role = if line.font_size > 14.0 {
            BlockRole::Heading
        } else {
            BlockRole::Paragraph
        };

        if block.role != role {
            return false;
        }

        // 2. Check Vertical/Horizontal proximity
        // Assuming Y grows upwards (PDF standard)

        // Horizontal merge (same line)
        // Check if Y ranges overlap significantly
        let y_overlap = block.bbox.y1.max(line.bbox.y1) < block.bbox.y2.min(line.bbox.y2);
        if y_overlap {
            return true;
        }

        // Vertical merge (next line in paragraph)
        // In the coordinate system we're using (Y increases downward or acts like screen coords):
        // - block.y1 is the TOP of the previous block
        // - block.y2 is the BOTTOM of the previous block
        // - line.y1 is the TOP of the current line
        // - line.y2 is the BOTTOM of the current line
        // Gap = Line TOP - Block BOTTOM
        let vertical_gap = line.bbox.y1 - block.bbox.y2;

        // Allow some gap (e.g. up to 4.0 * font size for PDFs with larger line spacing)
        // Negative gap means overlap, which we allow up to 5 units
        if vertical_gap >= -5.0 && vertical_gap < line.font_size * 4.0 {
            return true;
        }

        false
    }
}
