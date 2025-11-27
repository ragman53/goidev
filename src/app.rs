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

#[derive(Serialize, Deserialize)]
struct OpenDocumentArgs<'a> {
    path: &'a str,
}

#[component]
pub fn App() -> impl IntoView {
    let (path, set_path) = signal(String::new());
    let (document, set_document) = signal(Option::<ReflowDocument>::None);
    let (error_msg, set_error_msg) = signal(String::new());

    let update_path = move |ev| {
        let v = event_target_value(&ev);
        set_path.set(v);
    };

    let open_pdf = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let path = path.get_untracked();
            if path.is_empty() {
                return;
            }

            let args = serde_wasm_bindgen::to_value(&OpenDocumentArgs { path: &path }).unwrap();

            match invoke("open_document", args).await {
                Ok(val) => match serde_wasm_bindgen::from_value::<ReflowDocument>(val) {
                    Ok(doc) => {
                        set_document.set(Some(doc));
                        set_error_msg.set(String::new());
                    }
                    Err(e) => {
                        set_error_msg.set(format!("Failed to parse result: {:?}", e));
                    }
                },
                Err(e) => {
                    set_error_msg.set(format!(
                        "Error opening PDF: {}",
                        e.as_string().unwrap_or_else(|| "Unknown error".to_string())
                    ));
                }
            }
        });
    };

    view! {
        <main class="container">
            <h1>"GOIDEV PDF Reader"</h1>

            <form class="row" on:submit=open_pdf>
                <input
                    id="path-input"
                    placeholder="Enter absolute path to PDF..."
                    on:input=update_path
                />
                <button type="submit">"Open PDF"</button>
            </form>

            <p class="error">{ move || error_msg.get() }</p>

            {move || document.get().map(|doc| view! { <ReflowViewer document=doc/> })}
        </main>
    }
}
