//! folio-httpd — HTTP JSON API over the Folio engine.
//!
//! searxng-style resident service for the automa stack: the edge terminates
//! TLS and validates the stack bearer (same contract as search.016180.xyz).
//! When `FOLIO_BEARER` is set the server additionally enforces its own
//! `Authorization: Bearer` check (defense in depth; off by default because
//! the container is only reachable through the edge network).
//!
//! Endpoints (full parity with the folio-mcp tool surface — both transports
//! call the same functions in the `folio-mcp` service-layer crate):
//!   GET  /healthz                    → {status, name, version}
//!   POST /eval        {template, variables?}                     → eval result
//!   POST /eval_file   {name, variables?}                         → eval result
//!   POST /eval_batch  {template, variable_sets, compare_field?}  → sweep result
//!   GET  /docs?name=&compact=1       → documentation (the `folio` tool)
//!   GET  /quick                      → compact quick reference
//!   GET  /functions?category=        → function list
//!   GET  /constants                  → constants with sources
//!   GET|POST /decompose?value=       → φ/π/e pattern analysis
//!   GET  /files                      → available .fmd documents
//!
//! Eval responses: {markdown, cells[], errors[], isError} — `markdown` is the
//! complete, auditable results table; `cells` the structured per-cell data.
//!
//! Env: FOLIO_HTTP_ADDR (default 0.0.0.0:8080), FOLIO_DATA_PATH, FOLIO_BEARER.

use axum::{
    extract::{Query, Request, State},
    http::{header, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use folio::Folio;
use folio_mcp::{
    create_folio_with_isis, list_fmd_files, tool_decompose, tool_eval, tool_eval_batch,
    tool_eval_file, tool_folio, tool_list_constants, tool_list_functions, tool_quick, McpError,
};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

const VERSION: &str = env!("CARGO_PKG_VERSION");

struct AppState {
    folio: Folio,
}

/// Optional app-side bearer, read once. Unset/empty = edge-only auth.
fn expected_bearer() -> Option<&'static str> {
    static BEARER: OnceLock<Option<String>> = OnceLock::new();
    BEARER
        .get_or_init(|| env::var("FOLIO_BEARER").ok().filter(|s| !s.is_empty()))
        .as_deref()
}

async fn auth(req: Request, next: Next) -> Response {
    if let Some(expected) = expected_bearer() {
        let ok = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .map(|t| t.trim() == expected)
            .unwrap_or(false);
        if !ok {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": {"code": 401, "message": "missing or invalid bearer"}})),
            )
                .into_response();
        }
    }
    next.run(req).await
}

/// Reshape an MCP CallToolResult into a flat HTTP JSON body:
/// structuredContent fields at top level + `markdown` from the text content.
fn shape(res: Result<JsonValue, McpError>) -> Response {
    match res {
        Ok(v) => {
            let mut out = v
                .get("structuredContent")
                .cloned()
                .unwrap_or_else(|| json!({}));
            if let Some(text) = v.pointer("/content/0/text").and_then(|t| t.as_str()) {
                out["markdown"] = json!(text);
            }
            if let Some(is_err) = v.get("isError") {
                out["isError"] = is_err.clone();
            }
            (StatusCode::OK, Json(out)).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": {"code": e.code, "message": e.message, "data": e.data}})),
        )
            .into_response(),
    }
}

/// Run a tool on the blocking pool (evals are CPU-bound; keeps the runtime
/// responsive and contains panics as 500-class tool errors).
async fn run<F>(st: Arc<AppState>, f: F) -> Response
where
    F: FnOnce(&Folio) -> Result<JsonValue, McpError> + Send + 'static,
{
    let res = tokio::task::spawn_blocking(move || f(&st.folio))
        .await
        .unwrap_or_else(|e| {
            Err(McpError {
                code: -32000,
                message: format!("evaluation worker failed: {}", e),
                data: None,
            })
        });
    shape(res)
}

async fn healthz() -> Json<JsonValue> {
    Json(json!({"status": "ok", "name": "folio", "version": VERSION}))
}

async fn eval(State(st): State<Arc<AppState>>, Json(body): Json<JsonValue>) -> Response {
    run(st, move |f| tool_eval(f, body)).await
}

async fn eval_file(State(st): State<Arc<AppState>>, Json(body): Json<JsonValue>) -> Response {
    run(st, move |f| tool_eval_file(f, body)).await
}

async fn eval_batch(State(st): State<Arc<AppState>>, Json(body): Json<JsonValue>) -> Response {
    run(st, move |f| tool_eval_batch(f, body)).await
}

async fn decompose_post(State(st): State<Arc<AppState>>, Json(body): Json<JsonValue>) -> Response {
    run(st, move |f| tool_decompose(f, body)).await
}

async fn decompose_get(
    State(st): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let mut args = json!({});
    if let Some(v) = q.get("value") {
        args["value"] = json!(v);
    }
    run(st, move |f| tool_decompose(f, args)).await
}

async fn docs(
    State(st): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let mut args = json!({});
    if let Some(n) = q.get("name") {
        args["name"] = json!(n);
    }
    if let Some(c) = q.get("compact") {
        args["compact"] = json!(matches!(c.as_str(), "1" | "true"));
    }
    run(st, move |f| tool_folio(f, args)).await
}

async fn quick(State(st): State<Arc<AppState>>) -> Response {
    run(st, tool_quick).await
}

async fn functions(
    State(st): State<Arc<AppState>>,
    Query(q): Query<HashMap<String, String>>,
) -> Response {
    let mut args = json!({});
    if let Some(c) = q.get("category") {
        args["category"] = json!(c);
    }
    run(st, move |f| tool_list_functions(f, args)).await
}

async fn constants(State(st): State<Arc<AppState>>) -> Response {
    run(st, move |f| tool_list_constants(f, json!({}))).await
}

async fn files() -> Json<JsonValue> {
    Json(json!({ "files": list_fmd_files() }))
}

#[tokio::main]
async fn main() {
    let state = Arc::new(AppState {
        folio: create_folio_with_isis(),
    });

    let api = Router::new()
        .route("/eval", post(eval))
        .route("/eval_file", post(eval_file))
        .route("/eval_batch", post(eval_batch))
        .route("/decompose", get(decompose_get).post(decompose_post))
        .route("/docs", get(docs))
        .route("/quick", get(quick))
        .route("/functions", get(functions))
        .route("/constants", get(constants))
        .route("/files", get(files))
        .layer(middleware::from_fn(auth))
        .with_state(state);

    let app = Router::new().route("/healthz", get(healthz)).merge(api);

    let addr: SocketAddr = env::var("FOLIO_HTTP_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
        .parse()
        .expect("invalid FOLIO_HTTP_ADDR");

    eprintln!("folio-httpd v{} listening on {}", VERSION, addr);
    eprintln!("data path: {}", folio_mcp::data_path().display());
    eprintln!(
        "app-side bearer: {}",
        if expected_bearer().is_some() { "enforced" } else { "disabled (edge-only)" }
    );

    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind failed");
    axum::serve(listener, app).await.expect("server error");
}
