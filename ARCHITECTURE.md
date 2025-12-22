# Folio Architecture

## Design Principles

### 1. LLM-First
Everything is designed for LLM consumption:
- Structured errors (JSON-serializable)
- Self-documenting functions
- Deterministic output
- No hidden state

### 2. Never Break
The system must NEVER panic or crash. Every error is:
- Caught and converted to `FolioError`
- Propagated as `Value::Error`
- Reported with context and suggestions

### 3. Plugin-First
Core is minimal. Everything else is a plugin:
- Math functions → `FunctionPlugin`
- Pattern detection → `AnalyzerPlugin`
- Side effects → `CommandPlugin`

### 4. Explicit Precision
No floating point surprises:
- All numbers are `BigRational` (numerator/denominator)
- Precision is explicit (`@precision:100`)
- Conversion to decimal only at render time

---

## Core Types

### Number (`folio-core/src/number.rs`)

```rust
/// Arbitrary precision rational number
pub struct Number {
    inner: rug::Rational,  // GMP-backed
}

impl Number {
    // Construction
    pub fn from_str(s: &str) -> Result<Self, NumberError>;
    pub fn from_i64(n: i64) -> Self;
    pub fn from_ratio(num: i64, den: i64) -> Self;
    
    // Constants (lazy-initialized with precision)
    pub fn phi(precision: u32) -> Self;
    pub fn pi(precision: u32) -> Self;
    pub fn e(precision: u32) -> Self;
    
    // Operations (return Result, never panic)
    pub fn add(&self, other: &Self) -> Self;
    pub fn sub(&self, other: &Self) -> Self;
    pub fn mul(&self, other: &Self) -> Self;
    pub fn div(&self, other: &Self) -> Result<Self, NumberError>;
    pub fn pow(&self, exp: i32) -> Self;
    
    // Transcendental (computed to precision)
    pub fn sqrt(&self, precision: u32) -> Result<Self, NumberError>;
    pub fn ln(&self, precision: u32) -> Result<Self, NumberError>;
    pub fn exp(&self, precision: u32) -> Self;
    
    // Analysis
    pub fn as_decimal(&self, places: u32) -> String;
    pub fn is_integer(&self) -> bool;
    pub fn as_simple_fraction(&self, max_denom: u64) -> Option<(i64, i64)>;
}
```

### Value (`folio-core/src/value.rs`)

```rust
/// Runtime value in Folio
#[derive(Debug, Clone)]
pub enum Value {
    Number(Number),
    Text(String),
    Bool(bool),
    Object(HashMap<String, Value>),
    List(Vec<Value>),
    Null,
    Error(FolioError),
}

impl Value {
    // Safe accessors (never panic)
    pub fn as_number(&self) -> Option<&Number>;
    pub fn as_text(&self) -> Option<&str>;
    pub fn as_bool(&self) -> Option<bool>;
    pub fn as_object(&self) -> Option<&HashMap<String, Value>>;
    pub fn as_list(&self) -> Option<&[Value]>;
    pub fn is_error(&self) -> bool;
    
    // Field access for objects
    pub fn get(&self, key: &str) -> Value;  // Returns Error if not found
    
    // Type coercion (explicit)
    pub fn to_number(&self) -> Value;  // May return Error
    pub fn to_text(&self) -> Value;    // Always succeeds
    pub fn to_bool(&self) -> Value;    // Truthy/falsy
}
```

### FolioError (`folio-core/src/error.rs`)

```rust
/// Structured error for LLM consumption
#[derive(Debug, Clone, Serialize)]
pub struct FolioError {
    /// Machine-readable error code
    pub code: String,
    
    /// Human-readable message
    pub message: String,
    
    /// Suggestion for fixing
    pub suggestion: Option<String>,
    
    /// Where the error occurred
    pub context: Option<ErrorContext>,
    
    /// Severity level
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorContext {
    pub cell: Option<String>,
    pub formula: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub enum Severity {
    Warning,  // Computation continued with degraded result
    Error,    // Computation failed for this cell
    Fatal,    // Document cannot be evaluated
}

// Standard error codes
pub mod codes {
    pub const PARSE_ERROR: &str = "PARSE_ERROR";
    pub const DIV_ZERO: &str = "DIV_ZERO";
    pub const UNDEFINED_VAR: &str = "UNDEFINED_VAR";
    pub const UNDEFINED_FUNC: &str = "UNDEFINED_FUNC";
    pub const TYPE_ERROR: &str = "TYPE_ERROR";
    pub const ARG_COUNT: &str = "ARG_COUNT";
    pub const DOMAIN_ERROR: &str = "DOMAIN_ERROR";  // e.g., sqrt(-1)
    pub const OVERFLOW: &str = "OVERFLOW";
    pub const CIRCULAR_REF: &str = "CIRCULAR_REF";
}
```

---

## Plugin System

