use goidev_core::dto::ReflowDocument;
use goidev_core::reflow_engine::{Block, BlockRole};
use leptos::prelude::*;
use std::collections::BTreeMap;

#[component]
pub fn ReflowViewer(document: ReflowDocument) -> impl IntoView {
    // Group blocks by page
    let pages = {
        let mut map: BTreeMap<u32, Vec<Block>> = BTreeMap::new();
        for block in document.blocks {
            map.entry(block.page_num).or_default().push(block);
        }
        map
    };

    view! {
        <div class="reflow-viewer">
            <h1>{document.title}</h1>
            <div class="pages">
                {pages.into_iter().map(|(page_num, blocks)| {
                    let bg_style = if page_num % 2 != 0 {
                        "background-color: #ffffff; color: #333333; padding: 20px; margin-bottom: 20px; position: relative; border: 1px solid #eee;"
                    } else {
                        "background-color: #f8f9fa; color: #333333; padding: 20px; margin-bottom: 20px; position: relative; border: 1px solid #eee;"
                    };

                    view! {
                        <div class="page-container" style=bg_style>
                            <div class="page-number" style="position: absolute; top: 5px; right: 10px; color: #888; font-size: 0.8em; font-weight: bold;">
                                "Page " {page_num}
                            </div>
                            {blocks.into_iter().map(|block| {
                                render_block(block)
                            }).collect_view()}
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Render a block based on its role.
fn render_block(block: Block) -> AnyView {
    match block.role {
        BlockRole::Heading => view! {
            <h2 style="margin: 0 0 10px 0; line-height: 1.3;">{block.text}</h2>
        }.into_any(),

        BlockRole::Paragraph => view! {
            <p style="margin: 0 0 10px 0; line-height: 1.6;">{block.text}</p>
        }.into_any(),

        BlockRole::PageNumber | BlockRole::Header | BlockRole::Footer => {
            // Skip header/footer/page numbers in reflow view (already shown)
            view! { <span></span> }.into_any()
        },

        BlockRole::Footnote => view! {
            <div style="font-size: 0.85em; color: #666; border-top: 1px solid #ddd; padding-top: 5px; margin-top: 15px;">
                {block.text}
            </div>
        }.into_any(),

        BlockRole::Caption => view! {
            <p style="font-style: italic; text-align: center; color: #555; margin: 10px 0;">
                {block.text}
            </p>
        }.into_any(),

        BlockRole::Citation => view! {
            <div style="margin-left: 20px; margin-bottom: 8px; font-size: 0.9em; color: #444;">
                {block.text}
            </div>
        }.into_any(),

        BlockRole::Author => view! {
            <p style="font-weight: bold; text-align: center; margin-bottom: 5px;">
                {block.text}
            </p>
        }.into_any(),

        BlockRole::Abstract => view! {
            <blockquote style="font-style: italic; border-left: 3px solid #ccc; padding-left: 15px; margin: 15px 0; color: #555;">
                {block.text}
            </blockquote>
        }.into_any(),

        BlockRole::Reference => view! {
            <h3 style="margin: 20px 0 10px 0; border-bottom: 1px solid #ddd; padding-bottom: 5px;">
                {block.text}
            </h3>
        }.into_any(),
    }
}
