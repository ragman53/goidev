# GOIDEV Agent Spec: Vibe-Coding Workflow Control

This document defines how autonomous Agents collaborate to build GOIDEV using “vibe-coding” with strict Test-Driven Development (TDD). Agents act as a senior engineer pair-programming with a junior engineer (less than six months of experience). Agents must always consult and maintain the canonical plan in PLANS.md.

## Core Principles

- PLANS.md is the single source of truth. Before any action, read the entire PLANS.md and update its Progress, Decision Log, and Concrete Steps as work proceeds.
- Vibe-coding: iterate quickly in tiny, end-to-end slices. Prefer small, observable wins over big-bang changes. Keep the energy high, feedback loops short, and explanations junior-friendly.
- Test-Driven (TDD): write a failing test (Red) → make it pass (Green) → clean up (Refactor). Minimum: one happy-path test plus one edge case per feature.
- Pair-programming mindset: narrate intent, explain trade-offs, and capture learnings in PLANS.md.
- Speed with safety: small commits, local-only by default, reproducible steps, and clear rollback.

## Agent Roles and Responsibilities

- Planner Agent: tasks are decomposed into small steps from PLANS.md; writes or updates milestones, Concrete Steps, and acceptance criteria; notes assumptions explicitly.
- Coder Agent: generates code to satisfy current acceptance criteria; keeps diffs focused; runs builds/tests locally; updates PLANS.md Progress as steps complete.
- Tester Agent: creates tests first; covers happy path and at least one edge case; captures output snippets in PLANS.md Artifacts and Validation.
- Reviewer Agent: performs code review for correctness, readability, security, and performance; requests small follow-ups; records decisions in Decision Log.
- Teacher Agent: explains generated code and changes in plain language; helps the user grow coding knowledge; adds notes to Outcomes & Retrospective.

Deliverables per role:

- Planner: updated sections in PLANS.md (Plan of Work, Concrete Steps, Interfaces), clarified assumptions, and next-step checkboxes in Progress.
- Coder: code changes aligned with PLANS.md; no new lints; passing local build/tests; short commit messages.
- Tester: new test files and assertions; reproducible transcripts in PLANS.md Validation and Artifacts.
- Reviewer: comments, requested fixes, or approval; Decision Log entries reflecting trade-offs.
- Teacher: junior-friendly explanations, risks/limits, and quickstart notes appended to PLANS.md.

## End-to-End Workflow (Loop)

1) Planner reads PLANS.md fully, updates Plan of Work and Concrete Steps, and marks the next step in Progress.
2) Tester adds/updates a failing test for the step (Red) and documents expected failure in PLANS.md.
3) Coder implements the smallest change to pass the test (Green) and updates Progress.
4) Tester runs the full test suite; captures PASS/FAIL in PLANS.md Validation; adds artifacts as evidence.
5) Reviewer reviews diffs; either requests small changes (loop 3–4) or approves and logs decisions.
6) Teacher writes a short explanation of what changed and how to run/observe it; adds notes to Outcomes & Retrospective.
7) Planner selects the next small step and repeats the loop.

Always keep commit scope tiny and tied to a checked box in PLANS.md Progress.

## Definition of Done (Per Step)

- All acceptance criteria for the step in PLANS.md are satisfied and demonstrable.
- New/updated tests are present and green; no new warnings/lints introduced.
- PLANS.md Progress, Validation, Decision Log, and (if applicable) Outcomes & Retrospective updated.
- Commands work on Windows PowerShell (pwsh) unless stated otherwise.

## Operational Rules

- Source of truth: PLANS.md governs scope, sequence, and acceptance. If you discover drift, update PLANS.md immediately before coding further.
- Assumptions: when uncertain, state one or two reasonable assumptions in the Decision Log and proceed; revise if later contradicted.
- Edits: keep patches small and localized. Preserve existing public APIs unless the step explicitly calls for change.
- Security/Privacy: no external network calls unless the step requires it and PLANS.md allows it. Handle secrets safely.
- Logging: prefer concise logs during development and remove noisy debugging before declaring done.

## File Orientation

- Backend crate: goidev-core/
  - Key modules: src/pdf_parser.rs, src/reflow_engine.rs, src/nlp_engine.rs, src/storage_layer.rs, src/api.rs
- UI crate: dioxus-ui/
  - Key modules: src/app.rs, src/reflow_viewer.rs, src/components/
- Project docs: PLANS.md (canonical plan), this AGENTS.md (agent workflow control)

## Commit Convention (Recommended)

- feat: add new user-visible behavior
- fix: correct behavior or bug
- test: add or change tests
- docs: docs only
- refactor: code restructuring, no behavior change
- chore: repo maintenance

Prefix optional scope, for example: feat(pdf_parser): extract font_size. Reference a plan step, for example: refs: plan-step M1-S2.

