use crate::components::reflow_viewer::ReflowViewer;
use goidev_core::dto::ReflowDocument;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

// Tauri dialog API
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "dialog"], js_name = open, catch)]
    async fn dialog_open(options: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize, Deserialize)]
struct OpenDocumentArgs<'a> {
    path: &'a str,
}

#[derive(Serialize)]
struct DialogFilter {
    name: &'static str,
    extensions: Vec<&'static str>,
}

#[derive(Serialize)]
struct DialogOptions {
    multiple: bool,
    directory: bool,
    filters: Vec<DialogFilter>,
    title: &'static str,
}

#[derive(Clone, PartialEq)]
enum LoadingState {
    Idle,
    Loading,
    Ready,
}

#[component]
pub fn App() -> impl IntoView {
    let (path, set_path) = signal(String::new());
    let (document, set_document) = signal(Option::<ReflowDocument>::None);
    let (error_msg, set_error_msg) = signal(String::new());
    let (loading_state, set_loading_state) = signal(LoadingState::Idle);
    let (status_msg, set_status_msg) = signal(String::new());

    let update_path = move |ev| {
        let v = event_target_value(&ev);
        set_path.set(v);
    };

    // Open file using the provided path
    let open_file = move |file_path: String| {
        spawn_local(async move {
            if file_path.is_empty() {
                return;
            }

            // Clear previous state and show loading
            set_error_msg.set(String::new());
            set_document.set(None);
            set_loading_state.set(LoadingState::Loading);
            set_status_msg.set(format!("Opening {}...", file_path));

            let args = serde_wasm_bindgen::to_value(&OpenDocumentArgs { path: &file_path }).unwrap();

            match invoke("open_document", args).await {
                Ok(val) => match serde_wasm_bindgen::from_value::<ReflowDocument>(val) {
                    Ok(doc) => {
                        set_status_msg.set(format!("Loaded: {} ({} blocks)", doc.title, doc.blocks.len()));
                        set_document.set(Some(doc));
                        set_loading_state.set(LoadingState::Ready);
                    }
                    Err(e) => {
                        set_error_msg.set(format!("Failed to parse result: {:?}", e));
                        set_loading_state.set(LoadingState::Idle);
                        set_status_msg.set(String::new());
                    }
                },
                Err(e) => {
                    set_error_msg.set(format!(
                        "Error opening document: {}",
                        e.as_string().unwrap_or_else(|| "Unknown error".to_string())
                    ));
                    set_loading_state.set(LoadingState::Idle);
                    set_status_msg.set(String::new());
                }
            }
        });
    };

    // Form submit handler
    let open_pdf = move |ev: SubmitEvent| {
        ev.prevent_default();
        let path_val = path.get_untracked();
        open_file(path_val);
    };

    // File picker button handler
    let browse_file = move |_| {
        spawn_local(async move {
            let options = DialogOptions {
                multiple: false,
                directory: false,
                filters: vec![
                    DialogFilter {
                        name: "Documents",
                        extensions: vec!["pdf", "md", "markdown"],
                    },
                    DialogFilter {
                        name: "PDF Files",
                        extensions: vec!["pdf"],
                    },
                    DialogFilter {
                        name: "Markdown Files",
                        extensions: vec!["md", "markdown"],
                    },
                ],
                title: "Select a PDF or Markdown file",
            };

            let options_js = serde_wasm_bindgen::to_value(&options).unwrap();
            
            match dialog_open(options_js).await {
                Ok(result) => {
                    // Result can be a string (single file) or null (cancelled)
                    if let Some(file_path) = result.as_string() {
                        set_path.set(file_path.clone());
                        // Automatically open the selected file
                        open_file(file_path);
                    }
                }
                Err(e) => {
                    set_error_msg.set(format!(
                        "Error opening file dialog: {}",
                        e.as_string().unwrap_or_else(|| "Unknown error".to_string())
                    ));
                }
            }
        });
    };

    view! {
        <main class="container">
            <h1>"GOIDEV PDF Reader"</h1>

            <div class="file-selection">
                <form class="row" on:submit=open_pdf>
                    <input
                        id="path-input"
                        placeholder="Enter path to PDF or Markdown file..."
                        prop:value=move || path.get()
                        on:input=update_path
                        prop:disabled=move || loading_state.get() == LoadingState::Loading
                    />
                    <button
                        type="submit"
                        prop:disabled=move || loading_state.get() == LoadingState::Loading
                    >
                        {move || {
                            if loading_state.get() == LoadingState::Loading {
                                "Loading..."
                            } else {
                                "Open"
                            }
                        }}
                    </button>
                </form>
                
                <button
                    class="browse-button"
                    on:click=browse_file
                    prop:disabled=move || loading_state.get() == LoadingState::Loading
                >
                    "Browse..."
                </button>
            </div>

            <p class="status">{ move || status_msg.get() }</p>
            <p class="error">{ move || error_msg.get() }</p>

            {move || {
                if loading_state.get() == LoadingState::Loading {
                    Some(view! {
                        <div class="loading-indicator" style="text-align: center; padding: 40px; color: #666;">
                            <p>"Parsing PDF and creating markdown cache..."</p>
                            <p style="font-size: 0.9em;">"This may take a moment for the first time."</p>
                        </div>
                    })
                } else {
                    None
                }
            }}

            {move || document.get().map(|doc| view! { <ReflowViewer document=doc/> })}
        </main>
    }
}
