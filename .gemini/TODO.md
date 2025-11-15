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

Current Tasks

- [x] test(pdf_parser): Add red tests for happy path, empty page, and invalid page — refs: M1 mode: L0
- [>] Implement `goidev-core/src/pdf_parser.rs` MVP and tests — refs: M1 mode: L0 #high
- [ ] feat(pdf_parser): Refactor to process page content stream — refs: M1 mode: L0 #high
- [ ] feat(pdf_parser): Implement state tracking for text matrix (Tm) and font size (Tf) — refs: M1 mode: L0 #high
- [ ] feat(pdf_parser): Handle Tj/TJ operators to create TextChunks with correct position and font size — refs: M1 mode: L0 #high

Keep TODO.md concise and actionable.
