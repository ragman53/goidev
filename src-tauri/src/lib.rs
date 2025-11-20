use goidev_core::dto::ReflowDocument;
use goidev_core::pdf_parser::parse_pdf;
use goidev_core::reflow_engine::ReflowEngine;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn open_document(path: String) -> Result<ReflowDocument, String> {
    let lines = parse_pdf(&path).map_err(|e| e.to_string())?;
    let blocks = ReflowEngine::process(lines);

    // Extract filename as title for now
    let title = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Untitled")
        .to_string();

    Ok(ReflowDocument {
        doc_id: uuid::Uuid::new_v4().to_string(),
        title,
        blocks,
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, open_document])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
