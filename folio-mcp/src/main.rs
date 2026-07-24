//! Folio MCP Server
//!
//! MCP Server implementing spec 2025-11-25, with the MCP Apps (SEP-1865)
//! `io.modelcontextprotocol/ui` extension for verbatim, in-conversation
//! rendering of computation tables. Tool results carry `structuredContent`
//! (no `outputSchema` is declared for the eval tools: hosts MAY validate
//! structuredContent against it and silently strip it on any mismatch, which
//! blanks the widget); UI-capable hosts render the table widget, others fall
//! back to the markdown in the text content.
//! Set `FOLIO_NO_WIDGET=1` to omit the tool->widget linkage entirely so hosts
//! render the markdown text in chat instead of mounting a widget.
//!
//! Tools:
//! - eval: Evaluate a document template (MCP App: ui://folio/table)
//! - eval_file: Evaluate a .fmd file by reference (MCP App: ui://folio/table)
//! - eval_batch: Batch evaluation for parameter sweeps (MCP App: ui://folio/batch)
//! - folio: Get documentation for functions/constants
//! - quick: Compact quick reference
//! - list_functions: List available functions
//! - list_constants: List available constants
//! - decompose: Analyze a value for patterns
//!
//! Resources:
//! - folio://documents - List available .fmd files
//! - folio://documents/{name} - Get specific document
//! - ui://folio/table - MCP Apps widget for eval/eval_file results
//! - ui://folio/batch - MCP Apps widget for eval_batch comparison

use folio::Folio;
use folio_mcp::*;
use std::env;
use std::io::{self, BufRead, Write, IsTerminal};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

const PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "folio";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Bump on each diagnostic rebuild so the `version` tool and `serverInfo`
/// reveal exactly which build a client (e.g. Claude Desktop) is connected to.
const BUILD_TAG: &str = "2026-07-22-apps-spec-fallback";

const WIDGET_TABLE_HTML: &str = include_str!("widgets/table.html");
const WIDGET_BATCH_HTML: &str = include_str!("widgets/batch.html");

// MCP Protocol types
#[derive(Debug, Deserialize)]
struct McpRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<JsonValue>,
    method: String,
    #[serde(default)]
    params: Option<JsonValue>,
}

#[derive(Debug, Serialize)]
struct McpResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<McpError>,
}

fn main() {
    // Initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    // Initialize Folio with standard library and ISIS extensions
    let folio = create_folio_with_isis();

    // Force line-buffered stderr for Docker
    // This ensures logs appear immediately in MCP client
    
    eprintln!("Folio MCP Server v{} started", SERVER_VERSION);
    eprintln!("Protocol: {}", PROTOCOL_VERSION);
    eprintln!("Data path: {}", data_path().display());
    eprintln!("stdin is_terminal: {}", io::stdin().is_terminal());
    eprintln!("stdout is_terminal: {}", io::stdout().is_terminal());

    // List available files at startup
    let files = list_fmd_files();
    eprintln!("Available .fmd files: {}", files.len());
    for f in &files {
        eprintln!("  - {}: {:?}", f.name, f.description);
    }

    // Use BufReader for stdin (line-based protocol)
    let stdin = io::stdin();
    let mut reader = io::BufReader::new(stdin.lock());

    eprintln!("Server ready, waiting for requests...");

    // Negotiated at `initialize`; threaded into per-request handling so tools/list
    // can decide whether to attach MCP Apps UI linkage (consumed in a later task).
    let mut client_ui_support = false;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => {
                // EOF - client disconnected
                eprintln!("Client disconnected (EOF)");
                break;
            }
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                eprintln!("Received: {} bytes", line.len());

                // Parse request
                let request: McpRequest = match serde_json::from_str(line) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Error parsing request: {}", e);
                        let response = McpResponse {
                            jsonrpc: "2.0".to_string(),
                            id: None,
                            result: None,
                            error: Some(McpError {
                                code: -32700,
                                message: format!("Parse error: {}", e),
                                data: None,
                            }),
                        };
                        let mut stdout = io::stdout().lock();
                        let _ = writeln!(stdout, "{}", serde_json::to_string(&response).unwrap());
                        let _ = stdout.flush();
                        continue;
                    }
                };

                eprintln!("Processing: {}", request.method);

                // Handle request
                let response = handle_request(&folio, &request, &mut client_ui_support);

                // Notifications (no id) should NOT receive a response
                if request.id.is_none() {
                    eprintln!("Notification processed (no response): {}", request.method);
                    continue;
                }

                // Write response directly to stdout (no buffering)
                let response_json = serde_json::to_string(&response).unwrap();
                let mut stdout = io::stdout().lock();
                if let Err(e) = writeln!(stdout, "{}", response_json) {
                    eprintln!("Error writing response: {}", e);
                    break;
                }
                if let Err(e) = stdout.flush() {
                    eprintln!("Error flushing stdout: {}", e);
                    break;
                }
                drop(stdout); // Release lock immediately

                eprintln!("Sent response for: {}", request.method);
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }

    eprintln!("Server shutting down");
}

