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

## Current Tasks (Milestone 3)

- [x] **fix(pdf_parser):** Implement generic `ToUnicode` and `Encoding` handling to fix text decoding. — refs: M2 mode: L3 #high
- [ ] **feat(tauri):** Implement `open_document` Tauri command to call `parse_pdf` and `ReflowEngine`. — refs: M3 mode: L0 #high
- [ ] **feat(ui):** Create `ReflowViewer` Leptos component to display blocks. — refs: M3 mode: L0 #high
- [ ] **feat(ui):** Integrate file picker to open PDF. — refs: M3 mode: L0 #medium

Keep TODO.md concise and actionable.
