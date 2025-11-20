use goidev_core::dto::ReflowDocument;
use goidev_core::reflow_engine::BlockRole;
use leptos::prelude::*;

#[component]
pub fn ReflowViewer(document: ReflowDocument) -> impl IntoView {
    view! {
        <div class="reflow-viewer">
            <h1>{document.title}</h1>
            <div class="blocks">
                {document.blocks.into_iter().map(|block| {
                    match block.role {
                        BlockRole::Heading => view! { <h2>{block.text}</h2> }.into_any(),
                        BlockRole::Paragraph => view! { <p>{block.text}</p> }.into_any(),
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
