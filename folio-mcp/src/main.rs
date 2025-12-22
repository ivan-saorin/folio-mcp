//! Folio MCP Server
//!
//! MCP Server implementing spec 2025-11-25
//!
//! Tools:
//! - eval: Evaluate a document template
//! - eval_file: Evaluate a .fmd file by reference
//! - eval_batch: Batch evaluation for parameter sweeps
//! - help: Get documentation for functions/constants
//! - list_functions: List available functions
//! - list_constants: List available constants
//! - decompose: Analyze a value for patterns
//!
//! Resources:
//! - folio://documents - List available .fmd files
//! - folio://documents/{name} - Get specific document

use folio::Folio;
use folio_core::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write, IsTerminal};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

const PROTOCOL_VERSION: &str = "2025-11-25";
const SERVER_NAME: &str = "folio";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

const SACRED_MANTRA: &str = "\n\nSACRED MANTRA: Claude MUST SHOW to the user the calculated markdown exactly as received by this tool, only after you can further explain / elaborate.";

/// Get the data path from environment
fn data_path() -> PathBuf {
    env::var("FOLIO_DATA_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/app/folio"))
}

/// List all .fmd files in data path
fn list_fmd_files() -> Vec<FmdFileInfo> {
    let path = data_path();
    let mut files = Vec::new();

    // Check root and examples subdirectory
    for dir in [path.clone(), path.join("examples")] {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path.extension().map_or(false, |e| e == "fmd") {
                    if let Some(name) = file_path.file_stem().and_then(|s| s.to_str()) {
                        let metadata = fs::metadata(&file_path).ok();
                        files.push(FmdFileInfo {
                            name: name.to_string(),
                            path: file_path.to_string_lossy().to_string(),
                            size: metadata.as_ref().map(|m| m.len()),
                            description: extract_description(&file_path),
                        });
                    }
                }
            }
        }
    }

    files
}

/// Extract description from first line comment in .fmd file
fn extract_description(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let first_line = content.lines().next()?;
    if first_line.starts_with("<!-- ") && first_line.ends_with(" -->") {
        Some(first_line[5..first_line.len()-4].to_string())
    } else if first_line.starts_with("# ") {
        Some(first_line[2..].to_string())
    } else {
        None
    }
}

/// Load a .fmd file by name
fn load_fmd_file(name: &str) -> Result<String, String> {
    let base = data_path();

    // Try root first, then examples
    let candidates = [
        base.join(format!("{}.fmd", name)),
        base.join("examples").join(format!("{}.fmd", name)),
    ];

    for path in candidates {
        if path.exists() {
            return fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read '{}': {}", path.display(), e));
        }
    }

    Err(format!("File '{}' not found. Available: {:?}",
        name,
        list_fmd_files().iter().map(|f| &f.name).collect::<Vec<_>>()))
}

#[derive(Debug, Serialize)]
struct FmdFileInfo {
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

// MCP Protocol types
#[derive(Debug, Deserialize)]
struct McpRequest {
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

#[derive(Debug, Serialize)]
struct McpError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<JsonValue>,
}

/// Create Folio with standard library and ISIS extensions
fn create_folio_with_isis() -> Folio {
    // Load standard library
    let registry = folio_std::standard_registry();
    // Add ISIS extensions
    let registry = folio_isis::load_isis_extensions(registry);
    Folio::new(registry)
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
                let response = handle_request(&folio, &request);

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

fn handle_request(folio: &Folio, request: &McpRequest) -> McpResponse {
    let result = match request.method.as_str() {
        // Lifecycle
        "initialize" => handle_initialize(&request.params),
        "initialized" => Ok(json!({})),
        "ping" => Ok(json!({})),

        // Tools
        "tools/list" => handle_tools_list(),
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
            "version": SERVER_VERSION,
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
            }
        },
        "instructions": "Folio provides computational markdown documents with arbitrary precision arithmetic. Use 'folio()' to explore available functions. Load .fmd files from the data directory using 'eval_file'. IMPORTANT: Always display the computed results to the user in your response - MUST SHOW (SACRED MANTRA!!!) the calculated markdown exacly as the eval tool output, only after you can explain."
    }))
}

fn handle_tools_list() -> Result<JsonValue, McpError> {
    Ok(json!({
        "tools": [
            {
                "name": "eval",
                "description": "Evaluate a Folio markdown document with formulas. Returns computed results for all cells.",
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
                "description": "Evaluate a .fmd file from the data directory by name.",
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
                "description": "Evaluate a template with multiple variable sets for parameter sweeps.",
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
                "description": "Get documentation for a function, constant, or general help about Folio.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Function or constant name. Omit for general help."
                        }
                    }
                }
            },
            {
                "name": "list_functions",
                "description": "List all available functions, optionally by category.",
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
                "description": "List available mathematical constants with sources.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "decompose",
                "description": "Analyze a value for patterns involving φ, π, e.",
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
    }))
}

fn handle_resources_list() -> Result<JsonValue, McpError> {
    let files = list_fmd_files();

    let resources: Vec<JsonValue> = files.iter().map(|f| {
        json!({
            "uri": format!("folio://documents/{}", f.name),
            "name": f.name,
            "description": f.description.clone().unwrap_or_else(|| format!("Folio document: {}.fmd", f.name)),
            "mimeType": "text/markdown"
        })
    }).collect();

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

fn tool_eval(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
    let template = args.get("template")
        .and_then(|v| v.as_str())
        .ok_or(McpError {
            code: -32602,
            message: "Missing template argument".to_string(),
            data: None,
        })?;

    let variables: HashMap<String, Value> = args.get("variables")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), json_to_value(v))).collect())
        .unwrap_or_default();

    let result = folio.eval(template, &variables);

    let markdown_with_mantra = format!("{}{}", result.markdown, SACRED_MANTRA);

    Ok(json!({
        "content": [{ "type": "text", "text": markdown_with_mantra }],
        "values": result.values.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect::<HashMap<_, _>>(),
        "errors": result.errors.iter().map(|e| json!({"code": e.code, "message": e.message})).collect::<Vec<_>>(),
        "isError": !result.errors.is_empty()
    }))
}

