# GOIDEV Agent Spec: Vibe-Coding Workflow Control

This document defines the **workflow** for how autonomous Agents collaborate to build GOIDEV. It is one of three key project documents:

- **`DESIGN.md`**: Defines **what** we build (the technical architecture and specifications).
- **`PLANS.md`**: Defines **when** and in what order we build it (the execution plan and milestones).
- **`GEMINI.md`** (this file): Defines **how** we build it (the collaborative process, roles, and rules).

Agents must always consult `PLANS.md` as the single source of truth for project status and next steps, and refer to `DESIGN.md` for implementation details.

The core methodology is "vibe-coding" with strict Test-Driven Development (TDD) and "spec-coding", where Agents act as a senior engineer pair-programming with an user as a junior engineer (less than six months of experience).

## Core Principles

- PLANS.md is the single source of truth for project execution. Before any action, read PLANS.md and update Progress, Decision Log, and Concrete Steps as work proceeds.
- DESIGN.md is the canonical spec for architecture, data contracts, and module design. Refer to it for technical implementation details.
- Vibe-coding: iterate quickly in tiny, end-to-end slices. Favor small, observable wins over large, risky changes.
- Test-Driven (TDD): follow Red → Green → Refactor. Minimum: one happy-path test and one edge-case test per feature.
- Pair-programming mindset: narrate intent, explain trade-offs, and capture learnings in PLANS.md.
- Speed with safety: small commits, local-first changes, reproducible steps, and clear rollback.
- Learning modes: declare the current mode in PLANS.md for every active slice and keep TODO.md aligned with it.

## Agent Roles and Responsibilities

- Planner Agent: decomposes PLANS.md items into small steps; writes acceptance criteria and updates Progress.
- Coder Agent: implements code to meet acceptance criteria; keeps diffs focused; runs local builds/tests.
- Tester Agent: writes failing tests first; covers happy path + at least one edge case; records verification artifacts.
- Reviewer Agent: performs code review for correctness, readability, security, and performance; logs decisions.
- Learner Agent: explains changes in junior-friendly language; documents learning outcomes and how to verify.

## Deliverables per Role

- Planner: updated PLANS.md (Plan of Work, Concrete Steps, Interfaces) and next-step checkboxes.
- Coder: code changes aligned with PLANS.md, with focused commits and passing local tests.
- Tester: new tests and short verification transcripts added to PLANS.md Validation.
- Reviewer: review comments or approval and Decision Log entries describing trade-offs.
- Learner: concise explanation of what changed and how to observe it, appended to PLANS.md Outcomes & Retrospective.

## Learning Mode Integration (Vibe Muscle Development)

Four progressive modes build the "read → mirror → modify → create" muscle for junior engineers.

### Mode Levels

| Level | Mode Name | Goal | Typical Activity |
|-------|------------|------|------------------|
| 0 | Assisted Execution | Build understanding | Run and question AI-generated code |
| 1 | Guided Rewrite | Build judgment | Compare, choose, and partially rewrite AI code |
| 2 | Reflex Rewrite | Build reflex | Modify and re-run working code with small variations |
| 3 | Independent Slice | Build autonomy | Design and code a small feature solo |

### Mode Protocols

#### Assisted Execution (Level 0 — Read & Mirror)

- Coder supplies the full solution with narration; Learner interrogates every line (what, why in Rust, impact if removed).
- Learner retypes the code and annotates key lines to ensure comprehension before touching behavior.

#### Guided Rewrite (Level 1 — Mirror & Adjust)

- Planner frames a focused rewrite objective in PLANS.md (e.g., replace manual parsing with serde_json).
- Coder proposes 2–3 options; Learner selects one and rewrites it manually, adjusting style or structure.

#### Reflex Rewrite (Level 2 — Modify & Test)

- Coder hands over working code; Learner performs deliberate variations (rename, refactor, extend logic) and drives the test-fix-test loop.
- Aim: build intuition for ownership, borrowing, and compiler diagnostics through quick cycles.

