# GOIDEV ExecPlan: Local-First PDF Reader & Vocabulary Builder

This ExecPlan is a living document and must be maintained in accordance with PLANS.md requirements. Contributors must update the sections Progress, Surprises & Discoveries, Decision Log, and Outcomes & Retrospective as work proceeds. Treat this file as the single source of truth for scope, steps, and acceptance.

## Purpose / Big Picture

After implementing this plan, a user can open a local PDF, parse text with position and font size, reflow it into readable blocks, and capture vocabulary by double-clicking a word, saving its base form and context to SQLite. The behavior is observable end-to-end via a small desktop UI built with Dioxus.

## Progress

- [x] (2025-10-16 00:00Z) Establish ExecPlan with vibe-coding and agent workflow alignment.
- [ ] Milestone 1: pdf_parser MVP parses text with bbox and font_size from a page range (unit tests + small CLI/demo).
- [ ] Milestone 2: reflow_engine groups TextChunks into Blocks (paragraphs/headings) with basic heuristics (tests).
- [ ] Milestone 3: storage_layer schema + functions to persist words and contexts (tests).
- [ ] Milestone 4: nlp_engine extracts base form and sentence from a block (tests).
- [ ] Milestone 5: api connects the pipeline; Dioxus UI shows reflowed pages and supports dblclick capture.
- [ ] Polish: logging, small perf pass, and docs.

## Surprises & Discoveries

- Observation: TBD as we implement. Capture decoding quirks, layout edge-cases, or Windows path issues.
    Evidence: Include short error messages or test output when discovered.

## Decision Log

- Decision: Keep font_size in TextChunk for future heading detection and layout heuristics.
    Rationale: Enables better grouping and confidence scoring even if immediate reflow does not rely on it.
    Date/Author: 2025-10-16 / Agents

## Outcomes & Retrospective

To be filled at each completed milestone with a short summary: what was achieved, how it was verified, and remaining gaps.

## Context and Orientation

Repository layout:

- goidev-core/ (Rust backend crate)
  - src/pdf_parser.rs: parse PDF pages into TextChunk { text, bbox, font_size }
  - src/reflow_engine.rs: convert TextChunks → Blocks (Paragraph/Heading/ImageFallback)
  - src/nlp_engine.rs: clean word, stem to base form, find sentence
  - src/storage_layer.rs: SQLite schema and persistence
  - src/api.rs: async functions orchestrating modules
  - Cargo.toml: lists dependencies (lopdf, rusqlite, etc.)
