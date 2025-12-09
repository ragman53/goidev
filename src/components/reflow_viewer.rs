use goidev_core::dto::ReflowDocument;
use goidev_core::reflow_engine::{Block, BlockRole};
#[allow(unused_imports)]
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use std::collections::BTreeMap;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

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
    let (vocab_entries, set_vocab_entries) = signal(Vec::<VocabularyEntry>::new());
    let (vocab_panel_open, set_vocab_panel_open) = signal(false);

    let toggle_vocab_panel = {
        let vocab_panel_open = vocab_panel_open.clone();
        let set_vocab_panel_open = set_vocab_panel_open.clone();
        let set_vocab_entries = set_vocab_entries.clone();
        move |_| {
            let open = vocab_panel_open.get();
            if !open {
                let set_vocab_entries = set_vocab_entries.clone();
                spawn_local(async move {
                    if let Ok(result) = invoke("get_vocabulary", JsValue::NULL).await {
                        if let Ok(entries) =
                            serde_wasm_bindgen::from_value::<Vec<VocabularyEntry>>(result)
                        {
                            set_vocab_entries.set(entries);
                        }
                    }
                });
            }
            set_vocab_panel_open.set(!open);
        }
    };

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

    let doc_id = Rc::new(document.doc_id.clone());

    view! {
        <div class="reflow-viewer">
            <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px;">
                <h1>{document.title}</h1>
                <button type="button" style="padding: 8px 16px; background: #007acc; color: white; border: none; border-radius: 6px; cursor: pointer;" on:click=toggle_vocab_panel>
                    {move || if vocab_panel_open.get() { "Hide vocabulary" } else { "Vocabulary" }}
                </button>
            </div>
            {move || {
                if vocab_panel_open.get() {
                    let entries = vocab_entries.get().clone();
                    view! {
                        <div class="vocab-panel" style="position: fixed; top: 90px; right: 20px; width: 340px; max-height: 70vh; overflow-y: auto; background: white; border: 1px solid #ddd; border-radius: 10px; box-shadow: 0 15px 40px rgba(0,0,0,0.15); padding: 16px; z-index: 1000;">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                                <strong>Vocabulary</strong>
                                <button type="button" style="background: none; border: none; color: #007acc; font-weight: 600; cursor: pointer;" on:click=move |_| set_vocab_panel_open.set(false)>{"Close"}</button>
                            </div>
                            {entries.into_iter().map(|entry| {
                                let location = match (entry.source_doc.clone(), entry.page_num) {
                                    (Some(doc), Some(page)) => format!("{} · page {}", doc, page),
                                    (Some(doc), None) => doc,
                                    (None, Some(page)) => format!("Page {}", page),
                                    (None, None) => "Unknown source".to_string(),
                                };
                                view! {
                                    <div style="border-bottom: 1px solid #f0f0f0; padding: 8px 0;">
                                        <div style="font-weight: 600; font-size: 1rem;">{entry.word.clone()}</div>
                                        <div style="font-size: 0.9rem; color: #444;">{entry.base_form.clone()}{" · "}{entry.sentence.clone()}</div>
                                        <div style="font-size: 0.8rem; color: #777;">{location}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="vocab-panel" style="position: fixed; top: 90px; right: 20px; width: 340px; max-height: 70vh; overflow-y: auto; background: white; border: 1px solid #ddd; border-radius: 10px; box-shadow: 0 15px 40px rgba(0,0,0,0.15); padding: 16px; z-index: 1000; display: none;">
                        </div>
                    }.into_any()
                }
            }}
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

#[derive(Deserialize, Clone)]
#[allow(dead_code)]
struct VocabularyEntry {
    id: Option<i64>,
    word: String,
    base_form: String,
    sentence: String,
    source_doc: Option<String>,
    page_num: Option<u32>,
    created_at: i64,
    review_count: u32,
    next_review: Option<i64>,
    ease_factor: f32,
}

/// Helper to create the capture word handler
fn make_capture_handler(
    doc_id: Rc<String>,
    block_text: Rc<String>,
    page_num: u32,
    set_captured: WriteSignal<Option<String>>,
) -> impl Fn() + Clone + 'static {
    move || {
        let doc = Rc::clone(&doc_id);
        let text = Rc::clone(&block_text);
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
                block_text: text.as_ref().to_string(),
                doc_id: doc.as_ref().to_string(),
                page_num,
            };
            let args = to_value(&req).unwrap_or(JsValue::NULL);

            match invoke("capture_word", args).await {
                Ok(result) => {
                    if let Ok(resp) = serde_wasm_bindgen::from_value::<CaptureWordResponse>(result)
                    {
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

fn render_block(
    block: Block,
    doc_id: Rc<String>,
    set_captured: WriteSignal<Option<String>>,
) -> AnyView {
    // Add extra margin-top for paragraphs that start a new indented paragraph
    let paragraph_indent = if block.starts_new_paragraph {
        "text-indent: 1.5em;"
    } else {
        ""
    };

    let page_num = block.page_num;
    let block_text = Rc::new(block.text);

    match block.role {
        BlockRole::Heading { level } => {
            let style = match level {
                1 => {
                    "font-size: 1.5em; font-weight: bold; margin: 20px 0 15px 0; line-height: 1.3; cursor: text;"
                }
                2 => {
                    "font-size: 1.25em; font-weight: bold; margin: 15px 0 10px 0; line-height: 1.3; cursor: text;"
                }
                _ => {
                    "font-size: 1.1em; font-weight: bold; margin: 10px 0 8px 0; line-height: 1.3; cursor: text;"
                }
            };
            let handler = make_capture_handler(doc_id.clone(), block_text.clone(), page_num, set_captured);
            view! {
                <div
                    style=style
                    on:contextmenu=move |ev: MouseEvent| {
                        ev.prevent_default();
                        handler();
                    }
                >{block_text.as_str()}</div>
            }
            .into_any()
        }

        BlockRole::Paragraph => {
            let style = format!(
                "margin: 0 0 10px 0; line-height: 1.6; cursor: text; {}",
                paragraph_indent
            );
            let handler = make_capture_handler(doc_id.clone(), block_text.clone(), page_num, set_captured);
            view! {
                <p
                    style=style
                    on:contextmenu=move |ev: MouseEvent| {
                        ev.prevent_default();
                        handler();
                    }
                >{block_text.as_str()}</p>
            }
            .into_any()
        }

        BlockRole::PageNumber | BlockRole::Header | BlockRole::Footer => {
            // Skip header/footer/page numbers in reflow view (already shown)
            view! { <span></span> }.into_any()
        }

        BlockRole::Footnote => {
            let handler = make_capture_handler(doc_id.clone(), block_text.clone(), page_num, set_captured);
            view! {
                <div
                    style="font-size: 0.85em; color: #666; border-top: 1px solid #ddd; padding-top: 5px; margin-top: 15px; cursor: text;"
                    on:contextmenu=move |ev: MouseEvent| {
                        ev.prevent_default();
                        handler();
                    }
                >
                    {block_text.as_str()}
                </div>
            }.into_any()
        }

        BlockRole::Caption => {
            let handler = make_capture_handler(doc_id.clone(), block_text.clone(), page_num, set_captured);
            view! {
                <p
                    style="font-style: italic; text-align: center; color: #555; margin: 10px 0; cursor: text;"
                    on:contextmenu=move |ev: MouseEvent| {
                        ev.prevent_default();
                        handler();
                    }
                >
                    {block_text.as_str()}
                </p>
            }.into_any()
        }

        BlockRole::Citation => {
            let handler = make_capture_handler(doc_id.clone(), block_text.clone(), page_num, set_captured);
            view! {
                <div
                    style="margin-left: 20px; margin-bottom: 8px; font-size: 0.9em; color: #444; cursor: text;"
                    on:contextmenu=move |ev: MouseEvent| {
                        ev.prevent_default();
                        handler();
                    }
                >
                    {block_text.as_str()}
                </div>
            }.into_any()
        }

        BlockRole::Author => {
            let handler = make_capture_handler(doc_id.clone(), block_text.clone(), page_num, set_captured);
            view! {
                <p
                    style="font-weight: bold; text-align: center; margin-bottom: 5px; cursor: text;"
                    on:contextmenu=move |ev: MouseEvent| {
                        ev.prevent_default();
                        handler();
                    }
                >
                    {block_text.as_str()}
                </p>
            }
            .into_any()
        }

        BlockRole::Abstract => {
            let handler = make_capture_handler(doc_id.clone(), block_text.clone(), page_num, set_captured);
            view! {
                <blockquote
                    style="font-style: italic; border-left: 3px solid #ccc; padding-left: 15px; margin: 15px 0; color: #555; cursor: text;"
                    on:contextmenu=move |ev: MouseEvent| {
                        ev.prevent_default();
                        handler();
                    }
                >
                    {block_text.as_str()}
                </blockquote>
            }.into_any()
        }

        BlockRole::Reference => {
            let handler = make_capture_handler(doc_id.clone(), block_text.clone(), page_num, set_captured);
            view! {
                <h3
                    style="margin: 20px 0 10px 0; border-bottom: 1px solid #ddd; padding-bottom: 5px; cursor: text;"
                    on:contextmenu=move |ev: MouseEvent| {
                        ev.prevent_default();
                        handler();
                    }
                >
                    {block_text.as_str()}
                </h3>
            }.into_any()
        }
    }
}
