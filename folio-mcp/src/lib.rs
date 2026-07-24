//! Folio service layer — the tool implementations shared by every transport.
//!
//! `folio-mcp` (stdio MCP server, `src/main.rs`) and `folio-httpd` (HTTP JSON
//! API for the automa stack) are thin protocol adapters over this crate.
//! Tool functions return MCP-shaped CallToolResult JSON (`content` +
//! `structuredContent` + `isError`); each transport reshapes as needed.

use folio::Folio;
use folio_core::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use serde::Serialize;
use serde_json::{json, Value as JsonValue};

/// Get the data path from environment
pub fn data_path() -> PathBuf {
    env::var("FOLIO_DATA_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/app/folio"))
}

/// List all .fmd files in data path
pub fn list_fmd_files() -> Vec<FmdFileInfo> {
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
pub fn extract_description(path: &Path) -> Option<String> {
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

/// Extract the base name from various input formats:
/// - "mortgage" -> "mortgage"
/// - "mortgage.fmd" -> "mortgage"
/// - "/path/to/mortgage.fmd" -> "mortgage"
/// - "C:\path\to\mortgage.fmd" -> "mortgage"
/// - "examples/mortgage.fmd" -> "mortgage"
pub fn extract_fmd_name(input: &str) -> String {
    let input = input.trim();

    // Handle both forward and back slashes for cross-platform compatibility
    let normalized = input.replace('\\', "/");

    // Get the filename part (after last slash)
    let filename = normalized
        .rsplit('/')
        .next()
        .unwrap_or(&normalized);

    // Remove .fmd extension if present (case-insensitive)
    let name = if filename.to_lowercase().ends_with(".fmd") {
        &filename[..filename.len() - 4]
    } else {
        filename
    };

    name.to_string()
}

/// Load a .fmd file by name, filename, or path
/// Accepts multiple formats:
/// - name: "mortgage"
/// - filename: "mortgage.fmd"
/// - path: "/any/path/to/mortgage.fmd" or "C:\path\to\mortgage.fmd"
///
/// When running natively (not in Docker), the host path is tried directly first.
/// When running in Docker, the path won't exist so we fall back to name extraction.
pub fn load_fmd_file(input: &str) -> Result<String, String> {
    let input = input.trim();

    eprintln!("load_fmd_file: input='{}'", input);

    // First, try the input directly as a path (works for native execution)
    // This handles cases where the LLM provides a full valid path
    let direct_path = Path::new(input);
    if direct_path.is_absolute() && direct_path.exists() {
        eprintln!("load_fmd_file: found at direct path '{}'", input);
        return fs::read_to_string(direct_path)
            .map_err(|e| format!("Failed to read '{}': {}", input, e));
    }

    // Also try with .fmd extension added if not present
    if !input.to_lowercase().ends_with(".fmd") {
        let with_ext = format!("{}.fmd", input);
        let path_with_ext = Path::new(&with_ext);
        if path_with_ext.is_absolute() && path_with_ext.exists() {
            eprintln!("load_fmd_file: found at direct path with extension '{}'", with_ext);
            return fs::read_to_string(path_with_ext)
                .map_err(|e| format!("Failed to read '{}': {}", with_ext, e));
        }
    }

    // Extract just the name and try the data directory
    let base = data_path();
    let name = extract_fmd_name(input);

    eprintln!("load_fmd_file: extracted name='{}'", name);

    if name.is_empty() {
        return Err(format!(
            "Invalid file reference: '{}'. Please provide a file name like 'mortgage' or 'mortgage.fmd'. Available: {:?}",
            input,
            list_fmd_files().iter().map(|f| &f.name).collect::<Vec<_>>()
        ));
    }

    // Try multiple locations in the data directory
    let candidates = [
        base.join(format!("{}.fmd", name)),
        base.join("examples").join(format!("{}.fmd", name)),
        // Also try case variations
        base.join(format!("{}.fmd", name.to_lowercase())),
        base.join("examples").join(format!("{}.fmd", name.to_lowercase())),
    ];

    for path in candidates {
        if path.exists() {
            eprintln!("load_fmd_file: found at '{}'", path.display());
            return fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read '{}': {}", path.display(), e));
        }
    }

    // Provide helpful error with available files
    let available: Vec<_> = list_fmd_files().iter().map(|f| f.name.clone()).collect();
    Err(format!(
        "File '{}' not found (extracted from '{}'). Available files: {:?}",
        name, input, available
    ))
}

#[derive(Debug, Serialize)]
pub struct FmdFileInfo {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Tool-level error, JSON-RPC shaped. The MCP transport serializes it as the
/// JSON-RPC `error` object; the HTTP transport maps it to a 4xx JSON body.
#[derive(Debug, Serialize)]
pub struct McpError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<JsonValue>,
}

/// Create Folio with standard library, stats, sequences, and ISIS extensions
pub fn create_folio_with_isis() -> Folio {
    // Load standard library
    let registry = folio_std::standard_registry();
    // Add statistics functions
    let registry = folio_stats::load_stats_library(registry);
    // Add sequence functions
    let registry = folio_sequence::load_sequence_library(registry);
    // Add finance functions
    let registry = folio_finance::load_finance_library(registry);
    // Add ISIS extensions
    let registry = folio_isis::load_isis_extensions(registry);
    // Add matrix/linear algebra functions
    let registry = folio_matrix::load_matrix_library(registry);
    // Add physical units and conversions
    let registry = folio_units::load_units_library(registry);
    // Add text manipulation functions
    let registry = folio_text::load_text_library(registry);
    // Add kitchen/cooking functions
    let registry = folio_kitchen::load_kitchen_library(registry);
    Folio::new(registry)
}

/// Build a spec-compliant CallToolResult for a single-document evaluation.
pub fn eval_result_json(result: &folio::EvalResult) -> JsonValue {
    // Derive errors in document order from the ordered `cells`, looking up each
    // error cell's `Value::Error` for its code/message. Iterating the ordered
    // cells rather than the `values` HashMap keeps the output deterministic
    // (same input => same output), even with multiple error cells.
    let errors: Vec<JsonValue> = result.cells.iter()
        .filter(|c| c.is_error)
        .filter_map(|c| {
            if let Some(Value::Error(e)) = result.values.get(&c.name) {
                Some(json!({ "code": e.code, "message": e.message, "cell": c.name }))
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

pub fn tool_eval(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
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

    Ok(eval_result_json(&result))
}

pub fn tool_eval_file(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
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

    let mut out = eval_result_json(&result);
    out["structuredContent"]["sourceFile"] = json!(format!("{}.fmd", name));
    Ok(out)
}

pub fn tool_eval_batch(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
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
            "isError": result.values.values().any(|v| v.is_error())
        }));
    }

    let any_error = results.iter().any(|r| r["isError"].as_bool().unwrap_or(false));

    // Markdown fallback so the sweep is visible in clients without MCP Apps
    // widgets (e.g. Claude Code), mirroring eval's verbatim-text path.
    let mut summary_md = format!("Evaluated {} variable set(s).\n\n", results.len());
    if !comparison.is_empty() {
        let field = compare_field.unwrap_or("value");
        summary_md.push_str(&format!("| # | variables | {} |\n|---|-----------|--------|\n", field));
        for c in &comparison {
            summary_md.push_str(&format!("| {} | {} | {} |\n",
                c["index"], fmt_obj(&c["variables"]), c["value"].as_str().unwrap_or("")));
        }
    } else {
        summary_md.push_str("| # | variables | values |\n|---|-----------|--------|\n");
        for r in &results {
            summary_md.push_str(&format!("| {} | {} | {} |\n",
                r["index"], fmt_obj(&r["variables"]), fmt_obj(&r["values"])));
        }
    }

    Ok(json!({
        "content": [{
            "type": "text",
            "text": summary_md,
            "annotations": { "audience": ["user"], "priority": 1.0 }
        }],
        "structuredContent": {
            "runs": results,
            "comparison": comparison,
            "compareField": compare_field
        },
        "isError": any_error
    }))
}

pub fn tool_folio(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
    let name = args.get("name").and_then(|v| v.as_str());
    let compact = args.get("compact").and_then(|v| v.as_bool()).unwrap_or(false);

    // If no name provided, return overview (compact or full)
    if name.is_none() {
        let overview = if compact {
            generate_compact_overview(folio)
        } else {
            generate_folio_overview(folio)
        };
        return Ok(json!({
            "content": [{ "type": "text", "text": overview }]
        }));
    }

    let help = folio.help(name);

    Ok(json!({
        "content": [{ "type": "text", "text": format_help(&help) }],
        "structuredContent": { "help": value_to_json(&help) }
    }))
}

pub fn tool_quick(folio: &Folio) -> Result<JsonValue, McpError> {
    let quick_ref = generate_quick_reference(folio);
    Ok(json!({
        "content": [{ "type": "text", "text": quick_ref }]
    }))
}

pub fn generate_folio_overview(folio: &Folio) -> String {
    let mut out = String::new();

    out.push_str("# Folio - Markdown Computational Documents\n\n");
    out.push_str("Arbitrary precision arithmetic for LLMs. All calculations use exact rational arithmetic.\n\n");

    // Functions
    out.push_str("## Available Functions\n\n");
    out.push_str("| Function | Description | Usage |\n");
    out.push_str("|----------|-------------|-------|\n");

    if let Value::List(funcs) = folio.list_functions(None) {
        for func in funcs {
            if let Value::Object(map) = func {
                let name = map.get("name").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let desc = map.get("description").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let usage = map.get("usage").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                out.push_str(&format!("| `{}` | {} | `{}` |\n", name, desc, usage));
            }
        }
    }

    // Constants
    out.push_str("\n## Available Constants\n\n");
    out.push_str("| Constant | Value/Formula | Category | Source |\n");
    out.push_str("|----------|---------------|----------|--------|\n");

    if let Value::List(consts) = folio.list_constants() {
        for c in consts {
            if let Value::Object(map) = c {
                let name = map.get("name").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let formula = map.get("formula").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let category = map.get("category").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let source = map.get("source").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                out.push_str(&format!("| `{}` | {} | {} | {} |\n", name, formula, category, source));
            }
        }
    }

    // Operators
    out.push_str("\n## Operators\n\n");
    out.push_str("| Operator | Description | Example |\n");
    out.push_str("|----------|-------------|--------|\n");
    out.push_str("| `+` | Addition | `a + b` |\n");
    out.push_str("| `-` | Subtraction | `a - b` |\n");
    out.push_str("| `*` | Multiplication | `a * b` |\n");
    out.push_str("| `/` | Division | `a / b` |\n");
    out.push_str("| `^` | Power | `a ^ b` |\n");
    out.push_str("| `()` | Grouping | `(a + b) * c` |\n");

    // Document format
    out.push_str("\n## Document Format\n\n");
    out.push_str("Folio documents use markdown tables for calculations:\n\n");
    out.push_str("```markdown\n");
    out.push_str("## Section Name @precision:50\n\n");
    out.push_str("| name | formula | result |\n");
    out.push_str("|------|---------|--------|\n");
    out.push_str("| x | 10 | |\n");
    out.push_str("| y | x * 2 | |\n");
    out.push_str("| z | sqrt(y) | |\n");
    out.push_str("```\n\n");

    // Directives
    out.push_str("## Directives\n\n");
    out.push_str("| Directive | Description | Example |\n");
    out.push_str("|-----------|-------------|--------|\n");
    out.push_str("| `@precision:N` | Set decimal precision | `@precision:100` |\n");
    out.push_str("| `@sigfigs:N` | Display with N significant figures | `@sigfigs:6` |\n");

    out
}

pub fn generate_compact_overview(folio: &Folio) -> String {
    let mut out = String::new();

    out.push_str("# Folio Quick Reference\n\n");
    out.push_str("## Operators: + - * / ^ ()\n\n");

    // Group functions by category
    let mut by_category: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    if let Value::List(funcs) = folio.list_functions(None) {
        for func in funcs {
            if let Value::Object(map) = func {
                let name = map.get("name").and_then(|v| if let Value::Text(s) = v { Some(s.clone()) } else { None }).unwrap_or_default();
                let category = map.get("category").and_then(|v| if let Value::Text(s) = v { Some(s.clone()) } else { None }).unwrap_or_else(|| "other".to_string());
                by_category.entry(category).or_default().push(name);
            }
        }
    }

    // Sort categories and output
    let mut categories: Vec<_> = by_category.keys().cloned().collect();
    categories.sort();

    for cat in categories {
        if let Some(funcs) = by_category.get(&cat) {
            out.push_str(&format!("## {}\n", cat));
            out.push_str(&funcs.join(", "));
            out.push_str("\n\n");
        }
    }

    // Constants (just names)
    out.push_str("## Constants\n");
    if let Value::List(consts) = folio.list_constants() {
        let names: Vec<_> = consts.iter().filter_map(|c| {
            if let Value::Object(map) = c {
                map.get("name").and_then(|v| if let Value::Text(s) = v { Some(s.clone()) } else { None })
            } else { None }
        }).collect();
        out.push_str(&names.join(", "));
    }
    out.push_str("\n\n");

    out.push_str("Use `folio(name=\"function_name\")` for detailed help.\n");

    out
}

pub fn generate_quick_reference(_folio: &Folio) -> String {
    // Hand-crafted compact reference with Object return fields
    r#"# Folio Quick Reference

## Operators: + - * / ^ ()

## math
abs, ceil, floor, round, sqrt, pow, exp, ln

## trig
sin, cos, tan

## aggregate
sum

## utility
fields, head, tail, take, typeof, describe, len, nth

## stats/central
mean, median, mode, gmean, hmean, tmean, wmean

## stats/dispersion
variance, variance_p, stddev, stddev_p, range, iqr, mad, cv, se

## stats/position
min, max, percentile, quantile, q1, q3, rank, zscore

## stats/shape
skewness, kurtosis, count, product

## stats/bivariate
covariance, covariance_p, correlation, spearman

## stats/regression
linear_reg→{slope,intercept,r_squared,r,std_error,n}, slope, intercept, r_squared, predict, residuals

## stats/hypothesis
t_test_1→{t,p,df,ci_low,ci_high,mean_diff}
t_test_2→{t,p,df,ci_low,ci_high,mean_diff}
t_test_paired→{t,p,df,ci_low,ci_high,mean_diff}
chi_test→{chi_sq,p,df}
f_test→{f,p,df1,df2}
anova→{f,p,df_between,df_within,ss_between,ss_within}

## stats/confidence
ci→{low,high,margin,level}, moe

## stats/transform
normalize, standardize, cumsum, differences, lag, moving_avg, ewma

## stats/distribution
norm_pdf, norm_cdf, norm_inv, snorm_pdf, snorm_cdf, snorm_inv
t_pdf, t_cdf, t_inv, chi_pdf, chi_cdf, chi_inv, f_pdf, f_cdf, f_inv
binom_pmf, binom_cdf, poisson_pmf, poisson_cdf

## datetime
now, date, time, datetime, parseDate, parseTime
year, month, day, hour, minute, second, weekday, dayOfYear, week
formatDate, formatTime, formatDateTime
days, hours, minutes, seconds, milliseconds, weeks
addDays, addMonths, addYears, diff
isBefore, isAfter, isSameDay
sod, eod, som, eom, sow, eow, soq, eoq, soy, eoy
tomorrow, nextWeek, nextMonth, nextMonthWd
isWorkday, nextWorkday, prevWorkday, addWorkdays

## isis
ISIS, ISIS_INV

## Tips
- Use `fields(obj)` to discover Object fields
- Use `head(list, 5)` to peek at list contents
- Functions accept both `(a, b, c)` and `([a, b, c])` for lists
"#.to_string()
}

pub fn format_help(help: &Value) -> String {
    match help {
        Value::Object(map) => {
            let mut out = String::new();
            if let Some(Value::Text(n)) = map.get("name") { out.push_str(&format!("# {}\n\n", n)); }
            if let Some(Value::Text(d)) = map.get("description") { out.push_str(&format!("{}\n\n", d)); }
            if let Some(Value::Text(u)) = map.get("usage") { out.push_str(&format!("**Usage:** `{}`\n\n", u)); }
            if let Some(Value::Text(c)) = map.get("category") { out.push_str(&format!("**Category:** {}\n\n", c)); }
            if let Some(Value::List(examples)) = map.get("examples") {
                out.push_str("**Examples:**\n");
                for ex in examples {
                    if let Value::Text(e) = ex {
                        out.push_str(&format!("- `{}`\n", e));
                    }
                }
                out.push_str("\n");
            }
            if let Some(Value::List(related)) = map.get("related") {
                let related_str: Vec<_> = related.iter().filter_map(|r| {
                    if let Value::Text(s) = r { Some(format!("`{}`", s)) } else { None }
                }).collect();
                if !related_str.is_empty() {
                    out.push_str(&format!("**Related:** {}\n", related_str.join(", ")));
                }
            }
            out
        }
        Value::Error(e) => format!("Error: {}", e.message),
        _ => format!("{:?}", help),
    }
}

pub fn tool_list_functions(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
    let category = args.get("category").and_then(|v| v.as_str());
    let functions = folio.list_functions(category);

    // Build readable table
    let mut text = String::from("# Available Functions\n\n");
    text.push_str("| Function | Description | Usage |\n");
    text.push_str("|----------|-------------|-------|\n");

    if let Value::List(funcs) = &functions {
        for func in funcs {
            if let Value::Object(map) = func {
                let name = map.get("name").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let desc = map.get("description").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let usage = map.get("usage").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                text.push_str(&format!("| `{}` | {} | `{}` |\n", name, desc, usage));
            }
        }
    }

    Ok(json!({ "content": [{ "type": "text", "text": text }], "structuredContent": { "functions": value_to_json(&functions) } }))
}

pub fn tool_list_constants(folio: &Folio, _args: JsonValue) -> Result<JsonValue, McpError> {
    let constants = folio.list_constants();

    // Build readable table grouped by category
    let mut text = String::from("# Available Constants\n\n");
    text.push_str("| Constant | Value/Formula | Category | Source |\n");
    text.push_str("|----------|---------------|----------|--------|\n");

    if let Value::List(consts) = &constants {
        for c in consts {
            if let Value::Object(map) = c {
                let name = map.get("name").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let formula = map.get("formula").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let category = map.get("category").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                let source = map.get("source").and_then(|v| if let Value::Text(s) = v { Some(s.as_str()) } else { None }).unwrap_or("");
                text.push_str(&format!("| `{}` | {} | {} | {} |\n", name, formula, category, source));
            }
        }
    }

    text.push_str("\n**Note:** Particle masses are in MeV. Use constants directly in formulas, e.g., `m_e * c^2`\n");

    Ok(json!({ "content": [{ "type": "text", "text": text }], "structuredContent": { "constants": value_to_json(&constants) } }))
}

/// Format a JSON object {k: v, ...} as "k=v, k2=v2" for markdown summaries.
pub fn fmt_obj(v: &JsonValue) -> String {
    v.as_object()
        .map(|o| o.iter()
            .map(|(k, val)| {
                let vs = val.as_str().map(|s| s.to_string()).unwrap_or_else(|| val.to_string());
                format!("{}={}", k, vs)
            })
            .collect::<Vec<_>>()
            .join(", "))
        .unwrap_or_default()
}

/// Recognize whether a value is a simple expression of φ, π, or e: direct match,
/// small integer multiples (2π), unit fractions (π/3), small powers (φ^2), 1/K,
/// and √K — reported when within 0.1% relative error, ranked best-first.
pub fn tool_decompose(_folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
    let value_str = args.get("value")
        .and_then(|v| v.as_str())
        .ok_or(McpError { code: -32602, message: "Missing value".to_string(), data: None })?;

    let v: f64 = match value_str.trim().parse::<f64>() {
        Ok(x) => x,
        Err(_) => return Ok(json!({
            "content": [{ "type": "text", "text": format!("`{}` is not a numeric value.", value_str),
                "annotations": { "audience": ["user"], "priority": 1.0 } }],
            "structuredContent": { "value": value_str, "matches": [] },
            "isError": true
        })),
    };

    let consts = [("φ", 1.618033988749895_f64), ("π", std::f64::consts::PI), ("e", std::f64::consts::E)];
    let mut cands: Vec<(String, f64)> = Vec::new();
    for (name, k) in consts {
        cands.push((name.to_string(), k));
        for m in 2..=12 { cands.push((format!("{}{}", m, name), (m as f64) * k)); }
        for m in 2..=12 { cands.push((format!("{}/{}", name, m), k / (m as f64))); }
        for n in 2..=4 { cands.push((format!("{}^{}", name, n), k.powi(n))); }
        cands.push((format!("1/{}", name), 1.0 / k));
        cands.push((format!("√{}", name), k.sqrt()));
    }

    let mut matches: Vec<(String, f64, f64)> = Vec::new();
    if v != 0.0 {
        for (expr, ev) in &cands {
            let rel = ((v - ev) / v).abs();
            if rel < 1e-3 { matches.push((expr.clone(), *ev, rel)); }
        }
    }
    matches.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
    matches.truncate(5);

    let mut text = format!("## Decompose {}\n\n", value_str);
    if matches.is_empty() {
        text.push_str(&format!("No simple φ/π/e pattern found within 0.1% of {}.", value_str));
    } else {
        text.push_str("| expression | value | rel. error |\n|------------|-------|------------|\n");
        for (expr, ev, rel) in &matches {
            text.push_str(&format!("| {} | {:.10} | {:.2e} |\n", expr, ev, rel));
        }
    }

    let matches_json: Vec<JsonValue> = matches.iter().map(|(expr, ev, rel)| json!({
        "expression": expr,
        "approx": format!("{:.12}", ev),
        "relativeError": format!("{:.3e}", rel)
    })).collect();

    Ok(json!({
        "content": [{ "type": "text", "text": text, "annotations": { "audience": ["user"], "priority": 1.0 } }],
        "structuredContent": { "value": value_str, "matches": matches_json }
    }))
}

pub fn json_to_value(json: &JsonValue) -> Value {
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

pub fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => JsonValue::Bool(*b),
        Value::Number(n) => JsonValue::String(n.to_string()),
        Value::Text(s) => JsonValue::String(s.clone()),
        Value::DateTime(dt) => json!({"_type": "datetime", "value": dt.to_string(), "nanos": dt.as_nanos().to_string()}),
        Value::Duration(d) => json!({"_type": "duration", "value": d.to_string(), "nanos": d.as_nanos().to_string()}),
        Value::List(l) => JsonValue::Array(l.iter().map(value_to_json).collect()),
        Value::Object(o) => JsonValue::Object(o.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect()),
        Value::Error(e) => json!({"_error": {"code": e.code, "message": e.message}}),
    }
}
