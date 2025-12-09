use goidev_core::dto::ReflowDocument;
use goidev_core::reflow_engine::{Block, BlockRole};
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use std::collections::BTreeMap;
use std::sync::Arc;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

struct PageGroup {
    pdf_page: u32,
    doc_page: Option<String>,
    blocks: Vec<Block>,
}

#[component]
pub fn ReflowViewer(document: ReflowDocument) -> impl IntoView {
    let (captured_word, set_captured_word) = signal(None::<String>);
    let (vocab_entries, set_vocab_entries) = signal(Vec::<VocabularyEntry>::new());
    let (pending_capture, set_pending_capture) = signal(None::<PendingCapture>);

    let load_vocab = {
        let set_vocab_entries = set_vocab_entries;
        move || {
            spawn_local(async move {
                match invoke("get_vocabulary", JsValue::NULL).await {
                    Ok(result) => match serde_wasm_bindgen::from_value::<Vec<VocabularyEntry>>(result) {
                        Ok(entries) => set_vocab_entries.set(entries),
                        Err(err) => web_sys::console::error_1(&JsValue::from_str(&format!("vocab deserialize error: {:?}", err))),
                    },
                    Err(err) => web_sys::console::error_1(&JsValue::from_str(&format!("get_vocabulary invoke error: {:?}", err))),
                }
            });
        }
    };

    // Load vocab on mount so the side panel is populated immediately
    load_vocab();

    let pages: Vec<PageGroup> = {
        let mut map: BTreeMap<u32, PageGroup> = BTreeMap::new();
        for block in document.blocks {
            let entry = map.entry(block.page_num).or_insert_with(|| PageGroup {
                pdf_page: block.page_num,
                doc_page: None,
                blocks: Vec::new(),
            });
            if entry.doc_page.is_none() {
                entry.doc_page = block.doc_page_num.clone();
            }
            entry.blocks.push(block);
        }
        map.into_values().collect()
    };

    let doc_id = Arc::new(document.doc_id);

    view! {
        <div class="reflow-viewer">
            <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px;">
                <h1>{document.title}</h1>
                <span style="font-size: 0.95rem; color: #555;">{"Vocabulary"}</span>
            </div>

            <div style="display: grid; grid-template-columns: 1fr 340px; gap: 16px; align-items: start;">
                <div class="pages">
                    {pages.into_iter().map(|page_group| {
                        let bg_style = if page_group.pdf_page % 2 != 0 {
                            "background-color: #ffffff; color: #333333; padding: 20px; margin-bottom: 20px; position: relative; border: 1px solid #eee;"
                        } else {
                            "background-color: #f8f9fa; color: #333333; padding: 20px; margin-bottom: 20px; position: relative; border: 1px solid #eee;"
                        };

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
                                    render_block(block, doc_id.clone(), set_pending_capture)
                                }).collect_view()}
                            </div>
                        }
                    }).collect_view()}
                </div>

                {move || {
                    let entries = vocab_entries.get();
                    view! {
                        <aside class="vocab-panel" style="background: white; border: 1px solid #ddd; border-radius: 10px; box-shadow: 0 10px 25px rgba(0,0,0,0.12); padding: 16px; max-height: calc(100vh - 120px); overflow-y: auto; position: sticky; top: 70px;">
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                                <strong>Vocabulary</strong>
                            </div>
                            {entries.into_iter().map(|entry| {
                                let location = match (entry.source_doc.as_ref(), entry.page_num) {
                                    (Some(doc), Some(page)) => format!("{} · page {}", doc, page),
                                    (Some(doc), None) => doc.to_string(),
                                    (None, Some(page)) => format!("Page {}", page),
                                    (None, None) => "Unknown source".to_string(),
                                };
                                view! {
                                    <div style="border-bottom: 1px solid #f0f0f0; padding: 8px 0;">
                                        <div style="font-weight: 600; font-size: 1rem;">{entry.word.clone()}</div>
                                        <div style="font-size: 0.9rem; color: #444;">{entry.base_form.clone()}" · "{entry.sentence.clone()}</div>
                                        <div style="font-size: 0.8rem; color: #777;">{location}</div>
                                    </div>
                                }
                            }).collect_view()}
                        </aside>
                    }.into_any()
                }}
            </div>

            {move || pending_capture.get().map(|pending| {
                let set_pending_capture = set_pending_capture;
                let set_captured_word = set_captured_word;
                let load_vocab = load_vocab;
                let word = pending.word;
                let block_text = pending.block_text;
                let doc_id = pending.doc_id;
                let page_num = pending.page_num;
                let top = pending.y;
                let left = pending.x;
                view! {
                    <div class="capture-menu-overlay" style="position: fixed; inset: 0; z-index: 1100;" on:click=move |_| set_pending_capture.set(None)>
                        <div class="capture-menu" style=format!("position: fixed; top: {}px; left: {}px; background: white; border: 1px solid #ccc; border-radius: 8px; box-shadow: 0 8px 24px rgba(0,0,0,0.2); padding: 8px; min-width: 160px;", top, left) on:click=move |ev| ev.stop_propagation()>
                            <button
                                type="button"
                                style="width: 100%; padding: 8px 10px; background: #007acc; color: white; border: none; border-radius: 6px; cursor: pointer; margin-bottom: 6px;"
                                on:click=move |_| {
                                    let set_captured_word = set_captured_word;
                                    let set_pending_capture = set_pending_capture;
                                    let load_vocab = load_vocab;
                                    let selected_word = word.clone();
                                    let selected_block = block_text.clone();
                                    let selected_doc = doc_id.clone();
                                    spawn_local(async move {
                                        let req = CaptureWordRequest {
                                            word: selected_word.clone(),
                                            block_text: selected_block.as_ref().to_string(),
                                            doc_id: selected_doc.as_ref().to_string(),
                                            page_num,
                                        };
                                        let args = to_value(&CaptureWordArgs { request: req }).unwrap_or(JsValue::NULL);
                                        match invoke("capture_word", args).await {
                                            Ok(result) => match serde_wasm_bindgen::from_value::<CaptureWordResponse>(result) {
                                                Ok(resp) => {
                                                    if resp.success {
                                                        set_captured_word.set(Some(selected_word.clone()));
                                                        set_pending_capture.set(None);
                                                        load_vocab();
                                                        spawn_local(async move {
                                                            gloo_timers::future::TimeoutFuture::new(2000).await;
                                                            set_captured_word.set(None);
                                                        });
                                                        return;
                                                    }
                                                    let msg = resp.error.unwrap_or_else(|| "capture failed (success=false)".to_string());
                                                    web_sys::console::error_1(&JsValue::from_str(&format!("capture_word error: {}", msg)));
                                                }
                                                Err(err) => web_sys::console::error_1(&JsValue::from_str(&format!("capture_word deserialize error: {:?}", err))),
                                            },
                                            Err(err) => web_sys::console::error_1(&JsValue::from_str(&format!("capture_word invoke error: {:?}", err))),
                                        }
                                        set_pending_capture.set(None);
                                    });
                                }
                            >{"Catch the word"}</button>
                            <button
                                type="button"
                                style="width: 100%; padding: 6px 10px; background: #f5f5f5; color: #333; border: 1px solid #ddd; border-radius: 6px; cursor: pointer;"
                                on:click=move |_| set_pending_capture.set(None)
                            >{"Cancel"}</button>
                        </div>
                    </div>
                }
            })}

            {move || captured_word.get().map(|word| view! {
                <div class="capture-toast" style="position: fixed; bottom: 20px; right: 20px; background: #4CAF50; color: white; padding: 12px 20px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.2); z-index: 1000; animation: fadeIn 0.3s;">
                    {"✓ Captured: "}{word}
                </div>
            })}
        </div>
    }
}

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

