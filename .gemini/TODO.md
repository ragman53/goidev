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

## Current Tasks (Milestone 2)

- [x] **feat(reflow_engine):** Implement `Block` struct and `ReflowEngine` with heuristics for paragraph/heading grouping. — refs: M2 mode: L0 #high
- [x] **test(reflow_engine):** Add unit tests for line merging and heading detection. — refs: M2 mode: L0 #high
- [x] **test(reflow_engine):** Verify reflow on real PDF (`test.pdf`). — refs: M2 mode: L0 #medium

Keep TODO.md concise and actionable.