## TDD Checklist (Per Feature)

- Write: one failing happy-path test and one failing edge-case test
- Implement: minimal code to pass
- Refactor: clean code and tests; keep tests green
- Capture: transcripts or short outputs in PLANS.md Validation/Artifacts

## Vibe-Coding Guidance

- Start with a walking skeleton: smallest vertical slice that runs from PDF → chunks → reflow → UI marker.
- Prefer rough-but-working over perfect-but-late; refine in subsequent tiny steps.
- Keep explanations friendly and concrete; prefer “how to verify” over theory.

This spec guides Agents to act consistently, quickly, and transparently. When in doubt, update PLANS.md and keep moving.

# GOIDEV Specification: AI-Enhanced PDF Reader & Vocabulary Builder

Purpose: Canonical spec for building a local-first PDF reader with vocabulary capture, using Dioxus (desktop/web) and Rust backend. Designed for GitHub Copilot in VSCode, with embedded prompts for code generation. Targets junior engineers learning Rust.

## Architecture Overview

```
+--------------------------------------+        +------------------+
| Dioxus UI (Desktop/Web)              | <----> | Rust Backend     |
| - ReflowViewer                       |        +------------------+
| - Word Selection UI / Side Panel     |
+------------------^-------------------+
                   | (Async Calls)
+------------------v-------------------+
| goidev-core (Rust crate)             |
| - pdf_parser (lopdf)                 |
| - reflow_engine (heuristics)         |
| - ai_processor (Candle + SLM, opt.)  |
| - nlp_engine (unicode_segmentation,  |
|                      waken_snowball) |
| - storage_layer (rusqlite)           |
+--------------------------------------+
```f

## 1. Core Goals (MVP)

Open local PDF, render in continuous-scroll viewer (Kindle-like, no snapping).
Double-click word → capture cleaned word + sentence → store with base form (stem/lemma).
Track word occurrences and multiple contexts in SQLite.
Minimal UI for viewing captured words/contexts.
Local-first, performant (lazy loading), extensible for web monetization.

## 2. Project Setup

### 2.1 Repository Structure

```
goidev/
├── goidev-core/           # Rust crate: backend logic
│   ├── src/
│   │   ├── pdf_parser.rs
│   │   ├── reflow_engine.rs
│   │   ├── ai_processor.rs
│   │   ├── nlp_engine.rs
│   │   ├── storage_layer.rs
│   │   ├── api.rs
│   │   ├── lib.rs
│   ├── Cargo.toml
├── dioxus-ui/             # Dioxus frontend (desktop/web)
│   ├── src/
│   │   ├── app.rs
│   │   ├── reflow_viewer.rs
│   │   ├── components/
│   │   │   ├── page.rs
│   │   │   ├── block.rs
│   │   │   ├── word_capture_toast.rs
│   │   │   ├── word_side_panel.rs
│   ├── Cargo.toml
├── README.md
├── rust-toolchain.toml    # Pin Rust 1.90+
```

### 2.2 Dependencies (Cargo.toml)

**goidev-core/Cargo.toml:**

```toml
[package]
name = "goidev-core"
version = "0.1.0"
edition = "2024"

[dependencies]
lopdf = "0.38"          # PDF parsing
rusqlite = { version = "0.37", features = ["bundled"] } # SQLite
unicode_segmentation = "1.12"         # Sentence segmentation
waken_snowball = "0.1"   # Word normalization (replaced rust-stemmers)
serde = { version = "1.0", features = ["derive"] } # JSON
uuid = { version = "1.11", features = ["v4"] } # UUIDs
tokio = { version = "1.41", features = ["rt", "fs"] } # Async
candle-core = { version = "0.8", optional = true } # Optional AI
log = "0.4"             # Logging
env_logger = "0.11"     # Logging setup
pdfium-render = "0.8"   # Image fallback rendering
```

**dioxus-ui/Cargo.toml:**

```toml
[package]
name = "dioxus-ui"
version = "0.1.0"
edition = "2024"

[dependencies]
dioxus = { version = "0.6", features = ["desktop", "web"] } # Desktop + WASM
dioxus-logger = "0.6"   # Logging
serde = { version = "1.0", features = ["derive"] } # JSON
serde_json = "1.0"      # JSON parsing
goidev-core = { path = "../goidev-core" } # Backend crate
```

### 2.3 Setup Commands

```
# Init projects
cargo new goidev-core
cargo new dioxus-ui
cd dioxus-ui
cargo add dioxus --features desktop,web
cargo add dioxus-logger serde serde_json goidev-core --path ../goidev-core
cd ../goidev-core
cargo add lopdf rusqlite --features bundled rust-stemmers serde --features derive uuid --features v4 tokio --features rt,fs candle-core --optional log env_logger pdfium-render
# Install dioxus-cli for dev
cargo install dioxus-cli
# Run desktop
cd dioxus-ui
dioxus serve --platform desktop
# Run web
dioxus serve --platform web
```