fn tool_eval_file(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
    let name = args.get("name")
        .and_then(|v| v.as_str())
        .ok_or(McpError {
            code: -32602,
            message: "Missing name argument".to_string(),
            data: Some(json!({"available": list_fmd_files().iter().map(|f| &f.name).collect::<Vec<_>>()})),
        })?;

    let template = load_fmd_file(name).map_err(|e| McpError {
        code: -32602,
        message: e,
        data: None,
    })?;

    let variables: HashMap<String, Value> = args.get("variables")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), json_to_value(v))).collect())
        .unwrap_or_default();

    let result = folio.eval(&template, &variables);

    let markdown_with_mantra = format!("{}{}", result.markdown, SACRED_MANTRA);

    Ok(json!({
        "content": [{ "type": "text", "text": markdown_with_mantra }],
        "source_file": format!("{}.fmd", name),
        "values": result.values.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect::<HashMap<_, _>>(),
        "errors": result.errors.iter().map(|e| json!({"code": e.code, "message": e.message})).collect::<Vec<_>>(),
        "isError": !result.errors.is_empty()
    }))
}

fn tool_eval_batch(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
    let template = args.get("template")
        .and_then(|v| v.as_str())
        .ok_or(McpError { code: -32602, message: "Missing template".to_string(), data: None })?;

    let variable_sets = args.get("variable_sets")
        .and_then(|v| v.as_array())
        .ok_or(McpError { code: -32602, message: "Missing variable_sets".to_string(), data: None })?;

    let compare_field = args.get("compare_field").and_then(|v| v.as_str());
    let mut results = Vec::new();
    let mut comparison = Vec::new();

    for (i, vars) in variable_sets.iter().enumerate() {
        let variables: HashMap<String, Value> = vars.as_object()
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), json_to_value(v))).collect())
            .unwrap_or_default();

        let result = folio.eval(template, &variables);

        if let Some(field) = compare_field {
            if let Some(value) = result.values.get(field) {
                comparison.push(json!({"index": i, "variables": vars, "value": value_to_json(value)}));
            }
        }

        results.push(json!({
            "index": i,
            "variables": vars,
            "values": result.values.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect::<HashMap<_, _>>(),
            "has_errors": !result.errors.is_empty()
        }));
    }

    let batch_summary = format!("Evaluated {} sets{}", results.len(), SACRED_MANTRA);

    Ok(json!({
        "content": [{ "type": "text", "text": batch_summary }],
        "results": results,
        "comparison": if compare_field.is_some() { Some(comparison) } else { None }
    }))
}

fn tool_folio(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
    let name = args.get("name").and_then(|v| v.as_str());
    let help = folio.help(name);

    Ok(json!({
        "content": [{ "type": "text", "text": format_help(&help) }],
        "data": value_to_json(&help)
    }))
}

fn format_help(help: &Value) -> String {
    match help {
        Value::Object(map) => {
            let mut out = String::new();
            if let Some(Value::Text(n)) = map.get("name") { out.push_str(&format!("# {}\n\n", n)); }
            if let Some(Value::Text(d)) = map.get("description") { out.push_str(&format!("{}\n\n", d)); }
            if let Some(Value::Text(u)) = map.get("usage") { out.push_str(&format!("**Usage:** `{}`\n\n", u)); }
            out
        }
        Value::Error(e) => format!("Error: {}", e.message),
        _ => format!("{:?}", help),
    }
}

fn tool_list_functions(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
    let category = args.get("category").and_then(|v| v.as_str());
    let functions = folio.list_functions(category);
    Ok(json!({ "content": [{ "type": "text", "text": "Functions listed" }], "data": value_to_json(&functions) }))
}

fn tool_list_constants(folio: &Folio, _args: JsonValue) -> Result<JsonValue, McpError> {
    let constants = folio.list_constants();
    Ok(json!({ "content": [{ "type": "text", "text": "Constants listed" }], "data": value_to_json(&constants) }))
}

fn tool_decompose(_folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
    let value_str = args.get("value")
        .and_then(|v| v.as_str())
        .ok_or(McpError { code: -32602, message: "Missing value".to_string(), data: None })?;

    Ok(json!({
        "content": [{ "type": "text", "text": format!("Analysis of {}\n\nPattern detection pending implementation.", value_str) }],
        "value": value_str,
        "patterns": {},
        "_note": "DECOMPOSE implementation pending"
    }))
}

fn json_to_value(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() { Value::Number(folio_core::Number::from_i64(i)) }
            else { Value::Text(n.to_string()) }
        }
        JsonValue::String(s) => {
            match folio_core::Number::from_str(s) {
                Ok(n) => Value::Number(n),
                Err(_) => Value::Text(s.clone()),
            }
        }
        JsonValue::Array(arr) => Value::List(arr.iter().map(json_to_value).collect()),
        JsonValue::Object(obj) => Value::Object(obj.iter().map(|(k, v)| (k.clone(), json_to_value(v))).collect()),
    }
}

fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Number(n) => JsonValue::String(n.to_string()),
        Value::Text(s) => JsonValue::String(s.clone()),
        Value::List(l) => JsonValue::Array(l.iter().map(value_to_json).collect()),
        Value::Object(o) => JsonValue::Object(o.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect()),
        Value::Error(e) => json!({"_error": {"code": e.code, "message": e.message}}),
    }
}
