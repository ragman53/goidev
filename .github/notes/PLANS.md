# GOIDEV ExecPlan: Local-First PDF Reader & Vocabulary Builder

This ExecPlan is the canonical project plan. Keep Progress, Surprises, Decision Log, and Outcomes updated as work proceeds. Treat this file as the single source of truth for scope, steps, and acceptance.

## Purpose / Big Picture

Users can open a local PDF, extract text with position and font size, reflow it into readable blocks, and capture vocabulary by selecting a word. Captured data (base form + sentence context) is stored in SQLite. A Tauri-based desktop UI demonstrates end-to-end behavior, chosen for its stability, cross-platform capabilities, and potential for AI extensions.

## Progress

- [x] (2025-10-16) Establish ExecPlan with vibe-coding and agent workflow.
- [x] (2025-11-16) Architectural shift from Dioxus to Tauri for better stability and cross-platform support. Updated DESIGN.md and PLANS.md.
- [x] (2025-11-17) Scaffolded the new project structure (`goidev-core`, `src-tauri`, `ui`) as defined in `DESIGN.md`.
- [x] Milestone 1: pdf_parser MVP — Operator parsing with PdfState implemented; comprehensive text extraction working.
- [x] Milestone 2: reflow_engine — group TextLines into Blocks (paragraphs/headings) with heuristics (tests).
- [x] (2025-11-20) **Code Cleanup**: Cleaned and refactored `goidev-core` directory. Removed ~2.5 MB of temporary debug files, consolidated test suite, added comprehensive documentation.
- [x] (2025-11-20) **Encoding Fixes**: Resolved PDF text encoding issues (custom ligatures, special quotes, WinAnsiEncoding) with comprehensive `decode_pdf_str` function.
- [x] (2025-11-20) **Encoding Refactor**: Replaced hardcoded `decode_pdf_str` with generic `FontEncoding` system (ToUnicode, Encoding/Differences, WinAnsi fallback).
- [x] Milestone 3: Basic UI & Integration — Tauri command to invoke reflow; Leptos UI to render blocks.
- [x] (2025-12-01) **Enhanced Reflow**: Added heading levels (L1 ≥16pt, L2 ≥13pt), doc_page_num extraction, paragraph indentation detection.
- [x] (2025-12-01) **Role Detection**: Implemented multi-stage block classification (Header/Footer/Footnote/Caption/Citation/PageNumber).
- [x] (2025-12-02) **File Selection UI**: Added Browse button with tauri-plugin-dialog for PDF/Markdown file picker.
- [x] (2025-12-02) **Cache Directory Fix**: Moved cache files from sidecar to `~/.cache/goidev/` (AppData on Windows) to prevent file watcher triggers.
- [x] (2025-12-02) **23 Tests Passing**: Comprehensive test coverage for pdf_parser, reflow_engine, markdown roundtrip.
- [ ] Milestone 4: Word & Sentence Capture — NLP engine + Storage layer + UI selection for language learning.
- [ ] Milestone 5: Side Panel — vocabulary list display, word review, spaced repetition hooks.
- [ ] Polish: logging, perf pass, docs, and markdown exporter for RAG.

## Surprises & Discoveries

- **PDF Encoding Challenges (2025-11-20)**: Academic PDFs (e.g., Sage Publications) use custom byte mappings:

  - `0x8F`, `0x90` for single quotes (not standard WinAnsi)
  - `0x93`, `0x94` for ligatures `"fi"`, `"fl"` in some contexts, but double quotes in others
  - `0x02`, `0x03` for ligatures `"ffi"`, `"ff"`
  - Solution: Context-aware decoding in `decode_pdf_str` that detects custom encoding presence.
  - **Evidence**: `test-1.pdf` originally produced garbled text ("・ｽ") which is now correctly decoded.

- **Hidden Font Resources & ToUnicode (2025-11-20)**:
  - **Issue**: Some fonts (e.g., `F4` in test PDF) had no `Encoding` dictionary but used a `ToUnicode` CMap for mapping custom glyphs. `lopdf` also failed to find `Resources` on subsequent pages, implying reliance on inherited/global state.
  - **Solution**: Implemented `ToUnicode` parsing and persisted a `font_map` across pages to handle missing resource dictionaries.

- **Test Organization**: Initial test structure had overlapping files (`integration_tests.rs`, `reproduction_test.rs`) and many temporary debug output files. Consolidation improved clarity and reduced disk usage by 2.5 MB.

