# Folio MCP — Latest-Spec Update Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Update `folio-mcp` to the current MCP spec and render computation tables verbatim via MCP Apps, replacing the imperative `SACRED_MANTRA` payload-injection anti-pattern.

**Architecture:** Extend the existing hand-rolled synchronous JSON-RPC stdio server (no SDK migration). The core `folio` crate gains an ordered, renderer-formatted `EvalResult.cells`. The `folio-mcp` crate gains spec `structuredContent` + `outputSchema`, tool/content annotations, titles, fixed `isError`, an `io.modelcontextprotocol/ui` capability, two embedded HTML widgets (`ui://folio/table`, `ui://folio/batch`), and conditional `_meta.ui` linkage negotiated at `initialize`.

**Tech Stack:** Rust (workspace crates), `serde_json`, hand-rolled JSON-RPC 2.0 over stdio, MCP Apps (SEP-1865) `text/html;profile=mcp-app` widgets.

**Design doc:** `docs/plans/2026-06-09-folio-mcp-latest-spec-design.md`

**Conventions for the executing engineer:**
- Use @superpowers:test-driven-development for every task: write the failing test first, see it fail, implement, see it pass, commit.
- Tests for the binary crate live in `folio-mcp/src/main.rs` inside `#[cfg(test)] mod tests` (same-file tests can call private fns like `tool_eval`, `handle_tools_list`, `create_folio_with_isis`).
- Windows PowerShell. Run tests with `cargo test -p <crate>`. Chain with `;` not `&&`.
- Commit after each task. Branch is already `folio-mcp-latest-spec`.
- The MCP spec result type `CallToolResult` only defines `content`, `structuredContent`, `isError`, `_meta`. Any other top-level field is non-standard and must be removed.

---

## Task 0: Establish green baseline

**Step 1:** Run the full suite to confirm a clean starting point.

Run: `cargo test -p folio -p folio-mcp`
Expected: all existing tests PASS, both crates compile.

**Step 2:** If anything fails, STOP and report — do not start changes on a red baseline.

No commit.

---

## Task 1: Core — ordered, renderer-formatted `EvalResult.cells`

**Files:**
- Modify: `folio/src/eval.rs` (add `CellResult` near `EvalResult`, ~line 10-33)
- Modify: `folio/src/render.rs` (add `render_with_cells`, refactor `render` to delegate, ~line 44-101)
- Modify: `folio/src/lib.rs` (re-export `CellResult`; use `render_with_cells` in `Folio::eval`, ~line 8-71)

**Step 1: Write the failing test**

Add to the `tests` module in `folio/src/lib.rs`:

```rust
#[test]
fn test_eval_result_has_ordered_renderer_matched_cells() {
    let folio = test_folio();
    let doc = r#"
## Demo
| name | formula | result |
|------|---------|--------|
| a | 10 | |
| b | a * 2 | |
"#;
    let result = folio.eval(doc, &HashMap::new());

    // Ordered, with original formula text and section label.
    assert_eq!(result.cells.len(), 2);
    assert_eq!(result.cells[0].name, "a");
    assert_eq!(result.cells[1].name, "b");
    assert_eq!(result.cells[1].formula, "a * 2");
    assert_eq!(result.cells[0].section, "Demo");
    assert!(!result.cells[1].is_error);

    // Fidelity: each cell's result string is exactly what the markdown shows.
    assert!(
        result.markdown.contains(&format!(
            "| b | a * 2 | {} |",
            result.cells[1].result
        )),
        "cell result must match markdown verbatim; markdown was:\n{}",
        result.markdown
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p folio test_eval_result_has_ordered_renderer_matched_cells`
Expected: FAIL to compile — `result.cells` field does not exist.

**Step 3: Implement**

In `folio/src/eval.rs`, add the struct above `EvalResult` and a field to `EvalResult`:

```rust
/// One row of the rendered results table, formatted identically to the markdown.
#[derive(Debug, Clone)]
pub struct CellResult {
    pub name: String,
    pub formula: String,   // original formula text (Cell::raw_text)
    pub result: String,    // renderer-formatted, matches the markdown cell verbatim
    pub is_error: bool,
    pub section: String,   // owning section name, for grouping in the widget
}
```

Add `pub cells: Vec<CellResult>,` to `EvalResult` and to `EvalResult::parse_error` (use `cells: vec![]`).

In `folio/src/render.rs`, change the import line `use crate::ast::Document;` to:

```rust
use crate::ast::Document;
use crate::eval::CellResult;
```

Rename the body of the current `render` into a new `render_with_cells` that also collects cells, and make `render` delegate. Replace the `render` method (lines ~44-101) with:

```rust
    /// Render document with computed values (markdown only).
    pub fn render(
        &self,
        doc: &Document,
        values: &HashMap<String, Value>,
        external: &HashMap<String, Value>,
    ) -> String {
        self.render_with_cells(doc, values, external).0
    }

    /// Render document, returning the markdown AND the ordered structured cells.
    /// Both are produced from the same `render_value` call, so the cell `result`
    /// strings are byte-for-byte identical to the markdown table.
    pub fn render_with_cells(
        &self,
        doc: &Document,
        values: &HashMap<String, Value>,
        external: &HashMap<String, Value>,
    ) -> (String, Vec<CellResult>) {
        let mut output = String::new();
        let mut cells_out: Vec<CellResult> = Vec::new();

        // Render external variables section if any (not collected as cells).
        if !external.is_empty() {
            output.push_str("## External Variables\n\n");
            output.push_str("| name | value |\n");
            output.push_str("|------|-------|\n");
            let default_dt_formats = DateTimeFormats::default();
            for (name, value) in external {
                output.push_str(&format!(
                    "| {} | {} |\n",
                    name,
                    self.render_value(value, NumberFormat::default(), &default_dt_formats)
                ));
            }
            output.push('\n');
        }

        for section in &doc.sections {
            output.push_str(&format!("## {}", section.name));

            if !section.attributes.is_empty() {
                let attrs: Vec<String> = section
                    .attributes
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect();
                output.push_str(&format!(" @{}", attrs.join(",")));
            }
            output.push_str("\n\n");

            let num_format = self.get_number_format(&section.attributes);
            let dt_formats = self.get_datetime_formats(&section.attributes);

            output.push_str("| name | formula | result |\n");
            output.push_str("|------|---------|--------|\n");

            for row in &section.table.rows {
                for cell in &row.cells {
                    let value = values.get(&cell.name);
                    let result = value
                        .map(|v| self.render_value(v, num_format, &dt_formats))
                        .unwrap_or_default();
                    output.push_str(&format!(
                        "| {} | {} | {} |\n",
                        cell.name, cell.raw_text, result
                    ));
                    cells_out.push(CellResult {
                        name: cell.name.clone(),
                        formula: cell.raw_text.clone(),
                        result,
                        is_error: value.map(|v| v.is_error()).unwrap_or(false),
                        section: section.name.clone(),
                    });
                }
            }

            output.push('\n');
        }

        (output, cells_out)
    }
```

In `folio/src/lib.rs`:
- Change `pub use eval::{Evaluator, EvalResult};` to `pub use eval::{Evaluator, EvalResult, CellResult};`
- In `Folio::eval`, replace the render block (lines ~60-70) with:

```rust
        let renderer = Renderer::new();
        let (markdown, cells) = renderer.render_with_cells(&doc, &values, variables);

        EvalResult {
            markdown,
            values,
            cells,
            errors: ctx.trace.iter()
                .filter_map(|s| if let Value::Error(e) = &s.result { Some(e.clone()) } else { None })
                .collect(),
            warnings: vec![],
        }
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p folio`
Expected: PASS (new test + all existing).

**Step 5: Commit**

```bash
git add folio/src/eval.rs folio/src/render.rs folio/src/lib.rs
git commit -m "feat(core): add ordered, renderer-matched EvalResult.cells"
```

---

## Task 2: MCP — remove SACRED_MANTRA, relocate guidance, fix batch bug

**Files:**
- Modify: `folio-mcp/src/main.rs` (delete const at :32; `handle_initialize` :391-427; tool descriptions in `handle_tools_list` :429-570; `tool_eval` :779; `tool_eval_file` :811; `tool_eval_batch` :856)

**Step 1: Write the failing test**

Add a `#[cfg(test)] mod tests` block at the end of `folio-mcp/src/main.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn eval_args(template: &str) -> JsonValue {
        json!({ "template": template })
    }

    const SIMPLE_DOC: &str =
        "## T\n| name | formula | result |\n|------|---------|--------|\n| a | 1 | |\n";

    #[test]
    fn test_no_sacred_mantra_anywhere() {
        let folio = create_folio_with_isis();
        let res = tool_eval(&folio, eval_args(SIMPLE_DOC)).unwrap();
        let text = res["content"][0]["text"].as_str().unwrap();
        assert!(!text.to_uppercase().contains("SACRED"), "mantra leaked into eval content");

        let init = handle_initialize(&None).unwrap();
        let instr = init["instructions"].as_str().unwrap();
        assert!(!instr.to_uppercase().contains("SACRED"), "mantra in initialize instructions");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p folio-mcp test_no_sacred_mantra_anywhere`
Expected: FAIL — current output contains the mantra.

**Step 3: Implement**

