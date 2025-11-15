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
cargo add lopdf rusqlite --features bundled waken_snowball serde --features derive uuid --features v4 tokio --features rt,fs candle-core --optional log env_logger pdfium-render
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

/// Represents a bounding box for a text chunk or line.
/// Coordinates are in PDF points (1/72 inch), with the origin at the bottom-left of the page.
pub struct BBox { x: f32, y: f32, w: f32, h: f32 }

/// Represents a single word or contiguous piece of text on a line.
pub struct WordSpan {
    pub text: String,
    /// The x-coordinate of the span's left edge.
    pub x: f32,
    /// The width of the span.
    pub w: f32,
    /// The font size specific to this span.
    pub font_size: f32,
}

/// Represents a single line of text, containing one or more WordSpans.
pub struct TextLine {
    pub spans: Vec<WordSpan>,
    pub bbox: BBox,
}

// Generate a function parse_pdf(path: String, start_page: u32, end_page: u32) -> Result<Vec<TextLine>, String>
// Use lopdf to process the page content stream, interpreting operators like Tj, TJ, Tm, and Tf.
// Group text fragments into WordSpans and then group those into TextLines based on vertical position.
// Return a vector of TextLines for each page, which is a more semantic and efficient structure for the reflow engine.
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

### 6.3 ai_processor.rs (after MVP)

Purpose: PDF to Markdown or reflow-view by using Py03 with Docling

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
