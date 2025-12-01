# GOIDEV — Redesigned: Tauri + Leptos + goidev-core (Rust) + optional PyO3/docling

**Purpose (updated)**
Redesign GOIDEV to use Tauri as the application shell with a Leptos frontend (WASM) and a Rust `goidev-core` backend crate. Provide an *optional* `ai_processor` implemented by embedding Python via PyO3 and using `docling` (or similar Python libraries) for higher-fidelity PDF reflow and parsing assistance.

---

## 1. High-level architecture

```
+----------------------+     IPC/Commands     +-------------------------+
|  Tauri App (shell)   | <------------------> |  goidev-core (Rust)     |
|  - Leptos (WASM UI)  |   tauri::invoke()   |  - pdf_parser.rs        |
|  - UI: ReflowViewer  |                     |  - reflow_engine.rs     |
+----------------------+                     |  - nlp_engine.rs        |
                                             |  - storage_layer.rs     |
                                             |  - commands.rs         |
                                             |  - ai_processor (*)     |
                                             +-------------------------+
                                                         |
                                                         | optional: PyO3 embed
                                                         v
                                                  +----------------+
                                                  | Python: docling|
                                                  | (via PyO3)     |
                                                  +----------------+
```

Key notes:

* The UI is written in **Leptos** (Rust → WASM). This keeps the whole stack in Rust for better type-safety and developer ergonomics.
* Tauri provides the native shell and secure IPC (tauri commands). The app bundle remains local-first and offline-capable.
* `goidev-core` is the canonical Rust crate that encapsulates PDF parsing, reflow heuristics, NLP, and storage.
* `ai_processor` is optional and implemented by embedding Python using **PyO3** to call `docling` (or other Python libs). This is an opt-in build feature to avoid shipping the Python runtime by default.

---

## 2. Repository structure (recommended)

```
goidev/
├── goidev-core/             # Rust crate for core logic
│   ├── src/
│   │   ├── pdf_parser.rs
│   │   ├── reflow_engine.rs
│   │   ├── ai_processor.rs    # gate with feature `python-ai`
│   │   ├── nlp_engine.rs
│   │   ├── storage_layer.rs
│   │   ├── commands.rs       # Tauri commands exported here
│   │   └── lib.rs
│   └── Cargo.toml
├── src-tauri/               # Tauri shell
│   ├── src/main.rs
│   ├── tauri.conf.json
│   └── Cargo.toml
├── ui/                      # Leptos frontend app (WASM)
│   ├── Cargo.toml
│   ├── src/main.rs           # leptos::mount_to_body etc.
│   └── components/
│       ├── reflow_viewer.rs
│       ├── side_panel.rs
│       └── toasts.rs
├── README.md
└── rust-toolchain.toml
```

---

## 3. Cargo features & dependencies (high-level)

**goidev-core/Cargo.toml** (key entries):

```toml
[package]
name = "goidev-core"
edition = "2024"

[features]
default = ["pdf-basic"]
pdf-basic = ["lopdf"]
python-ai = ["pyo3", "ai-docling"]

[dependencies]
lopdf = { version = "0.38", optional = true }
pdfium-render = { version = "0.8", optional = true }
rusqlite = { version = "0.37", features = ["bundled"] }
unicode_segmentation = "1.12"
waken_snowball = "0.1"
serde = { version = "1.0", features = ["derive"] }
uuid = { version = "1.11", features = ["v4"] }
tokio = { version = "1.41", features = ["rt", "fs"] }
log = "0.4"
env_logger = "0.11"

# PyO3 & Python helper (optional)
pyo3 = { version = "0.20", optional = true, features = ["auto-initialize"] }
# A thin helper crate to invoke a docling-like Python module can be a dev-dep or separate package.
```

Notes:

* `python-ai` is an opt-in Cargo feature that enables `pyo3` usage and the `ai_processor` module.
* Keep heavy or optional deps behind feature flags to keep the default binary small.

---

## 4. IPC & Data Contracts (Leptos ↔ Tauri ↔ goidev-core)

Use Tauri commands (`#[tauri::command]`) in `goidev-core::commands` and invoke from the Leptos frontend using `@tauri-apps/api` bindings (or `tauri::invoke` from Rust WASM shim). Example JSON contracts:

**ReflowDocument** (Backend → UI)

```json
{
  "doc_id": "uuid-v4",
  "title": "...",
  "pages": [
    {
      "page_number": 1,
      "blocks": [ { "id":"p1_b1", "type":"paragraph", "text":"...", "bbox": {...}, "confidence": 0.95 } ]
    }
  ]
}
```

**WordSelectionRequest** (UI → Backend)

```json
{ "documentId":"uuid","pageNumber":3,"selectedWord":"running","blockText":"..." }
```

**WordSelectionResponse** (Backend → UI)

```json
{ "status":"success","data": { "word":"running","base_form":"run","sentence":"...", "occurrence_count":5 } }
```

---

## 5. Module responsibilities (detailed)

### `pdf_parser.rs`

* Use `lopdf` for a lightweight default pipeline to extract text, fonts, and the best-effort bounding boxes.
* Provide `parse_pdf(path, start, end) -> Result<Vec<TextChunk>>`.
* The parser will return a `Vec<TextLine>`.
* Each `TextLine` contains its bounding box and a vector of `WordSpan`s, which hold the text, position, and font size.

