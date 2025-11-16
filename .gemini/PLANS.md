# GOIDEV ExecPlan: Local-First PDF Reader & Vocabulary Builder

This ExecPlan is the canonical project plan. Keep Progress, Surprises, Decision Log, and Outcomes updated as work proceeds. Treat this file as the single source of truth for scope, steps, and acceptance.

## Purpose / Big Picture

Users can open a local PDF, extract text with position and font size, reflow it into readable blocks, and capture vocabulary by selecting a word. Captured data (base form + sentence context) is stored in SQLite. A Tauri-based desktop UI demonstrates end-to-end behavior, chosen for its stability, cross-platform capabilities, and potential for AI extensions.

## Progress

- [x] (2025-10-16) Establish ExecPlan with vibe-coding and agent workflow.
- [ ] (2025-11-16) Architectural shift from Dioxus to Tauri for better stability and cross-platform support. Updated DESIGN.md and PLANS.md.
- [ ] Milestone 1: pdf_parser MVP — extract TextChunk (text + bbox + font_size) from page ranges (unit tests + CLI/demo).
- [ ] Milestone 2: reflow_engine — group TextChunks into Blocks (paragraphs/headings) with heuristics (tests).
- [ ] Milestone 3: storage_layer — DB schema and functions to persist words and contexts (tests).
- [ ] Milestone 4: nlp_engine — extract base form and sentence from a block (tests). 
- [ ] Milestone 5: api + UI — wire pipeline; Tauri UI supports double-click capture and displays captured words.
- [ ] Polish: logging, perf pass, docs, and markdown exporter for RAG.

## Surprises & Discoveries

- Record parsing quirks, font encoding issues, layout edge-cases, and Windows path observations here with short evidence snippets.

## Decision Log

- **Decision**: Switched from Dioxus to Tauri as the main UI framework.
  - **Rationale**: Tauri offers greater stability, a clear path to mobile and desktop deployment, and better integration opportunities for future AI extensions. This aligns with the long-term vision of the project.
  - **Date/Author**: 2025-11-16 / Agents & User
- **Decision**: Keep font_size in TextChunk for heading detection and layout heuristics.
  - **Rationale**: Enables better grouping and confidence scoring even if reflow does not immediately require it.
  - **Date/Author**: 2025-10-16 / Agents

## Outcomes & Retrospective

- **M1 - Initial Implementation (2025-10-17)**: Successfully implemented a basic `parse_pdf` function using `lopdf::Document::extract_text`. This function passes the initial "happy path" test by returning a `Vec<TextChunk>` that is not empty. The current implementation uses dummy values for `bbox` and `font_size`, which will be addressed in the refactoring step. This completes the first "Red -> Green" cycle.
  - **Vibe Reflection**: The `lopdf::extract_text` helper is a great way to get a quick win and validate the overall structure. It hides a lot of complexity, which is perfect for a first pass but insufficient for our final goal of getting detailed coordinates.

- Populate after each milestone: what was achieved, verification steps, and remaining gaps.

## Data Contracts (Canonical)

## Milestone 1 — pdf_parser (MVP)

Goal:

- Implement parse_pdf(path, start_page, end_page) -> `Result<Vec<TextChunk>, String>`
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
- Reference `.github/references/pdf` for operator handling, font decoding, and PdfState patterns.
- For async callers, use tokio::task::spawn_blocking to avoid blocking the runtime.

Concrete steps:

1. Add TextChunk and BBox types to goidev-core/src/pdf_parser.rs.
2. Implement parse_pdf that:
   - Loads Document with lopdf.
   - Validates 1-based page range.
   - For each page: decode Content and interpret BT/ET, Tf, Tm, Td, TD, T*, Tj, TJ to build TextChunk entries.
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

- Implement reflow_engine grouping heuristics.
- Build a minimal Tauri viewer that renders blocks and supports double-click selection to call the pipeline via Tauri commands.
- Add storage_layer and nlp_engine integration.

Keep this document current. Update Progress, Decision Log, Surprises, and Outcomes as you complete steps.