## Decision Log

- **Decision**: Switched from Dioxus to Tauri as the main UI framework.
  - **Rationale**: Tauri offers greater stability, a clear path to mobile and desktop deployment, and better integration opportunities for future AI extensions. This aligns with the long-term vision of the project.
  - **Date/Author**: 2025-11-16 / Agents & User
- **Decision**: Keep font_size in TextLine for heading detection and layout heuristics.
  - **Rationale**: Enables better grouping and confidence scoring even if reflow does not immediately require it.
  - **Date/Author**: 2025-10-16 / Agents
- **Decision**: Implement context-aware PDF string decoding instead of pure WinAnsiEncoding.
  - **Rationale**: Some publishers (e.g., Sage) use custom byte mappings for ligatures and quotes that vary by context. A single fixed mapping table would produce incorrect output. Solution: detect custom encoding usage and apply context-dependent transformations.
  - **Date/Author**: 2025-11-20 / Agents
- **Decision**: Consolidate integration test files and mark debug utilities as `#[ignore]`.
  - **Rationale**: Reduced duplication between `integration_tests.rs` and `reproduction_test.rs`. Debug utilities like `debug_encoding.rs` should not run in standard test suite but remain available for troubleshooting.
  - **Date/Author**: 2025-11-20 / Agents
- **Decision**: Switch from hardcoded PDF decoding to generic font parsing.
  - **Rationale**: Hardcoding resolved specific byte issues but failed when fonts lacked explicit Encoding entries or used ToUnicode. A generic solution parsing `ToUnicode` and `Differences` is robust across different PDFs.
  - **Date/Author**: 2025-11-20 / Agents

## Outcomes & Retrospective

- **M1 - Initial Implementation (2025-10-17)**: Successfully implemented a basic `parse_pdf` function using `lopdf::Document::extract_text`. This function passes the initial "happy path" test by returning a `Vec<TextChunk>` that is not empty. The current implementation uses dummy values for `bbox` and `font_size`, which will be addressed in the refactoring step. This completes the first "Red -> Green" cycle.

  - **Vibe Reflection**: The `lopdf::extract_text` helper is a great way to get a quick win and validate the overall structure. It hides a lot of complexity, which is perfect for a first pass but insufficient for our final goal of getting detailed coordinates.

- **Project Scaffolding (2025-11-17)**: Successfully created the initial project structure using `cargo tauri init`. This sets up the `src-tauri` directory for the backend, and we have placeholders for the `goidev-core` and `ui` (Leptos) crates. The application runs in development mode with `cargo tauri dev`.

  - **Vibe Reflection**: Starting with the standard Tauri template gives us a solid, working foundation. We can now incrementally build out the `goidev-core` logic and the Leptos UI, knowing the shell is stable.

- **M1 - pdf_parser MVP (2025-11-19)**: Implemented `parse_pdf` using `lopdf`.

  - **Verification**: `cargo test` passes. `test_extract_text_with_position` confirms text extraction.
  - **Vibe Reflection**: `lopdf` works well for basic extraction. BBox and font size are currently placeholders, to be refined in M2/M3.

- **M2 - reflow_engine (2025-11-19)**: Implemented `ReflowEngine::process` to group `TextLine` instances into logical blocks.

  - **Verification**: `cargo test` passes. Unit tests verify paragraph grouping and heading detection based on font size.
  - **Vibe Reflection**: Heuristic-based reflow works well for simple layouts. May need refinement for complex multi-column or mixed-font documents.

- **Encoding Fixes (2025-11-20)**: Resolved garbled text issues in academic PDFs through context-aware `decode_pdf_str` function.

  - **Challenge**: Sage Publications PDFs use non-standard byte mappings (e.g., `0x8F`/`0x90` for quotes, `0x93`/`0x94` context-dependent for ligatures vs quotes).
  - **Solution**: Detect custom encoding presence and apply context-dependent transformations.
  - **Verification**: `test_reflow_complex_pdf` now passes with correctly decoded text; no more \"・ｽ\" garbled characters.
  - **Vibe Reflection**: Real-world PDFs are messier than expected. The decode function now handles UTF-16BE, WinAnsiEncoding, and custom publisher encodings robustly.