fn handle_request(folio: &Folio, request: &McpRequest, client_ui_support: &mut bool) -> McpResponse {
    let result = match request.method.as_str() {
        // Lifecycle
        "initialize" => {
            *client_ui_support = detect_ui_support(&request.params);
            handle_initialize(&request.params)
        }
        "initialized" => Ok(json!({})),
        "ping" => Ok(json!({})),

        // Tools
        "tools/list" => handle_tools_list(*client_ui_support),
        "tools/call" => handle_tool_call(folio, &request.params),

        // Resources
        "resources/list" => handle_resources_list(),
        "resources/read" => handle_resources_read(&request.params),

        // Prompts (for templates)
        "prompts/list" => handle_prompts_list(),
        "prompts/get" => handle_prompts_get(&request.params),

        _ => Err(McpError {
            code: -32601,
            message: format!("Method not found: {}", request.method),
            data: None,
        }),
    };

    match result {
        Ok(r) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: Some(r),
            error: None,
        },
        Err(e) => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id.clone(),
            result: None,
            error: Some(e),
        },
    }
}

/// Kill-switch: with FOLIO_NO_WIDGET=1 (or "true"), tools/list omits the
/// `_meta.ui` linkage so hosts show the markdown text in chat instead of
/// mounting a widget. The ui:// resources stay registered; only the tool
/// linkage decides whether a widget mounts.
fn widget_disabled() -> bool {
    env::var("FOLIO_NO_WIDGET")
        .map(|v| { let v = v.trim().to_ascii_lowercase(); v == "1" || v == "true" })
        .unwrap_or(false)
}

/// True if the client declared support for the MCP Apps UI extension.
fn detect_ui_support(params: &Option<JsonValue>) -> bool {
    params.as_ref()
        .and_then(|p| p.get("capabilities"))
        .and_then(|c| c.get("extensions"))
        .and_then(|e| e.get("io.modelcontextprotocol/ui"))
        .is_some()
}