## 3. Data Contracts

### 3.1 ReflowDocument (Backend → UI)

```json
{
  "doc_id": "uuid-v4",
  "title": "...",
  "pages": [
    {
      "page_number": 1,
      "blocks": [
        {
          "id": "page1_block1",
          "type": "paragraph",
          "text": "Reflowed text...",
          "confidence": 0.95,
          "bbox": {"x":0,"y":0,"w":600,"h":48}
        }
      ]
    }
  ]
}
```

### 3.2 Vocabulary Request/Response

**Request (UI → Backend):**

```json
{
  "documentId": "uuid",
  "pageNumber": 3,
  "selectedWord": "running",
  "blockText": "The dog was running across the field. It was very fast."
}
```

**Response:**

```json
{
  "status": "success",
  "data": {
    "word": "running",
    "base_form": "run",
    "sentence": "The dog was running across the field.",
    "occurrence_count": 5
  }
}
```

## 4. Database Schema (SQLite)

```sql
CREATE TABLE documents (
  id TEXT PRIMARY KEY, -- uuid
  title TEXT,
  path TEXT NOT NULL UNIQUE,
  imported_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE words (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  base_form TEXT NOT NULL UNIQUE,
  first_seen_form TEXT NOT NULL,
  occurrence_count INTEGER DEFAULT 1,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE word_contexts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  word_id INTEGER NOT NULL,
  document_id TEXT NOT NULL,
  page_number INTEGER,
  sentence TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (word_id) REFERENCES words (id) ON DELETE CASCADE,
  FOREIGN KEY (document_id) REFERENCES documents (id) ON DELETE CASCADE
);

CREATE INDEX idx_words_base_form ON words(base_form);
CREATE INDEX idx_word_contexts_word_id ON word_contexts(word_id);
```

## 5. API Endpoints (Rust Async Functions)

```rust
async fn open_document(path: String) -> Result<ReflowDocument, String>
async fn process_word_selection(payload: WordSelectionRequest) -> Result<WordSelectionResponse, String>
async fn get_word_list(filter: Option<String>, page: Option<i32>, sort: Option<String>) -> Result<Vec<WordSummary>, String>
async fn get_word_contexts(word_id: i64) -> Result<Vec<Context>, String>
```

## 6. Module Specifications (goidev-core)

### 6.1 pdf_parser.rs

Purpose: Parse PDF into TextChunks (text + coords + font info) using lopdf.
Key Logic: Lazy parse visible pages; use Tokio for background parsing.

Copilot Prompt:

```rust
// pdf_parser.rs
pub struct TextChunk {
    text: String,
    bbox: BBox,
    font_size: f32,
}
pub struct BBox {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}
// Generate a function parse_pdf(path: String, start_page: u32, end_page: u32) -> Result<Vec<TextChunk>, String> {
//   // Use lopdf to extract text chunks with coordinates and font info
//   // Support lazy parsing for specified page range
//   // Handle errors with meaningful messages
// }
```

### 6.2 reflow_engine.rs

Purpose: Group TextChunks into blocks (paragraphs, headers) with heuristics.
Key Logic: Join hyphenated words; assign confidence scores; fallback to pdfium-render for images.

Copilot Prompt:

```rust
// reflow_engine.rs
pub struct Block {
    id: String,
    kind: BlockKind,
    text: Option<String>,
    confidence: f32,
    bbox: BBox,
}
pub enum BlockKind {
    Paragraph,
    Heading,
    ImageFallback,
}
// Generate a function reflow_page(chunks: Vec<TextChunk>) -> Result<Vec<Block>, String> {
//   // Group chunks by proximity and font size into paragraphs/headers
//   // Join hyphenated words across lines
//   // Assign confidence scores (0.0-1.0) based on layout consistency
//   // Use pdfium-render for image fallback if text confidence < 0.5
// }
```

### 6.3 ai_processor.rs

Purpose: Optional text correction with Candle + quantized SLM (e.g., CoEdit).
Key Logic: Toggle via config; run on CPU; skip if disabled.

Copilot Prompt:

```rust
// ai_processor.rs
// Generate a function correct_text(text: String, enable_ai: bool) -> Result<(String, f32), String> {
//   // If enable_ai, use Candle with quantized CoEdit SLM for text correction
//   // Return corrected text and confidence score
//   // Fallback to original text with 0.9 confidence if disabled
// }

// ai_processor.rs
  //pub async fn generate_definition(word: &str, context_sentence: &str, enable_ai: bool) -> Result<(String, f32), String> {
      // If enable_ai: run Candle or local model to produce a concise definition scoped to the sentence.
      // Return (definition_text, confidence_score).
      // If disabled or model missing: return ("", 0.0) or a short fallback.
  //}
```