- **Code Cleanup (2025-11-20)**: Cleaned and refactored `goidev-core` directory for better maintainability.
  - **Removed**: 10 temporary files (~2.5 MB): debug outputs, experimental Rust files, Python analysis script.
  - **Consolidated**: Merged `integration_tests.rs` and `reproduction_test.rs`; marked `debug_encoding.rs` as `#[ignore]`.
  - **Improved**: Added comprehensive module-level documentation to all test files.
  - **Verification**: All 6 tests pass; build succeeds; `cargo tauri dev` runs successfully.
  - **Vibe Reflection**: Clean codebase makes it easier to navigate and maintain. Good practice to do periodic cleanup passes.

- **Generic Font Decoding (2025-11-20)**: Refactored `pdf_parser` to be fully context-aware.
  - **Implementation**: Parsing `ToUnicode` CMaps, `Encoding` dictionaries with `Differences`, and fallback to `WinAnsi`.
  - **Verification**: `test_reflow_complex_pdf` passes without any hardcoded replacement logic.
  - **Vibe Reflection**: Moving from "make it work" (hardcoding) to "make it right" (generic) was essential when the hardcoded solution failed on slightly different font structures (`F4` missing Encoding entry).

- **M3 - Basic UI & Integration (2025-11-21)**: Implemented `open_document` Tauri command and `ReflowViewer` component.
  - **UI Features**: Page-aware rendering with alternating backgrounds (`#ffffff` / `#f8f9fa`), explicit text color (`#333333`), and page numbering.
  - **Engine Update**: ReflowEngine now respects page boundaries (no merge across pages) and correctly handles vertical gaps for both Y-up (PDF) and Y-down (Screen) coordinates.
  - **Verification**: Manual visual verification of PDF rendering shows clear page separation and correct paragraph grouping.
  - **Vibe Reflection**: Simple visual cues like alternating backgrounds significantly improve the reading experience compared to a continuous wall of text. Explicit styling is crucial when moving from raw data to UI.

## Data Contracts (Canonical)

## Milestone 1 — pdf_parser (MVP)

Goal:

- Implement parse_pdf(path, start_page, end_page) -> `Result<Vec<TextLine>, String>`
- Ensure tests: one happy path and one edge case (empty page / missing ToUnicode)

Learning context:

- Mode L0 (Assisted Execution) — focus on reading traced reference code, retyping critical sections, and capturing questions about lopdf text operators.

Acceptance criteria:

- Passing unit tests demonstrating text extraction for a small test PDF.
- Returns TextChunk with text, bbox (points, origin=bottom-left), font_size.
- Document parse failures with clear error messages and non-fatal per-page warnings when appropriate.
- Provide a small CLI or test harness that prints extracted chunks for manual inspection.

Notes and constraints:

- Use lopdf for content stream decoding.
- Reference `.gemini/references/pdf` for operator handling, font decoding, and PdfState patterns.
- For async callers, use tokio::task::spawn_blocking to avoid blocking the runtime.

Concrete steps:

1. Add TextChunk and BBox types to goidev-core/src/pdf_parser.rs.
2. Implement parse_pdf that:
   - Loads Document with lopdf.
   - Validates 1-based page range.
   - For each page: decode Content and interpret BT/ET, Tf, Tm, Td, TD, T\*, Tj, TJ to build TextChunk entries.
   - Use Document::decode_text per-font where available; fall back to lossy UTF-8/UTF-16.
3. Write tests:
   - test_extract_text_simple: verifies non-empty TextChunk on a sample PDF page.
   - test_empty_page: verifies function returns empty Vec for pages with no text without panicking.
4. Add minimal CLI or integration test that prints chunks.
5. Update Progress and Validation outputs.

## Integration with RAG/Markdown

- Idea: docling + Py03 <-> Rust
- Purpose: parsing PDF documents and reflow.

---

## Acceptance & Verification (M1 example)

- **Initial Happy Path Test (2025-10-17)**: PASSED

  - **Run unit tests**:

    ```shell
    cd goidev-core
    cargo test -- --nocapture
    ```

  - **Output**:

    ```c
    running 1 test
    test tests::pdf_parser_test::test_extract_text_simple_happy_path ... ok

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s
    ```

- Keep structured outputs (TextChunk → Block) as canonical.
- Add deterministic to_markdown(blocks: &[Block]) -> String exporter to produce consistent Markdown for RAG/LLM ingestion.
- Persist both canonical structure and Markdown to storage for retrieval.