- Delete the `SACRED_MANTRA` const (line 32).
- In `tool_eval` and `tool_eval_file`, delete the `let markdown_with_mantra = ...;` line and use `result.markdown` directly (full result-shape rewrite happens in Task 3/4; for now just drop the mantra prefix so the test passes — e.g. `"text": result.markdown` / keep existing other fields).
- In `tool_eval_batch`, fix the summary line:
  `let batch_summary = format!("Evaluated {} variable set(s).", results.len());`
- In `handle_initialize`, replace the `instructions` string with a concise, non-coercive version:

```rust
        "instructions": "Folio evaluates Markdown Computational Documents with arbitrary-precision arithmetic. Use `folio()` to explore functions and `eval`/`eval_file` to compute. The eval tools return a complete, auditable results table that is designed to be shown to the user in full; present that table before adding commentary."
```

- In `handle_tools_list`, append one non-coercive sentence to the `eval` and `eval_file` descriptions, e.g. for `eval`:
  `"Evaluate a Folio markdown document with formulas. Returns a complete, auditable results table (one row per named cell) intended to be shown to the user in full."`

**Step 4: Run test to verify it passes**

Run: `cargo test -p folio-mcp`
Expected: PASS.

**Step 5: Commit**

```bash
git add folio-mcp/src/main.rs
git commit -m "refactor(mcp): remove SACRED_MANTRA payload injection; relocate guidance to descriptions/instructions; fix eval_batch summary"
```

---

## Task 3: MCP — eval/eval_file → structuredContent + annotations + isError

**Files:**
- Modify: `folio-mcp/src/main.rs` (`tool_eval` :763-787, `tool_eval_file` :789-820; add a shared helper)

**Step 1: Write the failing test**

Add to the `tests` module:

```rust
    #[test]
    fn test_eval_structured_content_and_annotations() {
        let folio = create_folio_with_isis();
        let doc = "## T\n| name | formula | result |\n|------|---------|--------|\n| a | 10 | |\n| b | a * 2 | |\n";
        let res = tool_eval(&folio, eval_args(doc)).unwrap();

        // User-facing text block carries the audience annotation.
        assert_eq!(res["content"][0]["annotations"]["audience"][0], "user");
        assert_eq!(res["content"][0]["annotations"]["priority"], 1.0);

        // structuredContent.cells is ordered and renderer-matched.
        let cells = res["structuredContent"]["cells"].as_array().unwrap();
        assert_eq!(cells[0]["name"], "a");
        assert_eq!(cells[1]["formula"], "a * 2");
        assert_eq!(cells[1]["isError"], false);

        // No non-standard top-level fields remain.
        assert!(res.get("values").is_none(), "non-standard top-level 'values' must be gone");
        assert_eq!(res["isError"], false);
    }

    #[test]
    fn test_eval_iserror_on_div_zero() {
        let folio = create_folio_with_isis();
        let doc = "## T\n| name | formula | result |\n|------|---------|--------|\n| x | 42 / 0 | |\n";
        let res = tool_eval(&folio, eval_args(doc)).unwrap();
        assert_eq!(res["isError"], true);
        let errs = res["structuredContent"]["errors"].as_array().unwrap();
        assert!(!errs.is_empty(), "div-by-zero must populate structured errors");
        assert_eq!(errs[0]["cell"], "x");
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p folio-mcp test_eval_structured_content_and_annotations`
Expected: FAIL — `structuredContent` / annotations not present; `values` still top-level.

**Step 3: Implement**

Add a shared helper near the eval tools:

```rust
/// Build a spec-compliant CallToolResult for a single-document evaluation.
fn eval_result_json(result: &folio::EvalResult) -> JsonValue {
    let errors: Vec<JsonValue> = result.values.iter()
        .filter_map(|(name, v)| {
            if let Value::Error(e) = v {
                Some(json!({ "code": e.code, "message": e.message, "cell": name }))
            } else {
                None
            }
        })
        .collect();
    let is_error = !errors.is_empty();

    let cells: Vec<JsonValue> = result.cells.iter().map(|c| json!({
        "name": c.name,
        "formula": c.formula,
        "result": c.result,
        "isError": c.is_error,
        "section": c.section,
    })).collect();

    json!({
        "content": [{
            "type": "text",
            "text": result.markdown,
            "annotations": { "audience": ["user"], "priority": 1.0 }
        }],
        "structuredContent": {
            "cells": cells,
            "markdown": result.markdown,
            "errors": errors,
            "isError": is_error
        },
        "isError": is_error
    })
}
```

Rewrite `tool_eval` body tail (after computing `result`) to `Ok(eval_result_json(&result))`.

