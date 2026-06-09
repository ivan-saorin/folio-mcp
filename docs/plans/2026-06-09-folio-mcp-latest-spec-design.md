# Folio MCP — Latest-Spec Update (Stage 1 + Stage 2)

**Date:** 2026-06-09
**Status:** Approved (brainstorming → writing-plans)
**Author:** Ivan Saorin (with Claude)

## Goal

Bring the `folio-mcp` server up to the current MCP spec and solve the
"show the computed table to the user verbatim" problem the *right* way —
without injecting imperative instructions into tool-result payloads.

Replaces the `SACRED_MANTRA` anti-pattern (an imperative string prepended to
the result `content`) with: (1) spec-correct structured output + metadata, and
(2) an MCP App (SEP-1865) that renders the table directly in the conversation,
independent of model discretion.

## Locked decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Scope | **Stage 1 (spec refactor) + Stage 2 (MCP App)** | Stage 1 is the immediate de-risk; Stage 2 is the durable verbatim-render fix. |
| Implementation | **Extend the hand-rolled JSON-RPC server** (no rmcp migration) | Every required change is additive JSON the current synchronous server already emits; an SDK migration is a larger, separate refactor that doesn't itself buy verbatim display. |
| `content` text block | **Rendered markdown table** (not serialized JSON) | The markdown is folio's product and the verbatim-display fallback for non-widget clients. A faithful, more-useful serialization of the same data than raw JSON. Documented deviation from the spec's "text = serialized JSON" SHOULD. |
| `eval_batch` | **Gets its own comparison widget** (`ui://folio/batch`) | Both compute paths render verbatim, not just single-document eval. |

## MCP Apps wire format (verified against the live spec, 2026-01-26)

The research report had two errors that this design corrects:

- **Tool → UI link** uses the *nested* path `_meta.ui.resourceUri` (+ optional
  `visibility: ["model","app"]`). **Not** a flat `_meta["io.modelcontextprotocol/ui"]` key.
- `"io.modelcontextprotocol/ui"` is the **extension identifier**, used in
  `initialize` → `capabilities.extensions` with `{ "mimeTypes": ["text/html;profile=mcp-app"] }`.
- UI resource mimeType is the literal `text/html;profile=mcp-app`, served via
  `resources/read` as a `text` field. Its own `_meta.ui` may carry `csp` /
  `prefersBorder`.
- The widget never reads our result directly — the **host** pushes the full
  `CallToolResult` via a `ui/notifications/tool-result` postMessage; the widget
  reads `structuredContent` from there. `structuredContent` is *not* injected
  into model context.
- Widget → host methods: `ui/initialize` (handshake), `tools/call`,
  `resources/read`, `ui/message`, etc. Host → widget notifications:
  `ui/notifications/tool-result`, `ui/notifications/tool-input`, etc.

## Architecture — where changes land

| Crate | Change | Risk |
|-------|--------|------|
| `folio` (core) | Extend `EvalResult` with ordered `cells: Vec<CellResult>`. Built from the parsed `doc` + the **renderer's** formatting so result strings match the markdown byte-for-byte (single source of truth). | Low — additive |
| `folio-mcp` | structuredContent + outputSchema; tool/content annotations; titles; capabilities; mantra removal/relocation; `isError` fix; `ui://folio/table` + `ui://folio/batch` resources; `_meta.ui` linkage; two embedded widget HTML files. | Low — JSON + static assets |

`parser::parse` is not `pub`, and the ordered cells already exist in `doc` at
eval time, so surfacing them from core is cleaner than re-parsing in the MCP layer.

### `CellResult` (core)

```rust
pub struct CellResult {
    pub name: String,        // cell.name
    pub formula: String,     // cell.raw_text (original formula text)
    pub result: String,      // rendered EXACTLY as in markdown (renderer-formatted)
    pub is_error: bool,
    pub section: String,     // section.name, for grouping in the widget
}
```
`EvalResult` gains `pub cells: Vec<CellResult>`. **Fidelity requirement:**
`result` must be produced by the same formatting path the `Renderer` uses
(precision/sigfigs honored), so widget table == markdown table verbatim.