- dioxus-ui/ (Rust UI crate)
  - src/app.rs, src/reflow_viewer.rs, src/components/*

Assumptions:

- OS: Windows 10/11; shell: PowerShell (pwsh).
- Rust stable toolchain installed; cargo builds both crates locally.
- No external network calls during normal development.

## Plan of Work

We will implement in small, testable slices. Each milestone has clear acceptance. The Planner Agent updates Progress and Concrete Steps; the Tester writes failing tests; the Coder implements the minimal pass; the Reviewer approves; the Teacher explains what changed.

Milestone 1 (pdf_parser MVP):

- Define TextChunk and BBox in goidev-core/src/pdf_parser.rs.
- Implement parse_pdf(path, start_page, end_page) using lopdf to decode content streams and capture Tj/TJ text, tracking font size (Tf) and a simple text matrix (Tm, Td, TD, T*). Coordinates are best-effort.
- Provide a tiny test harness and unit tests asserting non-empty extraction on a sample PDF.

Milestone 2 (reflow_engine MVP):

- Implement reflow_page(chunks) to return a list of Block that groups by proximity and font_size. Join lines and hyphenation. Assign confidence scores.
- Unit tests construct synthetic TextChunks to validate grouping and confidence behavior deterministically.

Milestone 3 (storage_layer):

- Define schema creation if missing. Implement save_word_and_context(...) with transactions and upsert logic.
- Tests cover first insert and repeated occurrences increment.

Milestone 4 (nlp_engine):

- Implement process_selection(word, block_text) returning (base_form, sentence) using unicode_segmentation and waken_snowball.
- Tests cover punctuation stripping and sentence detection.

Milestone 5 (api + UI glue):

- Implement async api functions. Wire a minimal Dioxus desktop view that loads a PDF (hard-coded path or file picker), shows reflowed text blocks, and on double-click, calls process_word_selection then storage_layer.
- Manual validation via running the desktop app.

## Concrete Steps

All commands below run in Windows PowerShell (pwsh) at the repository root unless otherwise noted.

1. Ensure dependencies in goidev-core/Cargo.toml. If missing, add at the specified versions:

    - lopdf, rusqlite with bundled feature, unicode_segmentation, waken_snowball, serde with derive, uuid v4, tokio (rt, fs), log, env_logger, pdfium-render (optional fallback).

2. Implement pdf_parser.rs MVP with types and parse_pdf function. Add unit tests in goidev-core/src/lib.rs or goidev-core/tests/pdf_parser_tests.rs using a tiny embedded or generated PDF fixture. If a real file is needed, include a small, license-free generated PDF during the test using lopdf to write one.

3. Run unit tests for the core crate:

    - pwsh command:

        - cd goidev-core
        - cargo test -- --nocapture

    Expectation: the new tests compile and pass after implementation.

4. Implement reflow_engine.rs grouping logic and tests using synthetic chunks.

5. Implement nlp_engine.rs, storage_layer.rs with tests.

6. Implement api.rs orchestration and a tiny Dioxus UI path calling into goidev-core.

7. Manual run (desktop):

    - pwsh commands:

        - cd dioxus-ui
        - cargo run

    Expectation: a window opens, text appears from a known PDF; double-click captures a word and shows confirmation.

## Validation and Acceptance

Milestone 1 acceptance:

- Running cargo test in goidev-core yields passing tests including pdf_parser tests.
- parse_pdf returns a non-empty list of TextChunk for a generated trivial PDF containing simple text.

Milestone 2 acceptance:

- reflow_engine unit tests pass, verifying grouping by proximity and font size and hyphen join.

Milestone 3 acceptance:

- storage_layer tests pass: words table upsert increments occurrence_count; contexts recorded with page number.

Milestone 4 acceptance:

- nlp_engine tests pass: base form extraction and sentence detection.

Milestone 5 acceptance:

- Manual interaction works: open app, see reflowed text, double-click word, observe saved context confirmation.

Quality gates per change:

- Build: PASS, Lint/Typecheck: PASS, Tests: PASS before declaring done. Capture short outputs in this file.

## Idempotence and Recovery

- Steps are additive and safe to repeat. If a step fails, fix code and re-run cargo build/test. Database operations create tables if missing; tests should use an in-memory SQLite connection to avoid persistent state.

## Artifacts and Notes

- Keep small evidence snippets here when running tests, for example:

    Example (after Milestone 1):

        running 1 test
        test pdf_parser_extracts_text ... ok
        test result: ok. 1 passed; 0 failed; 0 ignored

## Interfaces and Dependencies

Types and signatures to exist by the end of Milestone 1:

- In goidev-core/src/pdf_parser.rs

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct BBox { pub x: f32, pub y: f32, pub w: f32, pub h: f32 }

        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct TextChunk { pub text: String, pub bbox: BBox, pub font_size: f32 }

        pub fn parse_pdf(path: String, start_page: u32, end_page: u32) -> Result<Vec<TextChunk>, String>;

By the end of Milestone 2:

- In goidev-core/src/reflow_engine.rs

        pub enum BlockKind { Paragraph, Heading, ImageFallback }

        pub struct Block { pub id: String, pub kind: BlockKind, pub text: Option<String>, pub confidence: f32, pub bbox: BBox }

        pub fn reflow_page(chunks: Vec<TextChunk>) -> Result<Vec<Block>, String>;

By the end of Milestone 3:

- In goidev-core/src/storage_layer.rs

        pub struct WordSelectionRequest { pub document_id: String, pub page_number: u32, pub selected_word: String, pub block_text: String }

        pub struct WordSelectionResponse { pub base_form: String, pub sentence: String, pub occurrence_count: i64 }

        pub fn save_word_and_context(payload: WordSelectionRequest, base_form: String, sentence: String) -> Result<i64, String>;

By the end of Milestone 4:

- In goidev-core/src/nlp_engine.rs

        pub fn process_selection(word: String, block_text: String) -> Result<(String, String), String>;

By the end of Milestone 5:

- In goidev-core/src/api.rs

        pub async fn process_word_selection(payload: WordSelectionRequest) -> Result<WordSelectionResponse, String>;

## Notes for Agents

- Always update this file as you proceed. If you diverge, correct the plan first, then code.
- Keep commands PowerShell-compatible. Prefer explicit working directories and one command per line when documenting runs.
- Document at least one happy-path and one edge-case test per feature.

## Revision Note

- 2025-10-16: Rewrote PLANS.md into a self-contained ExecPlan aligned with vibe-coding Agents and TDD. Kept font_size in TextChunk for future layout heuristics.
