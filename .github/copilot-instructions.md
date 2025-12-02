# GOIDEV Copilot Instructions

Concise guidance for AI coding agents to be productive in this repo.

## Architecture Overview

- App stack: Tauri desktop app + Leptos UI + Rust core.
- Workspace roots:
	- `src-tauri/`: Tauri Rust app and config (`tauri.conf.json`).
	- `src/`: Leptos frontend (`app.rs`, components under `src/components/`).
	- `goidev-core/`: Pure Rust library with PDF parsing + reflow engine.
- Core library structure (`goidev-core/src`):
	- `pdf_parser.rs`: parse PDF into internal structs; uses `pdf_state.rs`.
	- `reflow_engine.rs`: converts parsed content into reflowed layout chunks.
	- `dto.rs`: data transfer structs shared with UI (serializeable).
	- `font_utils.rs`: font metrics/helpers used by reflow.
	- `lib.rs`: public API surface exposing parse + reflow.
- Tests live in `goidev-core/src/tests/` with integration tests and fixtures under `resources/`.

## Developer Workflows

- Build core library: `pwsh` → `cargo build -p goidev-core`.
- Run core tests: `pwsh` → `cargo test -p goidev-core`.
- Run desktop app: `pwsh` → `cargo tauri dev` from repo root.
- Frontend (Trunk for WASM/Leptos): `pwsh` → `trunk serve` (uses `Trunk.toml`).
- Common error logs: see `build_error.txt`, `wasm_error.txt`, `output.txt` for prior failures.

## Patterns and Conventions

- Rust-first: place parsing/reflow logic in `goidev-core`; UI calls through Tauri commands.
- DTO boundary: types in `goidev-core/src/dto.rs` are the contract between core and UI. Extend here when adding cross-component data.
- Tests: follow happy-path + one edge-case per feature; add fixtures under `goidev-core/src/tests/resources/`.
- Public API: export from `goidev-core/src/lib.rs`; do not leak internal module details.
- Font handling: centralize metrics in `font_utils.rs`; reflow engine reads font sizes from DTOs.

## Cross-Component Communication

- Tauri commands (Rust) call into `goidev-core` functions and return DTOs to the Leptos frontend.
- The Leptos viewer (`src/components/reflow_viewer.rs`) renders reflowed chunks from DTOs.
- Keep serialization stable (Serde) across the boundary; avoid breaking DTO field names.

## DTO Examples

- `goidev-core/src/dto.rs` exposes the main serializable document contract used by the UI. Example (simplified):

```rust
use goidev_core::reflow_engine::Block;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflowDocument {
		pub doc_id: String,
		pub title: String,
		pub blocks: Vec<Block>,
}
```

- `Block` (from `reflow_engine.rs`) contains layout + text used by the viewer:

```rust
pub struct Block {
		pub text: String,
		pub bbox: BBox,      // { x1,y1,x2,y2 }
		pub role: BlockRole, // Paragraph | Heading
		pub page_num: u32,
}
```

Keep these DTOs stable (Serde names/types); the frontend expects these fields.

## Tauri Commands (integration examples)

- Commands are defined in `src-tauri/src/lib.rs` and exported to the frontend via `tauri::generate_handler!`.
- Two useful commands to know:

	- `greet(name: &str) -> String` — test helper exposed as a Tauri command.
	- `open_document(path: String) -> Result<ReflowDocument, String>` — parses a PDF, runs the reflow engine, and returns a `ReflowDocument` DTO.

- Example frontend call (JS/TS using `@tauri-apps/api`):

```javascript
import { invoke } from '@tauri-apps/api/tauri'

// open a PDF and get a ReflowDocument
const doc = await invoke('open_document', { path: 'C:\\path\\to\\file.pdf' })
console.log(doc.title, doc.blocks.length)
```

Note: the Rust handler creates the `ReflowDocument` and assigns a `doc_id` (UUID) before returning.

## File Landmarks

- `goidev-core/src/pdf_parser.rs`: entry for parsing; look here when changing how PDFs are ingested.
- `goidev-core/src/reflow_engine.rs`: main layout logic; extend chunk generation or hyphenation here.
- `goidev-core/src/tests/reflow_engine_tests.rs`: examples of expected reflow outputs.
- `src-tauri/tauri.conf.json`: window/capabilities; update for platform settings.
- `Trunk.toml`: frontend build config for WASM/Leptos.

## Working Rules (Project-Specific)

- Keep changes small and focused; preserve public APIs unless explicitly changing them.
- Prefer adding tests in `goidev-core` before UI changes; validate core outputs.
- Windows-first: commands and scripts should run in PowerShell (`pwsh`).
- Avoid external network calls in core; keep pure functions where possible.

## Quick Start Examples

- Add a new reflow feature:
	1. Define/extend DTOs in `dto.rs`.
	2. Implement logic in `reflow_engine.rs` using `font_utils.rs` as needed.
	3. Expose via `lib.rs` and Tauri command.
	4. Cover with tests under `goidev-core/src/tests/`.
- Change PDF parsing:
	1. Update `pdf_parser.rs` and `pdf_state.rs`.
	2. Adjust DTO mapping in `lib.rs`.
	3. Add fixtures in `tests/resources/` and expand `pdf_parser_tests.rs`.

## Where to Look When Stuck

- Minimal `README.md` (Tauri + Leptos); rely on paths above and tests for behavior.
- Check `target/` build artifacts if linking fails; confirm crate names in `Cargo.toml` files.

This doc is intentionally concise. If any section feels unclear, point to the exact file/flow you need help with and we’ll expand with code-level examples.