## Stage 1 — spec-correct refactor

1. **Remove the anti-pattern.** Delete `SACRED_MANTRA` (main.rs:32); stop
   prepending it in `tool_eval` / `tool_eval_file` / `tool_eval_batch`. This
   also fixes the format-arg bug at main.rs:856
   (`"Evaluated {} sets{}"` with args swapped).
2. **Relocate guidance.** Put one concise, non-coercive sentence in each compute
   tool's `description` and a professional `instructions` string in `initialize`
   (the sanctioned model-facing homes). No imperatives in any result payload.
3. **structuredContent + outputSchema.** Replace ad-hoc top-level fields
   (`values`, `errors`, `source_file`, `data`, `results`, `comparison`) with
   spec `structuredContent`, and declare matching `outputSchema` per tool.
4. **Fix `isError`.** Derive `isError` and structured `errors[]` from any
   `Value::Error` in the results — **not** from `EvalResult.errors`, which is
   empty in practice because tracing is never enabled (lib.rs:66 reads
   `ctx.trace` but the context is built without `.with_tracing(true)`).
5. **Tool metadata.** Every tool gets `title` and
   `annotations: { readOnlyHint: true, openWorldHint: false }`. User-facing text
   blocks get `annotations: { audience: ["user"], priority: 1.0 }`.

### `eval` / `eval_file` result shape

```jsonc
{
  "content": [{ "type": "text",
    "text": "<rendered markdown table>",
    "annotations": { "audience": ["user"], "priority": 1.0 } }],
  "structuredContent": {
    "cells": [{ "name": "revenue", "formula": "120*1000", "result": "120000", "isError": false, "section": "..." }],
    "markdown": "<same table, verbatim>",
    "errors": [{ "code": "DIV_ZERO", "message": "...", "cell": "ratio" }],
    "isError": false
  },
  "isError": false
}
```

### `eval` / `eval_file` outputSchema (sketch)

```jsonc
{ "type": "object",
  "properties": {
    "cells": { "type": "array", "items": { "type": "object",
      "properties": { "name": {"type":"string"}, "formula": {"type":"string"},
        "result": {"type":"string"}, "isError": {"type":"boolean"},
        "section": {"type":"string"} },
      "required": ["name","formula","result"] } },
    "markdown": { "type": "string" },
    "errors": { "type": "array", "items": { "type": "object",
      "properties": { "code":{"type":"string"}, "message":{"type":"string"}, "cell":{"type":"string"} } } },
    "isError": { "type": "boolean" } },
  "required": ["cells"] }
```

### `eval_batch` structuredContent / outputSchema

```jsonc
{ "type": "object",
  "properties": {
    "runs": { "type": "array", "items": { "type": "object",
      "properties": { "index":{"type":"integer"}, "variables":{"type":"object"},
        "values":{"type":"object"}, "isError":{"type":"boolean"} },
      "required":["index","values"] } },
    "comparison": { "type": "array", "items": { "type": "object",
      "properties": { "index":{"type":"integer"}, "variables":{"type":"object"}, "value":{"type":"string"} } } },
    "compareField": { "type": "string" } },
  "required": ["runs"] }
```

## Stage 2 — MCP Apps

### Resources

Add two UI resources to `resources/list` and `resources/read`:

- `ui://folio/table` — single-document results widget (for `eval`, `eval_file`)
- `ui://folio/batch` — parameter-sweep comparison widget (for `eval_batch`)

Both: mimeType `text/html;profile=mcp-app`, returned as `contents[0].text`,
self-contained inline CSS/JS, **no external resources** → empty/minimal CSP, so
no domain-signing needed for local stdio use. The existing
`folio://documents/...` resources stay as-is.

