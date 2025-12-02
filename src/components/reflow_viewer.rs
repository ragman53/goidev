use goidev_core::dto::ReflowDocument;
use goidev_core::reflow_engine::{Block, BlockRole};
use leptos::prelude::*;
use std::collections::BTreeMap;

/// Metadata for a page group
struct PageGroup {
    pdf_page: u32,
    doc_page: Option<String>,
    blocks: Vec<Block>,
}

#[component]
pub fn ReflowViewer(document: ReflowDocument) -> impl IntoView {
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

    view! {
        <div class="reflow-viewer">
            <h1>{document.title}</h1>
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
    // Add extra margin-top for paragraphs that start a new indented paragraph
    let paragraph_indent = if block.starts_new_paragraph {
        "text-indent: 1.5em;"
    } else {
        ""
    };

    match block.role {
        BlockRole::Heading { level } => {
            let style = match level {
                1 => "font-size: 1.5em; font-weight: bold; margin: 20px 0 15px 0; line-height: 1.3;",
                2 => "font-size: 1.25em; font-weight: bold; margin: 15px 0 10px 0; line-height: 1.3;",
                _ => "font-size: 1.1em; font-weight: bold; margin: 10px 0 8px 0; line-height: 1.3;",
            };
            view! {
                <div style=style>{block.text}</div>
            }.into_any()
        },

        BlockRole::Paragraph => {
            let style = format!("margin: 0 0 10px 0; line-height: 1.6; {}", paragraph_indent);
            view! {
                <p style=style>{block.text}</p>
            }.into_any()
        },

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
