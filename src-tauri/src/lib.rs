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
/// 
/// This runs synchronously on a blocking thread pool to avoid issues with panics.
#[tauri::command]
fn open_document(path: String) -> Result<ReflowDocument, String> {
    println!("[open_document] START: {}", path);
    
    let p = Path::new(&path);
    
    // Validate file exists
    if !p.exists() {
        println!("[open_document] ERROR: File not found");
        return Err(format!("File not found: {}", path));
    }
    
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("");
    println!("[open_document] Extension: {}", ext);

    let (blocks, title) = match ext.to_lowercase().as_str() {
        "md" | "markdown" => {
            println!("[open_document] Loading markdown file");
            // Direct Markdown load (lenient import)
            let (blocks, _meta) = load_markdown(&path).map_err(|e| {
                println!("[open_document] ERROR loading markdown: {}", e);
                format!("Failed to load markdown: {}", e)
            })?;
            let title = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string();
            println!("[open_document] Loaded {} blocks from markdown", blocks.len());
            (blocks, title)
        }
        "pdf" => {
            // PDF file handling
            let sidecar = sidecar_path(&path);
            println!("[open_document] Sidecar path: {:?}", sidecar);

            let blocks = if sidecar.exists() && is_cache_valid(p, &sidecar) {
                // Cache hit - load from sidecar
                println!("[open_document] Cache hit - loading from sidecar");
                let (cached_blocks, _) = load_markdown(&sidecar).map_err(|e| {
                    println!("[open_document] ERROR loading cache: {}", e);
                    format!("Failed to load cache: {}", e)
                })?;
                println!("[open_document] Loaded {} blocks from cache", cached_blocks.len());
                cached_blocks
            } else {
                // Cache miss: parse PDF, reflow, save sidecar
                println!("[open_document] Cache miss - parsing PDF");
                
                let lines = match parse_pdf(&path) {
                    Ok(lines) => {
                        println!("[open_document] Parsed {} text lines", lines.len());
                        lines
                    },
                    Err(e) => {
                        println!("[open_document] ERROR parsing PDF: {}", e);
                        return Err(format!("Failed to parse PDF: {}", e));
                    }
                };
                
                println!("[open_document] Processing lines into blocks...");
                let fresh_blocks = ReflowEngine::process(lines);
                println!("[open_document] Generated {} blocks", fresh_blocks.len());

                // Compute source hash and save sidecar
                println!("[open_document] Computing hash and saving cache...");
                let source_hash = hash_file(&path).ok();
                let meta = MarkdownMeta { source_hash };
                
                match save_markdown(&fresh_blocks, &meta, &sidecar) {
                    Ok(_) => println!("[open_document] Saved cache to: {:?}", sidecar),
                    Err(e) => eprintln!("[open_document] Warning: failed to write sidecar cache: {}", e),
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
        _ => {
            println!("[open_document] ERROR: Unsupported file type");
            return Err(format!("Unsupported file type: .{}", ext));
        }
    };

    println!("[open_document] Creating ReflowDocument with {} blocks", blocks.len());
    let doc = ReflowDocument {
        doc_id: uuid::Uuid::new_v4().to_string(),
        title,
        blocks,
    };
    
    println!("[open_document] SUCCESS - returning document");
    Ok(doc)
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

/// Select a file using the native file dialog.
/// Returns the selected file path, or None if cancelled.
#[tauri::command]
async fn select_file() -> Option<String> {
    // Dialog is handled via JS API in the frontend
    None
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            open_document,
            save_document_markdown,
            select_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