### Widgets (embedded via `include_str!`)

- `folio-mcp/src/widgets/table.html` — on `ui/initialize`, listens for
  `ui/notifications/tool-result`, reads `structuredContent.cells`, renders a
  `name | formula | result` table grouped by `section`, error cells highlighted.
- `folio-mcp/src/widgets/batch.html` — renders `structuredContent.runs` /
  `comparison` as a comparison table.

Embedding via `include_str!` means they ship in the binary (Docker-friendly).

### Tool linkage + capability negotiation

- `eval` / `eval_file` → `_meta: { ui: { resourceUri: "ui://folio/table", visibility: ["model","app"] } }`.
- `eval_batch` → `_meta: { ui: { resourceUri: "ui://folio/batch", visibility: ["model","app"] } }`.
- `initialize` response advertises
  `capabilities.extensions: { "io.modelcontextprotocol/ui": { "mimeTypes": ["text/html;profile=mcp-app"] } }`.
- Capture the client's declared `capabilities.extensions["io.modelcontextprotocol/ui"]`
  at `initialize` into a small shared state; **only attach `_meta.ui`** in
  `tools/list` when the client supports it. Non-supporting hosts (e.g. Claude
  Code) never see the linkage and fall back to the `content` markdown — which is
  why Stage 1's text block is load-bearing.

## Per-tool treatment

| Tool | structuredContent | outputSchema | UI widget | readOnlyHint |
|------|------|------|------|------|
| `eval`, `eval_file` | `cells` + `markdown` + `errors` | yes | `ui://folio/table` | yes |
| `eval_batch` | `runs` + `comparison` | yes | `ui://folio/batch` | yes |
| `folio`, `quick`, `list_functions`, `list_constants` | move `data` under structuredContent | where structured | — | yes |
| `decompose` | clean shape only (behavior still stubbed) | minimal | — | yes |

## Error handling

Unchanged philosophy ("never crash, always explain"). Protocol-level failures
stay JSON-RPC errors; computation errors stay *inside* the result with
`isError: true` + structured `errors[]`. Core protocol version handling stays
as-is (echo the client's negotiated version); the UI extension is versioned
separately and degrades gracefully.

## Testing

Handlers are pure-ish (`params → Result<JsonValue, McpError>`), so add a
`#[cfg(test)]` module to `folio-mcp` asserting:

- `SACRED_MANTRA` text appears nowhere in any tool result or in `initialize`.
- `tool_eval` on a normal doc → ordered `cells`, `audience:["user"]`
  annotation, `isError:false`; on a div-by-zero doc → `isError:true` +
  structured `errors[]`.
- `tools/list` carries `outputSchema` + `readOnlyHint`/`openWorldHint` for all
  tools, and `_meta.ui` appears **only** after a UI-capable `initialize`.
- `handle_resources_read("ui://folio/table")` and `("ui://folio/batch")` return
  HTML with mimeType `text/html;profile=mcp-app`.
- `initialize` advertises the `io.modelcontextprotocol/ui` extension capability.

Core: a test that `EvalResult.cells` is ordered and its `result` strings equal
the corresponding markdown cells (fidelity).

Manual/visual widget verification via MCP Inspector or Claude Desktop is a
follow-up (cannot be unit-tested headless).

## Out of scope

- rmcp SDK migration.
- Remote/hosted connector distribution + Claude domain-signing / Connectors
  directory submission (local stdio MCP App needs none of this).
- Implementing `decompose`'s pattern detection (still stubbed).

## Sources

- [MCP Apps blog (launch, 2026-01-26)](https://blog.modelcontextprotocol.io/posts/2026-01-26-mcp-apps/)
- [SEP-1865 spec (apps.mdx, 2026-01-26)](https://github.com/modelcontextprotocol/ext-apps/blob/main/specification/2026-01-26/apps.mdx)
- [ext-apps repo](https://github.com/modelcontextprotocol/ext-apps/)