## Operational Notes

- Use TODO.md for short actionable tasks derived from PLANS.md.
- Keep tests green and include verification snippets in PLANS.md Validation.
- Translate PLANS.md and AGENTS.md to English as canonical; maintain optional localized summaries (README_JA.md) if needed.

## Learning Context Tracking

- Each milestone lists the active learning mode; update it when the team escalates (L0 → L1 → L2 → L3).
- When the mode changes, add a brief note to Progress and the Decision Log summarizing why the shift happened.
- Capture Learner reflections in Outcomes & Retrospective with the tag **Vibe Reflection**.

## Next Steps (after M1)

- Build a minimal Tauri viewer that renders blocks (M3). ✅ DONE
- Add storage_layer (M4) and nlp_engine (M5). ← **CURRENT FOCUS**
- Implement word collection flow (M6).

---

## Milestone 4 — Word & Sentence Capture (Language Learning Core)

**Goal**: Enable users to select words/sentences from reflowed PDF content and save them to a local vocabulary database for language learning.

**Learning context**: Mode L1 (Guided Implementation) — focus on NLP patterns, SQLite schema design, and UI text selection.

### Data Contracts (New)

**WordEntry** (stored in SQLite):
```rust
pub struct WordEntry {
    pub id: i64,
    pub word: String,           // Original selected word
    pub base_form: String,      // Lemmatized form (e.g., "running" → "run")
    pub sentence: String,       // Full sentence context
    pub source_doc: String,     // Document title/path
    pub page_num: u32,          // Page where captured
    pub created_at: i64,        // Unix timestamp
    pub review_count: u32,      // For spaced repetition
    pub next_review: Option<i64>, // Next review timestamp
}
```

**CaptureWordRequest** (UI → Backend):
```rust
pub struct CaptureWordRequest {
    pub word: String,
    pub block_text: String,     // Full block text for sentence extraction
    pub doc_id: String,
    pub page_num: u32,
}
```

**CaptureWordResponse** (Backend → UI):
```rust
pub struct CaptureWordResponse {
    pub success: bool,
    pub entry: Option<WordEntry>,
    pub error: Option<String>,
}
```

### Architecture

```
User clicks word in UI
        ↓
ReflowViewer detects selection → CaptureWordRequest
        ↓
Tauri command: capture_word()
        ↓
┌─────────────────────────────────────────┐
│ nlp_engine.rs                           │
│  - extract_sentence(block_text, word)   │
│  - get_base_form(word)                  │
└─────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────┐
│ storage_layer.rs                        │
│  - save_word(WordEntry)                 │
│  - get_vocabulary() → Vec<WordEntry>    │
│  - update_review(id, next_review)       │
└─────────────────────────────────────────┘
        ↓
CaptureWordResponse → UI (SidePanel updates)
```

### Implementation Steps

1. **nlp_engine.rs** (Pure Rust NLP):
   - Sentence boundary detection using `unicode_segmentation`
   - Word tokenization with punctuation handling
   - Base form extraction using `rust-stemmers` (Snowball algorithm)
   - Language detection hint (optional)

2. **storage_layer.rs** (SQLite):
   - Schema: `words` table with fields above
   - Indices on `base_form`, `created_at`, `next_review`
   - CRUD operations: save, get_all, get_by_base_form, update_review
   - Use `rusqlite` with bundled SQLite

3. **UI Text Selection**:
   - Enable word selection on click/tap in ReflowViewer
   - Highlight selected word
   - Show sentence context in tooltip/popup
   - "Save" button to trigger capture

4. **SidePanel Component**:
   - Display vocabulary list
   - Search/filter by word
   - Show review status
   - Export to Anki/CSV (future)

### Acceptance Criteria

- [ ] User can click a word in the viewer to select it
- [ ] Selected word's sentence is automatically extracted
- [ ] Base form (lemma) is computed for the word
- [ ] Word + context saved to SQLite database
- [ ] SidePanel shows captured vocabulary
- [ ] 5+ unit tests for nlp_engine
- [ ] 3+ integration tests for storage_layer

### Dependencies to Add

```toml
# goidev-core/Cargo.toml
unicode-segmentation = "1.12"
rust-stemmers = "1.2"
rusqlite = { version = "0.37", features = ["bundled"] }
```

---