#[derive(Serialize)]
struct CaptureWordArgs {
    request: CaptureWordRequest,
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

#[derive(Clone)]
struct PendingCapture {
    word: String,
    block_text: Arc<String>,
    doc_id: Arc<String>,
    page_num: u32,
    x: i32,
    y: i32,
}

fn render_block(
    block: Block,
    doc_id: Arc<String>,
    set_pending_capture: WriteSignal<Option<PendingCapture>>,
) -> AnyView {
    let paragraph_indent = if block.starts_new_paragraph {
        "text-indent: 1.5em;"
    } else {
        ""
    };

    let page_num = block.page_num;
    let block_text = Arc::new(block.text);

    let open_menu = {
        let doc_id = Arc::clone(&doc_id);
        let block_text = Arc::clone(&block_text);
        let set_pending_capture = set_pending_capture.clone();
        move |ev: MouseEvent| {
            let selected = web_sys::window()
                .and_then(|w| w.get_selection().ok().flatten())
                .map(|s| s.to_string())
                .map(|js| js.as_string().unwrap_or_default())
                .unwrap_or_default();

            if selected.trim().is_empty() {
                return;
            }

            ev.prevent_default();

            set_pending_capture.set(Some(PendingCapture {
                word: selected,
                block_text: block_text.clone(),
                doc_id: doc_id.clone(),
                page_num,
                x: ev.client_x(),
                y: ev.client_y(),
            }));
        }
    };

    match block.role {
        BlockRole::Heading { level } => {
            let style = match level {
                1 => "font-size: 1.5em; font-weight: bold; margin: 20px 0 15px 0; line-height: 1.3; cursor: text;",
                2 => "font-size: 1.25em; font-weight: bold; margin: 15px 0 10px 0; line-height: 1.3; cursor: text;",
                _ => "font-size: 1.1em; font-weight: bold; margin: 10px 0 8px 0; line-height: 1.3; cursor: text;",
            };
            view! { <div style=style on:contextmenu=move |ev: MouseEvent| open_menu(ev)>{block_text.as_str()}</div> }.into_any()
        }
        BlockRole::Paragraph => {
            let style = format!("margin: 0 0 10px 0; line-height: 1.6; cursor: text; {}", paragraph_indent);
            view! { <p style=style on:contextmenu=move |ev: MouseEvent| open_menu(ev)>{block_text.as_str()}</p> }.into_any()
        }
        BlockRole::PageNumber | BlockRole::Header | BlockRole::Footer => view! { <span></span> }.into_any(),
        BlockRole::Footnote => view! { <div style="font-size: 0.85em; color: #666; border-top: 1px solid #ddd; padding-top: 5px; margin-top: 15px; cursor: text;" on:contextmenu=move |ev: MouseEvent| open_menu(ev)>{block_text.as_str()}</div> }.into_any(),
        BlockRole::Caption => view! { <p style="font-style: italic; text-align: center; color: #555; margin: 10px 0; cursor: text;" on:contextmenu=move |ev: MouseEvent| open_menu(ev)>{block_text.as_str()}</p> }.into_any(),
        BlockRole::Citation => view! { <div style="margin-left: 20px; margin-bottom: 8px; font-size: 0.9em; color: #444; cursor: text;" on:contextmenu=move |ev: MouseEvent| open_menu(ev)>{block_text.as_str()}</div> }.into_any(),
        BlockRole::Author => view! { <p style="font-weight: bold; text-align: center; margin-bottom: 5px; cursor: text;" on:contextmenu=move |ev: MouseEvent| open_menu(ev)>{block_text.as_str()}</p> }.into_any(),
        BlockRole::Abstract => view! { <blockquote style="font-style: italic; border-left: 3px solid #ccc; padding-left: 15px; margin: 15px 0; color: #555; cursor: text;" on:contextmenu=move |ev: MouseEvent| open_menu(ev)>{block_text.as_str()}</blockquote> }.into_any(),
        BlockRole::Reference => view! { <h3 style="margin: 20px 0 10px 0; border-bottom: 1px solid #ddd; padding-bottom: 5px; cursor: text;" on:contextmenu=move |ev: MouseEvent| open_menu(ev)>{block_text.as_str()}</h3> }.into_any(),
    }
}