Rewrite `tool_eval_file` similarly, but preserve the source filename inside `structuredContent` (not as a top-level field). Easiest: build via the helper then inject:

```rust
    let mut out = eval_result_json(&result);
    out["structuredContent"]["sourceFile"] = json!(format!("{}.fmd", name));
    Ok(out)
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p folio-mcp`
Expected: PASS.

**Step 5: Commit**

```bash
git add folio-mcp/src/main.rs
git commit -m "feat(mcp): eval/eval_file return structuredContent + audience annotations + reliable isError"
```

---

## Task 4: MCP — eval_batch → structuredContent (runs/comparison)

**Files:**
- Modify: `folio-mcp/src/main.rs` (`tool_eval_batch` :822-863)

**Step 1: Write the failing test**

```rust
    #[test]
    fn test_eval_batch_structured_content() {
        let folio = create_folio_with_isis();
        let args = json!({
            "template": "## T\n| name | formula | result |\n|------|---------|--------|\n| out | x * 2 | |\n",
            "variable_sets": [ {"x": "5"}, {"x": "10"} ],
            "compare_field": "out"
        });
        let res = tool_eval_batch(&folio, args).unwrap();
        let runs = res["structuredContent"]["runs"].as_array().unwrap();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0]["index"], 0);
        let cmp = res["structuredContent"]["comparison"].as_array().unwrap();
        assert_eq!(cmp.len(), 2);
        assert!(res.get("results").is_none(), "non-standard top-level 'results' must be gone");
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p folio-mcp test_eval_batch_structured_content`
Expected: FAIL.

**Step 3: Implement**

Rewrite `tool_eval_batch` so each run records `isError` (any `Value::Error` in its values) and the final result uses `structuredContent`:

```rust
        results.push(json!({
            "index": i,
            "variables": vars,
            "values": result.values.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect::<HashMap<_, _>>(),
            "isError": result.values.values().any(|v| v.is_error())
        }));
```

And the return:

```rust
    let any_error = results.iter().any(|r| r["isError"].as_bool().unwrap_or(false));
    Ok(json!({
        "content": [{
            "type": "text",
            "text": format!("Evaluated {} variable set(s).", results.len()),
            "annotations": { "audience": ["user"], "priority": 1.0 }
        }],
        "structuredContent": {
            "runs": results,
            "comparison": comparison,
            "compareField": compare_field
        },
        "isError": any_error
    }))
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p folio-mcp`
Expected: PASS.

**Step 5: Commit**

```bash
git add folio-mcp/src/main.rs
git commit -m "feat(mcp): eval_batch returns structuredContent (runs/comparison)"
```

---

## Task 5: MCP — clean informational tools' result shapes

**Files:**
- Modify: `folio-mcp/src/main.rs` (`tool_folio` :865-887, `tool_list_functions` :1124-1145, `tool_list_constants` :1147-1170, `tool_decompose` :1172-1183)

**Step 1: Write the failing test**

```rust
    #[test]
    fn test_informational_tools_have_no_adhoc_top_level_fields() {
        let folio = create_folio_with_isis();

        let lf = tool_list_functions(&folio, json!({})).unwrap();
        assert!(lf.get("data").is_none(), "list_functions must not use top-level 'data'");
        assert!(lf["structuredContent"]["functions"].is_array());

        let lc = tool_list_constants(&folio, json!({})).unwrap();
        assert!(lc.get("data").is_none());
        assert!(lc["structuredContent"]["constants"].is_array());

        let dc = tool_decompose(&folio, json!({"value": "1.618"})).unwrap();
        assert!(dc.get("patterns").is_none(), "decompose must not use top-level 'patterns'");
        assert!(dc.get("_note").is_none());
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p folio-mcp test_informational_tools_have_no_adhoc_top_level_fields`
Expected: FAIL.

**Step 3: Implement**

- `tool_list_functions`: replace `"data": value_to_json(&functions)` with `"structuredContent": { "functions": value_to_json(&functions) }`.
- `tool_list_constants`: replace `"data": value_to_json(&constants)` with `"structuredContent": { "constants": value_to_json(&constants) }`.
- `tool_folio` (name branch): replace `"data": value_to_json(&help)` with `"structuredContent": { "help": value_to_json(&help) }`.
- `tool_decompose`: replace the ad-hoc `value`/`patterns`/`_note` with:
  `"structuredContent": { "value": value_str, "patterns": {}, "note": "DECOMPOSE implementation pending" }`

**Step 4: Run test to verify it passes**

Run: `cargo test -p folio-mcp`
Expected: PASS.

**Step 5: Commit**

```bash
git add folio-mcp/src/main.rs
git commit -m "refactor(mcp): move informational tool data under structuredContent"
```

