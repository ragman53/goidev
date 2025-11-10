# GOIDEV ExecPlan: Local-First PDF Reader & Vocabulary Builder

This ExecPlan is the canonical project plan. Keep Progress, Surprises, Decision Log, and Outcomes updated as work proceeds. Treat this file as the single source of truth for scope, steps, and acceptance.

## Purpose / Big Picture

Users can open a local PDF, extract text with position and font size, reflow it into readable blocks, and capture vocabulary by selecting a word. Captured data (base form + sentence context) is stored in SQLite. A small Dioxus-based desktop UI demonstrates end-to-end behavior.

## Progress

- [x] (2025-10-16) Establish ExecPlan with vibe-coding and agent workflow.
- [ ] Milestone 1: pdf_parser MVP — extract TextChunk (text + bbox + font_size) from page ranges (unit tests + CLI/demo).
- [ ] Milestone 2: reflow_engine — group TextChunks into Blocks (paragraphs/headings) with heuristics (tests).
- [ ] Milestone 3: storage_layer — DB schema and functions to persist words and contexts (tests).
- [ ] Milestone 4: nlp_engine — extract base form and sentence from a block (tests).
- [ ] Milestone 5: api + UI — wire pipeline; Dioxus UI supports double-click capture and displays captured words.
- [ ] Polish: logging, perf pass, docs, and markdown exporter for RAG.

## Surprises & Discoveries

- Record parsing quirks, font encoding issues, layout edge-cases, and Windows path observations here with short evidence snippets.

## Decision Log

- Decision: Keep font_size in TextChunk for heading detection and layout heuristics.
  - Rationale: Enables better grouping and confidence scoring even if reflow does not immediately require it.
  - Date/Author: 2025-10-16 / Agents

## Outcomes & Retrospective

- Populate after each milestone: what was achieved, verification steps, and remaining gaps.

## Data Contracts (Canonical)

- ReflowDocument (backend → UI): doc_id, title, pages → blocks with id, type, text, confidence, bbox.
- TextChunk: { text: String, bbox: {x,y,w,h}, font_size: f32 }
- Block: { id, kind, text, confidence, bbox }

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

## Acceptance & Verification (M1 example)

- Run unit tests:
  - cd goidev-core
  - cargo test -- --nocapture
- Manual check:
  - Provide sample.pdf in tests/resources
  - Run small CLI/test harness to print TextChunks and inspect bbox/font_size

## Next Steps (after M1)

- Implement reflow_engine grouping heuristics.
- Build minimal Dioxus viewer that renders blocks and supports double-click selection to call the pipeline.
- Add storage_layer and nlp_engine integration.

Keep this document current. Update Progress, Decision Log, Surprises, and Outcomes as you complete steps.