### Function Plugin (`folio-plugin/src/traits.rs`)

```rust
/// Pure function: inputs → output, no side effects
pub trait FunctionPlugin: Send + Sync {
    /// Metadata for documentation
    fn meta(&self) -> FunctionMeta;
    
    /// Execute function
    /// MUST NOT panic. Return Value::Error on failure.
    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value;
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub args: Vec<ArgMeta>,
    pub returns: &'static str,
    pub examples: Vec<&'static str>,
    pub category: &'static str,
    pub source: Option<&'static str>,  // Documentation URL
    pub related: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArgMeta {
    pub name: &'static str,
    pub typ: &'static str,
    pub description: &'static str,
    pub optional: bool,
    pub default: Option<&'static str>,
}
```

### Analyzer Plugin

```rust
/// Pattern detection in values
pub trait AnalyzerPlugin: Send + Sync {
    fn meta(&self) -> AnalyzerMeta;
    
    /// Quick confidence check (0-1)
    fn confidence(&self, value: &Number, ctx: &EvalContext) -> f64;
    
    /// Full analysis
    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value;
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyzerMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub detects: Vec<&'static str>,  // Constants this analyzer looks for
}
```

### Command Plugin

```rust
/// Commands can have side effects (explain, trace, export)
pub trait CommandPlugin: Send + Sync {
    fn meta(&self) -> CommandMeta;
    
    /// Execute command
    fn execute(&self, args: &[Value], ctx: &mut EvalContext) -> Value;
}
```

### Registry

```rust
pub struct PluginRegistry {
    functions: HashMap<String, Arc<dyn FunctionPlugin>>,
    analyzers: Vec<Arc<dyn AnalyzerPlugin>>,
    commands: HashMap<String, Arc<dyn CommandPlugin>>,
    constants: HashMap<String, ConstantDef>,
}

impl PluginRegistry {
    pub fn new() -> Self;
    
    // Builder pattern
    pub fn with_function<F: FunctionPlugin + 'static>(self, f: F) -> Self;
    pub fn with_analyzer<A: AnalyzerPlugin + 'static>(self, a: A) -> Self;
    pub fn with_command<C: CommandPlugin + 'static>(self, c: C) -> Self;
    pub fn with_constant(self, name: &str, value: Number, source: &str) -> Self;
    
    // Lookup
    pub fn get_function(&self, name: &str) -> Option<&dyn FunctionPlugin>;
    pub fn get_constant(&self, name: &str) -> Option<&ConstantDef>;
    
    // Documentation
    pub fn help(&self, name: Option<&str>) -> Value;
    pub fn list_functions(&self, category: Option<&str>) -> Value;
    pub fn list_constants(&self) -> Value;
}
```

---

## Parser

### Grammar (Pest)

```pest
document = { SOI ~ section* ~ EOI }

section = { section_header ~ table }

section_header = { "##" ~ " "* ~ name ~ attributes? ~ NEWLINE }

attributes = { "@" ~ attribute ~ ("," ~ attribute)* }
attribute = { name ~ ":" ~ value }

table = { table_header ~ table_separator ~ table_row+ }

table_header = { "|" ~ column_name ~ ("|" ~ column_name)+ ~ "|" ~ NEWLINE }
table_separator = { "|" ~ "-"+ ~ ("|" ~ "-"+)+ ~ "|" ~ NEWLINE }
table_row = { "|" ~ cell ~ ("|" ~ cell)+ ~ "|" ~ NEWLINE }

cell = { " "* ~ cell_content ~ " "* }
cell_content = { formula | text }

formula = { expression }
expression = { term ~ (("+" | "-") ~ term)* }
term = { factor ~ (("*" | "/" | "^") ~ factor)* }
factor = { function_call | number | variable | "(" ~ expression ~ ")" }

function_call = { name ~ "(" ~ arg_list? ~ ")" }
arg_list = { expression ~ ("," ~ expression)* }

variable = { name ~ ("." ~ name)* }
name = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
number = @{ "-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? ~ (("e" | "E") ~ "-"? ~ ASCII_DIGIT+)? }
text = @{ (!"|" ~ ANY)* }
```

### AST (`folio/src/ast.rs`)

```rust
pub struct Document {
    pub sections: Vec<Section>,
}

pub struct Section {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub table: Table,
}

pub struct Table {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
}

pub struct Row {
    pub cells: Vec<Cell>,
}

pub struct Cell {
    pub name: String,
    pub formula: Option<Expr>,
    pub raw_text: String,
}

pub enum Expr {
    Number(String),
    Variable(Vec<String>),  // ["decomp", "φ"] for decomp.φ
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
}

pub enum BinOp { Add, Sub, Mul, Div, Pow }
pub enum UnaryOp { Neg }
```

---

## Evaluator

### Dependency Resolution