---

## Task 6: MCP — tools/list metadata (title, annotations, outputSchema)

**Files:**
- Modify: `folio-mcp/src/main.rs` (`handle_tools_list` :429-570; change signature)

**Step 1: Write the failing test**

```rust
    #[test]
    fn test_tools_list_metadata() {
        let res = handle_tools_list(false).unwrap();
        let tools = res["tools"].as_array().unwrap();
        for t in tools {
            assert!(t["title"].is_string(), "{} missing title", t["name"]);
            assert_eq!(t["annotations"]["readOnlyHint"], true, "{} readOnlyHint", t["name"]);
            assert_eq!(t["annotations"]["openWorldHint"], false, "{} openWorldHint", t["name"]);
        }
        let eval = tools.iter().find(|t| t["name"] == "eval").unwrap();
        assert!(eval["outputSchema"]["properties"]["cells"].is_object());
        // No UI linkage when the client does not support the extension.
        assert!(eval.get("_meta").is_none());
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p folio-mcp test_tools_list_metadata`
Expected: FAIL — signature mismatch (`handle_tools_list` takes no args) and metadata absent.

**Step 3: Implement**

Change the signature to `fn handle_tools_list(client_ui_support: bool) -> Result<JsonValue, McpError>`. Update its caller in `handle_request` (Task 7 wires the bool; for now pass `false` to compile, Task 7 replaces it). Add to every tool object a `"title"` and `"annotations": { "readOnlyHint": true, "openWorldHint": false }`. Add `"outputSchema"` for `eval`, `eval_file`, `eval_batch`, `list_functions`, `list_constants`.

`eval` / `eval_file` outputSchema:

```rust
"outputSchema": {
    "type": "object",
    "properties": {
        "cells": { "type": "array", "items": { "type": "object",
            "properties": {
                "name": {"type":"string"}, "formula": {"type":"string"},
                "result": {"type":"string"}, "isError": {"type":"boolean"},
                "section": {"type":"string"}
            },
            "required": ["name","formula","result"] } },
        "markdown": { "type": "string" },
        "errors": { "type": "array", "items": { "type": "object",
            "properties": { "code":{"type":"string"}, "message":{"type":"string"}, "cell":{"type":"string"} } } },
        "isError": { "type": "boolean" }
    },
    "required": ["cells"]
}
```

`eval_batch` outputSchema:

```rust
"outputSchema": {
    "type": "object",
    "properties": {
        "runs": { "type": "array", "items": { "type": "object",
            "properties": { "index":{"type":"integer"}, "variables":{"type":"object"},
                "values":{"type":"object"}, "isError":{"type":"boolean"} },
            "required": ["index","values"] } },
        "comparison": { "type": "array" },
        "compareField": { "type": ["string","null"] }
    },
    "required": ["runs"]
}
```

`list_functions` / `list_constants` outputSchema: `{ "type":"object", "properties": { "functions"|"constants": {"type":"array"} }, "required": [...] }`.

Titles: `eval`→"Evaluate Folio Document", `eval_file`→"Evaluate Folio File", `eval_batch`→"Folio Parameter Sweep", `folio`→"Folio Help", `quick`→"Folio Quick Reference", `list_functions`→"List Folio Functions", `list_constants`→"List Folio Constants", `decompose`→"Decompose Value".

**Step 4: Run test to verify it passes**

Run: `cargo test -p folio-mcp`
Expected: PASS.

**Step 5: Commit**

```bash
git add folio-mcp/src/main.rs
git commit -m "feat(mcp): add tool titles, readOnly/openWorld annotations, and outputSchema"
```

---

## Task 7: MCP — capability negotiation (detect + advertise `io.modelcontextprotocol/ui`)

**Files:**
- Modify: `folio-mcp/src/main.rs` (`main` loop :269-344; `handle_request` :349-389; `handle_initialize` :391-427; add `detect_ui_support`)

**Step 1: Write the failing test**

```rust
    #[test]
    fn test_initialize_advertises_ui_extension() {
        let res = handle_initialize(&None).unwrap();
        assert_eq!(
            res["capabilities"]["extensions"]["io.modelcontextprotocol/ui"]["mimeTypes"][0],
            "text/html;profile=mcp-app"
        );
    }

    #[test]
    fn test_detect_ui_support() {
        let yes = Some(json!({
            "capabilities": { "extensions": { "io.modelcontextprotocol/ui": { "mimeTypes": ["text/html;profile=mcp-app"] } } }
        }));
        assert!(detect_ui_support(&yes));
        assert!(!detect_ui_support(&None));
        let no = Some(json!({ "capabilities": {} }));
        assert!(!detect_ui_support(&no));
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p folio-mcp test_detect_ui_support`
Expected: FAIL — `detect_ui_support` undefined; capability not advertised.

