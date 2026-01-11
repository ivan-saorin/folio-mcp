# Folio Architecture

## Design Principles

### 1. LLM-First

Everything is designed for LLM consumption:

* **Structured Errors:** Errors are JSON-serializable and provide context.
* **Self-Documenting Functions:** Tools expose their own schemas and usage examples.
* **Deterministic Output:** Given the same context and precision, results are reproducible.

### 2. Never Break

The system aims for high stability to prevent agent loops from crashing:

* Panics are forbidden in core logic (Results are captured in `Value::Error`).
* **Correction:** *Current implementation constraints require strict input validation to avoid unwraps in the server layer.*

### 3. Plugin-First

The Core is minimal. Functionality is extended via traits:

* Math functions → `FunctionPlugin`
* Pattern detection → `AnalyzerPlugin`
* Side effects → `CommandPlugin`

### 4. Configurable Decimal Precision

To balance performance and accuracy without C-dependencies:

* **Type:** All numbers are `BigFloat` (arbitrary precision decimals via `dashu`).
* **Behavior:** Precision is explicitly managed (Default: 50 digits, configurable via `@precision:N`).
* **Implementation:** Pure Rust (no GMP/system library dependencies).

---

## Core Types

### Number (`folio-core/src/number.rs`)

The fundamental numeric type used throughout the evaluator.

```rust
/// Arbitrary precision decimal number
/// Wraps dashu_float::DBig
pub struct Number {
    inner: dashu_float::DBig,
}

impl Number {
    // Construction
    // Parses strings using the current context's precision settings
    pub fn from_str(s: &str, precision: usize) -> Result<Self>;
    
    // Arithmetic
    // Operations maintain the defined working precision
    pub fn add(&self, other: &Self) -> Self;
    pub fn pow(&self, exp: &Self) -> Self; // Note: Currently O(N) complexity
}

```

### Value (`folio-core/src/value.rs`)

The enum representing any data passing through the system.

```rust
pub enum Value {
    Number(Number),
    String(String),
    Boolean(bool),
    // Structured error handling
    Error(FolioError), 
    // ...
}

```

---

## System Components

### 1. Folio Core (`folio-core`)

The library containing the AST, Parser, and Evaluator logic.

* **Dependencies:** `dashu` (math), `serde` (serialization).
* **Responsibility:** Pure function evaluation and state management.

### 2. Folio MCP (`folio-mcp`)

The server implementation for Claude Desktop (Model Context Protocol).

* **Role:** Exposes core functionality as MCP Tools (`evaluate`, `decompose`, etc.).
* **Transport:** Stdio (Docker/CLI).

---

## Data Flow

1. **Input:** User provides a `.fmd` (Folio Markdown) string or file.
2. **Parsing:** Text is parsed into an AST.
3. **Context Setup:** Global settings (Precision, constants) are initialized.
4. **Evaluation:**
* Math expressions are calculated using `DBig` at specific precision.
* Plugins are invoked for specialized functions (e.g., `DECOMPOSE`).


5. **Rendering:** Results are serialized to JSON/Markdown for the LLM.