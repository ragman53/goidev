# GOIDEV Copilot Instructions

Concise guidance for AI coding agents to be productive in this repo.

## Architecture Overview

- App stack: Tauri desktop app + Leptos UI + Rust core.
- Workspace roots:
  - `src-tauri/`: Tauri Rust app and config (`tauri.conf.json`).
  - `src/`: Leptos frontend (`app.rs`, components under `src/components/`).
  - `goidev-core/`: Pure Rust library with PDF parsing + reflow engine + Markdown caching.
- Core library structure (`goidev-core/src`):
  - `pdf_parser.rs`: parse PDF into internal structs; uses `pdf_state.rs`.
  - `reflow_engine.rs`: converts parsed content into reflowed layout chunks.
  - `markdown.rs`: serialize/deserialize blocks to Markdown (sidecar cache).
  - `dto.rs`: data transfer structs shared with UI (serializeable).
  - `font_utils.rs`: font metrics/helpers used by reflow.
  - `lib.rs`: public API surface exposing parse + reflow + markdown.
- Tests live in `goidev-core/tests/` with fixtures under `resources/`.

## Data Flow (Parse-Once Architecture)

```
PDF → parse_pdf() → Vec<TextLine> → ReflowEngine::process() → Vec<Block>
                                                                   ↓
                                              blocks_to_markdown() → .goidev.md sidecar
                                                                   ↓
                                                         ReflowDocument → UI
```

On subsequent opens, `open_document` checks the sidecar cache:
- If `doc.pdf.goidev.md` exists and its `source_hash` matches the PDF's SHA-256, load from cache.
- Otherwise, re-parse and regenerate the sidecar.

External Markdown (no goidev metadata) is imported leniently with synthetic page/bbox defaults.

## Developer Workflows

- Build core library: `cargo build -p goidev-core`
- Run core tests: `cargo test -p goidev-core`
- Run desktop app: `cargo tauri dev` from repo root
- Frontend (Trunk for WASM/Leptos): `trunk serve` (uses `Trunk.toml`)
- Common error logs: see `build_error.txt`, `wasm_error.txt`, `output.txt` for prior failures

## Patterns and Conventions

- Rust-first: place parsing/reflow/markdown logic in `goidev-core`; UI calls through Tauri commands.
- DTO boundary: types in `goidev-core/src/dto.rs` are the contract between core and UI.
- Markdown sidecar: `goidev-core/src/markdown.rs` handles cache serialization with YAML frontmatter (`source_hash`) and HTML comment metadata (`<!-- goidev:page=N bbox=... -->`).
- Tests: follow happy-path + one edge-case per feature; add fixtures under `goidev-core/tests/resources/`.
- Public API: export from `goidev-core/src/lib.rs`; do not leak internal module details.

## Cross-Component Communication

- Tauri commands (Rust) call into `goidev-core` functions and return DTOs to the Leptos frontend.
- The Leptos viewer (`src/components/reflow_viewer.rs`) renders reflowed chunks from DTOs.
- Keep serialization stable (Serde) across the boundary; avoid breaking DTO field names.

## DTO Examples

- `goidev-core/src/dto.rs` exposes the main serializable document contract used by the UI:

```rust
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
    pub bbox: BBox,      // { x1, y1, x2, y2 }
    pub role: BlockRole, // Semantic role (see below)
    pub page_num: u32,
}
```

- `BlockRole` enum defines semantic block types:

```rust
pub enum BlockRole {
    Paragraph,   // Main body text
    Heading,     // Section/document heading (large font)
    PageNumber,  // Page number in header/footer
    Header,      // Header content (top zone)
    Footer,      // Footer content (bottom zone)
    Footnote,    // Footnote text
    Caption,     // Figure/table caption
    Citation,    // Reference entry
    Author,      // Author/affiliation
    Abstract,    // Abstract section
    Reference,   // References section header
}
```

- `PageGeometry` (from `pdf_parser.rs`) provides page dimensions for zone detection:

```rust
pub struct PageGeometry {
    pub width: f32,
    pub height: f32,
    pub origin_x: f32,
    pub origin_y: f32,
}
```

Keep these DTOs stable (Serde names/types); the frontend expects these fields.

## Tauri Commands (integration examples)

- Commands are defined in `src-tauri/src/lib.rs` and exported via `tauri::generate_handler!`.
- Key commands:

  - `open_document(path: String) -> Result<ReflowDocument, String>` — opens PDF or Markdown; uses sidecar cache for PDFs.
  - `save_document_markdown(blocks, dest_path, source_hash)` — explicitly save blocks to Markdown.
  - `greet(name: &str) -> String` — test helper.

- Example frontend call:

```javascript
import { invoke } from '@tauri-apps/api/tauri'

const doc = await invoke('open_document', { path: 'C:\\path\\to\\file.pdf' })
console.log(doc.title, doc.blocks.length)
```

## Role Detection System

The reflow engine classifies text blocks using a multi-stage approach:

1. **Position-Based Detection** (via `PageGeometry`):
   - Header zone: top 8% of page → `Header` or `PageNumber`
   - Footer zone: bottom 8% of page → `Footer`, `Footnote`, or `PageNumber`

2. **Pattern-Based Detection** (regex in `reflow_engine.rs`):
   - Page numbers: `42`, `- 5 -`, `Page 1 of 10`
   - Footnotes: `1.`, `†`, `[1]` followed by text
   - Captions: `Figure 1:`, `Table 2.`, `Fig. 3:`
   - Citations: `[1] Author...`, `1. Author...`
   - References header: `References`, `Bibliography`, `Works Cited`
   - Abstract header: `Abstract`

3. **Font-Based Detection**:
   - Font size > 14pt → `Heading`
   - Otherwise → `Paragraph`

4. **State Tracking**:
   - After detecting `References` header, subsequent entries become `Citation`
   - New major heading resets the References section state

Zone thresholds and patterns are defined as constants in `reflow_engine.rs`.

## File Landmarks

- `goidev-core/src/pdf_parser.rs`: PDF parsing entry point.
- `goidev-core/src/reflow_engine.rs`: main layout/merge logic.
- `goidev-core/src/markdown.rs`: sidecar cache serialization/deserialization.
- `goidev-core/tests/markdown_tests.rs`: roundtrip and lenient import tests.
- `src-tauri/src/lib.rs`: Tauri command definitions with cache logic.
- `src-tauri/tauri.conf.json`: window/capabilities config.

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
  4. Cover with tests under `goidev-core/tests/`.

- Change PDF parsing:
  1. Update `pdf_parser.rs` and `pdf_state.rs`.
  2. Adjust DTO mapping in `lib.rs`.
  3. Add fixtures in `tests/resources/` and expand `pdf_parser_tests.rs`.

- Modify Markdown cache format:
  1. Update `markdown.rs` (frontmatter keys, comment format).
  2. Update `markdown_tests.rs` with roundtrip tests.
  3. Bump or invalidate existing caches if format changes are breaking.

## Where to Look When Stuck

- Minimal `README.md` (Tauri + Leptos); rely on paths above and tests for behavior.
- Check `target/` build artifacts if linking fails; confirm crate names in `Cargo.toml` files.

This doc is intentionally concise. If any section feels unclear, point to the exact file/flow you need help with and we'll expand with code-level examples.