fn handle_initialize(params: &Option<JsonValue>) -> Result<JsonValue, McpError> {
    let client_info = params.as_ref()
        .and_then(|p| p.get("clientInfo"))
        .and_then(|c| c.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    // Use client's protocol version for compatibility
    let client_protocol = params.as_ref()
        .and_then(|p| p.get("protocolVersion"))
        .and_then(|v| v.as_str())
        .unwrap_or(PROTOCOL_VERSION);

    eprintln!("Client connected: {} (protocol: {})", client_info, client_protocol);

    Ok(json!({
        "protocolVersion": client_protocol,
        "serverInfo": {
            "name": SERVER_NAME,
            "version": format!("{}+{}", SERVER_VERSION, BUILD_TAG),
            "description": "Markdown Computational Documents - Jupyter for LLMs"
        },
        "capabilities": {
            "tools": {
                "listChanged": false
            },
            "resources": {
                "subscribe": false,
                "listChanged": false
            },
            "prompts": {
                "listChanged": false
            },
            "extensions": {
                "io.modelcontextprotocol/ui": { "mimeTypes": ["text/html;profile=mcp-app"] }
            }
        },
        "instructions": "Folio evaluates Markdown Computational Documents with arbitrary-precision arithmetic. Use `folio()` to explore functions and `eval`/`eval_file` to compute. The eval tools return a complete, auditable results table that is designed to be shown to the user in full; present that table before adding commentary."
    }))
}

fn handle_tools_list(client_ui_support: bool) -> Result<JsonValue, McpError> {
    let mut value = json!({
        "tools": [
            {
                "name": "eval",
                "title": "Evaluate Folio Document",
                "description": "Evaluate a Folio markdown document with formulas. Returns a complete, auditable results table (one row per named cell) designed to be shown to the user in full; do not summarize, truncate, or paraphrase it.",
                "annotations": { "readOnlyHint": true, "openWorldHint": false },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "template": {
                            "type": "string",
                            "description": "Markdown document with formula tables"
                        },
                        "variables": {
                            "type": "object",
                            "description": "External variables to inject",
                            "additionalProperties": { "type": "string" }
                        },
                        "precision": {
                            "type": "integer",
                            "description": "Decimal precision (default: 50)",
                            "default": 50
                        }
                    },
                    "required": ["template"]
                }
            },
            {
                "name": "eval_file",
                "title": "Evaluate Folio File",
                "description": "Evaluate a .fmd file from the data directory by name. Returns a complete, auditable results table designed to be shown to the user in full; do not summarize, truncate, or paraphrase it.",
                "annotations": { "readOnlyHint": true, "openWorldHint": false },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "File name without .fmd extension"
                        },
                        "variables": {
                            "type": "object",
                            "description": "External variables to inject",
                            "additionalProperties": { "type": "string" }
                        },
                        "precision": {
                            "type": "integer",
                            "description": "Decimal precision (default: 50)"
                        }
                    },
                    "required": ["name"]
                }
            },
            {
                "name": "eval_batch",
                "title": "Folio Parameter Sweep",
                "description": "Evaluate a template with multiple variable sets for parameter sweeps.",
                "annotations": { "readOnlyHint": true, "openWorldHint": false },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "template": {
                            "type": "string",
                            "description": "Markdown document template"
                        },
                        "variable_sets": {
                            "type": "array",
                            "items": { "type": "object" },
                            "description": "Array of variable sets to evaluate"
                        },
                        "compare_field": {
                            "type": "string",
                            "description": "Field to compare across runs"
                        }
                    },
                    "required": ["template", "variable_sets"]
                }
            },
            {
                "name": "folio",
                "title": "Folio Help",
                "description": "Get documentation for a function, constant, or general help about Folio.",
                "annotations": { "readOnlyHint": true, "openWorldHint": false },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Function or constant name. Omit for general help."
                        },
                        "compact": {
                            "type": "boolean",
                            "description": "Return compact listing (function names only, ~400 tokens vs ~3000)",
                            "default": false
                        }
                    }
                }
            },
            {
                "name": "quick",
                "title": "Folio Quick Reference",
                "description": "Compact quick reference (~400 tokens). Lists function names grouped by category with Object return fields.",
                "annotations": { "readOnlyHint": true, "openWorldHint": false },
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "version",
                "title": "Folio Version",
                "description": "Returns the running Folio server version, build tag, MCP protocol version, and the full list of tools the server registers. Call this to confirm exactly which build is connected.",
                "annotations": { "readOnlyHint": true, "openWorldHint": false },
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "list_functions",
                "title": "List Folio Functions",
                "description": "List all available functions, optionally by category.",
                "annotations": { "readOnlyHint": true, "openWorldHint": false },
                "outputSchema": { "type": "object", "properties": { "functions": { "type": "array" } }, "required": ["functions"] },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "category": {
                            "type": "string",
                            "description": "Filter: math, trig, aggregate, isis",
                            "enum": ["math", "trig", "aggregate", "isis"]
                        }
                    }
                }
            },
            {
                "name": "list_constants",
                "title": "List Folio Constants",
                "description": "List available mathematical constants with sources.",
                "annotations": { "readOnlyHint": true, "openWorldHint": false },
                "outputSchema": { "type": "object", "properties": { "constants": { "type": "array" } }, "required": ["constants"] },
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "decompose",
                "title": "Decompose Value",
                "description": "Analyze a value for patterns involving φ, π, e.",
                "annotations": { "readOnlyHint": true, "openWorldHint": false },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "value": {
                            "type": "string",
                            "description": "Numeric value to analyze"
                        },
                        "precision": {
                            "type": "integer",
                            "description": "Analysis precision (default: 50)"
                        }
                    },
                    "required": ["value"]
                }
            }
        ]
    });
    // Compatibility: Claude Desktop drops a tool that carries BOTH a top-level
    // `title` and an `annotations` object together with required input params
    // (observed: eval/eval_file/eval_batch/decompose silently disappeared from
    // the tool surface). No other server combines tool-level `title` with
    // `annotations`. Relocate the human title into `annotations.title` — the
    // older, universally-supported location (e.g. Desktop Commander) — and drop
    // the top-level `title`, eliminating the field co-occurrence.
    if let Some(arr) = value["tools"].as_array_mut() {
        for t in arr.iter_mut() {
            if let Some(title) = t.as_object_mut().and_then(|o| o.remove("title")) {
                if let Some(ann) = t.get_mut("annotations").and_then(|a| a.as_object_mut()) {
                    ann.insert("title".to_string(), title);
                }
            }
        }
    }

    // Nested `_meta.ui.resourceUri` is the SEP-1865 stable form (the flat
    // `_meta["ui/resourceUri"]` key is deprecated). No linkage at all when the
    // FOLIO_NO_WIDGET kill-switch is set: the host then renders the markdown
    // text content in chat.
    if client_ui_support && !widget_disabled() {
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
}

