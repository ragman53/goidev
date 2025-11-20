use crate::components::reflow_viewer::ReflowViewer;
use goidev_core::dto::ReflowDocument;
use leptos::task::spawn_local;
use leptos::{ev::SubmitEvent, prelude::*};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
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
            match invoke("open_document", args).await.as_string() {
                // invoke returns a JsValue. We need to deserialize it into ReflowDocument.
                // Wait, invoke returns a Promise that resolves to the result.
                // If the command returns Result<T, E>, Tauri serializes Ok(T) or rejects with E.
                // So we need to handle the promise rejection for errors.
                // However, the simple `invoke` binding here might not handle Result types automatically if we just use `as_string()`.
                // `as_string()` only works if the return value is a string. ReflowDocument is an object.
                // We need serde_wasm_bindgen::from_value.
                _ => {}
            }

            // Let's try a safer approach with serde_wasm_bindgen
            let args = serde_wasm_bindgen::to_value(&OpenDocumentArgs { path: &path }).unwrap();

            // We need to handle the Result from rust.
            // Since we are using a raw extern "C" block, we have to be careful.
            // Ideally we should use `tauri_sys` or the `invoke` from `@tauri-apps/api` if we were in JS, but here we are in Rust.
            // The `invoke` defined here returns `JsValue`.

            match invoke("open_document", args).await {
                val => {
                    // Check if it's an error?
                    // Actually, if the command fails, the promise rejects.
                    // `invoke(...).await` in Rust/wasm-bindgen usually means it awaits the promise.
                    // If the promise rejects, `await` might panic or return an error if the signature was `async fn invoke(...) -> Result<JsValue, JsValue>`.
                    // The current signature is `async fn invoke(...) -> JsValue`. This implies it might not catch rejections?
                    // Let's update the signature to return Result<JsValue, JsValue> to handle errors.

                    match serde_wasm_bindgen::from_value::<ReflowDocument>(val) {
                        Ok(doc) => {
                            set_document.set(Some(doc));
                            set_error_msg.set(String::new());
                        }
                        Err(e) => {
                            // It might be that it returned an error string, but we tried to parse as ReflowDocument.
                            // Or maybe the invoke didn't fail but returned a string error?
                            // If the command returns Result<T, String>, Tauri sends the String as a rejection.
                            // So we MUST change the signature of invoke to handle rejections.
                            set_error_msg.set(format!("Failed to parse result: {:?}", e));
                        }
                    }
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