### 6.4 nlp_engine.rs

Purpose: Segment sentences (unicode_segmentation), normalize words (waken_snowball).
Key Logic: Find first-match sentence; stem to base_form.

Copilot Prompt:

```rust
// nlp_engine.rs
// Generate a function process_selection(word: String, block_text: String) -> Result<(String, String), String> {
//   // Clean word (trim, lowercase, strip punctuation)
//   // Stem with waken_snowball to get base_form
//   // Use unicode_segmentation to segment block_text and find first sentence containing word
//   // Return (base_form, sentence)
// }
```

### 6.5 storage_layer.rs

Purpose: Manage SQLite DB with migrations and transactions.
Key Logic: Upsert words, insert contexts, ensure consistency.

Copilot Prompt:

```rust
// storage_layer.rs
pub struct WordSelectionRequest {
    document_id: String,
    page_number: u32,
    selected_word: String,
    block_text: String,
}
pub struct WordSelectionResponse {
    base_form: String,
    sentence: String,
    occurrence_count: i64,
}
// Generate a function save_word_and_context(payload: WordSelectionRequest, base_form: String, sentence: String) -> Result<i64, String> {
//   // Use rusqlite to create tables if not exists
//   // Transaction: upsert words (increment occurrence_count), insert word_contexts
//   // Return new occurrence_count
// }
```

### 6.6 api.rs

Purpose: Handle async calls from UI.

Copilot Prompt:

```rust
// api.rs
// Generate async fn process_word_selection(payload: WordSelectionRequest) -> Result<WordSelectionResponse, String> {
//   // Call nlp_engine to get base_form and sentence
//   // Call storage_layer to save and get occurrence_count
//   // Return JSON-compatible response
// }
```

## 7. Dioxus UI Specifications (dioxus-ui)

### 7.1 app.rs

Purpose: Root component, manages document state.

Copilot Prompt:

```rust
// app.rs
// Generate a Dioxus component App that initializes dioxus-logger and manages a Resource<Option<ReflowDocument>>.
//   // Render ReflowViewer if document loaded, else show file picker.
//   // Include WordSidePanel and WordCaptureToast.
```

### 7.2 reflow_viewer.rs

Purpose: Continuous-scroll viewer with lazy loading.

Copilot Prompt:

```rust
// reflow_viewer.rs
// Generate a Dioxus component ReflowViewer with prop ReadSignal<Option<ReflowDocument>>.
//   // Implement continuous scroll with virtualization.
//   // Render Page components lazily.
//   // On dblclick in blocks, capture selection and call process_word_selection async.
```

### 7.3 components/page.rs

Purpose: Render blocks for a single page.

Copilot Prompt:

```rust
// components/page.rs
// Generate a Dioxus component Page with prop page: Page.
//   // Render list of Block components.
//   // Pass dblclick handler to blocks.
```

### 7.4 components/block.rs

Purpose: Render paragraph/heading/image; handle dblclick.

Copilot Prompt:

```rust
// components/block.rs
// Generate a Dioxus component Block with prop block: Block.
//   // Render text or image fallback based on confidence.
//   // On dblclick, use window.getSelection() to extract selected word and block text.
```

### 7.5 components/word_capture_toast.rs

Purpose: Show confirmation of saved word.

Copilot Prompt:

```rust
// components/word_capture_toast.rs
// Generate a Dioxus component WordCaptureToast with prop response: Option<WordSelectionResponse>.
//   // Show ephemeral toast with word, base_form, and occurrence_count.
```

### 7.6 components/word_side_panel.rs

Purpose: Display captured words and contexts.

Copilot Prompt:

```rust
// components/word_side_panel.rs
// Generate a Dioxus component WordSidePanel that fetches word list via get_word_list async.
//   // Display words; on click, fetch contexts with get_word_contexts.
```

## 8. Roadmap (8-10 Weeks)

Weeks 1-3: Setup repo, implement pdf_parser, reflow_engine, basic ReflowViewer.
Weeks 4-6: Add nlp_engine, storage_layer, process_word_selection, SidePanel.
Weeks 7-10: Polish UI, add tests, optional ai_processor, optimize with Tokio.

## 9. Edge Cases

Multi-word Selection: Tokenize and store individual words.
Scanned PDFs: Use pdfium-render for image fallback; skip OCR for MVP.
Hyphenated Words: Join in reflow_engine.
Large PDFs: Lazy parse, LRU cache for blocks.

## 10. Testing

Unit: reflow_engine, nlp_engine, storage_layer.
Integration: End-to-end PDF-to-vocab flow.
Metrics: Parse time, memory usage (local).

## 11. Notes for Copilot

Use log::info! for debugging.
Ensure async calls are non-blocking (Tokio).
Validate inputs (e.g., clean selectedWord).
Run cargo clippy for Rust style.
