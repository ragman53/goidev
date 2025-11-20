use crate::reflow_engine::Block;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflowDocument {
    pub doc_id: String,
    pub title: String,
    pub blocks: Vec<Block>,
}