```rust
pub struct DependencyGraph {
    /// cell_name → dependencies
    edges: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn from_document(doc: &Document) -> Result<Self, FolioError>;
    
    /// Returns cells in evaluation order
    pub fn topological_sort(&self) -> Result<Vec<String>, FolioError>;
    
    /// Detect cycles
    pub fn find_cycles(&self) -> Vec<Vec<String>>;
}
```

### Evaluation Context

```rust
pub struct EvalContext {
    /// Current precision setting
    pub precision: u32,
    
    /// Variable bindings (external + computed)
    pub variables: HashMap<String, Value>,
    
    /// Plugin registry
    pub registry: Arc<PluginRegistry>,
    
    /// Evaluation trace (for EXPLAIN)
    pub trace: Option<EvalTrace>,
}

pub struct EvalTrace {
    pub steps: Vec<TraceStep>,
}

pub struct TraceStep {
    pub cell: String,
    pub formula: String,
    pub result: Value,
    pub dependencies: Vec<String>,
}
```

### Evaluator

```rust
pub struct Evaluator {
    registry: Arc<PluginRegistry>,
}

impl Evaluator {
    pub fn eval(&self, doc: &Document, external: &HashMap<String, Value>) -> EvalResult;
    
    fn eval_expr(&self, expr: &Expr, ctx: &EvalContext) -> Value;
    
    fn eval_function(&self, name: &str, args: &[Expr], ctx: &EvalContext) -> Value;
}

pub struct EvalResult {
    /// Evaluated document with results
    pub document: Document,
    
    /// All computed values
    pub values: HashMap<String, Value>,
    
    /// Any errors encountered
    pub errors: Vec<FolioError>,
    
    /// Warnings (non-fatal)
    pub warnings: Vec<FolioError>,
}
```

---

## Renderer

```rust
pub struct Renderer;

impl Renderer {
    /// Render document back to markdown with results
    pub fn render(&self, doc: &Document, values: &HashMap<String, Value>) -> String;
    
    /// Render external variables section
    fn render_external(&self, external: &HashMap<String, Value>) -> String;
    
    /// Render a single value
    fn render_value(&self, value: &Value, precision: u32) -> String;
}
```

---

## MCP Interface

### Tools

```rust
pub struct FolioMcpServer {
    folio: Arc<Folio>,
}

impl FolioMcpServer {
    /// Evaluate a document
    /// Returns: { markdown: String, values: Object, errors: Array }
    async fn eval(&self, template: String, variables: Object, precision: Option<u32>) -> Value;
    
    /// Batch evaluation for parameter sweeps
    /// Returns: Array of eval results
    async fn eval_batch(&self, template: String, variable_sets: Array) -> Value;
    
    /// Get help for a function
    /// Returns: FunctionMeta as JSON
    async fn help(&self, name: Option<String>) -> Value;
    
    /// List available functions
    /// Returns: Array of function metadata
    async fn list_functions(&self, category: Option<String>) -> Value;
    
    /// List available constants
    /// Returns: Array of constant definitions with sources
    async fn list_constants(&self) -> Value;
    
    /// Decompose a value (direct access to analyzers)
    /// Returns: Object with pattern analysis
    async fn decompose(&self, value: String, precision: Option<u32>) -> Value;
}
```

---

## Error Handling Strategy

### Rule: Never Panic

```rust
// WRONG
fn divide(a: &Number, b: &Number) -> Number {
    if b.is_zero() {
        panic!("Division by zero");  // NO!
    }
    a / b
}

// CORRECT
fn divide(a: &Number, b: &Number) -> Value {
    if b.is_zero() {
        return Value::Error(FolioError {
            code: codes::DIV_ZERO.into(),
            message: "Division by zero".into(),
            suggestion: Some("Check that divisor is not zero".into()),
            context: None,
            severity: Severity::Error,
        });
    }
    Value::Number(a / b)
}
```

### Error Propagation

When a cell has an error, dependent cells get a propagated error:

```rust
fn eval_binary_op(left: Value, op: BinOp, right: Value) -> Value {
    // Propagate errors
    if let Value::Error(e) = &left {
        return Value::Error(e.clone().with_note("Propagated from left operand"));
    }
    if let Value::Error(e) = &right {
        return Value::Error(e.clone().with_note("Propagated from right operand"));
    }
    
    // ... actual operation
}
```

### Error Accumulation

Evaluation continues even when errors occur:

```rust
fn eval_document(&self, doc: &Document) -> EvalResult {
    let mut values = HashMap::new();
    let mut errors = Vec::new();
    
    for cell in self.evaluation_order(doc) {
        let result = self.eval_cell(cell, &values);
        
        if let Value::Error(e) = &result {
            errors.push(e.clone());
        }
        
        values.insert(cell.name.clone(), result);
        // Continue evaluating other cells!
    }
    
    EvalResult { values, errors, .. }
}
```