**Step 3: Implement**

Add the detector:

```rust
/// True if the client declared support for the MCP Apps UI extension.
fn detect_ui_support(params: &Option<JsonValue>) -> bool {
    params.as_ref()
        .and_then(|p| p.get("capabilities"))
        .and_then(|c| c.get("extensions"))
        .and_then(|e| e.get("io.modelcontextprotocol/ui"))
        .is_some()
}
```

In `handle_initialize`, add to the `capabilities` object:

```rust
            "extensions": {
                "io.modelcontextprotocol/ui": { "mimeTypes": ["text/html;profile=mcp-app"] }
            }
```

Thread the negotiated flag:
- In `main`, before the loop: `let mut client_ui_support = false;`
- Change `handle_request(&folio, &request)` to `handle_request(&folio, &request, &mut client_ui_support)`.
- Change `fn handle_request(folio: &Folio, request: &McpRequest, client_ui_support: &mut bool) -> McpResponse`.
- In the match, the `"initialize"` arm becomes:
  `"initialize" => { *client_ui_support = detect_ui_support(&request.params); handle_initialize(&request.params) }`
- The `"tools/list"` arm becomes: `"tools/list" => handle_tools_list(*client_ui_support),`

**Step 4: Run test to verify it passes**

Run: `cargo test -p folio-mcp`
Expected: PASS.

**Step 5: Commit**

```bash
git add folio-mcp/src/main.rs
git commit -m "feat(mcp): negotiate and advertise io.modelcontextprotocol/ui extension"
```

---

## Task 8: MCP — embed widgets and serve `ui://` resources

**Files:**
- Create: `folio-mcp/src/widgets/table.html`
- Create: `folio-mcp/src/widgets/batch.html`
- Modify: `folio-mcp/src/main.rs` (`handle_resources_list` :572-585; `handle_resources_read` :587-617; add consts)

**Step 1: Write the failing test**

```rust
    #[test]
    fn test_ui_resources_listed_and_readable() {
        let list = handle_resources_list().unwrap();
        let uris: Vec<&str> = list["resources"].as_array().unwrap().iter()
            .map(|r| r["uri"].as_str().unwrap()).collect();
        assert!(uris.contains(&"ui://folio/table"));
        assert!(uris.contains(&"ui://folio/batch"));

        for uri in ["ui://folio/table", "ui://folio/batch"] {
            let res = handle_resources_read(&Some(json!({ "uri": uri }))).unwrap();
            assert_eq!(res["contents"][0]["mimeType"], "text/html;profile=mcp-app");
            let html = res["contents"][0]["text"].as_str().unwrap();
            assert!(html.contains("ui/notifications/tool-result"), "{} widget must listen for results", uri);
            assert!(html.contains("ui/initialize"), "{} widget must handshake", uri);
        }
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p folio-mcp test_ui_resources_listed_and_readable`
Expected: FAIL.

**Step 3: Implement**

Create `folio-mcp/src/widgets/table.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<style>
  :root { color-scheme: light dark; }
  body { font: 13px/1.5 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; margin: 0; padding: 12px; }
  h3 { margin: 16px 0 6px; font: 600 13px/1.4 system-ui, sans-serif; }
  table { border-collapse: collapse; width: 100%; margin-bottom: 8px; }
  th, td { text-align: left; padding: 4px 10px; border-bottom: 1px solid rgba(128,128,128,.25); vertical-align: top; }
  th { font: 600 12px/1.4 system-ui, sans-serif; opacity: .7; }
  td.result { font-weight: 600; }
  tr.err td { background: rgba(220,40,40,.12); }
  td.result.err { color: #c0392b; }
  .empty { opacity: .55; padding: 12px; }
</style>
</head>
<body>
<div id="root"><div class="empty">Waiting for results…</div></div>
<script>
  function esc(s){return String(s==null?"":s).replace(/[&<>"]/g,function(m){return {"&":"&amp;","<":"&lt;",">":"&gt;","\"":"&quot;"}[m];});}
  function render(sc){
    var root=document.getElementById("root");
    if(!sc||!Array.isArray(sc.cells)||sc.cells.length===0){root.innerHTML='<div class="empty">No cells to display.</div>';return;}
    var groups={},order=[];
    sc.cells.forEach(function(c){var s=c.section||"";if(!(s in groups)){groups[s]=[];order.push(s);}groups[s].push(c);});
    var html="";
    order.forEach(function(s){
      if(s) html+="<h3>"+esc(s)+"</h3>";
      html+="<table><thead><tr><th>name</th><th>formula</th><th>result</th></tr></thead><tbody>";
      groups[s].forEach(function(c){
        var e=!!c.isError;
        html+='<tr class="'+(e?"err":"")+'"><td>'+esc(c.name)+"</td><td>"+esc(c.formula)+'</td><td class="result '+(e?"err":"")+'">'+esc(c.result)+"</td></tr>";
      });
      html+="</tbody></table>";
    });
    root.innerHTML=html;
  }
  window.addEventListener("message",function(ev){
    var m=ev.data||{};
    if(m.method==="ui/notifications/tool-result"&&m.params){render(m.params.structuredContent);}
  });
  window.parent.postMessage({jsonrpc:"2.0",id:1,method:"ui/initialize",params:{}},"*");
</script>
</body>
</html>
```