fn handle_resources_list() -> Result<JsonValue, McpError> {
    let files = list_fmd_files();

    let mut resources = vec![
        json!({ "uri": "ui://folio/table", "name": "Folio Results Table",
            "description": "Renders a Folio computation table verbatim.",
            "mimeType": "text/html;profile=mcp-app" }),
        json!({ "uri": "ui://folio/batch", "name": "Folio Comparison Table",
            "description": "Renders a Folio parameter-sweep comparison.",
            "mimeType": "text/html;profile=mcp-app" }),
    ];
    resources.extend(files.iter().map(|f| {
        json!({
            "uri": format!("folio://documents/{}", f.name),
            "name": f.name,
            "description": f.description.clone().unwrap_or_else(|| format!("Folio document: {}.fmd", f.name)),
            "mimeType": "text/markdown"
        })
    }));

    Ok(json!({ "resources": resources }))
}

fn handle_resources_read(params: &Option<JsonValue>) -> Result<JsonValue, McpError> {
    let uri = params.as_ref()
        .and_then(|p| p.get("uri"))
        .and_then(|u| u.as_str())
        .ok_or_else(|| McpError {
            code: -32602,
            message: "Missing uri parameter".to_string(),
            data: None,
        })?;

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

    let name = uri.strip_prefix("folio://documents/")
        .ok_or_else(|| McpError {
            code: -32602,
            message: format!("Invalid URI: {}. Expected folio://documents/{{name}}", uri),
            data: None,
        })?;

    let content = load_fmd_file(name).map_err(|e| McpError {
        code: -32602,
        message: e,
        data: None,
    })?;

    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "text/markdown",
            "text": content
        }]
    }))
}

fn handle_prompts_list() -> Result<JsonValue, McpError> {
    Ok(json!({
        "prompts": [
            {
                "name": "mortgage_calculator",
                "description": "Calculate monthly mortgage payment",
                "arguments": [
                    {"name": "principal", "description": "Loan amount", "required": true},
                    {"name": "rate", "description": "Annual rate (e.g., 0.065)", "required": true},
                    {"name": "years", "description": "Loan term in years", "required": true}
                ]
            },
            {
                "name": "compound_interest",
                "description": "Calculate compound interest",
                "arguments": [
                    {"name": "principal", "description": "Initial investment", "required": true},
                    {"name": "rate", "description": "Annual rate", "required": true},
                    {"name": "years", "description": "Time period", "required": true}
                ]
            },
            {
                "name": "isis_analysis",
                "description": "Analyze value using ISIS transform",
                "arguments": [
                    {"name": "value", "description": "Value to analyze", "required": true}
                ]
            }
        ]
    }))
}

