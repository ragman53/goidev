# GOIDEV: AI-Enhanced PDF Reader & Vocabulary Builder

## Architecture Overview

GOIDEV is a local-first PDF reader with vocabulary capture built in Rust using Dioxus for cross-platform UI (desktop/web). The system processes PDFs into reflowed text blocks for continuous-scroll reading, captures vocabulary through double-click selection, and stores words with contexts in SQLite.

**Core Components:**
- `goidev-core/`: Rust backend crate with PDF parsing, text reflow, NLP processing, and storage
- `dioxus-ui/`: Frontend with continuous-scroll viewer and vocabulary management UI

**Data Flow:**
```
PDF → TextChunks (lopdf) → Blocks (reflow_engine) → UI Display
Word Selection → NLP Processing (nlprule/rust-stemmers) → SQLite Storage
```

## Key Patterns & Conventions

### Async API Layer
Use async functions for all UI-backend communication. Return `Result<T, String>` for error handling.

```rust
// api.rs pattern
pub async fn process_word_selection(payload: WordSelectionRequest) -> Result<WordSelectionResponse, String> {
    // Call nlp_engine, then storage_layer
}
```

### Data Contracts
Use serde-derived structs for JSON-compatible request/response. Document IDs use UUID v4, word IDs auto-increment.

```rust
#[derive(Serialize, Deserialize)]
pub struct WordSelectionRequest {
    document_id: String,  // UUID
    page_number: u32,
    selected_word: String,
    block_text: String,
}
```

### SQLite Storage with Transactions
Use rusqlite transactions for data consistency. Upsert patterns for words, insert-only for contexts.

```rust
// storage_layer.rs pattern
pub fn save_word_and_context(/* params */) -> Result<i64, String> {
    let conn = get_connection()?;
    let tx = conn.transaction()?;
    // Upsert word, insert context
    tx.commit()?;
}
```

### Text Processing Pipeline
Clean inputs, stem words, segment sentences. Use confidence scores (0.0-1.0) for text quality.

```rust
// nlp_engine.rs pattern
pub fn process_selection(word: String, block_text: String) -> Result<(String, String), String> {
    let cleaned = word.trim().to_lowercase();  // Remove punctuation
    let base_form = stemmer.stem(&cleaned);
    let sentence = nlprule_segment_and_find(block_text, &word)?;
    Ok((base_form, sentence))
}
```

### Reflow Engine Heuristics
Group TextChunks by proximity/font size into Blocks. Join hyphenated words, assign confidence scores.

```rust
// reflow_engine.rs pattern
pub fn reflow_page(chunks: Vec<TextChunk>) -> Result<Vec<Block>, String> {
    // Proximity grouping, hyphen joining, confidence scoring
    // Fallback to pdfium-render if confidence < 0.5
}
```

## Developer Workflows

### Project Setup
```bash
# Create crates
cargo new goidev-core
cargo new dioxus-ui --bin

# Add dependencies (see spec.md for full lists)
cd dioxus-ui
cargo add dioxus --features desktop,web
cargo add goidev-core --path ../goidev-core

# Install tools
cargo install dioxus-cli

# Run development servers
dioxus serve --platform desktop  # Desktop app
dioxus serve --platform web      # Web app
```

### Build & Test Commands
```bash
# Standard Rust workflow
cargo check          # Fast compilation check
cargo build          # Debug build
cargo build --release  # Optimized build
cargo clippy         # Linting
cargo test           # Run tests

# Dioxus-specific
dioxus build --platform desktop  # Build desktop binary
dioxus build --platform web      # Build web assets
```

### Debugging
- Use `log::info!()` for debugging output (initialized via dioxus-logger)
- Async calls are non-blocking via Tokio runtime
- Validate inputs: trim/lowercase words, check UUID formats
- Monitor memory usage for large PDFs (lazy loading required)

## Integration Points

### External Dependencies
- **lopdf**: PDF text/position extraction
- **nlprule**: Sentence boundary detection
- **rust-stemmers**: Word normalization to base forms
- **rusqlite**: Local SQLite storage with transactions
- **pdfium-render**: Image fallback for scanned PDFs
- **candle-core** (optional): AI text correction with quantized models

### Cross-Component Communication
- UI calls backend via async function calls
- Backend returns JSON-serializable structs
- Error handling: String messages for user display
- State management: Dioxus signals for document/view state

## File Structure Reference

- `.github/instructions/spec.md`: Complete technical specification
- `goidev-core/src/`: Core modules (pdf_parser, reflow_engine, nlp_engine, storage_layer, api)
- `dioxus-ui/src/`: UI components (app, reflow_viewer, components/)
- `docs/`: Architecture documentation (currently empty)

## Quality Gates

- Run `cargo clippy` for style consistency
- Unit test core logic (reflow_engine, nlp_engine, storage_layer)
- Integration test end-to-end PDF-to-vocabulary flow
- Performance: Lazy PDF parsing, LRU block caching
- Local-first: No external API dependencies for core functionality</content>
<parameter name="filePath">c:\Users\ragma\dev\goidev\.github\copilot-instructions.md