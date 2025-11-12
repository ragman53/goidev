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

- [x] Explain `.github/references/pdf` parser components — refs: M1 mode: L0
- [>] Implement `goidev-core/src/pdf_parser.rs` MVP and tests — refs: M1 mode: L0 #high
- [x] Update PLANS.md with test evidence after parser implementation — refs: M1 mode: L0
- [ ] feat(pdf_parser): Refactor to extract real bbox and font_size — refs: M1 mode: L0 #high

Keep TODO.md concise and actionable.