Create `folio-mcp/src/widgets/batch.html`:

```html
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8" />
<style>
  :root { color-scheme: light dark; }
  body { font: 13px/1.5 ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; margin: 0; padding: 12px; }
  h3 { margin: 14px 0 6px; font: 600 13px/1.4 system-ui, sans-serif; }
  table { border-collapse: collapse; width: 100%; margin-bottom: 8px; }
  th, td { text-align: left; padding: 4px 10px; border-bottom: 1px solid rgba(128,128,128,.25); vertical-align: top; }
  th { font: 600 12px/1.4 system-ui, sans-serif; opacity: .7; }
  .empty { opacity: .55; padding: 12px; }
</style>
</head>
<body>
<div id="root"><div class="empty">Waiting for results…</div></div>
<script>
  function esc(s){return String(s==null?"":s).replace(/[&<>"]/g,function(m){return {"&":"&amp;","<":"&lt;",">":"&gt;","\"":"&quot;"}[m];});}
  function kv(o){return Object.keys(o||{}).map(function(k){return k+"="+esc(o[k]);}).join(", ");}
  function render(sc){
    var root=document.getElementById("root");
    if(!sc||!Array.isArray(sc.runs)){root.innerHTML='<div class="empty">No runs to display.</div>';return;}
    var html="";
    if(Array.isArray(sc.comparison)&&sc.comparison.length){
      html+="<h3>Comparison"+(sc.compareField?(" — "+esc(sc.compareField)):"")+"</h3>";
      html+="<table><thead><tr><th>#</th><th>variables</th><th>value</th></tr></thead><tbody>";
      sc.comparison.forEach(function(r){html+="<tr><td>"+esc(r.index)+"</td><td>"+kv(r.variables)+"</td><td>"+esc(r.value)+"</td></tr>";});
      html+="</tbody></table>";
    }
    html+="<h3>Runs</h3><table><thead><tr><th>#</th><th>variables</th><th>values</th></tr></thead><tbody>";
    sc.runs.forEach(function(r){html+="<tr><td>"+esc(r.index)+"</td><td>"+kv(r.variables)+"</td><td>"+kv(r.values)+"</td></tr>";});
    html+="</tbody></table>";
    root.innerHTML=html;
  }
  window.addEventListener("message",function(ev){
    var m=ev.data||{};
    if(m.method==="ui/notifications/tool-result"&&m.params){render(m.params.structuredContent);}
  });
  window.parent.postMessage({jsonrpc:"2.0",id:1,method:"ui/initialize",params:{}},"*");
</script>
</body>
</html>
```

In `main.rs`, add consts near the top:

```rust
const WIDGET_TABLE_HTML: &str = include_str!("widgets/table.html");
const WIDGET_BATCH_HTML: &str = include_str!("widgets/batch.html");
```

In `handle_resources_list`, prepend the two UI resources before the `.fmd` documents:

```rust
    let mut resources = vec![
        json!({ "uri": "ui://folio/table", "name": "Folio Results Table",
            "description": "Renders a Folio computation table verbatim.",
            "mimeType": "text/html;profile=mcp-app" }),
        json!({ "uri": "ui://folio/batch", "name": "Folio Comparison Table",
            "description": "Renders a Folio parameter-sweep comparison.",
            "mimeType": "text/html;profile=mcp-app" }),
    ];
    resources.extend(files.iter().map(|f| { /* existing folio://documents/... json */ }));
    Ok(json!({ "resources": resources }))
```

In `handle_resources_read`, branch on the `ui://` scheme before the `folio://documents/` logic:

```rust
    if let Some(html) = match uri {
        "ui://folio/table" => Some(WIDGET_TABLE_HTML),
        "ui://folio/batch" => Some(WIDGET_BATCH_HTML),
        _ => None,
    } {
        return Ok(json!({ "contents": [{
            "uri": uri,
            "mimeType": "text/html;profile=mcp-app",
            "text": html,
            "_meta": { "ui": { "prefersBorder": true } }
        }]}));
    }
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p folio-mcp`
Expected: PASS.

