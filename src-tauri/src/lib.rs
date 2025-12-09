use goidev_core::dto::ReflowDocument;
use goidev_core::markdown::{
    MarkdownMeta, hash_file, is_cache_valid, load_markdown, save_markdown, sidecar_path,
};
use goidev_core::nlp_engine;
use goidev_core::pdf_parser::parse_pdf;
use goidev_core::reflow_engine::ReflowEngine;
use goidev_core::storage_layer;
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
            println!(
                "[open_document] Loaded {} blocks from markdown",
                blocks.len()
            );
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
                println!(
                    "[open_document] Loaded {} blocks from cache",
                    cached_blocks.len()
                );
                cached_blocks
            } else {
                // Cache miss: parse PDF, reflow, save sidecar
                println!("[open_document] Cache miss - parsing PDF");

                let lines = match parse_pdf(&path) {
                    Ok(lines) => {
                        println!("[open_document] Parsed {} text lines", lines.len());
                        lines
                    }
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
                    Err(e) => eprintln!(
                        "[open_document] Warning: failed to write sidecar cache: {}",
                        e
                    ),
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

    println!(
        "[open_document] Creating ReflowDocument with {} blocks",
        blocks.len()
    );
    let doc = ReflowDocument {
        doc_id: uuid::Uuid::new_v4().to_string(),
        title,
        blocks,
    };

    println!("[open_document] SUCCESS - returning document");
    Ok(doc)
}

fn vocabulary_db_path() -> std::path::PathBuf {
    if let Some(mut dir) = dirs::data_local_dir() {
        dir.push("goidev");
        std::fs::create_dir_all(&dir).ok();
        dir.push("vocab.db");
        dir
    } else {
        std::path::PathBuf::from("./goidev_vocab.db")
    }
}

#[derive(serde::Deserialize)]
struct CaptureWordRequest {
    pub word: String,
    pub block_text: String,
    pub doc_id: String,
    pub page_num: u32,
}

#[derive(serde::Serialize)]
struct CaptureWordResponse {
    pub success: bool,
    pub entry: Option<storage_layer::WordEntry>,
    pub error: Option<String>,
}

/// Capture a selected word from the UI, extract sentence context and base form, and persist to SQLite.
#[tauri::command]
fn capture_word(request: CaptureWordRequest) -> Result<CaptureWordResponse, String> {
    println!("[capture_word] Processing: '{}'", request.word);
    println!(
        "[capture_word] request: {} (doc {}, page {})",
        request.word, request.doc_id, request.page_num
    );

    // Extract sentence context
    let sentence = nlp_engine::sentence_for_word(&request.block_text, &request.word)
        .unwrap_or_else(|| {
             println!("[capture_word] Warning: Exact sentence match failed for '{}', using block text.", request.word);
             request.block_text.clone()
        });

    // Base-form (stem) extraction
    let base_form = nlp_engine::get_base_form(&request.word);

    let db_path = vocabulary_db_path();

    // Open / initialize DB
    let conn = match storage_layer::init_db(db_path.to_str().unwrap_or("./goidev_vocab.db")) {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("failed to open db: {}", e);
            println!("[capture_word] {}", msg);
            return Ok(CaptureWordResponse {
                success: false,
                entry: None,
                error: Some(msg),
            });
        }
    };

    // Build WordEntry and persist
    let entry = storage_layer::WordEntry {
        id: None,
        word: request.word.clone(),
        base_form: base_form.clone(),
        sentence: sentence.clone(),
        source_doc: Some(request.doc_id.clone()),
        page_num: Some(request.page_num),
        created_at: 0,
        review_count: 0,
        next_review: None,
        ease_factor: 2.5,
    };

    match storage_layer::save_word(&conn, entry) {
        Ok(saved) => Ok(CaptureWordResponse {
            success: true,
            entry: Some(saved),
            error: None,
        }),
        Err(e) => {
            let msg = format!("failed to save word: {}", e);
            println!("[capture_word] {}", msg);
            Ok(CaptureWordResponse {
                success: false,
                entry: None,
                error: Some(msg),
            })
        }
    }
}

#[tauri::command]
fn get_vocabulary() -> Result<Vec<storage_layer::WordEntry>, String> {
    let db_path = vocabulary_db_path();
    let conn = storage_layer::init_db(db_path.to_str().unwrap_or("./goidev_vocab.db"))
        .map_err(|e| format!("failed to open db: {}", e))?;
    storage_layer::get_vocabulary(&conn).map_err(|e| format!("failed to load vocabulary: {}", e))
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
            select_file,
            capture_word,
            get_vocabulary
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