fn handle_prompts_get(params: &Option<JsonValue>) -> Result<JsonValue, McpError> {
    let params = params.as_ref().ok_or_else(|| McpError {
        code: -32602,
        message: "Missing params".to_string(),
        data: None,
    })?;

    let name = params.get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| McpError {
            code: -32602,
            message: "Missing name parameter".to_string(),
            data: None,
        })?;

    let arguments = params.get("arguments");

    // Map prompt names to file names and extract variable mappings
    let (file_name, var_mappings): (&str, Vec<(&str, &str)>) = match name {
        "mortgage_calculator" => ("mortgage", vec![
            ("principal", "principal"),
            ("rate", "annual_rate"),
            ("years", "years"),
        ]),
        "compound_interest" => ("compound_interest", vec![
            ("principal", "principal"),
            ("rate", "rate"),
            ("years", "years"),
        ]),
        "isis_analysis" => ("isis_analysis", vec![
            ("value", "target"),
        ]),
        // Also allow direct file names
        _ => (name, vec![]),
    };

    // Load the template
    let template = load_fmd_file(file_name).map_err(|e| McpError {
        code: -32602,
        message: e,
        data: Some(json!({ "available_prompts": ["mortgage_calculator", "compound_interest", "isis_analysis"] })),
    })?;

    // Build the variable injection instruction
    let mut var_instructions = String::new();
    if let Some(args) = arguments {
        if let Some(obj) = args.as_object() {
            for (arg_name, template_var) in &var_mappings {
                if let Some(value) = obj.get(*arg_name) {
                    if let Some(val_str) = value.as_str() {
                        var_instructions.push_str(&format!("- Set `{}` to `{}`\n", template_var, val_str));
                    }
                }
            }
        }
    }

    let prompt_text = if var_instructions.is_empty() {
        format!("Please evaluate this Folio document:\n\n```markdown\n{}\n```", template)
    } else {
        format!(
            "Please evaluate this Folio document with the following variable overrides:\n\n{}\n\n```markdown\n{}\n```",
            var_instructions, template
        )
    };

    Ok(json!({
        "description": format!("Folio prompt: {}", name),
        "messages": [{
            "role": "user",
            "content": {
                "type": "text",
                "text": prompt_text
            }
        }]
    }))
}

fn handle_tool_call(folio: &Folio, params: &Option<JsonValue>) -> Result<JsonValue, McpError> {
    let params = params.as_ref().ok_or(McpError {
        code: -32602,
        message: "Missing params".to_string(),
        data: None,
    })?;

    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or(McpError {
            code: -32602,
            message: "Missing tool name".to_string(),
            data: None,
        })?;

    let args = params.get("arguments").cloned().unwrap_or(json!({}));

    match name {
        "eval" => tool_eval(folio, args),
        "eval_file" => tool_eval_file(folio, args),
        "eval_batch" => tool_eval_batch(folio, args),
        "folio" => tool_folio(folio, args),
        "quick" => tool_quick(folio),
        "version" => tool_version(),
        "list_functions" => tool_list_functions(folio, args),
        "list_constants" => tool_list_constants(folio, args),
        "decompose" => tool_decompose(folio, args),
        _ => Err(McpError {
            code: -32602,
            message: format!("Unknown tool: {}", name),
            data: None,
        }),
    }
}

