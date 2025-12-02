# TODO — Short-Term Task Queue

Lightweight queue for immediate work (hours → days). Every item must trace back to PLANS.md.

Guidelines

- Keep tasks finishable within a single session (max half-day).
- Reference the related PLANS.md milestone (e.g., `refs: M1`) and note the active learning mode (`mode: L0`).
- Include owner `@handle` and optional priority tag (`#high/#medium/#low`).
- Track status with `[ ]` (open), `[>]` (in-progress), `[x]` (done).
- When complete, copy a 1–3 line verification note into PLANS.md Validation, then archive within 7 days.

# TODO — Short-Term Task Queue

Lightweight queue for immediate work (hours → days). Every item must trace back to PLANS.md.

Guidelines

- Keep tasks finishable within a single session (max half-day).
- Reference the related PLANS.md milestone (e.g., `refs: M1`) and note the active learning mode (`mode: L0`).
- Include owner `@handle` and optional priority tag (`#high/#medium/#low`).
- Track status with `[ ]` (open), `[>]` (in-progress), `[x]` (done).
- When complete, copy a 1–3 line verification note into PLANS.md Validation, then archive within 7 days.

Examples

- [ ] feat(pdf_parser): add failing test for parse_pdf empty-page case — refs: M1 mode: L0 #high @ragma
- [ ] test(reflow_engine): add hyphenation edge-case test — refs: M2 mode: L1 #medium

## Completed Tasks (Milestones 1-3)

- [x] **fix(pdf_parser):** Implement generic `ToUnicode` and `Encoding` handling to fix text decoding. — refs: M2 mode: L3 #high
- [x] **feat(tauri):** Implement `open_document` Tauri command to call `parse_pdf` and `ReflowEngine`. — refs: M3 mode: L0 #high
- [x] **feat(ui):** Create `ReflowViewer` Leptos component to display blocks. — refs: M3 mode: L0 #high
- [x] **feat(ui):** Integrate file picker to open PDF. — refs: M3 mode: L0 #medium
- [x] **feat(reflow):** Add heading levels (L1/L2), doc_page_num, paragraph indentation. — refs: M3 mode: L1 #medium
- [x] **fix(cache):** Move cache files to AppData to prevent app restart on first PDF open. — refs: M3 mode: L1 #high

## Current Tasks (Milestone 4: Word & Sentence Capture)

- [ ] **feat(core):** Add `nlp_engine.rs` with sentence boundary detection. — refs: M4 mode: L1 #high @ragma
- [ ] **feat(core):** Implement word tokenization and base form extraction. — refs: M4 mode: L1 #high @ragma
- [ ] **feat(core):** Create `storage_layer.rs` with SQLite schema for words/contexts. — refs: M4 mode: L1 #high @ragma
- [ ] **feat(ui):** Enable text selection in ReflowViewer (word/sentence highlight). — refs: M4 mode: L0 #high @ragma
- [ ] **feat(tauri):** Add `capture_word` command to process selection and save to DB. — refs: M4 mode: L0 #medium @ragma
- [ ] **feat(ui):** Create SidePanel component to display captured vocabulary. — refs: M4 mode: L0 #medium @ragma

## Backlog

- [ ] **feat(ui):** Show heading styles differently (larger font, bold). — refs: M3 mode: L0 #low
- [ ] **feat(ui):** Add page navigation / jump-to-page. — refs: M3 mode: L0 #low
- [ ] **feat(core):** Implement `ai_processor.rs` with PyO3 + docling for enhanced reflow. — refs: M7 mode: L2 #low

Keep TODO.md concise and actionable.
