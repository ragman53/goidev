use goidev_core::dto::ReflowDocument;
use goidev_core::reflow_engine::{Block, BlockRole};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use std::collections::BTreeMap;

/// Metadata for a page group
struct PageGroup {
    pdf_page: u32,
    doc_page: Option<String>,
    blocks: Vec<Block>,
}

#[component]
pub fn ReflowViewer(document: ReflowDocument) -> impl IntoView {
    // Signal for showing capture feedback
    let (captured_word, set_captured_word) = signal(Option::<String>::None);

    // Group blocks by page, capturing doc_page_num from the first block that has it
    let pages: Vec<PageGroup> = {
        let mut map: BTreeMap<u32, PageGroup> = BTreeMap::new();
        for block in document.blocks {
            let entry = map.entry(block.page_num).or_insert_with(|| PageGroup {
                pdf_page: block.page_num,
                doc_page: None,
                blocks: Vec::new(),
            });
            // Capture doc_page_num from first block that has it
            if entry.doc_page.is_none() && block.doc_page_num.is_some() {
                entry.doc_page = block.doc_page_num.clone();
            }
            entry.blocks.push(block);
        }
        map.into_values().collect()
    };

    let doc_id = document.doc_id.clone();

    view! {
        <div class="reflow-viewer">
            <h1>{document.title}</h1>
            // Capture feedback toast
            {move || captured_word.get().map(|word| view! {
                <div class="capture-toast" style="position: fixed; bottom: 20px; right: 20px; background: #4CAF50; color: white; padding: 12px 20px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.2); z-index: 1000; animation: fadeIn 0.3s;">
                    {"✓ Captured: "}{word}
                </div>
            })}
            <div class="pages">
                {pages.into_iter().map(|page_group| {
                    let bg_style = if page_group.pdf_page % 2 != 0 {
                        "background-color: #ffffff; color: #333333; padding: 20px; margin-bottom: 20px; position: relative; border: 1px solid #eee;"
                    } else {
                        "background-color: #f8f9fa; color: #333333; padding: 20px; margin-bottom: 20px; position: relative; border: 1px solid #eee;"
                    };

                    // Show doc page number if available, otherwise PDF page
                    let page_label = match &page_group.doc_page {
                        Some(doc_num) => format!("Page {}", doc_num),
                        None => format!("Page {}", page_group.pdf_page),
                    };

                    view! {
                        <div class="page-container" style=bg_style>
                            <div class="page-number" style="position: absolute; top: 5px; right: 10px; color: #888; font-size: 0.8em; font-weight: bold;">
                                {page_label}
                            </div>
                            {page_group.blocks.into_iter().map(|block| {
                                render_block(block, doc_id.clone(), set_captured_word)
                            }).collect_view()}
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Render a block based on its role.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
struct CaptureWordRequest {
    word: String,
    block_text: String,
    doc_id: String,
    page_num: u32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct CaptureWordResponse {
    success: bool,
    error: Option<String>,
}

/// Helper to create the capture word handler
fn make_capture_handler(
    doc_id: String,
    block_text: String,
    page_num: u32,
    set_captured: WriteSignal<Option<String>>,
) -> impl Fn() + Clone + 'static {
    move || {
        let doc = doc_id.clone();
        let text = block_text.clone();
        let set_word = set_captured;
        spawn_local(async move {
            let selected = web_sys::window()
                .and_then(|w| w.get_selection().ok().flatten())
                .map(|s| s.to_string())
                .map(|js| js.as_string().unwrap_or_default())
                .unwrap_or_default();
            
            if selected.trim().is_empty() {
                return;
            }
            
            let req = CaptureWordRequest {
                word: selected.clone(),
                block_text: text,
                doc_id: doc,
                page_num,
            };
            let args = to_value(&req).unwrap_or(JsValue::NULL);
            
            match invoke("capture_word", args).await {
                Ok(result) => {
                    if let Ok(resp) = serde_wasm_bindgen::from_value::<CaptureWordResponse>(result) {
                        if resp.success {
                            set_word.set(Some(selected.clone()));
                            // Clear after 2 seconds
                            spawn_local(async move {
                                gloo_timers::future::TimeoutFuture::new(2000).await;
                                set_word.set(None);
                            });
                        }
                    }
                }
                Err(_) => {}
            }
        });
    }
}

fn render_block(block: Block, doc_id: String, set_captured: WriteSignal<Option<String>>) -> AnyView {
    // Add extra margin-top for paragraphs that start a new indented paragraph
    let paragraph_indent = if block.starts_new_paragraph {
        "text-indent: 1.5em;"
    } else {
        ""
    };

    let page_num = block.page_num;
    let block_text = block.text.clone();

    match block.role {
        BlockRole::Heading { level } => {
            let style = match level {
                1 => "font-size: 1.5em; font-weight: bold; margin: 20px 0 15px 0; line-height: 1.3; cursor: text;",
                2 => "font-size: 1.25em; font-weight: bold; margin: 15px 0 10px 0; line-height: 1.3; cursor: text;",
                _ => "font-size: 1.1em; font-weight: bold; margin: 10px 0 8px 0; line-height: 1.3; cursor: text;",
            };
            let handler = make_capture_handler(doc_id, block_text, page_num, set_captured);
            view! {
                <div style=style on:dblclick=move |_| handler()>{block.text}</div>
            }.into_any()
        },

        BlockRole::Paragraph => {
            let style = format!("margin: 0 0 10px 0; line-height: 1.6; cursor: text; {}", paragraph_indent);
            let handler = make_capture_handler(doc_id, block_text, page_num, set_captured);
            view! {
                <p style=style on:dblclick=move |_| handler()>{block.text}</p>
            }.into_any()
        },

        BlockRole::PageNumber | BlockRole::Header | BlockRole::Footer => {
            // Skip header/footer/page numbers in reflow view (already shown)
            view! { <span></span> }.into_any()
        },

        BlockRole::Footnote => {
            let handler = make_capture_handler(doc_id, block_text, page_num, set_captured);
            view! {
                <div style="font-size: 0.85em; color: #666; border-top: 1px solid #ddd; padding-top: 5px; margin-top: 15px; cursor: text;"
                     on:dblclick=move |_| handler()>
                    {block.text}
                </div>
            }.into_any()
        },

        BlockRole::Caption => {
            let handler = make_capture_handler(doc_id, block_text, page_num, set_captured);
            view! {
                <p style="font-style: italic; text-align: center; color: #555; margin: 10px 0; cursor: text;"
                   on:dblclick=move |_| handler()>
                    {block.text}
                </p>
            }.into_any()
        },

        BlockRole::Citation => {
            let handler = make_capture_handler(doc_id, block_text, page_num, set_captured);
            view! {
                <div style="margin-left: 20px; margin-bottom: 8px; font-size: 0.9em; color: #444; cursor: text;"
                     on:dblclick=move |_| handler()>
                    {block.text}
                </div>
            }.into_any()
        },

        BlockRole::Author => {
            let handler = make_capture_handler(doc_id, block_text, page_num, set_captured);
            view! {
                <p style="font-weight: bold; text-align: center; margin-bottom: 5px; cursor: text;"
                   on:dblclick=move |_| handler()>
                    {block.text}
                </p>
            }.into_any()
        },

        BlockRole::Abstract => {
            let handler = make_capture_handler(doc_id, block_text, page_num, set_captured);
            view! {
                <blockquote style="font-style: italic; border-left: 3px solid #ccc; padding-left: 15px; margin: 15px 0; color: #555; cursor: text;"
                            on:dblclick=move |_| handler()>
                    {block.text}
                </blockquote>
            }.into_any()
        },

        BlockRole::Reference => {
            let handler = make_capture_handler(doc_id, block_text, page_num, set_captured);
            view! {
                <h3 style="margin: 20px 0 10px 0; border-bottom: 1px solid #ddd; padding-bottom: 5px; cursor: text;"
                    on:dblclick=move |_| handler()>
                    {block.text}
                </h3>
            }.into_any()
        },
    }
}