/// Diagnostic tool: report the running build (so we can confirm exactly which
/// server a client like Claude Desktop is connected to) plus the full list of
/// tools the server registers — useful when a client silently hides some.
fn tool_version() -> Result<JsonValue, McpError> {
    let names: Vec<String> = handle_tools_list(false)
        .ok()
        .and_then(|v| v["tools"].as_array().map(|arr| {
            arr.iter().filter_map(|t| t["name"].as_str().map(String::from)).collect()
        }))
        .unwrap_or_default();
    let text = format!(
        "Folio MCP server\n- version: {}\n- build: {}\n- protocol: {}\n- tools registered by server ({}): {}",
        SERVER_VERSION, BUILD_TAG, PROTOCOL_VERSION, names.len(), names.join(", ")
    );
    Ok(json!({
        "content": [{ "type": "text", "text": text, "annotations": { "audience": ["user"], "priority": 1.0 } }],
        "structuredContent": {
            "version": SERVER_VERSION,
            "build": BUILD_TAG,
            "protocol": PROTOCOL_VERSION,
            "toolCount": names.len(),
            "registeredTools": names
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_args(template: &str) -> JsonValue {
        json!({ "template": template })
    }

    const SIMPLE_DOC: &str =
        "## T\n| name | formula | result |\n|------|---------|--------|\n| a | 1 | |\n";

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

    #[test]
    fn test_eval_errors_are_document_ordered() {
        // Two error cells: the errors array must be in document order (a then b),
        // deterministically — not in HashMap-iteration order.
        let folio = create_folio_with_isis();
        let doc = "## T\n| name | formula | result |\n|------|---------|--------|\n| a | 1 / 0 | |\n| b | 2 / 0 | |\n";
        let res = tool_eval(&folio, eval_args(doc)).unwrap();
        let errs = res["structuredContent"]["errors"].as_array().unwrap();
        assert_eq!(errs.len(), 2);
        assert_eq!(errs[0]["cell"], "a");
        assert_eq!(errs[1]["cell"], "b");
    }

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

    #[test]
    fn test_eval_batch_text_has_markdown_table() {
        let folio = create_folio_with_isis();
        let args = json!({
            "template": "## T\n| name | formula | result |\n|------|---------|--------|\n| out | x * 2 | |\n",
            "variable_sets": [ {"x": "5"}, {"x": "10"} ],
            "compare_field": "out"
        });
        let res = tool_eval_batch(&folio, args).unwrap();
        let text = res["content"][0]["text"].as_str().unwrap();
        // Non-widget clients (Claude Code) must see the sweep data as markdown.
        assert!(text.contains("| # | variables | out |"), "expected comparison table header:\n{}", text);
        assert!(text.contains("x=5"), "expected variables listed:\n{}", text);
    }

    #[test]
    fn test_decompose_recognizes_constants() {
        let folio = create_folio_with_isis();
        let exprs = |res: &JsonValue| -> Vec<String> {
            res["structuredContent"]["matches"].as_array().unwrap().iter()
                .filter_map(|x| x["expression"].as_str().map(String::from)).collect()
        };
        let pi = tool_decompose(&folio, json!({"value": "3.141592653589793"})).unwrap();
        assert!(exprs(&pi).contains(&"π".to_string()), "π: {:?}", exprs(&pi));
        let two_pi = tool_decompose(&folio, json!({"value": "6.283185307179586"})).unwrap();
        assert!(exprs(&two_pi).contains(&"2π".to_string()), "2π: {:?}", exprs(&two_pi));
        let phi_sq = tool_decompose(&folio, json!({"value": "2.618033988749895"})).unwrap();
        assert!(exprs(&phi_sq).contains(&"φ^2".to_string()), "φ^2: {:?}", exprs(&phi_sq));
        let none = tool_decompose(&folio, json!({"value": "42"})).unwrap();
        assert!(none["structuredContent"]["matches"].as_array().unwrap().is_empty(), "42 should not match");
    }

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

    #[test]
    fn test_tools_list_metadata() {
        let res = handle_tools_list(false).unwrap();
        let tools = res["tools"].as_array().unwrap();
        for t in tools {
            // Claude Desktop drops tools that carry BOTH a top-level `title` and an
            // `annotations` object alongside required params, so the human title
            // lives in `annotations.title` (Desktop Commander's proven pattern).
            assert!(t.get("title").is_none(), "{} must not carry a top-level title", t["name"]);
            assert!(t["annotations"]["title"].is_string(), "{} missing annotations.title", t["name"]);
            assert_eq!(t["annotations"]["readOnlyHint"], true, "{} readOnlyHint", t["name"]);
            assert_eq!(t["annotations"]["openWorldHint"], false, "{} openWorldHint", t["name"]);
        }
        let eval = tools.iter().find(|t| t["name"] == "eval").unwrap();
        // No outputSchema on the eval tools: hosts MAY validate structuredContent
        // against a declared schema and strip it on mismatch, blanking the widget.
        assert!(eval.get("outputSchema").is_none(), "eval must not declare outputSchema");
        // No UI linkage when the client does not support the extension.
        assert!(eval.get("_meta").is_none());
    }

    #[test]
    fn test_version_tool_is_surfaceable_and_reports_registry() {
        // The diagnostic `version` tool must be in the always-shown category:
        // no required params, no _meta, title relocated into annotations.title.
        let res = handle_tools_list(false).unwrap();
        let tools = res["tools"].as_array().unwrap();
        let v = tools.iter().find(|t| t["name"] == "version").expect("version tool registered");
        assert!(v["inputSchema"].get("required").is_none(), "version must have no required params");
        assert!(v.get("_meta").is_none(), "version must have no _meta");
        assert!(v["annotations"]["title"].is_string(), "version title in annotations");

        // tool_version reports the build tag and the full server-side registry,
        // so a client that hides tools can still be shown what the server sent.
        let out = tool_version().unwrap();
        assert_eq!(out["structuredContent"]["build"], BUILD_TAG);
        let names: Vec<&str> = out["structuredContent"]["registeredTools"].as_array().unwrap()
            .iter().filter_map(|n| n.as_str()).collect();
        assert!(names.contains(&"eval"), "registry must include eval");
        assert!(names.contains(&"version"), "registry must include version itself");
        assert!(names.len() >= 9, "expected >= 9 registered tools, got {}", names.len());
    }

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
            assert!(html.contains("ui/notifications/initialized"), "{} widget must complete the handshake (initialized)", uri);
            assert!(html.contains("ui/notifications/size-changed"), "{} widget must report its content size", uri);
        }
    }
}
