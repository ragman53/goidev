use goidev_core::dto::ReflowDocument;
use goidev_core::markdown::{
    hash_file, is_cache_valid, load_markdown, save_markdown, sidecar_path, MarkdownMeta,
};
use goidev_core::pdf_parser::parse_pdf;
use goidev_core::reflow_engine::ReflowEngine;
use std::path::Path;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

/// Open a document (PDF or Markdown).
/// - For PDFs: check sidecar cache; if valid, load from cache; else parse, reflow, cache.
/// - For Markdown: load directly.
#[tauri::command]
async fn open_document(path: String) -> Result<ReflowDocument, String> {
    let p = Path::new(&path);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");

    let (blocks, title) = match ext.to_lowercase().as_str() {
        "md" | "markdown" => {
            // Direct Markdown load (lenient import)
            let (blocks, _meta) = load_markdown(&path).map_err(|e| e.to_string())?;
            let title = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string();
            (blocks, title)
        }
        _ => {
            // Assume PDF (or other parseable format)
            let sidecar = sidecar_path(&path);

            let blocks = if sidecar.exists() && is_cache_valid(p, &sidecar) {
                // Cache hit
                let (cached_blocks, _) = load_markdown(&sidecar).map_err(|e| e.to_string())?;
                cached_blocks
            } else {
                // Cache miss: parse PDF, reflow, save sidecar
                let lines = parse_pdf(&path).map_err(|e| e.to_string())?;
                let fresh_blocks = ReflowEngine::process(lines);

                // Compute source hash and save sidecar
                let source_hash = hash_file(&path).ok();
                let meta = MarkdownMeta { source_hash };
                if let Err(e) = save_markdown(&fresh_blocks, &meta, &sidecar) {
                    eprintln!("Warning: failed to write sidecar cache: {}", e);
                }

                fresh_blocks
            };

            let title = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string();
            (blocks, title)
        }
    };

    Ok(ReflowDocument {
        doc_id: uuid::Uuid::new_v4().to_string(),
        title,
        blocks,
    })
}

/// Explicitly save current document blocks to a Markdown file.
#[tauri::command]
async fn save_document_markdown(
    blocks: Vec<goidev_core::reflow_engine::Block>,
    dest_path: String,
    source_hash: Option<String>,
) -> Result<(), String> {
    let meta = MarkdownMeta { source_hash };
    save_markdown(&blocks, &meta, &dest_path).map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            open_document,
            save_document_markdown
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