### `reflow_engine.rs`

 * Group contiguous `TextLine`s into `Block`s (paragraphs/headings) using heuristics: font_size gaps, vertical gaps, left margins, and font-style changes.
* Produce `ReflowDocument` data structure matching the data contract.
* Expose deterministic `to_markdown()` for RAG/export.

### `nlp_engine.rs`

* Minimal pure-Rust lemmatization & sentence-splitting: use `waken_snowball` and `unicode_segmentation`.
* Return base form (lemma) and the sentence containing the selected token.

### `storage_layer.rs`

* SQLite schema (same as prior DESIGN.md). Use `rusqlite`.
* Implement migrations, indices, and accessors for words & contexts.

### `commands.rs` (Tauri commands)

* `open_document(path: String) -> ReflowDocument`
* `process_word_selection(payload: WordSelectionRequest) -> WordSelectionResponse`
* `get_word_list(filter, page, sort) -> Vec<WordSummary>`
* `get_word_contexts(word_id) -> Vec<Context>`

### `ai_processor.rs` (optional; feature `python-ai`)

* If enabled, embed Python via PyO3 and call into `docling` functions to get improved layout, OCR, or reflow heuristics.
* Provide a fallback Rust-only implementation when disabled.

Implementation suggestions for `ai_processor`:

* Initialize Python with `pyo3::prepare_freethreaded_python()` at app start when the feature is active.
* Keep the API surface narrow: expose `ai_reflow_enhance(chunks: Vec<TextChunk>) -> Vec<Block>`.
* Catch Python exceptions and fall back gracefully to Rust heuristics.

Security & packaging note: embedding Python increases bundle complexity; provide a build flag to include an embedded Python runtime or detect a system Python at runtime.

---

## 6. Build & developer commands (examples)

Prereqs:

* Rust (nightly not required), cargo-leptos (optional), Node not required.
* Tauri prerequisites: `cargo-tauri` tool and platform toolchains.

Example dev flow (without python-ai):

```bash
# from repo root
# 1) Build Leptos frontend -> wasm and assets
cd ui
cargo leptos build --release   # or the leptos build flow you prefer

# 2) Add goidev-core as dependency to src-tauri
cd ../src-tauri
cargo add ../goidev-core --path ../goidev-core

# 3) Run Tauri dev
cargo tauri dev
```

With python-ai feature (developer machine with Python & dependencies):

```bash
# create venv and install docling or equivalent
python -m venv .venv && source .venv/bin/activate
pip install docling    # or your chosen package

# enable feature when building tauri
cd src-tauri
cargo tauri dev --features "python-ai"
```

Packaging: only enable `python-ai` for users who opt in (or bundle a lightweight Python runtime during packaging).

---

## 7. Roadmap (8–10 weeks, updated)

* **Weeks 1–2:** Project scaffolding, Tauri + Leptos starter, minimal `open_document` that returns simple `ReflowDocument` from `lopdf`.
* **Weeks 3–4:** `pdf_parser` → `reflow_engine` MVP, basic UI rendering (continuous scroll), double-click selection plumbing to `process_word_selection`.
* **Weeks 5–6:** `nlp_engine` + storage layer; full word capture flow and side-panel UI.
* **Weeks 7–8:** Tests, UX polish, export to Markdown, performance pass.
* **Weeks 9–10 (opt):** Integrate `python-ai` feature using PyO3 + docling for improved reflow. Add CI matrix with & without `python-ai`.

Acceptance criteria for Milestone 1 (MVP):

* `parse_pdf` returns non-empty `Vec<TextChunk>` for sample PDFs and includes bbox & font metadata.
* UI can open a local PDF, show blocks, and capture a double-clicked word that is saved to SQLite.

---

## 8. Testing & CI

* Unit tests for `pdf_parser` parsing and `reflow_engine` grouping heuristics.
* Integration tests that simulate Tauri commands using `tauri::test` harness (or direct unit tests on `commands.rs`).
* CI should run with default features (no `python-ai`) to keep runner lightweight. Add an optional workflow for `python-ai` on self-hosted runners if needed.

---

## 9. Notes, tradeoffs, and recommendations

* **Why Leptos (WASM Rust) vs JS frameworks?**

  * Single-language (Rust) reduces context switching and centralizes types. Leptos works well with Tauri when compiled to Wasm.
  * If you anticipate many 3rd-party JS UI libs, you may choose a JS frontend; otherwise Leptos is a strong Rust-native choice.

* **Why PyO3 + docling optional?**

  * Python currently has richer PDF/OCR/layout tooling. Embedding it offers higher-quality reflow without reimplementing complex heuristics in Rust.
  * But embedding Python increases bundle complexity and potential security surface — keep it opt-in.

* **Packaging**

  * Provide two package tiers: "lite" (Rust-only) and "ai" (bundled Python). Let users choose on download/install.

---

## 10. Next steps (actions you can take now)

1. Confirm you want Leptos (WASM) rather than a JS frontend. If yes, I can convert the existing React examples in `DESIGN.md` to Leptos components.
2. Decide whether `ai_processor` should be `embedded` (PyO3) or `external` (spawn a Python subprocess). I can provide both implementation sketches.
3. If you want, I will update `PLANS.md` with milestone-level tasks and concrete TODO entries for the next 4 sprints.

---

*End of redesigned spec.*