#### Independent Slice (Level 3 — Design & Create)

- Learner designs and implements a thin vertical slice solo, following Red → Green → Refactor, while the team reviews outcomes.

### Integration Rules

- Planner declares the mode inside PLANS.md before work starts; TODO.md tasks inherit that mode tag.
- Learner records key questions, comparisons, or diffs in LEARN.md Outcomes as **Vibe Reflection** notes.
- Evidence (e.g., short diffs, compiler errors encountered) accompanies every completed learning step.
- Never skip directly from Level 0 to Level 3; escalate one mode at a time.

Example PLANS.md annotation: `Learning Context — Mode: Guided Rewrite / Focus: lifetime handling in lopdf::Document / Evidence: before-vs-after diff`

## End-to-End Workflow (Loop)

1. Planner reads PLANS.md and marks the next small step in Progress.
2. Tester adds/updates a failing test for the step (Red) and documents expected failure in PLANS.md.
3. Coder implements the minimal change to pass the test (Green) and updates Progress.
4. Tester runs tests, captures PASS/FAIL in PLANS.md Validation, and adds artifacts.
5. Reviewer reviews diffs; iterate 3–4 or approve and record decisions.
6. Learner documents the change and verification steps in Outcomes & Retrospective.
7. Repeat with the next step.

Always keep commit scope minimal and tied to a checked box in PLANS.md Progress.

## Definition of Done (Per Step)

- All acceptance criteria in PLANS.md are satisfied and demonstrable.
- New/updated tests exist and are green; no new warnings/lints.
- PLANS.md Progress, Validation, and Decision Log are updated.
- Commands work on Windows PowerShell (pwsh) unless noted otherwise.

## Operational Rules

- PLANS.md governs scope and acceptance. If scope drifts, update PLANS.md before continuing.
- When uncertain, state assumptions in the Decision Log and proceed; revise as needed.
- Keep changes small and preserve public APIs unless explicitly changing them.
- No external network calls unless PLANS.md permits them. Handle secrets safely.
- Agent conversation with the user will be in Japanese, while all code, comments, and documentation must be in English.
- Remove noisy debug logging before marking a step done.
- Write comments in English for all files.

## File Orientation

- Canonical plan: PLANS.md
- Short-run work queue: TODO.md (see below)

## TODO.md (Purpose and Rules)

Purpose:

- TODO.md is a lightweight, short-term task queue for immediate actionable items (hours → days). It complements PLANS.md, which remains the single source of truth.

Contents:

- One-line tasks (Markdown checkboxes) with 1–2 lines of context.
- Each task must reference the related PLANS.md milestone or item (e.g., refs: M1).
- Tag the active learning mode (e.g., mode: L0) so tasks reinforce the current practice step.

Format example:

- [ ] feat(pdf_parser): add failing test for parse_pdf empty-page case — refs: M1 mode: L0
- [ ] test(reflow_engine): add hyphenation edge-case — refs: M2 mode: L1

Rules:

- Task scope limited to what one person can finish in one session (max half-day).
- Annotate owner using @handle (optional) and priority tag (#high/#medium/#low).
- Mark in-progress with [>] and done with [x]. On completion, summarize verification in PLANS.md Validation.
- Planner Agent must update PLANS.md before adding tasks that change scope.

Agent usage:

- Planner emits tasks into TODO.md for the next short iterations.
- Coder/Tester/Reviewer reflect status changes in TODO.md and copy verification summaries into PLANS.md.

Lifecycle:

- Completed tasks should be archived within 7 days. Keep TODO.md focused and short.

## Vibe-Coding Guidance

- Begin with a walking skeleton: smallest vertical slice from PDF → chunks → reflow → UI marker.
- Prefer rough-but-working over perfect-but-late; refine in subsequent tiny steps.
- Keep mentoring explanations short, actionable, and targeted at junior engineers.

This spec guides Agents to act consistently, quickly, and transparently. When in doubt, update PLANS.md and continue.