**Step 5: Commit**

```bash
git add folio-mcp/src/widgets/table.html folio-mcp/src/widgets/batch.html folio-mcp/src/main.rs
git commit -m "feat(mcp): embed table/batch widgets and serve ui:// resources"
```

---

## Task 9: MCP — conditional `_meta.ui` tool linkage

**Files:**
- Modify: `folio-mcp/src/main.rs` (`handle_tools_list` — add `_meta` to eval/eval_file/eval_batch when `client_ui_support`)

**Step 1: Write the failing test**

```rust
    #[test]
    fn test_ui_meta_linkage_is_conditional() {
        let off = handle_tools_list(false).unwrap();
        let eval_off = off["tools"].as_array().unwrap().iter().find(|t| t["name"]=="eval").unwrap();
        assert!(eval_off.get("_meta").is_none());

        let on = handle_tools_list(true).unwrap();
        let eval_on = on["tools"].as_array().unwrap().iter().find(|t| t["name"]=="eval").unwrap();
        assert_eq!(eval_on["_meta"]["ui"]["resourceUri"], "ui://folio/table");
        let batch_on = on["tools"].as_array().unwrap().iter().find(|t| t["name"]=="eval_batch").unwrap();
        assert_eq!(batch_on["_meta"]["ui"]["resourceUri"], "ui://folio/batch");
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p folio-mcp test_ui_meta_linkage_is_conditional`
Expected: FAIL.

**Step 3: Implement**

After building the tools array in `handle_tools_list`, when `client_ui_support` is true, attach `_meta` to the relevant tools. Simplest robust approach — build the array into a mutable `Vec<JsonValue>` (or post-process the existing `json!` array), then:

```rust
    let mut value = json!({ "tools": tools });
    if client_ui_support {
        if let Some(arr) = value["tools"].as_array_mut() {
            for t in arr.iter_mut() {
                let uri = match t["name"].as_str() {
                    Some("eval") | Some("eval_file") => Some("ui://folio/table"),
                    Some("eval_batch") => Some("ui://folio/batch"),
                    _ => None,
                };
                if let Some(uri) = uri {
                    t["_meta"] = json!({ "ui": { "resourceUri": uri, "visibility": ["model","app"] } });
                }
            }
        }
    }
    Ok(value)
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p folio-mcp`
Expected: PASS — and re-run the full suite `cargo test -p folio -p folio-mcp`.

**Step 5: Commit**

```bash
git add folio-mcp/src/main.rs
git commit -m "feat(mcp): attach _meta.ui tool linkage for UI-capable clients"
```

---

## Task 10: Docs + final verification

**Files:**
- Modify: `folio-mcp/src/main.rs` (top-of-file `//!` doc block: tools/resources list, note MCP Apps)
- Modify: `README.md` (add a short "MCP Apps / verbatim rendering" note under Usage; ensure no stale mantra references)

**Step 1:** Update the `//!` header in `main.rs` to mention the `ui://folio/table` and `ui://folio/batch` resources and that the eval tools are MCP Apps-enabled.

**Step 2:** Add a brief README subsection explaining that in MCP Apps-capable hosts the eval tools render their table in a widget, and in other hosts the same table is returned as markdown.

**Step 3 (REQUIRED — @superpowers:verification-before-completion):** Run the full suite and a release build, and paste the actual output into the final report:

```
cargo test -p folio -p folio-mcp
cargo build --release -p folio-mcp
```

Expected: all tests PASS; release build succeeds. Do not claim completion without this output.

**Step 4 (manual, note as follow-up):** Verify widget rendering in a host — MCP Inspector or Claude Desktop — by calling `eval` and confirming the table widget renders and matches the markdown. This cannot be unit-tested headless.

**Step 5: Commit**

```bash
git add folio-mcp/src/main.rs README.md
git commit -m "docs: document MCP Apps widgets and verbatim rendering"
```

---

## Done criteria

- `cargo test -p folio -p folio-mcp` green; `cargo build --release -p folio-mcp` succeeds.
- No occurrence of `SACRED` in source or any tool/initialize output.
- `eval`/`eval_file`/`eval_batch` return `structuredContent` + `isError`; no non-standard top-level fields.
- `tools/list` carries titles, `readOnlyHint`/`openWorldHint`, `outputSchema`; `_meta.ui` present only after a UI-capable `initialize`.
- `ui://folio/table` and `ui://folio/batch` are listed and readable with mimeType `text/html;profile=mcp-app`.
- `initialize` advertises `io.modelcontextprotocol/ui`.
- Manual widget check performed (or explicitly logged as outstanding).
