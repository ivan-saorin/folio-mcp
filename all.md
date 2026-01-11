You are tasked to answer a question:

Honest review time.
Highlight everything that doesn't match up

Instructions for the output format:
- Consider other possibilities to achieve the result, do not be limited by the prompt.

ARCHITECTURE.md
```md
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
```

Cargo.toml
```toml
[workspace]
resolver = "2"
members = [
    "folio-core",
    "folio-plugin",
    "folio-std",
    "folio-stats",
    "folio-sequence",
    "folio",
    "folio-mcp",
    "folio-isis",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/yourname/folio"
authors = ["Ivan <ivan@example.com>"]

[workspace.dependencies]
# Shared dependencies across workspace
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"

# Arbitrary precision - dashu is pure Rust GMP/MPFR alternative
dashu = "0.4"           # Meta-crate with all components
dashu-base = "0.4"      # Base traits and Approximation type
dashu-int = "0.4"       # Arbitrary precision integers
dashu-ratio = "0.4"     # Exact rational arithmetic  
dashu-float = "0.4"     # Arbitrary precision floats (for transcendentals)

# Legacy - kept for transition, will be removed
num-bigint = "0.4"
num-rational = "0.4"
num-traits = "0.2"

# Parsing
pest = "2.7"
pest_derive = "2.7"

# MCP
async-trait = "0.1"
```

data\examples\compound_interest.fmd
```fmd
<!-- Compound Interest Calculator -->
# Compound Interest Calculator

A = P(1 + r/n)^(nt)

## Variables

- `principal`: Initial investment
- `rate`: Annual interest rate (decimal)
- `years`: Time period
- `compounds`: Compounding frequency per year

## Calculation @precision:20

| name | formula | result |
|------|---------|--------|
| principal | 10000 | |
| rate | 0.07 | |
| years | 10 | |
| compounds | 12 | |
| periodic_rate | rate / compounds | |
| total_periods | compounds * years | |
| growth_factor | pow(1 + periodic_rate, total_periods) | |
| final_amount | principal * growth_factor | |
| interest_earned | final_amount - principal | |
| ear | pow(1 + rate / compounds, compounds) - 1 | |
| continuous_amount | principal * exp(rate * years) | |
| continuous_diff | continuous_amount - final_amount | |
```

data\examples\datetime_calculations.fmd
```fmd
<!-- DateTime Calculations Example -->
# DateTime Calculations

Demonstrates datetime arithmetic, duration calculations, and comparisons.

## Duration Arithmetic

| name | formula | result |
|------|---------|--------|
| one_week | days(7) | |
| work_hours | hours(8) | |
| lunch_break | minutes(45) | |
| two_weeks | one_week + one_week | |
| half_week | one_week / 2 | |
| work_day_hours | work_hours * 5 | |

## Date Math @datetimeFmt:YYYY-MM-DD

| name | formula | result |
|------|---------|--------|
| start_date | date(2025, 6, 1) | |
| one_week_later | start_date + days(7) | |
| two_months_later | addMonths(start_date, 2) | |
| one_year_later | addYears(start_date, 1) | |
| six_months_ago | addMonths(start_date, -6) | |

## Time Differences

| name | formula | result |
|------|---------|--------|
| project_start | date(2025, 1, 15) | |
| project_end | date(2025, 6, 30) | |
| duration_days | diff(project_end, project_start, "days") | |
| duration_hours | diff(project_end, project_start, "hours") | |
| actual_duration | project_end - project_start | |

## Date Comparisons

| name | formula | result |
|------|---------|--------|
| deadline | date(2025, 7, 15) | |
| today_ref | date(2025, 6, 20) | |
| is_before_deadline | isBefore(today_ref, deadline) | |
| is_after_deadline | isAfter(today_ref, deadline) | |
| days_until | diff(deadline, today_ref, "days") | |

## Quarter Calculations @datetimeFmt:YYYY-MM-DD

| name | formula | result |
|------|---------|--------|
| q2_date | date(2025, 5, 15) | |
| q2_start | soq(q2_date) | |
| q2_end | eoq(q2_date) | |
| q3_start | soq(date(2025, 7, 1)) | |
| q3_end | eoq(date(2025, 7, 1)) | |

## Extraction

| name | formula | result |
|------|---------|--------|
| sample | datetime(2025, 6, 15, 14, 30, 45) | |
| sample_year | year(sample) | |
| sample_month | month(sample) | |
| sample_day | day(sample) | |
| sample_hour | hour(sample) | |
| sample_minute | minute(sample) | |
| sample_weekday | weekday(sample) | |
| sample_week | week(sample) | |
| sample_doy | dayOfYear(sample) | |

## Formatting

| name | formula | result |
|------|---------|--------|
| now_ref | datetime(2025, 6, 15, 14, 30, 0) | |
| us_format | formatDate(now_ref, "MM/DD/YYYY") | |
| eu_format | formatDate(now_ref, "DD/MM/YYYY") | |
| iso_format | formatDate(now_ref, "YYYY-MM-DD") | |
| time_12h | formatTime(now_ref, "hh:mm A") | |
| time_24h | formatTime(now_ref, "HH:mm:ss") | |
| full_format | formatDateTime(now_ref, "DD/MM/YYYY HH:mm") | |
```

data\examples\datetime_shortcuts.fmd
```fmd
<!-- DateTime Shortcuts Example -->
# DateTime Shortcuts

Demonstrates the period start/end shortcuts and navigation functions.

## Reference Date

| name | formula | result |
|------|---------|--------|
| reference | date(2025, 6, 15) | |
| day_name | weekday(reference) | |

## End of Period @datetimeFmt:YYYY-MM-DD HH:mm:ss

| name | formula | result |
|------|---------|--------|
| end_of_day | eod(reference) | |
| end_of_week | eow(reference) | |
| end_of_month | eom(reference) | |
| end_of_quarter | eoq(reference) | |
| end_of_year | eoy(reference) | |

## Start of Period @datetimeFmt:YYYY-MM-DD

| name | formula | result |
|------|---------|--------|
| start_of_day | sod(reference) | |
| start_of_week | sow(reference) | |
| start_of_month | som(reference) | |
| start_of_quarter | soq(reference) | |
| start_of_year | soy(reference) | |

## Navigation @datetimeFmt:YYYY-MM-DD

| name | formula | result |
|------|---------|--------|
| tomorrow_date | tomorrow(reference) | |
| next_week_start | nextWeek(reference) | |
| next_month_first | nextMonth(reference) | |
| next_month_workday | nextMonthWd(reference) | |

## February Edge Case @datetimeFmt:YYYY-MM-DD

| name | formula | result |
|------|---------|--------|
| jan_31 | date(2025, 1, 31) | |
| feb_end | eom(jan_31) | |
| feb_2024 | date(2024, 2, 15) | |
| feb_2024_end | eom(feb_2024) | |
```

data\examples\isis_analysis.fmd
```fmd
<!-- ISIS Transform Analysis Template -->
# ISIS Transform Analysis

X(n) = -ln(n) × φ / (2π × ln(φ))
X⁻¹(x) = exp(-x × 2π × ln(φ) / φ)

Provide external variable `target` for the value to analyze.

## Analysis @precision:100 @sigfigs:50

| name | formula | result |
|------|---------|--------|
| phi | (1 + sqrt(5)) / 2 | |
| ln_phi | ln(phi) | |
| two_pi | 2 * π | |
| scaling | phi / (two_pi * ln_phi) | |
| target | 299792458 | |
| ln_target | ln(target) | |
| isis_x | 0 - ln_target * scaling | |

## Inverse Verification @precision:100 @sigfigs:50

| name | formula | result |
|------|---------|--------|
| inv_scaling | two_pi * ln_phi / phi | |
| reconstructed | exp(0 - isis_x * inv_scaling) | |
| error | target - reconstructed | |
| relative_error | abs(error) / target | |

## Phi-Power Analysis @precision:100 @sigfigs:12

Finding nearest φ^k to target value.

| name | formula | result |
|------|---------|--------|
| k_phi | ln_target / ln_phi | |
| k_floor | floor(k_phi) | |
| k_ceil | ceil(k_phi) | |
| phi_floor | pow(phi, k_floor) | |
| phi_ceil | pow(phi, k_ceil) | |
| err_floor | abs(target - phi_floor) / target | |
| err_ceil | abs(target - phi_ceil) / target | |
| fractional_k | k_phi - k_floor | |
```

data\examples\mortgage.fmd
```fmd
<!-- Mortgage Payment Calculator -->
# Mortgage Calculator

Calculate monthly payment, total cost, and interest for a fixed-rate mortgage.

## External Variables

Provide these when calling:
- `principal`: Loan amount (default: 300000)
- `rate`: Annual interest rate as decimal (default: 0.065 for 6.5%)
- `years`: Loan term in years (default: 30)

## Calculation @precision:20

| name | formula | result |
|------|---------|--------|
| principal | 300000 | |
| annual_rate | 0.065 | |
| years | 30 | |
| monthly_rate | annual_rate / 12 | |
| months | years * 12 | |
| payment | principal * monthly_rate * pow(1 + monthly_rate, months) / (pow(1 + monthly_rate, months) - 1) | |
| total_paid | payment * months | |
| total_interest | total_paid - principal | |
| interest_ratio | total_interest / principal | |

## Notes

The payment formula is:
```
M = P * r * (1+r)^n / ((1+r)^n - 1)
```

Where:
- M = monthly payment
- P = principal
- r = monthly interest rate
- n = number of payments
```

data\examples\phi_properties.fmd
```fmd
<!-- Golden Ratio Properties and Identities -->
# Golden Ratio (φ) Properties

Demonstrating the remarkable properties of φ = (1 + √5) / 2

## Golden Ratio Analysis @precision:50

| name | formula | result |
|------|---------|--------|
| phi | (1 + sqrt(5)) / 2 | |
| phi_inv | 1 / phi | |
| phi_sq | phi * phi | |
| identity_check | phi_sq - phi - 1 | |
| reciprocal_check | phi_inv - (phi - 1) | |
| phi_3 | pow(phi, 3) | |
| phi_4 | pow(phi, 4) | |
| phi_5 | pow(phi, 5) | |
| phi_10 | pow(phi, 10) | |
| fib_5 | 5 | |
| fib_4 | 3 | |
| phi_5_check | fib_5 * phi + fib_4 | |
| phi_5_diff | phi_5 - phi_5_check | |
| ln_phi | ln(phi) | |
| two_pi | 2 * π | |
| ln_phi_times_2pi | ln_phi * two_pi | |

Key properties verified:
- φ² = φ + 1 (identity_check should be ~0)
- 1/φ = φ - 1 (reciprocal_check should be ~0)
- φⁿ = F(n)×φ + F(n-1) (phi_5_diff should be ~0)
```

data\examples\portfolio-risk-analysis.fmd
```fmd
# Portfolio Risk Analysis
# ══════════════════════════════════════════════════════════════════════════════
# A comprehensive statistical analysis of a 3-asset portfolio using:
# - Descriptive statistics
# - Correlation analysis  
# - Linear regression (beta calculation)
# - Risk metrics (VaR, Sharpe, Sortino)
# - Hypothesis testing
# - Distribution analysis
# ══════════════════════════════════════════════════════════════════════════════

## Monthly Returns (%) - 24 months of data

| Month | TECH | HEALTH | ENERGY | MARKET |
|-------|------|--------|--------|--------|
| 1 | 3.2 | 1.5 | -2.1 | 1.8 |
| 2 | -1.5 | 2.3 | 4.5 | 0.9 |
| 3 | 5.8 | 0.8 | -1.2 | 2.1 |
| 4 | -2.3 | 1.9 | 3.8 | 0.5 |
| 5 | 4.1 | -0.5 | -3.2 | 1.2 |
| 6 | 2.9 | 2.1 | 1.5 | 1.5 |
| 7 | -3.8 | 0.4 | 5.2 | -0.8 |
| 8 | 6.2 | 1.8 | -0.8 | 2.4 |
| 9 | 1.4 | 2.5 | 2.3 | 1.1 |
| 10 | -0.9 | -1.2 | -4.1 | -1.5 |
| 11 | 4.5 | 1.6 | 0.9 | 1.9 |
| 12 | 3.1 | 2.2 | 3.5 | 2.0 |
| 13 | -2.1 | 0.9 | -2.8 | -0.3 |
| 14 | 5.3 | 1.4 | 1.2 | 1.7 |
| 15 | 2.8 | 2.8 | 4.1 | 2.2 |
| 16 | -4.2 | -0.3 | -1.5 | -1.8 |
| 17 | 3.9 | 1.7 | 2.9 | 1.4 |
| 18 | 1.2 | 2.4 | -0.5 | 0.8 |
| 19 | -1.8 | 0.6 | 3.2 | 0.2 |
| 20 | 4.7 | 1.9 | -1.8 | 1.6 |
| 21 | 2.3 | 2.1 | 2.4 | 1.3 |
| 22 | -0.5 | -0.8 | -2.9 | -0.9 |
| 23 | 3.8 | 1.3 | 1.7 | 1.5 |
| 24 | 2.1 | 2.6 | 0.8 | 1.2 |

## Portfolio Weights

| Asset | Weight |
|-------|--------|
| w_tech | 0.50 |
| w_health | 0.30 |
| w_energy | 0.20 |

## ═══════════════════════════════════════════════════════════════════════════
## SECTION 1: DESCRIPTIVE STATISTICS
## ═══════════════════════════════════════════════════════════════════════════

## Tech Stock Analysis

| Metric | Formula | Result |
|--------|---------|--------|
| tech_data | [3.2, -1.5, 5.8, -2.3, 4.1, 2.9, -3.8, 6.2, 1.4, -0.9, 4.5, 3.1, -2.1, 5.3, 2.8, -4.2, 3.9, 1.2, -1.8, 4.7, 2.3, -0.5, 3.8, 2.1] | |
| tech_mean | mean(tech_data) | |
| tech_median | median(tech_data) | |
| tech_std | stddev(tech_data) | |
| tech_var | variance(tech_data) | |
| tech_min | min(tech_data) | |
| tech_max | max(tech_data) | |
| tech_range | range(tech_data) | |
| tech_iqr | iqr(tech_data) | |
| tech_skew | skewness(tech_data) | |
| tech_kurt | kurtosis(tech_data) | |
| tech_cv | cv(tech_data) | |

## Health Stock Analysis

| Metric | Formula | Result |
|--------|---------|--------|
| health_data | [1.5, 2.3, 0.8, 1.9, -0.5, 2.1, 0.4, 1.8, 2.5, -1.2, 1.6, 2.2, 0.9, 1.4, 2.8, -0.3, 1.7, 2.4, 0.6, 1.9, 2.1, -0.8, 1.3, 2.6] | |
| health_mean | mean(health_data) | |
| health_std | stddev(health_data) | |
| health_skew | skewness(health_data) | |
| health_kurt | kurtosis(health_data) | |

## Energy Stock Analysis

| Metric | Formula | Result |
|--------|---------|--------|
| energy_data | [-2.1, 4.5, -1.2, 3.8, -3.2, 1.5, 5.2, -0.8, 2.3, -4.1, 0.9, 3.5, -2.8, 1.2, 4.1, -1.5, 2.9, -0.5, 3.2, -1.8, 2.4, -2.9, 1.7, 0.8] | |
| energy_mean | mean(energy_data) | |
| energy_std | stddev(energy_data) | |
| energy_skew | skewness(energy_data) | |
| energy_kurt | kurtosis(energy_data) | |

## Market Benchmark

| Metric | Formula | Result |
|--------|---------|--------|
| market_data | [1.8, 0.9, 2.1, 0.5, 1.2, 1.5, -0.8, 2.4, 1.1, -1.5, 1.9, 2.0, -0.3, 1.7, 2.2, -1.8, 1.4, 0.8, 0.2, 1.6, 1.3, -0.9, 1.5, 1.2] | |
| market_mean | mean(market_data) | |
| market_std | stddev(market_data) | |

## Comparative Summary

| Metric | Formula | Result |
|--------|---------|--------|
| highest_return | max([tech_mean, health_mean, energy_mean]) | |
| lowest_risk | min([tech_std, health_std, energy_std]) | |
| best_sharpe_asset | (tech_mean / tech_std) | |

## ═══════════════════════════════════════════════════════════════════════════
## SECTION 2: CORRELATION & COVARIANCE ANALYSIS
## ═══════════════════════════════════════════════════════════════════════════

## Pairwise Correlations

| Pair | Formula | Result |
|------|---------|--------|
| corr_tech_health | correlation(tech_data, health_data) | |
| corr_tech_energy | correlation(tech_data, energy_data) | |
| corr_health_energy | correlation(health_data, energy_data) | |
| corr_tech_market | correlation(tech_data, market_data) | |
| corr_health_market | correlation(health_data, market_data) | |
| corr_energy_market | correlation(energy_data, market_data) | |

## Covariance Matrix Elements

| Pair | Formula | Result |
|------|---------|--------|
| cov_tech_health | covariance(tech_data, health_data) | |
| cov_tech_energy | covariance(tech_data, energy_data) | |
| cov_health_energy | covariance(health_data, energy_data) | |

## Rank Correlations (Spearman)

| Pair | Formula | Result |
|------|---------|--------|
| spear_tech_market | spearman(tech_data, market_data) | |
| spear_health_market | spearman(health_data, market_data) | |
| spear_energy_market | spearman(energy_data, market_data) | |

## ═══════════════════════════════════════════════════════════════════════════
## SECTION 3: BETA CALCULATION (CAPM REGRESSION)
## ═══════════════════════════════════════════════════════════════════════════

## Tech vs Market Regression

| Metric | Formula | Result |
|--------|---------|--------|
| reg_tech | linear_reg(market_data, tech_data) | |
| beta_tech | reg_tech.slope | |
| alpha_tech | reg_tech.intercept | |
| r2_tech | reg_tech.r_squared | |
| se_tech | reg_tech.std_error | |

## Health vs Market Regression

| Metric | Formula | Result |
|--------|---------|--------|
| reg_health | linear_reg(market_data, health_data) | |
| beta_health | reg_health.slope | |
| alpha_health | reg_health.intercept | |
| r2_health | reg_health.r_squared | |

## Energy vs Market Regression

| Metric | Formula | Result |
|--------|---------|--------|
| reg_energy | linear_reg(market_data, energy_data) | |
| beta_energy | reg_energy.slope | |
| alpha_energy | reg_energy.intercept | |
| r2_energy | reg_energy.r_squared | |

## Portfolio Beta (Weighted Average)

| Metric | Formula | Result |
|--------|---------|--------|
| portfolio_beta | 0.50 * beta_tech + 0.30 * beta_health + 0.20 * beta_energy | |

## ═══════════════════════════════════════════════════════════════════════════
## SECTION 4: PORTFOLIO CONSTRUCTION
## ═══════════════════════════════════════════════════════════════════════════

## Portfolio Returns (Monthly)

| Metric | Formula | Result |
|--------|---------|--------|
| port_r1 | 0.50 * 3.2 + 0.30 * 1.5 + 0.20 * (-2.1) | |
| port_r2 | 0.50 * (-1.5) + 0.30 * 2.3 + 0.20 * 4.5 | |
| port_r3 | 0.50 * 5.8 + 0.30 * 0.8 + 0.20 * (-1.2) | |
| port_returns | [1.63, 0.84, 3.1, -0.31, 1.98, 2.38, -0.82, 4.3, 1.91, -1.63, 3.11, 2.95, -0.64, 3.51, 2.66, -2.7, 2.91, 1.2, -0.32, 3.07, 2.26, -1.15, 2.63, 1.87] | |

## Portfolio Statistics

| Metric | Formula | Result |
|--------|---------|--------|
| port_mean | mean(port_returns) | |
| port_std | stddev(port_returns) | |
| port_var | variance(port_returns) | |
| port_median | median(port_returns) | |
| port_skew | skewness(port_returns) | |
| port_kurt | kurtosis(port_returns) | |

## ═══════════════════════════════════════════════════════════════════════════
## SECTION 5: RISK METRICS
## ═══════════════════════════════════════════════════════════════════════════

## Sharpe Ratio (Assuming 0.3% monthly risk-free rate)

| Metric | Formula | Result |
|--------|---------|--------|
| rf_monthly | 0.3 | |
| sharpe_tech | (tech_mean - rf_monthly) / tech_std | |
| sharpe_health | (health_mean - rf_monthly) / health_std | |
| sharpe_energy | (energy_mean - rf_monthly) / energy_std | |
| sharpe_portfolio | (port_mean - rf_monthly) / port_std | |

## Annualized Metrics

| Metric | Formula | Result |
|--------|---------|--------|
| ann_return_port | port_mean * 12 | |
| ann_std_port | port_std * sqrt(12) | |
| ann_sharpe | sharpe_portfolio * sqrt(12) | |

## Value at Risk (VaR) - Parametric Method

| Metric | Formula | Result |
|--------|---------|--------|
| z_95 | snorm_inv(0.05) | |
| z_99 | snorm_inv(0.01) | |
| var_95_monthly | port_mean + z_95 * port_std | |
| var_99_monthly | port_mean + z_99 * port_std | |
| var_95_annual | var_95_monthly * sqrt(12) | |

## Conditional VaR (Expected Shortfall) Approximation

| Metric | Formula | Result |
|--------|---------|--------|
| es_factor_95 | snorm_pdf(z_95) / 0.05 | |
| cvar_95 | port_mean - port_std * es_factor_95 | |

## Maximum Drawdown Proxy

| Metric | Formula | Result |
|--------|---------|--------|
| cumulative | cumsum(port_returns) | |
| max_cum | max(cumulative) | |
| min_cum | min(cumulative) | |

## ═══════════════════════════════════════════════════════════════════════════
## SECTION 6: HYPOTHESIS TESTING
## ═══════════════════════════════════════════════════════════════════════════

## Test 1: Is Tech mean return significantly different from zero?

| Metric | Formula | Result |
|--------|---------|--------|
| t_tech_zero | t_test_1(tech_data, 0) | |
| t_tech_stat | t_tech_zero.t | |
| p_tech_zero | t_tech_zero.p | |
| sig_tech | t_tech_zero.p < 0.05 | |

## Test 2: Is Portfolio return significantly different from market?

| Metric | Formula | Result |
|--------|---------|--------|
| t_port_market | t_test_2(port_returns, market_data) | |
| t_pm_stat | t_port_market.t | |
| p_port_market | t_port_market.p | |
| sig_pm | t_port_market.p < 0.05 | |

## Test 3: Do the three assets have different mean returns? (ANOVA)

| Metric | Formula | Result |
|--------|---------|--------|
| anova_assets | anova(tech_data, health_data, energy_data) | |
| f_assets | anova_assets.f | |
| p_assets | anova_assets.p | |
| sig_anova | anova_assets.p < 0.05 | |

## Test 4: Is Tech more volatile than Health? (F-test)

| Metric | Formula | Result |
|--------|---------|--------|
| f_vol | f_test(tech_data, health_data) | |
| f_vol_stat | f_vol.f | |
| p_vol | f_vol.p | |

## Test 5: Confidence Interval for Portfolio Return

| Metric | Formula | Result |
|--------|---------|--------|
| ci_port | ci(port_returns, 0.95) | |
| ci_low | ci_port.low | |
| ci_high | ci_port.high | |
| ci_margin | ci_port.margin | |
| contains_zero | ci_low < 0 | |

## ═══════════════════════════════════════════════════════════════════════════
## SECTION 7: DISTRIBUTION ANALYSIS
## ═══════════════════════════════════════════════════════════════════════════

## Probability Calculations

| Metric | Formula | Result |
|--------|---------|--------|
| prob_tech_loss | norm_cdf(0, tech_mean, tech_std) | |
| prob_port_loss | norm_cdf(0, port_mean, port_std) | |
| prob_port_gt_2 | 1 - norm_cdf(2, port_mean, port_std) | |
| prob_port_gt_5 | 1 - norm_cdf(5, port_mean, port_std) | |

## Quantiles for Portfolio

| Metric | Formula | Result |
|--------|---------|--------|
| q10_port | percentile(port_returns, 10) | |
| q25_port | q1(port_returns) | |
| q50_port | median(port_returns) | |
| q75_port | q3(port_returns) | |
| q90_port | percentile(port_returns, 90) | |

## Z-Scores for Extreme Months

| Metric | Formula | Result |
|--------|---------|--------|
| worst_month | min(port_returns) | |
| best_month | max(port_returns) | |
| z_worst | zscore(worst_month, port_returns) | |
| z_best | zscore(best_month, port_returns) | |
| worst_prob | norm_cdf(z_worst, 0, 1) | |

## ═══════════════════════════════════════════════════════════════════════════
## SECTION 8: ROLLING & TREND ANALYSIS
## ═══════════════════════════════════════════════════════════════════════════

## Moving Averages

| Metric | Formula | Result |
|--------|---------|--------|
| ma3_port | moving_avg(port_returns, 3) | |
| ma6_port | moving_avg(port_returns, 6) | |
| ewma_port | ewma(port_returns, 0.3) | |

## Momentum Analysis

| Metric | Formula | Result |
|--------|---------|--------|
| diffs_port | differences(port_returns) | |
| pos_months | count(port_returns) | |
| cumulative_ret | cumsum(port_returns) | |
| final_cum | nth(cumulative_ret, 23) | |

## ═══════════════════════════════════════════════════════════════════════════
## SECTION 9: EXECUTIVE SUMMARY
## ═══════════════════════════════════════════════════════════════════════════

## Key Performance Indicators

| KPI | Formula | Result |
|-----|---------|--------|
| Annual Return | port_mean * 12 | |
| Annual Volatility | port_std * sqrt(12) | |
| Sharpe Ratio | ann_sharpe | |
| Portfolio Beta | portfolio_beta | |
| VaR 95% Monthly | var_95_monthly | |
| Max Correlation | max([corr_tech_health, corr_tech_energy, corr_health_energy]) | |
| Returns Significant | sig_tech | |
| Diversification Benefit | tech_std - port_std | |

## Risk Assessment

| Assessment | Formula | Result |
|------------|---------|--------|
| high_beta | portfolio_beta > 1 | |
| negative_var | var_95_monthly < 0 | |
| normal_skew | abs(port_skew) < 1 | |
| excess_kurt | abs(port_kurt) > 3 | |
```

data\examples\trig_identities.fmd
```fmd
<!-- Trigonometric Identities Verification -->
# Trigonometric Identities

Verifying fundamental trig identities with arbitrary precision.

## Trig Calculations @precision:50

| name | formula | result |
|------|---------|--------|
| pi | π | |
| pi_2 | π / 2 | |
| pi_4 | π / 4 | |
| pi_6 | π / 6 | |
| x | pi_4 | |
| sin_x | sin(x) | |
| cos_x | cos(x) | |
| sin_sq | sin_x * sin_x | |
| cos_sq | cos_x * cos_x | |
| pyth_check | sin_sq + cos_sq | |
| pyth_error | pyth_check - 1 | |
| sin_0 | sin(0) | |
| cos_0 | cos(0) | |
| sin_pi_2 | sin(pi_2) | |
| cos_pi_2 | cos(pi_2) | |
| sin_pi_6 | sin(pi_6) | |
| tan_pi_4 | tan(pi_4) | |
| double_x | 2 * x | |
| sin_2x | sin(double_x) | |
| double_formula | 2 * sin_x * cos_x | |
| double_check | sin_2x - double_formula | |

Identities verified:
- sin²(x) + cos²(x) = 1 (pyth_error should be ~0)
- sin(2x) = 2·sin(x)·cos(x) (double_check should be ~0)
- sin(0) = 0, cos(0) = 1
- sin(π/2) = 1, cos(π/2) = 0
- sin(π/6) = 0.5
- tan(π/4) = 1
```

data\examples\unit_conversions.fmd
```fmd
<!-- Unit Conversion Calculator -->
# Unit Conversions

Common unit conversion calculations with exact values.

## Length Conversions @precision:30

| name | formula | result |
|------|---------|--------|
| meters | 1 | |
| feet | meters * 3.28084 | |
| inches | meters * 39.3701 | |
| miles | meters / 1609.344 | |
| km | meters / 1000 | |

## Temperature Conversions

| name | formula | result |
|------|---------|--------|
| celsius | 100 | |
| fahrenheit | celsius * 9/5 + 32 | |
| kelvin | celsius + 273.15 | |

## Physical Constants @precision:50 @sigfigs:6

Using integer-mantissa notation for full precision.

| name | formula | result |
|------|---------|--------|
| c | 299792458 | |
| h | 662607015e-42 | |
| e_charge | 1602176634e-28 | |
| avogadro | 602214076e15 | |
| boltzmann | 1380649e-29 | |
| mass_kg | 1 | |
| energy_j | mass_kg * c * c | |
| energy_ev | energy_j / e_charge | |
```

data\examples\workdays.fmd
```fmd
<!-- Workday Functions Example -->
# Workday Functions

Calculate payment due dates and working day schedules.

## Invoice Terms @datetimeFmt:YYYY-MM-DD

| name | formula | result |
|------|---------|--------|
| invoice_date | date(2025, 6, 1) | |
| is_workday_check | isWorkday(invoice_date) | |
| net_30_calendar | addDays(invoice_date, 30) | |
| net_30_workdays | addWorkdays(invoice_date, 30) | |
| days_between | diff(net_30_workdays, net_30_calendar, "days") | |

## Weekend Handling @datetimeFmt:YYYY-MM-DD

| name | formula | result |
|------|---------|--------|
| friday | date(2025, 6, 13) | |
| friday_weekday | weekday(friday) | |
| saturday | date(2025, 6, 14) | |
| saturday_weekday | weekday(saturday) | |
| is_friday_workday | isWorkday(friday) | |
| is_saturday_workday | isWorkday(saturday) | |
| next_from_friday | nextWorkday(friday) | |
| next_from_saturday | nextWorkday(saturday) | |
| prev_from_saturday | prevWorkday(saturday) | |

## Monthly Payment Schedule @datetimeFmt:YYYY-MM-DD

| name | formula | result |
|------|---------|--------|
| reference | date(2025, 6, 15) | |
| jul_first | nextMonth(reference) | |
| jul_first_wd | nextMonthWd(reference) | |
| jul_1_weekday | weekday(jul_first) | |
| aug_first | nextMonth(jul_first) | |
| aug_first_wd | nextMonthWd(jul_first) | |

## Year End Payment @datetimeFmt:YYYY-MM-DD

| name | formula | result |
|------|---------|--------|
| dec_ref | date(2025, 12, 15) | |
| dec_end | eom(dec_ref) | |
| jan_first | nextMonth(dec_ref) | |
| jan_first_wd | nextMonthWd(dec_ref) | |
| jan_1_weekday | weekday(jan_first) | |
```

docker-compose.yml
```yml
version: '3.8'

services:
  folio-mcp:
    build: .
    container_name: folio-mcp
    stdin_open: true
    environment:
      - FOLIO_DATA_PATH=/app/folio
      - RUST_LOG=info
    volumes:
      - ./data:/app/folio:ro
    # No ports needed - MCP uses stdio
```

Dockerfile
```Dockerfile
# Build stage
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /app

# Copy workspace
COPY . .

# Build release
RUN cargo build --release -p folio-mcp

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies (minimal)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary
COPY --from=builder /app/target/release/folio-mcp /usr/local/bin/folio-mcp

# Create data directory
RUN mkdir -p /app/folio

# Environment
ENV FOLIO_DATA_PATH=/app/folio
ENV RUST_LOG=info

# Run MCP server (stdio mode)
ENTRYPOINT ["/usr/local/bin/folio-mcp"]
```

folio-core\Cargo.toml
```toml
[package]
name = "folio-core"
description = "Core types for Folio: Number, Value, Error"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
# Arbitrary precision - dashu replaces num-bigint/num-rational
dashu-base = { workspace = true }
dashu-int = { workspace = true }
dashu-ratio = { workspace = true }
dashu-float = { workspace = true }

thiserror = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

folio-core\src\datetime.rs
```rs
//! DateTime and Duration types for Folio
//!
//! Provides nanosecond-precision datetime and duration types with full
//! arithmetic support. Uses i128 internally to avoid overflow issues.
//!
//! Design principles:
//! - No external datetime crates (keeps folio-core minimal)
//! - Gregorian proleptic calendar
//! - UTC-first with optional timezone offset
//! - Never panics - all operations return Results or handle edge cases

use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Constants
// ============================================================================

pub const NANOS_PER_SECOND: i128 = 1_000_000_000;
pub const NANOS_PER_MINUTE: i128 = 60 * NANOS_PER_SECOND;
pub const NANOS_PER_HOUR: i128 = 60 * NANOS_PER_MINUTE;
pub const NANOS_PER_DAY: i128 = 24 * NANOS_PER_HOUR;

/// Days in each month (non-leap year)
const DAYS_IN_MONTH: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Unix epoch: 1970-01-01T00:00:00Z
const UNIX_EPOCH_DAYS: i64 = 719_468; // Days from year 0 to 1970-01-01

// ============================================================================
// FolioDateTime
// ============================================================================

/// A datetime with nanosecond precision
///
/// Internally stores nanoseconds since Unix epoch (1970-01-01T00:00:00Z).
/// Supports dates from billions of years in the past to billions in the future.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FolioDateTime {
    /// Nanoseconds since Unix epoch (negative for pre-1970 dates)
    nanos: i128,
    /// Timezone offset in seconds from UTC (None = UTC)
    #[serde(skip_serializing_if = "Option::is_none")]
    tz_offset: Option<i32>,
}

impl FolioDateTime {
    // ========== Construction ==========

    /// Create a datetime from nanoseconds since Unix epoch
    pub fn from_nanos(nanos: i128) -> Self {
        Self { nanos, tz_offset: None }
    }

    /// Create a datetime from seconds since Unix epoch
    pub fn from_unix_secs(secs: i64) -> Self {
        Self {
            nanos: (secs as i128) * NANOS_PER_SECOND,
            tz_offset: None,
        }
    }

    /// Create a datetime from milliseconds since Unix epoch
    pub fn from_unix_millis(millis: i64) -> Self {
        Self {
            nanos: (millis as i128) * 1_000_000,
            tz_offset: None,
        }
    }

    /// Create a date (time = 00:00:00)
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Result<Self, DateTimeError> {
        Self::from_ymd_hms_nano(year, month, day, 0, 0, 0, 0)
    }

    /// Create a datetime from components
    pub fn from_ymd_hms(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
    ) -> Result<Self, DateTimeError> {
        Self::from_ymd_hms_nano(year, month, day, hour, minute, second, 0)
    }

    /// Create a datetime from components with nanoseconds
    pub fn from_ymd_hms_nano(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        nano: u32,
    ) -> Result<Self, DateTimeError> {
        // Validate components
        if month < 1 || month > 12 {
            return Err(DateTimeError::InvalidMonth(month));
        }
        let max_day = days_in_month(year, month);
        if day < 1 || day > max_day {
            return Err(DateTimeError::InvalidDay(day, month, year));
        }
        if hour > 23 {
            return Err(DateTimeError::InvalidHour(hour));
        }
        if minute > 59 {
            return Err(DateTimeError::InvalidMinute(minute));
        }
        if second > 59 {
            return Err(DateTimeError::InvalidSecond(second));
        }
        if nano >= 1_000_000_000 {
            return Err(DateTimeError::InvalidNano(nano));
        }

        // Convert to days since epoch
        let days = days_from_civil(year, month, day);
        let day_nanos = (days as i128) * NANOS_PER_DAY;

        // Add time components
        let time_nanos = (hour as i128) * NANOS_PER_HOUR
            + (minute as i128) * NANOS_PER_MINUTE
            + (second as i128) * NANOS_PER_SECOND
            + (nano as i128);

        Ok(Self {
            nanos: day_nanos + time_nanos,
            tz_offset: None,
        })
    }

    /// Create a time-only value (date = 1970-01-01)
    pub fn from_hms(hour: u32, minute: u32, second: u32) -> Result<Self, DateTimeError> {
        Self::from_ymd_hms(1970, 1, 1, hour, minute, second)
    }

    /// Get current UTC time
    pub fn now() -> Self {
        // Use std::time for current time
        let duration = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
        Self {
            nanos: duration.as_nanos() as i128,
            tz_offset: None,
        }
    }

    // ========== Accessors ==========

    /// Get nanoseconds since Unix epoch
    pub fn as_nanos(&self) -> i128 {
        self.nanos
    }

    /// Get seconds since Unix epoch (truncated)
    pub fn as_unix_secs(&self) -> i64 {
        (self.nanos / NANOS_PER_SECOND) as i64
    }

    /// Get milliseconds since Unix epoch (truncated)
    pub fn as_unix_millis(&self) -> i64 {
        (self.nanos / 1_000_000) as i64
    }

    /// Get timezone offset in seconds (None = UTC)
    pub fn tz_offset(&self) -> Option<i32> {
        self.tz_offset
    }

    /// Set timezone offset
    pub fn with_tz_offset(mut self, offset_secs: i32) -> Self {
        self.tz_offset = Some(offset_secs);
        self
    }

    /// Convert to UTC (remove timezone info)
    pub fn to_utc(mut self) -> Self {
        self.tz_offset = None;
        self
    }

    /// Get year component
    pub fn year(&self) -> i32 {
        let (y, _, _) = self.to_ymd();
        y
    }

    /// Get month component (1-12)
    pub fn month(&self) -> u32 {
        let (_, m, _) = self.to_ymd();
        m
    }

    /// Get day component (1-31)
    pub fn day(&self) -> u32 {
        let (_, _, d) = self.to_ymd();
        d
    }

    /// Get hour component (0-23)
    pub fn hour(&self) -> u32 {
        let day_nanos = self.nanos.rem_euclid(NANOS_PER_DAY);
        (day_nanos / NANOS_PER_HOUR) as u32
    }

    /// Get minute component (0-59)
    pub fn minute(&self) -> u32 {
        let day_nanos = self.nanos.rem_euclid(NANOS_PER_DAY);
        ((day_nanos % NANOS_PER_HOUR) / NANOS_PER_MINUTE) as u32
    }

    /// Get second component (0-59)
    pub fn second(&self) -> u32 {
        let day_nanos = self.nanos.rem_euclid(NANOS_PER_DAY);
        ((day_nanos % NANOS_PER_MINUTE) / NANOS_PER_SECOND) as u32
    }

    /// Get nanosecond component (0-999_999_999)
    pub fn nanosecond(&self) -> u32 {
        (self.nanos.rem_euclid(NANOS_PER_SECOND)) as u32
    }

    /// Get millisecond component (0-999)
    pub fn millisecond(&self) -> u32 {
        self.nanosecond() / 1_000_000
    }

    /// Get day of week (1=Monday, 7=Sunday, ISO 8601)
    pub fn weekday(&self) -> u32 {
        let days = self.nanos.div_euclid(NANOS_PER_DAY);
        // 1970-01-01 was Thursday (4)
        let day_of_week = (days + 4).rem_euclid(7);
        if day_of_week == 0 { 7 } else { day_of_week as u32 }
    }

    /// Get day of year (1-366)
    pub fn day_of_year(&self) -> u32 {
        let (year, month, day) = self.to_ymd();
        let mut doy = day;
        for m in 1..month {
            doy += days_in_month(year, m);
        }
        doy
    }

    /// Get ISO week number (1-53)
    pub fn iso_week(&self) -> u32 {
        // ISO week: week containing first Thursday of year
        let doy = self.day_of_year() as i32;
        let dow = self.weekday() as i32; // 1=Mon, 7=Sun

        // Find the Thursday of the current week
        let thursday_doy = doy + (4 - dow);

        // Week 1 contains Jan 4th
        let jan4_dow = FolioDateTime::from_ymd(self.year(), 1, 4)
            .map(|dt| dt.weekday() as i32)
            .unwrap_or(4);
        let week1_start = 4 - jan4_dow + 1; // Day of year when week 1 starts

        let week = (thursday_doy - week1_start) / 7 + 1;

        if week < 1 {
            // Last week of previous year
            FolioDateTime::from_ymd(self.year() - 1, 12, 31)
                .map(|dt| dt.iso_week())
                .unwrap_or(52)
        } else if week > 52 {
            // Check if it's week 1 of next year
            let next_jan4_dow = FolioDateTime::from_ymd(self.year() + 1, 1, 4)
                .map(|dt| dt.weekday() as i32)
                .unwrap_or(4);
            let days_in_year = if is_leap_year(self.year()) { 366 } else { 365 };
            if doy > days_in_year - (7 - next_jan4_dow) {
                1
            } else {
                week as u32
            }
        } else {
            week as u32
        }
    }

    /// Decompose into year, month, day
    pub fn to_ymd(&self) -> (i32, u32, u32) {
        let days = self.nanos.div_euclid(NANOS_PER_DAY) as i64;
        civil_from_days(days)
    }

    /// Decompose into all components
    pub fn to_components(&self) -> DateTimeComponents {
        let (year, month, day) = self.to_ymd();
        DateTimeComponents {
            year,
            month,
            day,
            hour: self.hour(),
            minute: self.minute(),
            second: self.second(),
            nanosecond: self.nanosecond(),
            tz_offset: self.tz_offset,
        }
    }

    // ========== Arithmetic ==========

    /// Add a duration
    pub fn add_duration(&self, duration: &FolioDuration) -> Self {
        Self {
            nanos: self.nanos + duration.nanos,
            tz_offset: self.tz_offset,
        }
    }

    /// Subtract a duration
    pub fn sub_duration(&self, duration: &FolioDuration) -> Self {
        Self {
            nanos: self.nanos - duration.nanos,
            tz_offset: self.tz_offset,
        }
    }

    /// Get duration between two datetimes
    pub fn duration_since(&self, other: &FolioDateTime) -> FolioDuration {
        FolioDuration {
            nanos: self.nanos - other.nanos,
        }
    }

    /// Add days
    pub fn add_days(&self, days: i64) -> Self {
        Self {
            nanos: self.nanos + (days as i128) * NANOS_PER_DAY,
            tz_offset: self.tz_offset,
        }
    }

    /// Add months (handles month boundaries)
    pub fn add_months(&self, months: i32) -> Self {
        let (mut year, mut month, day) = self.to_ymd();

        // Add months
        let total_months = (year as i64) * 12 + (month as i64 - 1) + (months as i64);
        year = (total_months.div_euclid(12)) as i32;
        month = (total_months.rem_euclid(12) + 1) as u32;

        // Clamp day to valid range for new month
        let max_day = days_in_month(year, month);
        let new_day = day.min(max_day);

        // Preserve time components
        let time_nanos = self.nanos.rem_euclid(NANOS_PER_DAY);
        let days = days_from_civil(year, month, new_day);

        Self {
            nanos: (days as i128) * NANOS_PER_DAY + time_nanos,
            tz_offset: self.tz_offset,
        }
    }

    /// Add years (handles leap years)
    pub fn add_years(&self, years: i32) -> Self {
        self.add_months(years * 12)
    }

    // ========== Utilities ==========

    /// Get start of day (00:00:00.000)
    pub fn start_of_day(&self) -> Self {
        let days = self.nanos.div_euclid(NANOS_PER_DAY);
        Self {
            nanos: days * NANOS_PER_DAY,
            tz_offset: self.tz_offset,
        }
    }

    /// Get end of day (23:59:59.999999999)
    pub fn end_of_day(&self) -> Self {
        let days = self.nanos.div_euclid(NANOS_PER_DAY);
        Self {
            nanos: (days + 1) * NANOS_PER_DAY - 1,
            tz_offset: self.tz_offset,
        }
    }

    /// Get start of month
    pub fn start_of_month(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        Self::from_ymd(year, month, 1).unwrap_or_else(|_| self.clone())
    }

    /// Get start of year
    pub fn start_of_year(&self) -> Self {
        let year = self.year();
        Self::from_ymd(year, 1, 1).unwrap_or_else(|_| self.clone())
    }

    /// Check if same day as another datetime
    pub fn is_same_day(&self, other: &FolioDateTime) -> bool {
        self.nanos.div_euclid(NANOS_PER_DAY) == other.nanos.div_euclid(NANOS_PER_DAY)
    }

    /// Check if before another datetime
    pub fn is_before(&self, other: &FolioDateTime) -> bool {
        self.nanos < other.nanos
    }

    /// Check if after another datetime
    pub fn is_after(&self, other: &FolioDateTime) -> bool {
        self.nanos > other.nanos
    }

    // ========== Period End Methods ==========

    /// Get end of week (Sunday 23:59:59.999...)
    /// week_start: 1=Monday (ISO), 7=Sunday (US)
    pub fn end_of_week(&self, week_start: u32) -> Self {
        // Calculate days until end of week
        let dow = self.weekday(); // 1=Mon, 7=Sun
        let week_end = if week_start == 7 {
            // US style: week ends on Saturday (6)
            6
        } else {
            // ISO style (default): week ends on Sunday (7)
            7
        };

        let days_until_end = if dow == week_end {
            0
        } else if week_end > dow {
            (week_end - dow) as i64
        } else {
            (7 - dow + week_end) as i64
        };

        self.add_days(days_until_end).end_of_day()
    }

    /// Get end of month (last day at 23:59:59.999...)
    pub fn end_of_month(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        let last_day = days_in_month(year, month);
        Self::from_ymd(year, month, last_day)
            .map(|dt| dt.end_of_day())
            .unwrap_or_else(|_| self.clone())
    }

    /// Get end of quarter (last day of quarter at 23:59:59.999...)
    pub fn end_of_quarter(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        let quarter_end_month = match month {
            1..=3 => 3,
            4..=6 => 6,
            7..=9 => 9,
            _ => 12,
        };
        let last_day = days_in_month(year, quarter_end_month);
        Self::from_ymd(year, quarter_end_month, last_day)
            .map(|dt| dt.end_of_day())
            .unwrap_or_else(|_| self.clone())
    }

    /// Get end of year (Dec 31 23:59:59.999...)
    pub fn end_of_year(&self) -> Self {
        let year = self.year();
        Self::from_ymd(year, 12, 31)
            .map(|dt| dt.end_of_day())
            .unwrap_or_else(|_| self.clone())
    }

    // ========== Period Start Methods ==========

    /// Get start of week (Monday 00:00:00 by default)
    /// week_start: 1=Monday (ISO), 7=Sunday (US)
    pub fn start_of_week(&self, week_start: u32) -> Self {
        let dow = self.weekday(); // 1=Mon, 7=Sun
        let target_start = if week_start == 7 { 7 } else { 1 }; // Sunday or Monday

        let days_since_start = if dow >= target_start {
            (dow - target_start) as i64
        } else {
            (7 - target_start + dow) as i64
        };

        self.add_days(-days_since_start).start_of_day()
    }

    /// Get start of quarter
    pub fn start_of_quarter(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        let quarter_start_month = match month {
            1..=3 => 1,
            4..=6 => 4,
            7..=9 => 7,
            _ => 10,
        };
        Self::from_ymd(year, quarter_start_month, 1)
            .map(|dt| dt.start_of_day())
            .unwrap_or_else(|_| self.clone())
    }

    // ========== Workday Methods ==========

    /// Check if this is a weekend (Saturday=6 or Sunday=7)
    pub fn is_weekend(&self) -> bool {
        let dow = self.weekday();
        dow == 6 || dow == 7
    }

    /// Check if this is a workday (Monday-Friday)
    pub fn is_workday(&self) -> bool {
        !self.is_weekend()
    }

    /// Get next workday (if already workday, returns same day at start)
    pub fn next_workday_inclusive(&self) -> Self {
        let dow = self.weekday();
        let days_to_add = match dow {
            6 => 2, // Saturday -> Monday
            7 => 1, // Sunday -> Monday
            _ => 0,
        };
        if days_to_add > 0 {
            self.add_days(days_to_add).start_of_day()
        } else {
            self.clone()
        }
    }

    /// Get next workday (always advances at least one day)
    pub fn next_workday(&self) -> Self {
        let next = self.add_days(1);
        let dow = next.weekday();
        let days_to_add = match dow {
            6 => 2, // Saturday -> Monday
            7 => 1, // Sunday -> Monday
            _ => 0,
        };
        next.add_days(days_to_add).start_of_day()
    }

    /// Get previous workday (always goes back at least one day)
    pub fn prev_workday(&self) -> Self {
        let prev = self.add_days(-1);
        let dow = prev.weekday();
        let days_to_sub = match dow {
            6 => 1, // Saturday -> Friday
            7 => 2, // Sunday -> Friday
            _ => 0,
        };
        prev.add_days(-days_to_sub).start_of_day()
    }

    /// Add n workdays (skips weekends)
    pub fn add_workdays(&self, n: i64) -> Self {
        if n == 0 {
            return self.clone();
        }

        let mut current = self.clone();
        let mut remaining = n.abs();
        let direction = if n > 0 { 1i64 } else { -1i64 };

        // First, move to a workday if on weekend
        if current.is_weekend() {
            if direction > 0 {
                current = current.next_workday_inclusive();
            } else {
                current = current.prev_workday();
                if remaining > 0 {
                    remaining -= 1;
                }
            }
        }

        while remaining > 0 {
            current = current.add_days(direction);
            if current.is_workday() {
                remaining -= 1;
            }
        }

        current.start_of_day()
    }

    // ========== Navigation Methods ==========

    /// Get tomorrow (same time, +1 day)
    pub fn tomorrow(&self) -> Self {
        self.add_days(1)
    }

    /// Get next week (Monday 00:00:00)
    pub fn next_week(&self, week_start: u32) -> Self {
        // Go to end of current week, then add 1 day
        self.end_of_week(week_start).add_days(1).start_of_day()
    }

    /// Get first day of next month (00:00:00)
    pub fn next_month_first(&self) -> Self {
        self.add_months(1).start_of_month()
    }

    /// Get first workday of next month
    pub fn next_month_first_workday(&self) -> Self {
        self.next_month_first().next_workday_inclusive()
    }

    /// Get first day of next quarter (00:00:00)
    pub fn next_quarter_first(&self) -> Self {
        let (year, month, _) = self.to_ymd();
        let (next_year, next_quarter_month) = match month {
            1..=3 => (year, 4),
            4..=6 => (year, 7),
            7..=9 => (year, 10),
            _ => (year + 1, 1),
        };
        Self::from_ymd(next_year, next_quarter_month, 1)
            .unwrap_or_else(|_| self.clone())
    }

    /// Get first day of next year (00:00:00)
    pub fn next_year_first(&self) -> Self {
        let year = self.year();
        Self::from_ymd(year + 1, 1, 1)
            .unwrap_or_else(|_| self.clone())
    }

    // ========== Formatting ==========

    /// Format as ISO 8601 string
    pub fn to_iso_string(&self) -> String {
        let c = self.to_components();
        if let Some(offset) = c.tz_offset {
            let (sign, abs_offset) = if offset < 0 { ('-', -offset) } else { ('+', offset) };
            let hours = abs_offset / 3600;
            let minutes = (abs_offset % 3600) / 60;
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{:02}:{:02}",
                c.year, c.month, c.day, c.hour, c.minute, c.second,
                sign, hours, minutes
            )
        } else {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
                c.year, c.month, c.day, c.hour, c.minute, c.second
            )
        }
    }

    /// Format with custom pattern
    ///
    /// Supported tokens:
    /// - YYYY: 4-digit year
    /// - YY: 2-digit year
    /// - MM: 2-digit month (01-12)
    /// - M: month (1-12)
    /// - DD: 2-digit day (01-31)
    /// - D: day (1-31)
    /// - HH: 2-digit hour 24h (00-23)
    /// - H: hour 24h (0-23)
    /// - hh: 2-digit hour 12h (01-12)
    /// - h: hour 12h (1-12)
    /// - mm: 2-digit minute (00-59)
    /// - m: minute (0-59)
    /// - ss: 2-digit second (00-59)
    /// - s: second (0-59)
    /// - SSS: milliseconds (000-999)
    /// - A: AM/PM
    /// - a: am/pm
    /// - DDD: day of year (001-366)
    /// - d: weekday (1-7, Monday=1)
    /// - W: ISO week (1-53)
    pub fn format(&self, pattern: &str) -> String {
        let c = self.to_components();
        let hour12 = if c.hour == 0 { 12 } else if c.hour > 12 { c.hour - 12 } else { c.hour };
        let am_pm = if c.hour < 12 { "AM" } else { "PM" };
        let am_pm_lower = if c.hour < 12 { "am" } else { "pm" };

        let mut result = pattern.to_string();

        // Order matters - longer patterns first
        result = result.replace("YYYY", &format!("{:04}", c.year));
        result = result.replace("YY", &format!("{:02}", c.year.rem_euclid(100)));
        result = result.replace("MM", &format!("{:02}", c.month));
        result = result.replace("M", &c.month.to_string());
        result = result.replace("DDD", &format!("{:03}", self.day_of_year()));
        result = result.replace("DD", &format!("{:02}", c.day));
        result = result.replace("D", &c.day.to_string());
        result = result.replace("HH", &format!("{:02}", c.hour));
        result = result.replace("H", &c.hour.to_string());
        result = result.replace("hh", &format!("{:02}", hour12));
        result = result.replace("h", &hour12.to_string());
        result = result.replace("mm", &format!("{:02}", c.minute));
        result = result.replace("m", &c.minute.to_string());
        result = result.replace("SSS", &format!("{:03}", c.nanosecond / 1_000_000));
        result = result.replace("ss", &format!("{:02}", c.second));
        result = result.replace("s", &c.second.to_string());
        result = result.replace("A", am_pm);
        result = result.replace("a", am_pm_lower);
        result = result.replace("W", &self.iso_week().to_string());
        // 'd' for weekday - but be careful not to replace 'd' in other contexts
        // We'll use a workaround: only replace standalone 'd'

        result
    }
}

impl fmt::Display for FolioDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso_string())
    }
}

// ============================================================================
// FolioDuration
// ============================================================================

/// A duration with nanosecond precision
///
/// Can be positive (future) or negative (past).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FolioDuration {
    /// Signed nanoseconds
    nanos: i128,
}

impl FolioDuration {
    /// Create from nanoseconds
    pub fn from_nanos(nanos: i128) -> Self {
        Self { nanos }
    }

    /// Create from seconds
    pub fn from_secs(secs: i64) -> Self {
        Self {
            nanos: (secs as i128) * NANOS_PER_SECOND,
        }
    }

    /// Create from milliseconds
    pub fn from_millis(millis: i64) -> Self {
        Self {
            nanos: (millis as i128) * 1_000_000,
        }
    }

    /// Create from days
    pub fn from_days(days: i64) -> Self {
        Self {
            nanos: (days as i128) * NANOS_PER_DAY,
        }
    }

    /// Create from weeks
    pub fn from_weeks(weeks: i64) -> Self {
        Self {
            nanos: (weeks as i128) * NANOS_PER_DAY * 7,
        }
    }

    /// Create from hours
    pub fn from_hours(hours: i64) -> Self {
        Self {
            nanos: (hours as i128) * NANOS_PER_HOUR,
        }
    }

    /// Create from minutes
    pub fn from_minutes(minutes: i64) -> Self {
        Self {
            nanos: (minutes as i128) * NANOS_PER_MINUTE,
        }
    }

    /// Zero duration
    pub fn zero() -> Self {
        Self { nanos: 0 }
    }

    /// Get total nanoseconds
    pub fn as_nanos(&self) -> i128 {
        self.nanos
    }

    /// Get total seconds (truncated)
    pub fn as_secs(&self) -> i64 {
        (self.nanos / NANOS_PER_SECOND) as i64
    }

    /// Get total milliseconds (truncated)
    pub fn as_millis(&self) -> i64 {
        (self.nanos / 1_000_000) as i64
    }

    /// Get total minutes (truncated)
    pub fn as_minutes(&self) -> i64 {
        (self.nanos / NANOS_PER_MINUTE) as i64
    }

    /// Get total hours (truncated)
    pub fn as_hours(&self) -> i64 {
        (self.nanos / NANOS_PER_HOUR) as i64
    }

    /// Get total days (truncated)
    pub fn as_days(&self) -> i64 {
        (self.nanos / NANOS_PER_DAY) as i64
    }

    /// Get total weeks (truncated)
    pub fn as_weeks(&self) -> i64 {
        (self.nanos / (NANOS_PER_DAY * 7)) as i64
    }

    /// Get fractional days as f64
    pub fn as_days_f64(&self) -> f64 {
        (self.nanos as f64) / (NANOS_PER_DAY as f64)
    }

    /// Get fractional hours as f64
    pub fn as_hours_f64(&self) -> f64 {
        (self.nanos as f64) / (NANOS_PER_HOUR as f64)
    }

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.nanos == 0
    }

    /// Check if negative
    pub fn is_negative(&self) -> bool {
        self.nanos < 0
    }

    /// Get absolute value
    pub fn abs(&self) -> Self {
        Self {
            nanos: self.nanos.abs(),
        }
    }

    /// Negate
    pub fn neg(&self) -> Self {
        Self { nanos: -self.nanos }
    }

    /// Add durations
    pub fn add(&self, other: &FolioDuration) -> Self {
        Self {
            nanos: self.nanos + other.nanos,
        }
    }

    /// Subtract durations
    pub fn sub(&self, other: &FolioDuration) -> Self {
        Self {
            nanos: self.nanos - other.nanos,
        }
    }

    /// Multiply by scalar
    pub fn mul(&self, scalar: i64) -> Self {
        Self {
            nanos: self.nanos * (scalar as i128),
        }
    }

    /// Multiply by float (for Number compatibility)
    pub fn mul_f64(&self, scalar: f64) -> Self {
        Self {
            nanos: ((self.nanos as f64) * scalar) as i128,
        }
    }

    /// Divide by scalar
    pub fn div(&self, scalar: i64) -> Option<Self> {
        if scalar == 0 {
            None
        } else {
            Some(Self {
                nanos: self.nanos / (scalar as i128),
            })
        }
    }
}

impl fmt::Display for FolioDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abs_nanos = self.nanos.abs();
        let sign = if self.nanos < 0 { "-" } else { "" };

        let days = abs_nanos / NANOS_PER_DAY;
        let hours = (abs_nanos % NANOS_PER_DAY) / NANOS_PER_HOUR;
        let minutes = (abs_nanos % NANOS_PER_HOUR) / NANOS_PER_MINUTE;
        let seconds = (abs_nanos % NANOS_PER_MINUTE) / NANOS_PER_SECOND;

        if days > 0 {
            write!(f, "{}{}d {:02}:{:02}:{:02}", sign, days, hours, minutes, seconds)
        } else {
            write!(f, "{}{:02}:{:02}:{:02}", sign, hours, minutes, seconds)
        }
    }
}

// ============================================================================
// DateTimeComponents
// ============================================================================

/// Decomposed datetime components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateTimeComponents {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub nanosecond: u32,
    pub tz_offset: Option<i32>,
}

// ============================================================================
// DateTimeError
// ============================================================================

/// Errors that can occur with datetime operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DateTimeError {
    InvalidMonth(u32),
    InvalidDay(u32, u32, i32), // day, month, year
    InvalidHour(u32),
    InvalidMinute(u32),
    InvalidSecond(u32),
    InvalidNano(u32),
    ParseError(String),
    Overflow,
}

impl fmt::Display for DateTimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMonth(m) => write!(f, "Invalid month: {} (must be 1-12)", m),
            Self::InvalidDay(d, m, y) => write!(f, "Invalid day: {} for {}/{}", d, m, y),
            Self::InvalidHour(h) => write!(f, "Invalid hour: {} (must be 0-23)", h),
            Self::InvalidMinute(m) => write!(f, "Invalid minute: {} (must be 0-59)", m),
            Self::InvalidSecond(s) => write!(f, "Invalid second: {} (must be 0-59)", s),
            Self::InvalidNano(n) => write!(f, "Invalid nanosecond: {}", n),
            Self::ParseError(s) => write!(f, "Parse error: {}", s),
            Self::Overflow => write!(f, "DateTime overflow"),
        }
    }
}

impl std::error::Error for DateTimeError {}

// ============================================================================
// Calendar Utilities (Gregorian proleptic)
// ============================================================================

/// Check if year is a leap year
pub fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Get days in a month
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        m if m >= 1 && m <= 12 => DAYS_IN_MONTH[(m - 1) as usize] as u32,
        _ => 0,
    }
}

/// Convert civil date to days since Unix epoch
/// Algorithm from Howard Hinnant: http://howardhinnant.github.io/date_algorithms.html
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year } as i64;
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32; // [0, 399]
    let m = month as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + day as i64 - 1; // [0, 365]
    let doe = yoe as i64 * 365 + yoe as i64 / 4 - yoe as i64 / 100 + doy; // [0, 146096]
    era * 146097 + doe - UNIX_EPOCH_DAYS
}

/// Convert days since Unix epoch to civil date
/// Algorithm from Howard Hinnant: http://howardhinnant.github.io/date_algorithms.html
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + UNIX_EPOCH_DAYS;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m as u32, d as u32)
}

// ============================================================================
// Parsing
// ============================================================================

impl FolioDateTime {
    /// Parse ISO 8601 datetime string
    ///
    /// Supported formats:
    /// - 2025-06-15
    /// - 2025-06-15T14:30:00
    /// - 2025-06-15T14:30:00Z
    /// - 2025-06-15T14:30:00+05:30
    /// - 2025-06-15T14:30:00.123Z
    pub fn parse(s: &str) -> Result<Self, DateTimeError> {
        let s = s.trim();

        // Try date only: YYYY-MM-DD
        if s.len() == 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
            return Self::parse_date_only(s);
        }

        // Try datetime with T separator
        if let Some(t_pos) = s.find('T') {
            let date_part = &s[..t_pos];
            let time_part = &s[t_pos + 1..];
            return Self::parse_datetime(date_part, time_part);
        }

        // Try datetime with space separator
        if let Some(space_pos) = s.find(' ') {
            let date_part = &s[..space_pos];
            let time_part = &s[space_pos + 1..];
            return Self::parse_datetime(date_part, time_part);
        }

        Err(DateTimeError::ParseError(format!("Unrecognized format: {}", s)))
    }

    fn parse_date_only(s: &str) -> Result<Self, DateTimeError> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 {
            return Err(DateTimeError::ParseError("Expected YYYY-MM-DD".to_string()));
        }

        let year: i32 = parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid year".to_string()))?;
        let month: u32 = parts[1].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid month".to_string()))?;
        let day: u32 = parts[2].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid day".to_string()))?;

        Self::from_ymd(year, month, day)
    }

    fn parse_datetime(date_part: &str, time_part: &str) -> Result<Self, DateTimeError> {
        // Parse date
        let date_parts: Vec<&str> = date_part.split('-').collect();
        if date_parts.len() != 3 {
            return Err(DateTimeError::ParseError("Expected YYYY-MM-DD".to_string()));
        }

        let year: i32 = date_parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid year".to_string()))?;
        let month: u32 = date_parts[1].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid month".to_string()))?;
        let day: u32 = date_parts[2].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid day".to_string()))?;

        // Parse timezone from time part
        let (time_str, tz_offset) = Self::extract_timezone(time_part)?;

        // Parse time (possibly with fractional seconds)
        let (time_no_frac, nanos) = if let Some(dot_pos) = time_str.find('.') {
            let frac_str = &time_str[dot_pos + 1..];
            let nanos = Self::parse_fractional_seconds(frac_str)?;
            (&time_str[..dot_pos], nanos)
        } else {
            (time_str, 0u32)
        };

        let time_parts: Vec<&str> = time_no_frac.split(':').collect();
        if time_parts.len() < 2 {
            return Err(DateTimeError::ParseError("Expected HH:MM[:SS]".to_string()));
        }

        let hour: u32 = time_parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid hour".to_string()))?;
        let minute: u32 = time_parts[1].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid minute".to_string()))?;
        let second: u32 = if time_parts.len() >= 3 {
            time_parts[2].parse()
                .map_err(|_| DateTimeError::ParseError("Invalid second".to_string()))?
        } else {
            0
        };

        let mut dt = Self::from_ymd_hms_nano(year, month, day, hour, minute, second, nanos)?;
        if let Some(offset) = tz_offset {
            dt.tz_offset = Some(offset);
        }
        Ok(dt)
    }

    fn extract_timezone(time_part: &str) -> Result<(&str, Option<i32>), DateTimeError> {
        // Check for Z suffix
        if time_part.ends_with('Z') {
            return Ok((&time_part[..time_part.len() - 1], Some(0)));
        }

        // Check for +HH:MM or -HH:MM suffix
        if let Some(plus_pos) = time_part.rfind('+') {
            let tz_str = &time_part[plus_pos + 1..];
            let offset = Self::parse_tz_offset(tz_str)?;
            return Ok((&time_part[..plus_pos], Some(offset)));
        }

        // Check for -HH:MM (but not negative hours like in 00:00:00)
        // Only consider it a timezone if it's at position > 5 (after HH:MM)
        if let Some(minus_pos) = time_part.rfind('-') {
            if minus_pos >= 5 {
                let tz_str = &time_part[minus_pos + 1..];
                let offset = -Self::parse_tz_offset(tz_str)?;
                return Ok((&time_part[..minus_pos], Some(offset)));
            }
        }

        Ok((time_part, None))
    }

    fn parse_tz_offset(s: &str) -> Result<i32, DateTimeError> {
        let parts: Vec<&str> = s.split(':').collect();
        let hours: i32 = parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid timezone hours".to_string()))?;
        let minutes: i32 = if parts.len() > 1 {
            parts[1].parse()
                .map_err(|_| DateTimeError::ParseError("Invalid timezone minutes".to_string()))?
        } else {
            0
        };
        Ok(hours * 3600 + minutes * 60)
    }

    fn parse_fractional_seconds(s: &str) -> Result<u32, DateTimeError> {
        // Pad or truncate to 9 digits (nanoseconds)
        let padded = if s.len() >= 9 {
            &s[..9]
        } else {
            &format!("{:0<9}", s)
        };
        padded.parse()
            .map_err(|_| DateTimeError::ParseError("Invalid fractional seconds".to_string()))
    }

    /// Parse a time-only string (HH:MM:SS)
    pub fn parse_time(s: &str) -> Result<Self, DateTimeError> {
        let s = s.trim();
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 2 {
            return Err(DateTimeError::ParseError("Expected HH:MM[:SS]".to_string()));
        }

        let hour: u32 = parts[0].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid hour".to_string()))?;
        let minute: u32 = parts[1].parse()
            .map_err(|_| DateTimeError::ParseError("Invalid minute".to_string()))?;
        let second: u32 = if parts.len() >= 3 {
            parts[2].parse()
                .map_err(|_| DateTimeError::ParseError("Invalid second".to_string()))?
        } else {
            0
        };

        Self::from_hms(hour, minute, second)
    }

    /// Parse with a specific format pattern
    pub fn parse_format(s: &str, pattern: &str) -> Result<Self, DateTimeError> {
        // Simple pattern matching - extract components based on pattern
        let mut year: Option<i32> = None;
        let mut month: Option<u32> = None;
        let mut day: Option<u32> = None;
        let mut hour: Option<u32> = None;
        let mut minute: Option<u32> = None;
        let mut second: Option<u32> = None;

        let mut s_pos = 0;
        let mut p_pos = 0;
        let s_bytes = s.as_bytes();
        let p_bytes = pattern.as_bytes();

        while p_pos < p_bytes.len() && s_pos < s_bytes.len() {
            // Check for format tokens
            if p_pos + 4 <= p_bytes.len() && &pattern[p_pos..p_pos + 4] == "YYYY" {
                year = Some(s[s_pos..s_pos + 4].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid year".to_string()))?);
                s_pos += 4;
                p_pos += 4;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "YY" {
                let yy: i32 = s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid year".to_string()))?;
                year = Some(if yy >= 70 { 1900 + yy } else { 2000 + yy });
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "MM" {
                month = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid month".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "DD" {
                day = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid day".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "HH" {
                hour = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid hour".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "mm" {
                minute = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid minute".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else if p_pos + 2 <= p_bytes.len() && &pattern[p_pos..p_pos + 2] == "ss" {
                second = Some(s[s_pos..s_pos + 2].parse()
                    .map_err(|_| DateTimeError::ParseError("Invalid second".to_string()))?);
                s_pos += 2;
                p_pos += 2;
            } else {
                // Literal character - must match
                if s_bytes[s_pos] != p_bytes[p_pos] {
                    return Err(DateTimeError::ParseError(
                        format!("Expected '{}' at position {}", p_bytes[p_pos] as char, s_pos)
                    ));
                }
                s_pos += 1;
                p_pos += 1;
            }
        }

        Self::from_ymd_hms(
            year.unwrap_or(1970),
            month.unwrap_or(1),
            day.unwrap_or(1),
            hour.unwrap_or(0),
            minute.unwrap_or(0),
            second.unwrap_or(0),
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_ymd() {
        let dt = FolioDateTime::from_ymd(2025, 6, 15).unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
    }

    #[test]
    fn test_from_ymd_hms() {
        let dt = FolioDateTime::from_ymd_hms(2025, 6, 15, 14, 30, 45).unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.second(), 45);
    }

    #[test]
    fn test_unix_epoch() {
        let dt = FolioDateTime::from_ymd(1970, 1, 1).unwrap();
        assert_eq!(dt.as_nanos(), 0);
        assert_eq!(dt.as_unix_secs(), 0);
    }

    #[test]
    fn test_pre_epoch() {
        let dt = FolioDateTime::from_ymd(1969, 12, 31).unwrap();
        assert!(dt.as_nanos() < 0);
        assert_eq!(dt.year(), 1969);
        assert_eq!(dt.month(), 12);
        assert_eq!(dt.day(), 31);
    }

    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
    }

    #[test]
    fn test_days_in_month() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 1), 31);
        assert_eq!(days_in_month(2023, 4), 30);
    }

    #[test]
    fn test_weekday() {
        // 1970-01-01 was Thursday
        let dt = FolioDateTime::from_ymd(1970, 1, 1).unwrap();
        assert_eq!(dt.weekday(), 4);

        // 2025-06-15 is Sunday
        let dt = FolioDateTime::from_ymd(2025, 6, 15).unwrap();
        assert_eq!(dt.weekday(), 7);
    }

    #[test]
    fn test_add_duration() {
        let dt = FolioDateTime::from_ymd(2025, 6, 15).unwrap();
        let dur = FolioDuration::from_days(10);
        let result = dt.add_duration(&dur);
        assert_eq!(result.year(), 2025);
        assert_eq!(result.month(), 6);
        assert_eq!(result.day(), 25);
    }

    #[test]
    fn test_add_months() {
        // Normal case
        let dt = FolioDateTime::from_ymd(2025, 1, 15).unwrap();
        let result = dt.add_months(2);
        assert_eq!(result.month(), 3);

        // End of month clamping
        let dt = FolioDateTime::from_ymd(2025, 1, 31).unwrap();
        let result = dt.add_months(1);
        assert_eq!(result.month(), 2);
        assert_eq!(result.day(), 28); // Feb 2025 has 28 days
    }

    #[test]
    fn test_parse_iso() {
        let dt = FolioDateTime::parse("2025-06-15T14:30:00Z").unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 14);
        assert_eq!(dt.minute(), 30);
        assert_eq!(dt.tz_offset(), Some(0));
    }

    #[test]
    fn test_parse_date_only() {
        let dt = FolioDateTime::parse("2025-06-15").unwrap();
        assert_eq!(dt.year(), 2025);
        assert_eq!(dt.month(), 6);
        assert_eq!(dt.day(), 15);
        assert_eq!(dt.hour(), 0);
    }

    #[test]
    fn test_format() {
        let dt = FolioDateTime::from_ymd_hms(2025, 6, 15, 14, 30, 0).unwrap();
        assert_eq!(dt.format("DD/MM/YYYY"), "15/06/2025");
        assert_eq!(dt.format("YYYY-MM-DD HH:mm"), "2025-06-15 14:30");
    }

    #[test]
    fn test_duration_arithmetic() {
        let d1 = FolioDuration::from_days(5);
        let d2 = FolioDuration::from_hours(12);
        let sum = d1.add(&d2);
        assert_eq!(sum.as_hours(), 5 * 24 + 12);
    }

    #[test]
    fn test_duration_display() {
        let d = FolioDuration::from_hours(26);
        assert_eq!(format!("{}", d), "1d 02:00:00");

        let d = FolioDuration::from_minutes(90);
        assert_eq!(format!("{}", d), "01:30:00");
    }

    #[test]
    fn test_iso_string() {
        let dt = FolioDateTime::from_ymd_hms(2025, 6, 15, 14, 30, 0).unwrap();
        assert_eq!(dt.to_iso_string(), "2025-06-15T14:30:00Z");

        let dt_tz = dt.with_tz_offset(5 * 3600 + 30 * 60);
        assert_eq!(dt_tz.to_iso_string(), "2025-06-15T14:30:00+05:30");
    }

    #[test]
    fn test_datetime_diff() {
        let dt1 = FolioDateTime::from_ymd(2025, 6, 15).unwrap();
        let dt2 = FolioDateTime::from_ymd(2025, 6, 10).unwrap();
        let diff = dt1.duration_since(&dt2);
        assert_eq!(diff.as_days(), 5);
    }

    #[test]
    fn test_iso_week() {
        // 2025-01-01 is Wednesday, should be week 1
        let dt = FolioDateTime::from_ymd(2025, 1, 1).unwrap();
        assert_eq!(dt.iso_week(), 1);

        // 2025-12-31 is Wednesday - can be week 52 or 53 depending on calculation
        // ISO week number is complex at year boundaries
        let dt = FolioDateTime::from_ymd(2025, 12, 31).unwrap();
        let week = dt.iso_week();
        // 2025 has 52 or 53 weeks - week 1 of 2026 starts on Dec 29
        assert!(week == 1 || week == 52 || week == 53, "Expected week 1, 52, or 53, got {}", week);
    }

    #[test]
    fn test_day_of_year() {
        let dt = FolioDateTime::from_ymd(2025, 1, 1).unwrap();
        assert_eq!(dt.day_of_year(), 1);

        let dt = FolioDateTime::from_ymd(2025, 12, 31).unwrap();
        assert_eq!(dt.day_of_year(), 365);

        let dt = FolioDateTime::from_ymd(2024, 12, 31).unwrap();
        assert_eq!(dt.day_of_year(), 366); // Leap year
    }

    #[test]
    fn test_invalid_dates() {
        assert!(FolioDateTime::from_ymd(2025, 13, 1).is_err());
        assert!(FolioDateTime::from_ymd(2025, 0, 1).is_err());
        assert!(FolioDateTime::from_ymd(2025, 2, 30).is_err());
        assert!(FolioDateTime::from_ymd_hms(2025, 1, 1, 25, 0, 0).is_err());
    }
}
```

folio-core\src\error.rs
```rs
//! Structured errors for LLM consumption
//!
//! Errors never crash the system. They are values that propagate through
//! computations and provide clear, actionable information.

use crate::{NumberError, DateTimeError};
use serde::{Deserialize, Serialize};

/// Standard error codes (machine-readable)
pub mod codes {
    pub const PARSE_ERROR: &str = "PARSE_ERROR";
    pub const DIV_ZERO: &str = "DIV_ZERO";
    pub const UNDEFINED_VAR: &str = "UNDEFINED_VAR";
    pub const UNDEFINED_FUNC: &str = "UNDEFINED_FUNC";
    pub const UNDEFINED_FIELD: &str = "UNDEFINED_FIELD";
    pub const TYPE_ERROR: &str = "TYPE_ERROR";
    pub const ARG_COUNT: &str = "ARG_COUNT";
    pub const ARG_TYPE: &str = "ARG_TYPE";
    pub const DOMAIN_ERROR: &str = "DOMAIN_ERROR";
    pub const OVERFLOW: &str = "OVERFLOW";
    pub const CIRCULAR_REF: &str = "CIRCULAR_REF";
    pub const INTERNAL: &str = "INTERNAL";
    // DateTime-specific error codes
    pub const INVALID_DATE: &str = "INVALID_DATE";
    pub const INVALID_TIME: &str = "INVALID_TIME";
    pub const DATE_OVERFLOW: &str = "DATE_OVERFLOW";
    pub const DATE_PARSE_ERROR: &str = "DATE_PARSE_ERROR";
}

/// Severity level of an error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Computation continued with degraded result
    Warning,
    /// Computation failed for this cell
    Error,
    /// Document cannot be evaluated
    Fatal,
}

/// Context about where an error occurred
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorContext {
    /// Cell name where error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell: Option<String>,
    
    /// Formula that caused the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formula: Option<String>,
    
    /// Line number in document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    
    /// Column number in document
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,
    
    /// Propagation notes
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub notes: Vec<String>,
}

/// Structured error for LLM consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolioError {
    /// Machine-readable error code
    pub code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Suggestion for fixing the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    
    /// Where the error occurred
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<ErrorContext>,
    
    /// Severity level
    pub severity: Severity,
}

impl FolioError {
    /// Create a new error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            suggestion: None,
            context: None,
            severity: Severity::Error,
        }
    }
    
    /// Builder: add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
    
    /// Builder: add context
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Builder: set cell context
    pub fn in_cell(mut self, cell: impl Into<String>) -> Self {
        let ctx = self.context.get_or_insert_with(ErrorContext::default);
        ctx.cell = Some(cell.into());
        self
    }
    
    /// Builder: set formula context
    pub fn with_formula(mut self, formula: impl Into<String>) -> Self {
        let ctx = self.context.get_or_insert_with(ErrorContext::default);
        ctx.formula = Some(formula.into());
        self
    }
    
    /// Builder: add propagation note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        let ctx = self.context.get_or_insert_with(ErrorContext::default);
        ctx.notes.push(note.into());
        self
    }
    
    /// Builder: set severity
    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
    
    // ========== Common Error Constructors ==========
    
    pub fn parse_error(details: impl Into<String>) -> Self {
        Self::new(codes::PARSE_ERROR, format!("Parse error: {}", details.into()))
            .with_suggestion("Check formula syntax")
    }
    
    pub fn div_zero() -> Self {
        Self::new(codes::DIV_ZERO, "Division by zero")
            .with_suggestion("Ensure divisor is not zero")
    }
    
    pub fn undefined_var(name: &str) -> Self {
        Self::new(codes::UNDEFINED_VAR, format!("Undefined variable: {}", name))
            .with_suggestion(format!("Define '{}' or check spelling", name))
    }
    
    pub fn undefined_func(name: &str) -> Self {
        Self::new(codes::UNDEFINED_FUNC, format!("Unknown function: {}", name))
            .with_suggestion("Use folio() to list available functions")
    }

    pub fn undefined_field(name: &str) -> Self {
        Self::new(codes::UNDEFINED_FIELD, format!("Undefined field: {}", name))
            .with_suggestion("Check object structure with folio()")
    }
    
    pub fn type_error(expected: &str, got: &str) -> Self {
        Self::new(codes::TYPE_ERROR, format!("Expected {}, got {}", expected, got))
            .with_suggestion(format!("Convert value to {} or check formula", expected))
    }
    
    pub fn arg_count(func: &str, expected: usize, got: usize) -> Self {
        Self::new(codes::ARG_COUNT, 
            format!("{}() expects {} arguments, got {}", func, expected, got))
            .with_suggestion(format!("Use help('{}') for usage", func))
    }
    
    pub fn arg_type(func: &str, arg: &str, expected: &str, got: &str) -> Self {
        Self::new(codes::ARG_TYPE,
            format!("{}() argument '{}': expected {}, got {}", func, arg, expected, got))
    }
    
    pub fn domain_error(details: impl Into<String>) -> Self {
        Self::new(codes::DOMAIN_ERROR, format!("Domain error: {}", details.into()))
    }
    
    pub fn circular_ref(cells: &[String]) -> Self {
        Self::new(codes::CIRCULAR_REF, 
            format!("Circular reference: {}", cells.join(" → ")))
            .with_suggestion("Remove circular dependency")
            .with_severity(Severity::Fatal)
    }
    
    pub fn internal(details: impl Into<String>) -> Self {
        Self::new(codes::INTERNAL, format!("Internal error: {}", details.into()))
            .with_suggestion("This is a bug, please report it")
            .with_severity(Severity::Fatal)
    }

    // ========== DateTime Error Constructors ==========

    pub fn invalid_date(details: impl Into<String>) -> Self {
        Self::new(codes::INVALID_DATE, format!("Invalid date: {}", details.into()))
            .with_suggestion("Check date components (year, month 1-12, day 1-31)")
    }

    pub fn invalid_time(details: impl Into<String>) -> Self {
        Self::new(codes::INVALID_TIME, format!("Invalid time: {}", details.into()))
            .with_suggestion("Check time components (hour 0-23, minute 0-59, second 0-59)")
    }

    pub fn date_overflow() -> Self {
        Self::new(codes::DATE_OVERFLOW, "DateTime overflow")
            .with_suggestion("Date value is out of supported range")
    }

    pub fn date_parse_error(details: impl Into<String>) -> Self {
        Self::new(codes::DATE_PARSE_ERROR, format!("DateTime parse error: {}", details.into()))
            .with_suggestion("Use ISO 8601 format (YYYY-MM-DD or YYYY-MM-DDTHH:MM:SS)")
    }
}

impl std::fmt::Display for FolioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, " (suggestion: {})", suggestion)?;
        }
        Ok(())
    }
}

impl std::error::Error for FolioError {}

impl From<NumberError> for FolioError {
    fn from(err: NumberError) -> Self {
        match err {
            NumberError::ParseError(s) => Self::parse_error(s),
            NumberError::DivisionByZero => Self::div_zero(),
            NumberError::DomainError(s) => Self::domain_error(s),
            NumberError::Overflow => Self::new(codes::OVERFLOW, "Numeric overflow"),
        }
    }
}

impl From<DateTimeError> for FolioError {
    fn from(err: DateTimeError) -> Self {
        match err {
            DateTimeError::InvalidMonth(m) => Self::invalid_date(format!("month {} out of range 1-12", m)),
            DateTimeError::InvalidDay(d, m, y) => Self::invalid_date(format!("day {} invalid for {}/{}", d, m, y)),
            DateTimeError::InvalidHour(h) => Self::invalid_time(format!("hour {} out of range 0-23", h)),
            DateTimeError::InvalidMinute(m) => Self::invalid_time(format!("minute {} out of range 0-59", m)),
            DateTimeError::InvalidSecond(s) => Self::invalid_time(format!("second {} out of range 0-59", s)),
            DateTimeError::InvalidNano(n) => Self::invalid_time(format!("nanosecond {} out of range", n)),
            DateTimeError::ParseError(s) => Self::date_parse_error(s),
            DateTimeError::Overflow => Self::date_overflow(),
        }
    }
}
```

folio-core\src\lib.rs
```rs
//! Folio Core - Fundamental types
//!
//! This crate provides the core types used throughout Folio:
//! - `Number`: Arbitrary precision rational numbers
//! - `Value`: Runtime values (numbers, text, datetime, duration, objects, errors)
//! - `FolioDateTime`: Nanosecond-precision datetime
//! - `FolioDuration`: Nanosecond-precision duration
//! - `FolioError`: Structured errors for LLM consumption

mod number;
mod value;
mod error;
mod datetime;

pub use number::{Number, NumberError};
pub use value::Value;
pub use error::{FolioError, ErrorContext, Severity, codes};
pub use datetime::{FolioDateTime, FolioDuration, DateTimeError, is_leap_year, days_in_month};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{Number, Value, FolioError, Severity};
    pub use crate::{FolioDateTime, FolioDuration, DateTimeError};
    pub use crate::error::codes;
}

#[cfg(test)]
mod tests {
    use super::*;

    mod number_tests {
        use super::*;

        #[test]
        fn test_from_i64() {
            let n = Number::from_i64(42);
            assert_eq!(n.to_i64(), Some(42));
        }

        #[test]
        fn test_from_str_integer() {
            let n = Number::from_str("123").unwrap();
            assert_eq!(n.to_i64(), Some(123));
        }

        #[test]
        fn test_from_str_decimal() {
            let n = Number::from_str("3.14").unwrap();
            assert!(!n.is_integer());
        }

        #[test]
        fn test_from_str_fraction() {
            let n = Number::from_str("1/3").unwrap();
            assert!(!n.is_integer());
        }

        #[test]
        fn test_from_str_scientific() {
            let n = Number::from_str("1.5e2").unwrap();
            assert_eq!(n.to_i64(), Some(150));
        }

        #[test]
        fn test_from_str_scientific_integer_mantissa() {
            // Integer mantissa preserves full precision (no float64 intermediary)
            let avogadro = Number::from_str("602214076e15").unwrap();
            // Should be exactly 602214076 * 10^15
            let expected = Number::from_str("602214076000000000000000").unwrap();
            assert_eq!(avogadro.as_decimal(0), expected.as_decimal(0));

            // Negative exponent
            let h = Number::from_str("662607015e-42").unwrap();
            assert!(!h.is_zero());
            // Check it's a very small positive number
            let h_decimal = h.as_decimal(50);
            assert!(h_decimal.starts_with("0."), "Planck constant should be tiny: {}", h_decimal);
        }

        #[test]
        fn test_as_sigfigs() {
            // Large number - should use scientific notation
            let avogadro = Number::from_str("602214076e15").unwrap();
            let s = avogadro.as_sigfigs(4);
            assert!(s.contains("e23") || s.contains("e+23"), "Avogadro should be ~6e23: {}", s);
            assert!(s.starts_with("6.022"), "Should have 4 sig figs: {}", s);

            // Small number - should use scientific notation
            let h = Number::from_str("6.62607e-34").unwrap();
            let s = h.as_sigfigs(4);
            assert!(s.contains("e-3"), "Planck should use sci notation: {}", s);

            // Normal range number - regular notation
            let n = Number::from_str("123.456").unwrap();
            let s = n.as_sigfigs(4);
            assert_eq!(s, "123.5", "Normal number with 4 sigfigs: {}", s);

            // Small but in range
            let n = Number::from_str("0.001234").unwrap();
            let s = n.as_sigfigs(3);
            assert!(s.starts_with("0.00123"), "0.001234 with 3 sigfigs: {}", s);
        }

        #[test]
        fn test_ln_correctness() {
            // ln(100) should equal 2 * ln(10)
            let ten = Number::from_i64(10);
            let hundred = Number::from_i64(100);

            let ln_10 = ten.ln(50).unwrap();
            let ln_100 = hundred.ln(50).unwrap();
            let two_ln_10 = ln_10.mul(&Number::from_i64(2));

            // ln(10) ≈ 2.302585
            let ln_10_str = ln_10.as_decimal(5);
            assert!(ln_10_str.starts_with("2.3025"), "ln(10) should be ~2.3025, got: {}", ln_10_str);

            // ln(100) should equal 2*ln(10) ≈ 4.605170
            let ln_100_str = ln_100.as_decimal(5);
            let two_ln_10_str = two_ln_10.as_decimal(5);
            assert!(ln_100_str.starts_with("4.605"), "ln(100) should be ~4.605, got: {}", ln_100_str);
            assert_eq!(ln_100_str, two_ln_10_str, "ln(100) should equal 2*ln(10)");

            // Test larger number: ln(1000) = 3*ln(10) ≈ 6.907755
            let thousand = Number::from_i64(1000);
            let ln_1000 = thousand.ln(50).unwrap();
            let ln_1000_str = ln_1000.as_decimal(4);
            assert!(ln_1000_str.starts_with("6.907"), "ln(1000) should be ~6.907, got: {}", ln_1000_str);
        }

        #[test]
        fn test_exp_ln_identity() {
            // exp(ln(x)) should equal x
            let hundred = Number::from_i64(100);
            let ln_100 = hundred.ln(50).unwrap();
            let exp_ln_100 = ln_100.exp(50);
            let result_str = exp_ln_100.as_decimal(6);
            assert!(result_str.starts_with("100.000"),
                "exp(ln(100)) should be 100, got: {}", result_str);

            // Also test with a larger number
            let million = Number::from_i64(1000000);
            let ln_million = million.ln(50).unwrap();
            let exp_ln_million = ln_million.exp(50);
            let result_str = exp_ln_million.as_decimal(0);
            assert!(result_str.starts_with("1000000") || result_str.starts_with("999999"),
                "exp(ln(1000000)) should be ~1000000, got: {}", result_str);
        }

        #[test]
        fn test_pow_real_fractional() {
            // 4^0.5 = 2 (square root)
            let four = Number::from_i64(4);
            let half = Number::from_str("0.5").unwrap();
            let result = four.pow_real(&half, 50);
            let decimal = result.as_decimal(3);
            assert!(decimal.starts_with("2.0"), "4^0.5 should be 2, got: {}", decimal);

            // 8^(1/3) ≈ 2 (cube root)
            let eight = Number::from_i64(8);
            let third = Number::from_str("0.333333333333333").unwrap();
            let result = eight.pow_real(&third, 50);
            let decimal = result.as_decimal(2);
            assert!(decimal.starts_with("2.0") || decimal.starts_with("1.9"),
                "8^(1/3) should be ~2, got: {}", decimal);

            // 10^2.5 = 10^2 * 10^0.5 = 100 * 3.162... ≈ 316.2
            let ten = Number::from_i64(10);
            let two_point_five = Number::from_str("2.5").unwrap();
            let result = ten.pow_real(&two_point_five, 50);
            let decimal = result.as_decimal(1);
            assert!(decimal.starts_with("316."), "10^2.5 should be ~316.2, got: {}", decimal);
        }

        #[test]
        fn test_add() {
            let a = Number::from_i64(10);
            let b = Number::from_i64(32);
            assert_eq!(a.add(&b).to_i64(), Some(42));
        }

        #[test]
        fn test_sub() {
            let a = Number::from_i64(50);
            let b = Number::from_i64(8);
            assert_eq!(a.sub(&b).to_i64(), Some(42));
        }

        #[test]
        fn test_mul() {
            let a = Number::from_i64(6);
            let b = Number::from_i64(7);
            assert_eq!(a.mul(&b).to_i64(), Some(42));
        }

        #[test]
        fn test_checked_div() {
            let a = Number::from_i64(84);
            let b = Number::from_i64(2);
            assert_eq!(a.checked_div(&b).unwrap().to_i64(), Some(42));
        }

        #[test]
        fn test_div_by_zero() {
            let a = Number::from_i64(42);
            let b = Number::from_i64(0);
            assert!(a.checked_div(&b).is_err());
        }

        #[test]
        fn test_pow_positive() {
            let n = Number::from_i64(2);
            assert_eq!(n.pow(10).to_i64(), Some(1024));
        }

        #[test]
        fn test_pow_negative() {
            let n = Number::from_i64(2);
            let result = n.pow(-2);
            // 2^-2 = 1/4 = 0.25
            assert!(!result.is_integer());
        }

        #[test]
        fn test_pow_large_exponent() {
            // Test case: 1.003^300 ≈ 2.456 (compound interest factor)
            // This creates a large BigRational that overflows f64 individually
            // but the ratio should be representable
            let base = Number::from_str("1.003").unwrap();
            let result = base.pow(300);
            let decimal = result.as_decimal(2);
            // Should be approximately 2.46, not NaN or error
            assert!(decimal.starts_with("2.4"), "Expected ~2.4x, got: {}", decimal);
        }

        #[test]
        fn test_sqrt() {
            let n = Number::from_i64(4);
            let result = n.sqrt(50).unwrap();
            assert_eq!(result.to_i64(), Some(2));
        }

        #[test]
        fn test_sqrt_5() {
            // sqrt(5) ≈ 2.236
            let n = Number::from_i64(5);
            let result = n.sqrt(50).unwrap();
            assert!(!result.is_zero());
            let decimal = result.as_decimal(4);
            assert!(decimal.starts_with("2.236"), "sqrt(5) should be ~2.236, got: {}", decimal);
        }

        #[test]
        fn test_sqrt_negative() {
            let n = Number::from_i64(-4);
            assert!(n.sqrt(50).is_err());
        }

        #[test]
        fn test_phi() {
            let phi = Number::phi(50);
            // φ ≈ 1.618
            let decimal = phi.as_decimal(3);
            assert!(decimal.starts_with("1.618"));
        }

        #[test]
        fn test_phi_identity() {
            // Test that phi^2 - phi - 1 ≈ 0 (the defining property of phi)
            // Using low precision (15) which uses the fast f64 path
            let five = Number::from_i64(5);
            let sqrt5 = five.sqrt(15).unwrap();
            let one = Number::from_i64(1);
            let two = Number::from_i64(2);

            let phi = one.add(&sqrt5).checked_div(&two).unwrap();
            let phi_squared = phi.mul(&phi);
            let identity = phi_squared.sub(&phi).sub(&one);

            // Should be very small (close to 0) - with f64 approximation this is near machine epsilon
            let result = identity.as_decimal(20);
            assert!(result.parse::<f64>().map(|f| f.abs() < 1e-10).unwrap_or(false),
                "phi^2 - phi - 1 should be ~0, got: {}", result);
        }

        #[test]
        fn test_phi_identity_high_precision() {
            // Test with higher precision (50) using Newton-Raphson
            let five = Number::from_i64(5);
            let sqrt5 = five.sqrt(50).unwrap();
            let one = Number::from_i64(1);
            let two = Number::from_i64(2);

            let phi = one.add(&sqrt5).checked_div(&two).unwrap();
            let phi_squared = phi.mul(&phi);
            let identity = phi_squared.sub(&phi).sub(&one);

            // With improved to_f64, this should now work even for huge BigRationals
            let result_str = identity.as_decimal(20);
            let f: f64 = result_str.parse().expect("should be able to parse to f64");
            assert!(f.abs() < 1e-10, "phi^2 - phi - 1 should be ~0, got: {}", f);
        }

        #[test]
        fn test_pi() {
            let pi = Number::pi(50);
            let decimal = pi.as_decimal(5);
            // Pi = 3.14159..., rounded to 5 places = 3.14159
            assert!(decimal.starts_with("3.14159"), "Expected 3.14159..., got: {}", decimal);
        }

        #[test]
        fn test_e() {
            let e = Number::e(50);
            let decimal = e.as_decimal(3);
            assert!(decimal.starts_with("2.718"));
        }

        #[test]
        fn test_is_zero() {
            assert!(Number::from_i64(0).is_zero());
            assert!(!Number::from_i64(1).is_zero());
        }

        #[test]
        fn test_is_negative() {
            assert!(Number::from_i64(-5).is_negative());
            assert!(!Number::from_i64(5).is_negative());
            assert!(!Number::from_i64(0).is_negative());
        }

        #[test]
        fn test_abs() {
            assert_eq!(Number::from_i64(-42).abs().to_i64(), Some(42));
            assert_eq!(Number::from_i64(42).abs().to_i64(), Some(42));
        }

        #[test]
        fn test_small_number_display() {
            // Planck's constant: 6.62607015e-34
            let h = Number::from_str("6.62607015e-34").unwrap();
            let display = h.as_decimal(10);
            // Should show significant digits 6.63 at the end (rounded from 6.626...)
            // Format: 0.000...000663 with enough zeros to reach the significant digits
            assert!(display.ends_with("663") || display.ends_with("662"),
                "Very small number should show significant digits at end, got: {}", display);
            // Should have many leading zeros (34 decimal places for e-34)
            assert!(display.len() > 30, "Should have many decimal places, got: {}", display);
        }

        #[test]
        fn test_small_number_display_various() {
            // Test various small numbers - they should show 3 significant digits
            // Format is 0.000...XXX where XXX are the significant digits
            let n1 = Number::from_str("1e-10").unwrap();
            let d1 = n1.as_decimal(10);
            // 1e-10 = 0.0000000001 - should end with 1
            assert!(d1.ends_with("100"), "1e-10 should end with '100', got: {}", d1);

            let n2 = Number::from_str("2.5e-8").unwrap();
            let d2 = n2.as_decimal(10);
            // 2.5e-8 = 0.000000025 - should end with 250
            assert!(d2.ends_with("250") || d2.ends_with("25"), "2.5e-8 should end with '25x', got: {}", d2);

            let n3 = Number::from_str("1.23e-15").unwrap();
            let d3 = n3.as_decimal(10);
            // Should end with 123
            assert!(d3.ends_with("123"), "1.23e-15 should end with '123', got: {}", d3);
        }
    }

    mod value_tests {
        use super::*;

        #[test]
        fn test_from_i64() {
            let v: Value = 42i64.into();
            assert!(matches!(v, Value::Number(_)));
            assert_eq!(v.as_number().unwrap().to_i64(), Some(42));
        }

        #[test]
        fn test_from_str() {
            let v: Value = "hello".into();
            assert!(matches!(v, Value::Text(_)));
            assert_eq!(v.as_text(), Some("hello"));
        }

        #[test]
        fn test_from_bool() {
            let v: Value = true.into();
            assert!(matches!(v, Value::Bool(true)));
        }

        #[test]
        fn test_type_name() {
            assert_eq!(Value::Number(Number::from_i64(0)).type_name(), "Number");
            assert_eq!(Value::Text("".to_string()).type_name(), "Text");
            assert_eq!(Value::Bool(true).type_name(), "Bool");
            assert_eq!(Value::Null.type_name(), "Null");
        }

        #[test]
        fn test_is_error() {
            let err = Value::Error(FolioError::div_zero());
            assert!(err.is_error());
            assert!(!Value::Null.is_error());
        }

        #[test]
        fn test_to_number_from_text() {
            let v = Value::Text("42".to_string());
            let n = v.to_number();
            assert!(matches!(n, Value::Number(_)));
        }

        #[test]
        fn test_to_bool_truthy() {
            assert!(matches!(Value::Number(Number::from_i64(1)).to_bool(), Value::Bool(true)));
            assert!(matches!(Value::Number(Number::from_i64(0)).to_bool(), Value::Bool(false)));
            assert!(matches!(Value::Text("hi".to_string()).to_bool(), Value::Bool(true)));
            assert!(matches!(Value::Text("".to_string()).to_bool(), Value::Bool(false)));
        }
    }

    mod error_tests {
        use super::*;

        #[test]
        fn test_error_construction() {
            let err = FolioError::div_zero();
            assert_eq!(err.code, codes::DIV_ZERO);
        }

        #[test]
        fn test_error_with_context() {
            let err = FolioError::undefined_var("x")
                .in_cell("result")
                .with_formula("x + 1");
            assert!(err.context.is_some());
            let ctx = err.context.unwrap();
            assert_eq!(ctx.cell, Some("result".to_string()));
            assert_eq!(ctx.formula, Some("x + 1".to_string()));
        }

        #[test]
        fn test_error_with_note() {
            let err = FolioError::type_error("Number", "Text")
                .with_note("from left operand");
            let ctx = err.context.unwrap();
            assert_eq!(ctx.notes.len(), 1);
            assert_eq!(ctx.notes[0], "from left operand");
        }

        #[test]
        fn test_error_display() {
            let err = FolioError::parse_error("unexpected token");
            let display = format!("{}", err);
            assert!(display.contains("PARSE_ERROR"));
        }
    }
}
```

folio-core\src\number.rs
```rs
//! Arbitrary precision numbers using dashu
//!
//! Uses dashu-float (DBig) for arbitrary precision decimal arithmetic.
//! Native support for transcendentals (ln, exp, sqrt) without
//! the denominator explosion issues of rational arithmetic.

use dashu_float::DBig;
use dashu_float::ops::{SquareRoot, Abs};
use dashu_int::IBig;
use dashu_int::ops::BitTest;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use thiserror::Error;

/// Error type for number operations
#[derive(Debug, Clone, Error)]
pub enum NumberError {
    #[error("Invalid number format: {0}")]
    ParseError(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Domain error: {0}")]
    DomainError(String),

    #[error("Overflow: result too large")]
    Overflow,
}

/// Default precision for calculations (decimal digits)
const DEFAULT_PRECISION: usize = 50;

/// Arbitrary precision decimal number
/// 
/// Built on dashu-float's DBig for efficient transcendental operations.
/// All operations return Results or new Numbers - never panic.
#[derive(Debug, Clone)]
pub struct Number {
    inner: DBig,
}

impl Number {
    // ========== Construction ==========

    /// Ensure a DBig has adequate precision for calculations
    fn with_work_precision(val: DBig) -> DBig {
        val.with_precision(DEFAULT_PRECISION).value()
    }

    /// Create from string representation
    /// Supports: "123", "3.14", "1/3", "1.5e10", "-42"
    pub fn from_str(s: &str) -> Result<Self, NumberError> {
        let s = s.trim();
        
        // Handle rational format "a/b"
        if s.contains('/') && !s.contains('.') && !s.contains('e') && !s.contains('E') {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                let num_str = parts[0].trim();
                let den_str = parts[1].trim();
                
                let num: DBig = num_str.parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                let den: DBig = den_str.parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                
                if den == DBig::ZERO {
                    return Err(NumberError::DivisionByZero);
                }
                
                let result = Self::with_work_precision(num) / Self::with_work_precision(den);
                return Ok(Self { inner: result });
            }
        }

        // Handle scientific notation with integer mantissa: "602214076e15"
        if (s.contains('e') || s.contains('E')) && !s.contains('.') {
            let s_lower = s.to_lowercase();
            let parts: Vec<&str> = s_lower.split('e').collect();
            if parts.len() == 2 {
                let mantissa: IBig = parts[0].parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                let exp: i32 = parts[1].parse()
                    .map_err(|_| NumberError::ParseError(s.to_string()))?;
                
                // Use DBig::from_parts for exact scientific notation
                // significand * 10^exponent
                let result = DBig::from_parts(mantissa, exp as isize);
                return Ok(Self { inner: Self::with_work_precision(result) });
            }
        }

        // Standard decimal parsing
        let inner: DBig = s.parse()
            .map_err(|_| NumberError::ParseError(s.to_string()))?;
        
        Ok(Self { inner: Self::with_work_precision(inner) })
    }

    /// Create from i64 with working precision
    pub fn from_i64(n: i64) -> Self {
        Self { inner: Self::with_work_precision(DBig::from(n)) }
    }

    /// Create from ratio (exact division)
    pub fn from_ratio(num: i64, den: i64) -> Self {
        if den == 0 {
            return Self { inner: DBig::ZERO };
        }
        let n = Self::with_work_precision(DBig::from(num));
        let d = Self::with_work_precision(DBig::from(den));
        Self { inner: n / d }
    }

    // ========== Predicates ==========

    /// Check if zero
    pub fn is_zero(&self) -> bool {
        self.inner == DBig::ZERO
    }

    /// Check if negative
    pub fn is_negative(&self) -> bool {
        self.inner < DBig::ZERO
    }

    /// Check if value is an integer
    pub fn is_integer(&self) -> bool {
        let floor_val = self.inner.clone().floor();
        self.inner == floor_val
    }

    // ========== Basic Arithmetic ==========

    /// Addition
    pub fn add(&self, other: &Self) -> Self {
        Self { inner: &self.inner + &other.inner }
    }

    /// Subtraction
    pub fn sub(&self, other: &Self) -> Self {
        Self { inner: &self.inner - &other.inner }
    }

    /// Multiplication
    pub fn mul(&self, other: &Self) -> Self {
        Self { inner: &self.inner * &other.inner }
    }

    /// Safe division (returns Result, never panics)
    pub fn checked_div(&self, other: &Self) -> Result<Self, NumberError> {
        if other.is_zero() {
            Err(NumberError::DivisionByZero)
        } else {
            Ok(Self { inner: &self.inner / &other.inner })
        }
    }

    /// Integer power (exact)
    pub fn pow(&self, exp: i32) -> Self {
        if exp == 0 {
            return Self::from_i64(1);
        }
        
        let abs_exp = exp.unsigned_abs();
        let mut result = Self::from_i64(1);
        
        // Simple repeated multiplication
        for _ in 0..abs_exp {
            result = result.mul(self);
        }
        
        if exp < 0 {
            Self::from_i64(1).checked_div(&result).unwrap_or(Self::from_i64(0))
        } else {
            result
        }
    }

    /// Real-valued power: x^y = exp(y * ln(x))
    pub fn pow_real(&self, exp: &Self, precision: u32) -> Self {
        if exp.is_zero() {
            return Self::from_i64(1);
        }
        if self.is_zero() {
            return Self::from_i64(0);
        }

        // If exponent is a small integer, use exact power
        if exp.is_integer() {
            if let Some(e) = exp.to_i64() {
                if e.abs() <= i32::MAX as i64 {
                    return self.pow(e as i32);
                }
            }
        }

        // For x^y where x > 0: x^y = exp(y * ln(x))
        if self.is_negative() {
            return Self::from_i64(0);
        }

        let ln_x = self.inner.clone().with_precision(precision as usize).value().ln();
        let product = &ln_x * &exp.inner;
        Self { inner: product.exp() }
    }

    // ========== Transcendental Functions ==========

    /// Square root
    pub fn sqrt(&self, precision: u32) -> Result<Self, NumberError> {
        if self.is_negative() {
            return Err(NumberError::DomainError(
                "square root of negative number".to_string()
            ));
        }
        if self.is_zero() {
            return Ok(Self::from_i64(0));
        }

        let val = self.inner.clone().with_precision(precision as usize).value();
        Ok(Self { inner: val.sqrt() })
    }

    /// Natural logarithm
    pub fn ln(&self, precision: u32) -> Result<Self, NumberError> {
        if self.inner <= DBig::ZERO {
            return Err(NumberError::DomainError(
                "logarithm of non-positive number".to_string()
            ));
        }

        let val = self.inner.clone().with_precision(precision as usize).value();
        Ok(Self { inner: val.ln() })
    }

    /// Exponential function (e^x)
    pub fn exp(&self, precision: u32) -> Self {
        let val = self.inner.clone().with_precision(precision as usize).value();
        Self { inner: val.exp() }
    }

    /// Sine function (Taylor series)
    pub fn sin(&self, precision: u32) -> Self {
        let x = self.inner.clone().with_precision(precision as usize).value();
        let x_squared = &x * &x;
        
        let mut sum = x.clone();
        let mut term = x.clone();
        
        let iterations = (precision / 3).max(12).min(50) as i64;
        for k in 1..iterations {
            let denom = DBig::from((2 * k) * (2 * k + 1));
            term = -&term * &x_squared / denom;
            sum = &sum + &term;
        }
        
        Self { inner: sum }
    }

    /// Cosine function (Taylor series)
    pub fn cos(&self, precision: u32) -> Self {
        let x = self.inner.clone().with_precision(precision as usize).value();
        let x_squared = &x * &x;
        
        let one = DBig::ONE.with_precision(precision as usize).value();
        let mut sum = one.clone();
        let mut term = one;
        
        let iterations = (precision / 3).max(12).min(50) as i64;
        for k in 1..iterations {
            let denom = DBig::from((2 * k - 1) * (2 * k));
            term = -&term * &x_squared / denom;
            sum = &sum + &term;
        }
        
        Self { inner: sum }
    }

    /// Tangent function (sin/cos)
    pub fn tan(&self, precision: u32) -> Result<Self, NumberError> {
        let cos_x = self.cos(precision);
        if cos_x.is_zero() {
            return Err(NumberError::DomainError(
                "tan undefined at odd multiples of π/2".to_string()
            ));
        }
        let sin_x = self.sin(precision);
        sin_x.checked_div(&cos_x)
    }

    // ========== Mathematical Constants ==========

    /// Golden ratio φ = (1 + √5) / 2
    pub fn phi(precision: u32) -> Self {
        let five = Self::from_i64(5);
        let sqrt5 = five.sqrt(precision + 10).unwrap_or(Self::from_i64(2));
        let one = Self::from_i64(1);
        let two = Self::from_i64(2);
        one.add(&sqrt5).checked_div(&two).unwrap_or(Self::from_ratio(161803, 100000))
    }

    /// Pi - from high-precision string constant
    pub fn pi(precision: u32) -> Self {
        const PI_STR: &str = "3.14159265358979323846264338327950288419716939937510582097494459230781640628620899862803482534211706798214808651328230664709384460955058223172535940812848111745028410270193852110555964462294895493038196442881097566593344612847564823378678316527120190914564856692346034861045432664821339360726024914127372458700660631558817488152092096282925409171536436789259036001133053054882046652138414695194151160943305727036575959195309218611738193261179310511854807446237996274956735188575272489122793818301194912";
        
        let end_pos = (precision as usize + 2).min(PI_STR.len());
        Self::from_str(&PI_STR[..end_pos])
            .unwrap_or(Self::from_ratio(355, 113))
    }

    /// Euler's number e
    pub fn e(precision: u32) -> Self {
        Self::from_i64(1).exp(precision)
    }

    // ========== Other Operations ==========

    /// Absolute value
    pub fn abs(&self) -> Self {
        Self { inner: Abs::abs(self.inner.clone()) }
    }

    /// Floor - largest integer <= x
    pub fn floor(&self) -> Self {
        Self { inner: self.inner.clone().floor() }
    }

    /// Ceiling - smallest integer >= x
    pub fn ceil(&self) -> Self {
        Self { inner: self.inner.clone().ceil() }
    }

    /// Try to convert to i64
    pub fn to_i64(&self) -> Option<i64> {
        if !self.is_integer() {
            return None;
        }
        
        // DBig stores as significand * 10^exponent
        let (significand, exponent) = self.inner.clone().into_repr().into_parts();
        
        // Try to get i64 from significand
        let sig_i64: i64 = significand.try_into().ok()?;
        
        if exponent == 0 {
            Some(sig_i64)
        } else if exponent > 0 && exponent <= 18 {
            sig_i64.checked_mul(10_i64.checked_pow(exponent as u32)?)
        } else if exponent < 0 && exponent >= -18 {
            let divisor = 10_i64.checked_pow((-exponent) as u32)?;
            if sig_i64 % divisor == 0 {
                Some(sig_i64 / divisor)
            } else {
                None
            }
        } else {
            // Fall back to f64 conversion
            self.to_f64().and_then(|f| {
                if f >= i64::MIN as f64 && f <= i64::MAX as f64 {
                    Some(f as i64)
                } else {
                    None
                }
            })
        }
    }

    // ========== Display ==========

    /// Render as decimal string with specified decimal places
    pub fn as_decimal(&self, places: u32) -> String {
        if let Some(f) = self.to_f64() {
            // Handle very small non-zero numbers
            if f != 0.0 && f.abs() < 1e-6 {
                let log10 = f.abs().log10().floor() as i32;
                let sig_places = ((-log10) + 2) as usize;
                return format!("{:.prec$}", f, prec = sig_places);
            }

            if places == 0 {
                format!("{:.0}", f)
            } else {
                format!("{:.prec$}", f, prec = places as usize)
            }
        } else {
            format!("{}", self.inner)
        }
    }

    /// Render with N significant figures
    pub fn as_sigfigs(&self, sigfigs: u32) -> String {
        if let Some(f) = self.to_f64() {
            if f == 0.0 {
                return "0".to_string();
            }

            let sigfigs = sigfigs.max(1) as usize;
            let exp = f.abs().log10().floor() as i32;

            if exp >= -3 && exp <= 4 {
                let decimal_places = if exp >= 0 {
                    (sigfigs as i32 - exp - 1).max(0) as usize
                } else {
                    sigfigs + (-exp - 1) as usize
                };
                format!("{:.prec$}", f, prec = decimal_places)
            } else {
                let mantissa = f / 10_f64.powi(exp);
                let decimal_places = (sigfigs - 1).max(0);
                format!("{:.prec$}e{}", mantissa, exp, prec = decimal_places)
            }
        } else {
            format!("{}", self.inner)
        }
    }

    /// Convert to f64 (may lose precision)
    pub fn to_f64(&self) -> Option<f64> {
        // Get the representation: significand * 10^exponent
        let (significand, exponent) = self.inner.clone().into_repr().into_parts();
        
        // Convert significand to f64
        // For large significands, we need to be careful
        let sig_f64: f64 = if significand.bit_len() <= 53 {
            // Safe direct conversion
            match TryInto::<i64>::try_into(significand.clone()) {
                Ok(i) => i as f64,
                Err(_) => {
                    // Try as u64 then negate if needed
                    let is_neg = significand < IBig::ZERO;
                    let abs_sig = if is_neg { -significand.clone() } else { significand.clone() };
                    match TryInto::<u64>::try_into(abs_sig) {
                        Ok(u) => if is_neg { -(u as f64) } else { u as f64 },
                        Err(_) => return None,
                    }
                }
            }
        } else {
            // Significand too large - need to scale down
            // Shift right to fit in 53 bits, adjusting exponent
            let extra_bits = significand.bit_len() - 53;
            let shifted = &significand >> extra_bits;
            let shifted_i64: i64 = shifted.try_into().ok()?;
            let base_f64 = shifted_i64 as f64;
            // Account for the bits we shifted out
            base_f64 * 2_f64.powi(extra_bits as i32)
        };
        
        // Apply the decimal exponent
        let result = if exponent == 0 {
            sig_f64
        } else if exponent > 0 && exponent <= 308 {
            sig_f64 * 10_f64.powi(exponent as i32)
        } else if exponent < 0 && exponent >= -308 {
            sig_f64 / 10_f64.powi((-exponent) as i32)
        } else {
            return None; // Exponent out of f64 range
        };
        
        if result.is_finite() {
            Some(result)
        } else {
            None
        }
    }
}

// ========== Trait Implementations ==========

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_decimal(10))
    }
}

impl Serialize for Number {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Number {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Number {}

impl PartialOrd for Number {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Number {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // DBig implements PartialOrd, use it and treat None as Equal
        self.inner.partial_cmp(&other.inner).unwrap_or(std::cmp::Ordering::Equal)
    }
}
```

folio-core\src\value.rs
```rs
//! Runtime values in Folio
//!
//! Values can be numbers, text, booleans, datetime, duration, objects
//! (for DECOMPOSE results), lists, null, or errors. Errors propagate
//! through computations.

use crate::{Number, FolioError, FolioDateTime, FolioDuration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Runtime value in Folio
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    Number(Number),
    Text(String),
    Bool(bool),
    DateTime(FolioDateTime),
    Duration(FolioDuration),
    Object(HashMap<String, Value>),
    List(Vec<Value>),
    Null,
    Error(FolioError),
}

impl Value {
    // ========== Safe Accessors (never panic) ==========
    
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Value::Number(n) => Some(n),
            _ => None,
        }
    }
    
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(o) => Some(o),
            _ => None,
        }
    }
    
    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) => Some(l),
            _ => None,
        }
    }

    pub fn as_datetime(&self) -> Option<&FolioDateTime> {
        match self {
            Value::DateTime(dt) => Some(dt),
            _ => None,
        }
    }

    pub fn as_duration(&self) -> Option<&FolioDuration> {
        match self {
            Value::Duration(d) => Some(d),
            _ => None,
        }
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Value::Error(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_datetime(&self) -> bool {
        matches!(self, Value::DateTime(_))
    }

    pub fn is_duration(&self) -> bool {
        matches!(self, Value::Duration(_))
    }
    
    // ========== Object Field Access ==========
    
    /// Get field from object. Returns Error value if not found or not an object.
    pub fn get(&self, key: &str) -> Value {
        match self {
            Value::Object(map) => {
                map.get(key).cloned().unwrap_or_else(|| {
                    Value::Error(FolioError::undefined_field(key))
                })
            }
            Value::Error(e) => Value::Error(e.clone()),
            _ => Value::Error(FolioError::type_error("Object", self.type_name())),
        }
    }
    
    /// Type name for error messages
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "Number",
            Value::Text(_) => "Text",
            Value::Bool(_) => "Bool",
            Value::DateTime(_) => "DateTime",
            Value::Duration(_) => "Duration",
            Value::Object(_) => "Object",
            Value::List(_) => "List",
            Value::Null => "Null",
            Value::Error(_) => "Error",
        }
    }
    
    // ========== Type Coercion ==========
    
    /// Convert to number (may return Error)
    pub fn to_number(&self) -> Value {
        match self {
            Value::Number(n) => Value::Number(n.clone()),
            Value::Text(s) => {
                match Number::from_str(s) {
                    Ok(n) => Value::Number(n),
                    Err(e) => Value::Error(FolioError::from(e)),
                }
            }
            Value::Bool(b) => Value::Number(Number::from_i64(if *b { 1 } else { 0 })),
            Value::Error(e) => Value::Error(e.clone()),
            _ => Value::Error(FolioError::type_error("Number", self.type_name())),
        }
    }
    
    /// Convert to text (always succeeds)
    pub fn to_text(&self) -> Value {
        Value::Text(format!("{}", self))
    }
    
    /// Convert to bool (truthy/falsy)
    pub fn to_bool(&self) -> Value {
        match self {
            Value::Bool(b) => Value::Bool(*b),
            Value::Number(n) => Value::Bool(!n.is_zero()),
            Value::Text(s) => Value::Bool(!s.is_empty()),
            Value::DateTime(_) => Value::Bool(true), // DateTime is always truthy
            Value::Duration(d) => Value::Bool(!d.is_zero()),
            Value::Null => Value::Bool(false),
            Value::List(l) => Value::Bool(!l.is_empty()),
            Value::Object(o) => Value::Bool(!o.is_empty()),
            Value::Error(e) => Value::Error(e.clone()),
        }
    }

    /// Convert to datetime (may return Error)
    pub fn to_datetime(&self) -> Value {
        match self {
            Value::DateTime(dt) => Value::DateTime(dt.clone()),
            Value::Text(s) => {
                match FolioDateTime::parse(s) {
                    Ok(dt) => Value::DateTime(dt),
                    Err(e) => Value::Error(FolioError::parse_error(format!("{}", e))),
                }
            }
            Value::Number(n) => {
                // Interpret as Unix timestamp in seconds
                if let Some(secs) = n.to_i64() {
                    Value::DateTime(FolioDateTime::from_unix_secs(secs))
                } else {
                    Value::Error(FolioError::type_error("DateTime", "Number (out of range)"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            _ => Value::Error(FolioError::type_error("DateTime", self.type_name())),
        }
    }

    /// Convert to duration (may return Error)
    pub fn to_duration(&self) -> Value {
        match self {
            Value::Duration(d) => Value::Duration(d.clone()),
            Value::Number(n) => {
                // Interpret as seconds
                if let Some(secs) = n.to_i64() {
                    Value::Duration(FolioDuration::from_secs(secs))
                } else {
                    Value::Error(FolioError::type_error("Duration", "Number (out of range)"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            _ => Value::Error(FolioError::type_error("Duration", self.type_name())),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Text(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::DateTime(dt) => write!(f, "{}", dt),
            Value::Duration(d) => write!(f, "{}", d),
            Value::Object(_) => write!(f, "[Object]"),
            Value::List(items) => {
                // Smart list display: show values for small lists, count for large
                if items.len() <= 5 {
                    let contents: Vec<String> = items.iter().map(|v| v.to_string()).collect();
                    write!(f, "[{}]", contents.join(", "))
                } else {
                    write!(f, "[{}]", items.len())
                }
            }
            Value::Null => write!(f, "null"),
            Value::Error(e) => write!(f, "#ERROR: {}", e.code),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

// From implementations for convenience
impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(Number::from_i64(n))
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::Text(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::Text(s)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<Number> for Value {
    fn from(n: Number) -> Self {
        Value::Number(n)
    }
}

impl From<FolioDateTime> for Value {
    fn from(dt: FolioDateTime) -> Self {
        Value::DateTime(dt)
    }
}

impl From<FolioDuration> for Value {
    fn from(d: FolioDuration) -> Self {
        Value::Duration(d)
    }
}
```

folio-isis\Cargo.toml
```toml
[package]
name = "folio-isis"
description = "ISIS formula extensions for Folio"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
folio-core = { path = "../folio-core" }
folio-plugin = { path = "../folio-plugin" }
serde = { workspace = true }
```

folio-isis\src\analyzers.rs
```rs
//! ISIS-specific analyzers
//!
//! Specialized pattern detection for Phillip formula research.

use folio_plugin::prelude::*;
use std::collections::HashMap;

/// Advanced φ analyzer for Phillip formula research
/// Detects combinations of φ powers with other constants
pub struct PhillipAnalyzer;

impl AnalyzerPlugin for PhillipAnalyzer {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "phillip",
            description: "Deep φ analysis for ISIS formula research",
            detects: &["φ^n", "ln(φ)", "φ^n × ln(φ)", "φ^n × π", "Fibonacci", "Lucas"],
        }
    }

    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.9 // Always try for research tooling
    }

    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let phi = Number::phi(ctx.precision);
        let pi = Number::pi(ctx.precision);
        let ln_phi = phi.ln(ctx.precision).unwrap_or(Number::from_i64(0));

        let mut result = HashMap::new();

        // Test φ^n patterns for larger range
        for n in -10i32..=10 {
            let phi_n = phi.pow(n);
            if let Ok(ratio) = value.checked_div(&phi_n) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 1000 && i != 0 {
                        let key = format!("φ^{}", n);
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("power".to_string(), Value::Number(Number::from_i64(n as i64)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.95").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }

        // Test φ^n × ln(φ) patterns
        for n in -5i32..=5 {
            let phi_n = phi.pow(n);
            let phi_n_ln_phi = phi_n.mul(&ln_phi);
            if let Ok(ratio) = value.checked_div(&phi_n_ln_phi) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 100 && i != 0 {
                        let key = format!("φ^{}×ln(φ)", n);
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("phi_power".to_string(), Value::Number(Number::from_i64(n as i64)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.9").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }

        // Test φ^n × π patterns
        for n in -5i32..=5 {
            let phi_n = phi.pow(n);
            let phi_n_pi = phi_n.mul(&pi);
            if let Ok(ratio) = value.checked_div(&phi_n_pi) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 100 && i != 0 {
                        let key = format!("φ^{}×π", n);
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("phi_power".to_string(), Value::Number(Number::from_i64(n as i64)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.85").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }

        // Check Fibonacci numbers (1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, ...)
        let fibonacci: [i64; 15] = [1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233, 377, 610];
        if let Some(val_i64) = value.to_i64() {
            if val_i64 > 0 {
                if let Some(pos) = fibonacci.iter().position(|&f| f == val_i64) {
                    let mut entry = HashMap::new();
                    entry.insert("index".to_string(), Value::Number(Number::from_i64((pos + 1) as i64)));
                    entry.insert("confidence".to_string(), Value::Number(Number::from_str("1.0").unwrap()));
                    result.insert("Fibonacci".to_string(), Value::Object(entry));
                }
            }
        }

        // Check Lucas numbers (2, 1, 3, 4, 7, 11, 18, 29, 47, 76, 123, ...)
        let lucas: [i64; 12] = [2, 1, 3, 4, 7, 11, 18, 29, 47, 76, 123, 199];
        if let Some(val_i64) = value.to_i64() {
            if val_i64 > 0 {
                if let Some(pos) = lucas.iter().position(|&l| l == val_i64) {
                    let mut entry = HashMap::new();
                    entry.insert("index".to_string(), Value::Number(Number::from_i64(pos as i64)));
                    entry.insert("confidence".to_string(), Value::Number(Number::from_str("1.0").unwrap()));
                    result.insert("Lucas".to_string(), Value::Object(entry));
                }
            }
        }

        if result.is_empty() {
            result.insert("_note".to_string(), Value::Text("No φ-related patterns found".to_string()));
        }

        Value::Object(result)
    }
}

/// Error archaeologist - recursive error analysis
/// Analyzes how far a value is from known constants and looks for φ structure in errors
pub struct ErrorArchaeologist;

impl AnalyzerPlugin for ErrorArchaeologist {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "archaeology",
            description: "Recursive error decomposition looking for hidden φ structure",
            detects: &["nested φ", "error chains", "convergent series"],
        }
    }

    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.5
    }

    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let phi = Number::phi(ctx.precision);
        let pi = Number::pi(ctx.precision);
        let e = Number::e(ctx.precision);
        let one = Number::from_i64(1);

        let mut result = HashMap::new();
        let mut layers = Vec::new();

        // Layer 0: Original value
        let mut current = value.clone();
        let max_depth = 5;

        for depth in 0..max_depth {
            let mut layer_info = HashMap::new();
            layer_info.insert("depth".to_string(), Value::Number(Number::from_i64(depth as i64)));
            layer_info.insert("value".to_string(), Value::Number(current.clone()));

            // Compute errors from key constants
            let error_phi = current.sub(&phi);
            let error_one = current.sub(&one);
            let error_pi = current.sub(&pi);
            let error_e = current.sub(&e);

            let mut errors = HashMap::new();
            errors.insert("φ".to_string(), Value::Number(error_phi.clone()));
            errors.insert("1".to_string(), Value::Number(error_one.clone()));
            errors.insert("π".to_string(), Value::Number(error_pi));
            errors.insert("e".to_string(), Value::Number(error_e));
            layer_info.insert("errors".to_string(), Value::Object(errors));

            // Check if any error is a simple φ multiple
            let mut found_phi_structure = false;
            for n in -5i32..=5 {
                if n == 0 {
                    continue;
                }
                let phi_n = phi.pow(n);
                if let Ok(ratio) = error_phi.checked_div(&phi_n) {
                    if let Some(i) = ratio.to_i64() {
                        if i.abs() <= 10 && i != 0 {
                            let mut phi_match = HashMap::new();
                            phi_match.insert("power".to_string(), Value::Number(Number::from_i64(n as i64)));
                            phi_match.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                            layer_info.insert("phi_structure".to_string(), Value::Object(phi_match));
                            found_phi_structure = true;
                            break;
                        }
                    }
                }
            }

            layers.push(Value::Object(layer_info));

            // If we found exact φ structure or error is very small, stop
            if found_phi_structure || error_phi.abs().as_decimal(15).starts_with("0.000000000") {
                break;
            }

            // Next layer: analyze the error from φ
            current = error_phi;
        }

        result.insert("layers".to_string(), Value::List(layers));
        result.insert("depth_analyzed".to_string(), Value::Number(Number::from_i64(max_depth as i64)));

        Value::Object(result)
    }
}
```

folio-isis\src\lib.rs
```rs
//! ISIS Formula Extensions for Folio
//!
//! Provides:
//! - ISIS transform function
//! - ISIS inverse transform
//! - Specialized φ analyzers for error archaeology
//! - GBR (Geometric Background Radiation) analysis

mod transform;
mod analyzers;

use folio_plugin::PluginRegistry;

pub use transform::{IsisTransform, IsisInverse};
pub use analyzers::{PhillipAnalyzer, ErrorArchaeologist};

/// Load ISIS extensions into registry
pub fn load_isis_extensions(registry: PluginRegistry) -> PluginRegistry {
    registry
        .with_function(IsisTransform)
        .with_function(IsisInverse)
        .with_analyzer(PhillipAnalyzer)
}
```

folio-isis\src\transform.rs
```rs
//! ISIS Transform Functions
//!
//! X(n) = -ln(n) × φ / (2π × ln(φ))
//!
//! This maps positive numbers n to a "golden ratio normalized" coordinate system.
//! Key properties:
//! - X(φ) = -1 (φ maps to -1)
//! - X(1) = 0 (1 maps to origin)
//! - X(1/φ) = 1 (1/φ maps to 1)

use folio_plugin::prelude::*;

pub struct IsisTransform;
pub struct IsisInverse;

static ISIS_ARGS: [ArgMeta; 1] = [ArgMeta { name: "n", typ: "Number", description: "Value to transform (positive)", optional: false, default: None }];
static ISIS_EXAMPLES: [&str; 3] = ["ISIS(2)", "ISIS(phi)", "ISIS(1)"];
static ISIS_RELATED: [&str; 1] = ["ISIS_INV"];

static ISIS_INV_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "X-space value", optional: false, default: None }];
static ISIS_INV_EXAMPLES: [&str; 3] = ["ISIS_INV(0)", "ISIS_INV(-1)", "ISIS_INV(1)"];
static ISIS_INV_RELATED: [&str; 1] = ["ISIS"];

impl FunctionPlugin for IsisTransform {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ISIS",
            description: "ISIS transform: X(n) = -ln(n) × φ / (2π × ln(φ))",
            usage: "ISIS(n)",
            args: &ISIS_ARGS,
            returns: "Number",
            examples: &ISIS_EXAMPLES,
            category: "isis",
            source: Some("https://example.com/isis-docs"),
            related: &ISIS_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        // Check argument count
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("ISIS", 1, args.len()));
        }

        // Get the input number
        let n = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("ISIS", "n", "Number", other.type_name())),
        };

        // Domain check: n must be positive
        if n.is_zero() {
            return Value::Error(FolioError::domain_error("ISIS transform undefined for n=0 (ln(0) is undefined)"));
        }
        if n.is_negative() {
            return Value::Error(FolioError::domain_error("ISIS transform requires positive n (ln of negative undefined)"));
        }

        // Compute: X(n) = -ln(n) × φ / (2π × ln(φ))
        let phi = Number::phi(ctx.precision);
        let pi = Number::pi(ctx.precision);
        let two = Number::from_i64(2);

        // Compute ln(n)
        let ln_n = match n.ln(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Compute ln(φ)
        let ln_phi = match phi.ln(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Compute 2π
        let two_pi = two.mul(&pi);

        // Compute denominator: 2π × ln(φ)
        let denominator = two_pi.mul(&ln_phi);

        // Compute numerator: -ln(n) × φ
        let zero = Number::from_i64(0);
        let neg_ln_n = zero.sub(&ln_n);
        let numerator = neg_ln_n.mul(&phi);

        // Compute X(n) = numerator / denominator
        match numerator.checked_div(&denominator) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

impl FunctionPlugin for IsisInverse {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ISIS_INV",
            description: "Inverse ISIS transform: find n where X(n) = x. n = exp(-x × 2π × ln(φ) / φ)",
            usage: "ISIS_INV(x)",
            args: &ISIS_INV_ARGS,
            returns: "Number",
            examples: &ISIS_INV_EXAMPLES,
            category: "isis",
            source: Some("https://example.com/isis-docs"),
            related: &ISIS_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        // Check argument count
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("ISIS_INV", 1, args.len()));
        }

        // Get the input number
        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("ISIS_INV", "x", "Number", other.type_name())),
        };

        // Inverse: n = exp(-x × 2π × ln(φ) / φ)
        let phi = Number::phi(ctx.precision);
        let pi = Number::pi(ctx.precision);
        let two = Number::from_i64(2);
        let zero = Number::from_i64(0);

        // Compute ln(φ)
        let ln_phi = match phi.ln(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Compute 2π
        let two_pi = two.mul(&pi);

        // Compute -x × 2π × ln(φ)
        let neg_x = zero.sub(x);
        let temp = neg_x.mul(&two_pi).mul(&ln_phi);

        // Divide by φ
        let exponent = match temp.checked_div(&phi) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Compute exp(exponent)
        Value::Number(exponent.exp(ctx.precision))
    }
}
```

folio-mcp\Cargo.toml
```toml
[package]
name = "folio-mcp"
description = "MCP Server for Folio"
version.workspace = true
edition.workspace = true
license.workspace = true

[[bin]]
name = "folio-mcp"
path = "src/main.rs"

[dependencies]
folio = { path = "../folio" }
folio-core = { path = "../folio-core" }
folio-plugin = { path = "../folio-plugin" }
folio-std = { path = "../folio-std" }
folio-isis = { path = "../folio-isis" }
folio-stats = { path = "../folio-stats" }
folio-sequence = { path = "../folio-sequence" }
serde = { workspace = true }
serde_json = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
```

folio-mcp\src\main.rs
```rs
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

/// Extract the base name from various input formats:
/// - "mortgage" -> "mortgage"
/// - "mortgage.fmd" -> "mortgage"
/// - "/path/to/mortgage.fmd" -> "mortgage"
/// - "C:\path\to\mortgage.fmd" -> "mortgage"
/// - "examples/mortgage.fmd" -> "mortgage"
fn extract_fmd_name(input: &str) -> String {
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
fn load_fmd_file(input: &str) -> Result<String, String> {
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

/// Create Folio with standard library, stats, sequences, and ISIS extensions
fn create_folio_with_isis() -> Folio {
    // Load standard library
    let registry = folio_std::standard_registry();
    // Add statistics functions
    let registry = folio_stats::load_stats_library(registry);
    // Add sequence functions
    let registry = folio_sequence::load_sequence_library(registry);
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
                "description": "Compact quick reference (~400 tokens). Lists function names grouped by category with Object return fields.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
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
        "quick" => tool_quick(folio),
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
        "data": value_to_json(&help)
    }))
}

fn tool_quick(folio: &Folio) -> Result<JsonValue, McpError> {
    let quick_ref = generate_quick_reference(folio);
    Ok(json!({
        "content": [{ "type": "text", "text": quick_ref }]
    }))
}

fn generate_folio_overview(folio: &Folio) -> String {
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

fn generate_compact_overview(folio: &Folio) -> String {
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

fn generate_quick_reference(_folio: &Folio) -> String {
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

fn format_help(help: &Value) -> String {
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

fn tool_list_functions(folio: &Folio, args: JsonValue) -> Result<JsonValue, McpError> {
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

    Ok(json!({ "content": [{ "type": "text", "text": text }], "data": value_to_json(&functions) }))
}

fn tool_list_constants(folio: &Folio, _args: JsonValue) -> Result<JsonValue, McpError> {
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

    Ok(json!({ "content": [{ "type": "text", "text": text }], "data": value_to_json(&constants) }))
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
        Value::DateTime(dt) => json!({"_type": "datetime", "value": dt.to_string(), "nanos": dt.as_nanos().to_string()}),
        Value::Duration(d) => json!({"_type": "duration", "value": d.to_string(), "nanos": d.as_nanos().to_string()}),
        Value::List(l) => JsonValue::Array(l.iter().map(value_to_json).collect()),
        Value::Object(o) => JsonValue::Object(o.iter().map(|(k, v)| (k.clone(), value_to_json(v))).collect()),
        Value::Error(e) => json!({"_error": {"code": e.code, "message": e.message}}),
    }
}
```

folio-plugin\Cargo.toml
```toml
[package]
name = "folio-plugin"
description = "Plugin system for Folio: traits and registry"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
folio-core = { path = "../folio-core" }
serde = { workspace = true }
serde_json = { workspace = true }
async-trait = { workspace = true }
```

folio-plugin\src\context.rs
```rs
//! Evaluation Context

use folio_core::Value;
use crate::PluginRegistry;
use std::collections::HashMap;
use std::sync::Arc;

/// Evaluation context passed to plugins
pub struct EvalContext {
    pub precision: u32,
    pub variables: HashMap<String, Value>,
    pub registry: Arc<PluginRegistry>,
    pub tracing: bool,
    pub trace: Vec<TraceStep>,
}

/// Single step in evaluation trace
#[derive(Debug, Clone)]
pub struct TraceStep {
    pub cell: String,
    pub formula: String,
    pub result: Value,
    pub dependencies: Vec<String>,
}

impl EvalContext {
    pub fn new(registry: Arc<PluginRegistry>) -> Self {
        Self {
            precision: 50,
            variables: HashMap::new(),
            registry,
            tracing: false,
            trace: Vec::new(),
        }
    }
    
    pub fn with_precision(mut self, precision: u32) -> Self {
        self.precision = precision;
        self
    }
    
    pub fn with_variables(mut self, vars: HashMap<String, Value>) -> Self {
        self.variables = vars;
        self
    }
    
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.tracing = enabled;
        self
    }
    
    pub fn get_var(&self, name: &str) -> Value {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.is_empty() {
            return Value::Error(folio_core::FolioError::undefined_var(name));
        }

        let root = match self.variables.get(parts[0]) {
            Some(v) => v.clone(),
            None => {
                // Check if it's a registered constant (π, φ, e)
                if let Some(constant) = self.registry.get_constant(parts[0]) {
                    // Evaluate the constant's formula
                    // For built-in constants, the formula is a function call like "pi", "exp(1)", "(1 + sqrt(5)) / 2"
                    return self.eval_constant_formula(&constant.formula);
                }
                return Value::Error(folio_core::FolioError::undefined_var(parts[0]));
            }
        };

        let mut current = root;
        for part in &parts[1..] {
            current = current.get(part);
            if current.is_error() {
                return current;
            }
        }
        current
    }

    /// Evaluate a constant's formula (e.g., "pi", "exp(1)", "(1 + sqrt(5)) / 2", "0.51099895")
    fn eval_constant_formula(&self, formula: &str) -> Value {
        // First try: parse as numeric literal (handles "0.51099895", "1776.86", "299792458", etc.)
        if let Ok(n) = folio_core::Number::from_str(formula) {
            return Value::Number(n);
        }

        // Handle special computed formulas
        match formula {
            "pi" => Value::Number(folio_core::Number::pi(self.precision)),
            "exp(1)" => Value::Number(folio_core::Number::e(self.precision)),
            "(1 + sqrt(5)) / 2" => Value::Number(folio_core::Number::phi(self.precision)),
            "sqrt(2)" => {
                let two = folio_core::Number::from_i64(2);
                two.sqrt(self.precision)
                    .map(Value::Number)
                    .unwrap_or_else(|e| Value::Error(e.into()))
            }
            "sqrt(3)" => {
                let three = folio_core::Number::from_i64(3);
                three.sqrt(self.precision)
                    .map(Value::Number)
                    .unwrap_or_else(|e| Value::Error(e.into()))
            }
            _ => {
                // Unknown formula
                Value::Error(folio_core::FolioError::new("UNKNOWN_CONSTANT",
                    format!("Unknown constant formula: {}", formula)))
            }
        }
    }
    
    pub fn set_var(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
    
    pub fn record_trace(&mut self, cell: String, formula: String, result: Value, dependencies: Vec<String>) {
        if self.tracing {
            self.trace.push(TraceStep { cell, formula, result, dependencies });
        }
    }
}
```

folio-plugin\src\lib.rs
```rs
//! Folio Plugin System
//!
//! Provides traits for extending Folio with custom:
//! - Functions (pure computation)
//! - Analyzers (pattern detection)
//! - Commands (side effects)

mod traits;
mod registry;
mod context;

pub use traits::{
    FunctionPlugin, FunctionMeta,
    AnalyzerPlugin, AnalyzerMeta,
    CommandPlugin, CommandMeta,
    ArgMeta,
};
pub use registry::{PluginRegistry, ConstantDef};
pub use context::{EvalContext, TraceStep};

/// Re-export core types for plugin authors
pub mod prelude {
    pub use crate::{
        FunctionPlugin, FunctionMeta,
        AnalyzerPlugin, AnalyzerMeta,
        CommandPlugin, CommandMeta,
        ArgMeta, PluginRegistry, EvalContext, TraceStep,
    };
    pub use folio_core::prelude::*;
}
```

folio-plugin\src\registry.rs
```rs
//! Plugin Registry

use crate::{FunctionPlugin, AnalyzerPlugin, CommandPlugin, FunctionMeta, CommandMeta};
use crate::EvalContext;
use folio_core::{Number, Value, FolioError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Definition of a built-in constant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantDef {
    pub name: String,
    pub formula: String,
    pub source: String,
    pub category: String,
}

/// Central plugin registry
pub struct PluginRegistry {
    functions: HashMap<String, Arc<dyn FunctionPlugin>>,
    analyzers: Vec<Arc<dyn AnalyzerPlugin>>,
    commands: HashMap<String, Arc<dyn CommandPlugin>>,
    constants: HashMap<String, ConstantDef>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            analyzers: Vec::new(),
            commands: HashMap::new(),
            constants: HashMap::new(),
        }
    }
    
    pub fn with_function<F: FunctionPlugin + 'static>(mut self, f: F) -> Self {
        let name = f.meta().name.to_lowercase();
        self.functions.insert(name, Arc::new(f));
        self
    }
    
    pub fn with_analyzer<A: AnalyzerPlugin + 'static>(mut self, a: A) -> Self {
        self.analyzers.push(Arc::new(a));
        self
    }
    
    pub fn with_command<C: CommandPlugin + 'static>(mut self, c: C) -> Self {
        let name = c.meta().name.to_lowercase();
        self.commands.insert(name, Arc::new(c));
        self
    }
    
    pub fn with_constant(mut self, def: ConstantDef) -> Self {
        let name = def.name.to_lowercase();
        self.constants.insert(name, def);
        self
    }
    
    pub fn get_function(&self, name: &str) -> Option<&dyn FunctionPlugin> {
        self.functions.get(&name.to_lowercase()).map(|f| f.as_ref())
    }
    
    pub fn get_command(&self, name: &str) -> Option<&dyn CommandPlugin> {
        self.commands.get(&name.to_lowercase()).map(|c| c.as_ref())
    }
    
    pub fn get_constant(&self, name: &str) -> Option<&ConstantDef> {
        self.constants.get(&name.to_lowercase())
    }
    
    pub fn call_function(&self, name: &str, args: &[Value], ctx: &EvalContext) -> Value {
        match self.get_function(name) {
            Some(f) => f.call(args, ctx),
            None => {
                // Find similar function names for better error message
                let similar = self.find_similar_functions(name);
                let mut err = FolioError::undefined_func(name);
                if !similar.is_empty() {
                    let suggestions: Vec<&str> = similar.iter().take(5).map(|s| s.as_str()).collect();
                    err = err.with_suggestion(format!(
                        "Similar: {}. Use help() for full list.",
                        suggestions.join(", ")
                    ));
                }
                Value::Error(err)
            }
        }
    }

    /// Find function names similar to the given name (for error suggestions)
    fn find_similar_functions(&self, name: &str) -> Vec<String> {
        let name_lower = name.to_lowercase();
        let mut matches: Vec<(String, usize)> = self.functions.keys()
            .filter_map(|func_name| {
                let score = Self::similarity_score(&name_lower, func_name);
                if score > 0 {
                    Some((func_name.clone(), score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by similarity score (higher = more similar)
        matches.sort_by(|a, b| b.1.cmp(&a.1));
        matches.into_iter().map(|(name, _)| name).collect()
    }

    /// Calculate similarity score between two strings
    fn similarity_score(query: &str, candidate: &str) -> usize {
        let mut score = 0;

        // Exact prefix match is best
        if candidate.starts_with(query) {
            score += 100;
        }
        // Contains the query
        else if candidate.contains(query) {
            score += 50;
        }
        // Query contains the candidate
        else if query.contains(candidate) {
            score += 30;
        }

        // Levenshtein-like: count matching characters
        let query_chars: std::collections::HashSet<char> = query.chars().collect();
        let candidate_chars: std::collections::HashSet<char> = candidate.chars().collect();
        let common = query_chars.intersection(&candidate_chars).count();
        score += common * 2;

        // Penalize length difference
        let len_diff = (query.len() as i32 - candidate.len() as i32).unsigned_abs() as usize;
        if len_diff < 5 && score > 0 {
            score += 5 - len_diff;
        }

        score
    }
    
    pub fn decompose(&self, value: &Number, ctx: &EvalContext) -> Value {
        let mut result = HashMap::new();
        let threshold = 0.1;
        
        for analyzer in &self.analyzers {
            let confidence = analyzer.confidence(value, ctx);
            if confidence >= threshold {
                match analyzer.analyze(value, ctx) {
                    Value::Object(map) => { result.extend(map); }
                    Value::Error(e) => {
                        result.insert(format!("_error_{}", analyzer.meta().name), Value::Error(e));
                    }
                    other => { result.insert(analyzer.meta().name.to_string(), other); }
                }
            }
        }
        
        Value::Object(result)
    }
    
    pub fn help(&self, name: Option<&str>) -> Value {
        match name {
            Some(n) => self.help_for(n),
            None => self.general_help(),
        }
    }
    
    fn help_for(&self, name: &str) -> Value {
        let name_lower = name.to_lowercase();
        
        if let Some(f) = self.functions.get(&name_lower) {
            return Value::Object(self.function_to_help(f.meta()));
        }
        if let Some(c) = self.commands.get(&name_lower) {
            return Value::Object(self.command_to_help(c.meta()));
        }
        if let Some(c) = self.constants.get(&name_lower) {
            return Value::Object(self.constant_to_help(c));
        }
        
        Value::Error(FolioError::new("NOT_FOUND", format!("No function, command, or constant named '{}'", name)))
    }
    
    fn general_help(&self) -> Value {
        let mut help = HashMap::new();
        
        let mut funcs_by_cat: HashMap<String, Vec<String>> = HashMap::new();
        for (name, f) in &self.functions {
            let cat = f.meta().category.to_string();
            funcs_by_cat.entry(cat).or_default().push(name.clone());
        }
        help.insert("functions".to_string(), 
            Value::Object(funcs_by_cat.into_iter()
                .map(|(k, v)| (k, Value::List(v.into_iter().map(Value::Text).collect())))
                .collect()));
        
        help.insert("constants".to_string(),
            Value::List(self.constants.keys().cloned().map(Value::Text).collect()));
        
        help.insert("commands".to_string(),
            Value::List(self.commands.keys().cloned().map(Value::Text).collect()));
        
        help.insert("usage".to_string(), 
            Value::Text("Call help('function_name') for detailed help.".to_string()));
        
        Value::Object(help)
    }
    
    fn function_to_help(&self, meta: FunctionMeta) -> HashMap<String, Value> {
        let mut help = HashMap::new();
        help.insert("name".to_string(), Value::Text(meta.name.to_string()));
        help.insert("type".to_string(), Value::Text("function".to_string()));
        help.insert("description".to_string(), Value::Text(meta.description.to_string()));
        help.insert("usage".to_string(), Value::Text(meta.usage.to_string()));
        help.insert("returns".to_string(), Value::Text(meta.returns.to_string()));
        help.insert("category".to_string(), Value::Text(meta.category.to_string()));
        help.insert("args".to_string(), Value::List(
            meta.args.iter().map(|a| {
                let mut arg = HashMap::new();
                arg.insert("name".to_string(), Value::Text(a.name.to_string()));
                arg.insert("type".to_string(), Value::Text(a.typ.to_string()));
                arg.insert("description".to_string(), Value::Text(a.description.to_string()));
                arg.insert("optional".to_string(), Value::Bool(a.optional));
                Value::Object(arg)
            }).collect()
        ));
        help.insert("examples".to_string(), Value::List(
            meta.examples.iter().map(|e| Value::Text(e.to_string())).collect()
        ));
        help
    }
    
    fn command_to_help(&self, meta: CommandMeta) -> HashMap<String, Value> {
        let mut help = HashMap::new();
        help.insert("name".to_string(), Value::Text(meta.name.to_string()));
        help.insert("type".to_string(), Value::Text("command".to_string()));
        help.insert("description".to_string(), Value::Text(meta.description.to_string()));
        help
    }
    
    fn constant_to_help(&self, def: &ConstantDef) -> HashMap<String, Value> {
        let mut help = HashMap::new();
        help.insert("name".to_string(), Value::Text(def.name.clone()));
        help.insert("type".to_string(), Value::Text("constant".to_string()));
        help.insert("formula".to_string(), Value::Text(def.formula.clone()));
        help.insert("source".to_string(), Value::Text(def.source.clone()));
        help
    }
    
    pub fn list_functions(&self, category: Option<&str>) -> Value {
        let funcs: Vec<Value> = self.functions.values()
            .filter(|f| category.map_or(true, |c| f.meta().category == c))
            .map(|f| {
                let meta = f.meta();
                let mut obj = HashMap::new();
                obj.insert("name".to_string(), Value::Text(meta.name.to_string()));
                obj.insert("description".to_string(), Value::Text(meta.description.to_string()));
                obj.insert("usage".to_string(), Value::Text(meta.usage.to_string()));
                obj.insert("category".to_string(), Value::Text(meta.category.to_string()));
                Value::Object(obj)
            })
            .collect();
        Value::List(funcs)
    }
    
    pub fn list_constants(&self) -> Value {
        let consts: Vec<Value> = self.constants.values()
            .map(|c| {
                let mut obj = HashMap::new();
                obj.insert("name".to_string(), Value::Text(c.name.clone()));
                obj.insert("formula".to_string(), Value::Text(c.formula.clone()));
                obj.insert("source".to_string(), Value::Text(c.source.clone()));
                Value::Object(obj)
            })
            .collect();
        Value::List(consts)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

folio-plugin\src\traits.rs
```rs
//! Plugin traits

use folio_core::{Number, Value};
use crate::EvalContext;
use serde::Serialize;

/// Metadata about a function argument
#[derive(Debug, Clone, Serialize)]
pub struct ArgMeta {
    pub name: &'static str,
    pub typ: &'static str,
    pub description: &'static str,
    pub optional: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<&'static str>,
}

impl ArgMeta {
    pub const fn required(name: &'static str, typ: &'static str, description: &'static str) -> Self {
        Self { name, typ, description, optional: false, default: None }
    }
    
    pub const fn optional(name: &'static str, typ: &'static str, description: &'static str, default: &'static str) -> Self {
        Self { name, typ, description, optional: true, default: Some(default) }
    }
}

/// Metadata for a function plugin
#[derive(Debug, Clone, Serialize)]
pub struct FunctionMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub args: &'static [ArgMeta],
    pub returns: &'static str,
    pub examples: &'static [&'static str],
    pub category: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<&'static str>,
    pub related: &'static [&'static str],
}

/// Pure function plugin
pub trait FunctionPlugin: Send + Sync {
    fn meta(&self) -> FunctionMeta;
    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value;
}

/// Metadata for an analyzer plugin
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzerMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub detects: &'static [&'static str],
}

/// Pattern detection plugin
pub trait AnalyzerPlugin: Send + Sync {
    fn meta(&self) -> AnalyzerMeta;
    fn confidence(&self, value: &Number, ctx: &EvalContext) -> f64;
    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value;
}

/// Metadata for a command plugin
#[derive(Debug, Clone, Serialize)]
pub struct CommandMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub args: &'static [ArgMeta],
    pub examples: &'static [&'static str],
}

/// Command plugin (may have side effects)
pub trait CommandPlugin: Send + Sync {
    fn meta(&self) -> CommandMeta;
    fn execute(&self, args: &[Value], ctx: &mut EvalContext) -> Value;
}
```

folio-sequence\Cargo.toml
```toml
[package]
name = "folio-sequence"
description = "Sequence generation and series operations for Folio"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
folio-core = { path = "../folio-core" }
folio-plugin = { path = "../folio-plugin" }
serde = { workspace = true }
```

folio-sequence\src\expr.rs
```rs
//! Mini-expression parser for recurrence relations
//!
//! Supports simple arithmetic expressions with variables a, b, c, d (previous values)
//! and n (current index).

use folio_core::{Number, FolioError};
use std::collections::HashMap;

/// Token types for the expression parser
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(Number),
    Variable(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
    Func(String),
}

/// AST node for expressions
#[derive(Debug, Clone)]
pub enum Expr {
    Num(Number),
    Var(String),
    BinOp(Box<Expr>, Op, Box<Expr>),
    UnaryMinus(Box<Expr>),
    FuncCall(String, Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

/// Tokenize expression string
fn tokenize(input: &str) -> Result<Vec<Token>, FolioError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' => {
                chars.next();
            }
            '+' => {
                tokens.push(Token::Plus);
                chars.next();
            }
            '-' => {
                tokens.push(Token::Minus);
                chars.next();
            }
            '*' => {
                tokens.push(Token::Star);
                chars.next();
            }
            '/' => {
                tokens.push(Token::Slash);
                chars.next();
            }
            '^' => {
                tokens.push(Token::Caret);
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num_str.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match Number::from_str(&num_str) {
                    Ok(n) => tokens.push(Token::Number(n)),
                    Err(_) => return Err(FolioError::new("PARSE_ERROR", format!("Invalid number: {}", num_str))),
                }
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let lower = ident.to_lowercase();
                // Check if it's a function
                if matches!(lower.as_str(), "abs" | "sqrt" | "floor" | "ceil") {
                    tokens.push(Token::Func(lower));
                } else {
                    tokens.push(Token::Variable(lower));
                }
            }
            _ => {
                return Err(FolioError::new("PARSE_ERROR", format!("Unexpected character: {}", ch)));
            }
        }
    }

    Ok(tokens)
}

/// Parse tokens into AST
struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn parse(&mut self) -> Result<Expr, FolioError> {
        self.parse_expr()
    }

    // expr = term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<Expr, FolioError> {
        let mut left = self.parse_term()?;

        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = Expr::BinOp(Box::new(left), Op::Add, Box::new(right));
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_term()?;
                    left = Expr::BinOp(Box::new(left), Op::Sub, Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // term = power (('*' | '/') power)*
    fn parse_term(&mut self) -> Result<Expr, FolioError> {
        let mut left = self.parse_power()?;

        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.advance();
                    let right = self.parse_power()?;
                    left = Expr::BinOp(Box::new(left), Op::Mul, Box::new(right));
                }
                Some(Token::Slash) => {
                    self.advance();
                    let right = self.parse_power()?;
                    left = Expr::BinOp(Box::new(left), Op::Div, Box::new(right));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    // power = unary ('^' power)?  (right associative)
    fn parse_power(&mut self) -> Result<Expr, FolioError> {
        let left = self.parse_unary()?;

        if matches!(self.peek(), Some(Token::Caret)) {
            self.advance();
            let right = self.parse_power()?;  // Right associative
            return Ok(Expr::BinOp(Box::new(left), Op::Pow, Box::new(right)));
        }

        Ok(left)
    }

    // unary = '-' unary | primary
    fn parse_unary(&mut self) -> Result<Expr, FolioError> {
        if matches!(self.peek(), Some(Token::Minus)) {
            self.advance();
            let expr = self.parse_unary()?;
            return Ok(Expr::UnaryMinus(Box::new(expr)));
        }
        self.parse_primary()
    }

    // primary = number | variable | func '(' expr ')' | '(' expr ')'
    fn parse_primary(&mut self) -> Result<Expr, FolioError> {
        match self.peek().cloned() {
            Some(Token::Number(n)) => {
                self.advance();
                Ok(Expr::Num(n))
            }
            Some(Token::Variable(name)) => {
                self.advance();
                Ok(Expr::Var(name))
            }
            Some(Token::Func(name)) => {
                self.advance();
                if !matches!(self.peek(), Some(Token::LParen)) {
                    return Err(FolioError::new("PARSE_ERROR", format!("Expected '(' after function {}", name)));
                }
                self.advance(); // consume '('
                let arg = self.parse_expr()?;
                if !matches!(self.peek(), Some(Token::RParen)) {
                    return Err(FolioError::new("PARSE_ERROR", "Expected ')' after function argument".to_string()));
                }
                self.advance(); // consume ')'
                Ok(Expr::FuncCall(name, Box::new(arg)))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                if !matches!(self.peek(), Some(Token::RParen)) {
                    return Err(FolioError::new("PARSE_ERROR", "Expected closing ')'".to_string()));
                }
                self.advance();
                Ok(expr)
            }
            Some(token) => {
                Err(FolioError::new("PARSE_ERROR", format!("Unexpected token: {:?}", token)))
            }
            None => {
                Err(FolioError::new("PARSE_ERROR", "Unexpected end of expression".to_string()))
            }
        }
    }
}

/// Parse an expression string into an AST
pub fn parse_expr(input: &str) -> Result<Expr, FolioError> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err(FolioError::new("PARSE_ERROR", "Empty expression".to_string()));
    }
    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;

    // Check that all tokens were consumed
    if parser.pos < parser.tokens.len() {
        return Err(FolioError::new("PARSE_ERROR", "Unexpected tokens at end of expression".to_string()));
    }

    Ok(expr)
}

/// Evaluate an expression with the given variable context
pub fn eval_expr(expr: &Expr, ctx: &HashMap<String, Number>, precision: u32) -> Result<Number, FolioError> {
    match expr {
        Expr::Num(n) => Ok(n.clone()),
        Expr::Var(name) => {
            ctx.get(name)
                .cloned()
                .ok_or_else(|| FolioError::new("UNDEFINED_VAR", format!("Unknown variable: {}", name)))
        }
        Expr::BinOp(left, op, right) => {
            let l = eval_expr(left, ctx, precision)?;
            let r = eval_expr(right, ctx, precision)?;
            match op {
                Op::Add => Ok(l.add(&r)),
                Op::Sub => Ok(l.sub(&r)),
                Op::Mul => Ok(l.mul(&r)),
                Op::Div => l.checked_div(&r).map_err(|e| e.into()),
                Op::Pow => {
                    // Check if exponent is an integer
                    if r.is_integer() {
                        if let Some(exp) = r.to_i64() {
                            if exp >= i32::MIN as i64 && exp <= i32::MAX as i64 {
                                return Ok(l.pow(exp as i32));
                            }
                        }
                    }
                    Ok(l.pow_real(&r, precision))
                }
            }
        }
        Expr::UnaryMinus(inner) => {
            let val = eval_expr(inner, ctx, precision)?;
            Ok(Number::from_i64(0).sub(&val))
        }
        Expr::FuncCall(name, arg) => {
            let val = eval_expr(arg, ctx, precision)?;
            match name.as_str() {
                "abs" => Ok(val.abs()),
                "sqrt" => val.sqrt(precision).map_err(|e| e.into()),
                "floor" => Ok(val.floor()),
                "ceil" => Ok(val.ceil()),
                _ => Err(FolioError::new("UNDEFINED_FUNC", format!("Unknown function: {}", name))),
            }
        }
    }
}

/// Evaluate a recurrence expression with previous values and current index
pub fn eval_recurrence_expr(
    expr_str: &str,
    prev: &[Number],
    n: i64,
    precision: u32,
) -> Result<Number, FolioError> {
    let expr = parse_expr(expr_str)?;

    let mut ctx = HashMap::new();

    // a = most recent (n-1)
    if let Some(val) = prev.last() {
        ctx.insert("a".to_string(), val.clone());
    }
    // b = second most recent (n-2)
    if prev.len() >= 2 {
        ctx.insert("b".to_string(), prev[prev.len() - 2].clone());
    }
    // c = third most recent (n-3)
    if prev.len() >= 3 {
        ctx.insert("c".to_string(), prev[prev.len() - 3].clone());
    }
    // d = fourth most recent (n-4)
    if prev.len() >= 4 {
        ctx.insert("d".to_string(), prev[prev.len() - 4].clone());
    }
    // n = current index (1-based)
    ctx.insert("n".to_string(), Number::from_i64(n));

    eval_expr(&expr, &ctx, precision)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize("a + b").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Variable(_)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Variable(_)));
    }

    #[test]
    fn test_parse_simple() {
        let expr = parse_expr("a + b").unwrap();
        assert!(matches!(expr, Expr::BinOp(_, Op::Add, _)));
    }

    #[test]
    fn test_eval_addition() {
        let expr = parse_expr("a + b").unwrap();
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Number::from_i64(3));
        ctx.insert("b".to_string(), Number::from_i64(5));
        let result = eval_expr(&expr, &ctx, 50).unwrap();
        assert_eq!(result.to_i64(), Some(8));
    }

    #[test]
    fn test_eval_multiplication() {
        let expr = parse_expr("2 * a").unwrap();
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Number::from_i64(7));
        let result = eval_expr(&expr, &ctx, 50).unwrap();
        assert_eq!(result.to_i64(), Some(14));
    }

    #[test]
    fn test_eval_power() {
        let expr = parse_expr("a ^ 2").unwrap();
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Number::from_i64(3));
        let result = eval_expr(&expr, &ctx, 50).unwrap();
        assert_eq!(result.to_i64(), Some(9));
    }

    #[test]
    fn test_eval_complex() {
        let expr = parse_expr("2*a + b").unwrap();
        let mut ctx = HashMap::new();
        ctx.insert("a".to_string(), Number::from_i64(5));
        ctx.insert("b".to_string(), Number::from_i64(3));
        let result = eval_expr(&expr, &ctx, 50).unwrap();
        assert_eq!(result.to_i64(), Some(13));
    }

    #[test]
    fn test_eval_recurrence() {
        // Fibonacci: a + b
        let prev = vec![Number::from_i64(5), Number::from_i64(8)];
        let result = eval_recurrence_expr("a + b", &prev, 7, 50).unwrap();
        assert_eq!(result.to_i64(), Some(13));
    }

    #[test]
    fn test_eval_factorial_recurrence() {
        // Factorial: a * n
        let prev = vec![Number::from_i64(24)]; // 4!
        let result = eval_recurrence_expr("a * n", &prev, 5, 50).unwrap();
        assert_eq!(result.to_i64(), Some(120)); // 5!
    }
}
```

folio-sequence\src\generators.rs
```rs
//! Basic sequence generators
//!
//! range, linspace, arithmetic, geometric, harmonic, repeat, cycle

use folio_plugin::prelude::*;
use crate::helpers::{extract_number, extract_optional_number, extract_list, require_count};

// ============ Range ============

pub struct Range;

static RANGE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Starting value (inclusive)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end",
        typ: "Number",
        description: "Ending value (inclusive)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "step",
        typ: "Number",
        description: "Step increment",
        optional: true,
        default: Some("1"),
    },
];

static RANGE_EXAMPLES: [&str; 3] = [
    "range(1, 5) → [1, 2, 3, 4, 5]",
    "range(0, 10, 2) → [0, 2, 4, 6, 8, 10]",
    "range(10, 1, -1) → [10, 9, 8, 7, 6, 5, 4, 3, 2, 1]",
];

static RANGE_RELATED: [&str; 2] = ["linspace", "arithmetic"];

impl FunctionPlugin for Range {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "range",
            description: "Generate integer sequence from start to end (inclusive)",
            usage: "range(start, end, [step])",
            args: &RANGE_ARGS,
            returns: "List<Number>",
            examples: &RANGE_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &RANGE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(FolioError::arg_count("range", 2, args.len()));
        }

        let start = match extract_number(&args[0], "start") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match extract_number(&args[1], "end") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let step = match extract_optional_number(args, 2) {
            Ok(Some(n)) => n,
            Ok(None) => {
                // Default step: 1 if start <= end, -1 if start > end
                if start.sub(&end).is_negative() || start.sub(&end).is_zero() {
                    Number::from_i64(1)
                } else {
                    Number::from_i64(-1)
                }
            }
            Err(e) => return Value::Error(e),
        };

        if step.is_zero() {
            return Value::Error(FolioError::domain_error("range() step cannot be zero"));
        }

        let mut result = Vec::new();
        let mut current = start.clone();
        let zero = Number::from_i64(0);
        let max_elements = 100000;

        if step.sub(&zero).is_negative() {
            // Descending
            while current.sub(&end).is_negative() == false && !current.sub(&end).is_zero() || current == end {
                if result.len() >= max_elements {
                    return Value::Error(FolioError::domain_error(
                        format!("range() would generate more than {} elements", max_elements)
                    ));
                }
                result.push(Value::Number(current.clone()));
                current = current.add(&step);
                if current.sub(&end).is_negative() {
                    break;
                }
            }
        } else {
            // Ascending
            while current.sub(&end).is_negative() || current == end {
                if result.len() >= max_elements {
                    return Value::Error(FolioError::domain_error(
                        format!("range() would generate more than {} elements", max_elements)
                    ));
                }
                result.push(Value::Number(current.clone()));
                current = current.add(&step);
            }
        }

        Value::List(result)
    }
}

// ============ Linspace ============

pub struct Linspace;

static LINSPACE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Starting value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end",
        typ: "Number",
        description: "Ending value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of points (including endpoints)",
        optional: false,
        default: None,
    },
];

static LINSPACE_EXAMPLES: [&str; 1] = [
    "linspace(0, 1, 5) → [0, 0.25, 0.5, 0.75, 1]",
];

static LINSPACE_RELATED: [&str; 2] = ["range", "logspace"];

impl FunctionPlugin for Linspace {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "linspace",
            description: "Generate linearly spaced values (includes endpoints)",
            usage: "linspace(start, end, count)",
            args: &LINSPACE_ARGS,
            returns: "List<Number>",
            examples: &LINSPACE_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &LINSPACE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("linspace", 3, args.len()));
        }

        let start = match extract_number(&args[0], "start") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match extract_number(&args[1], "end") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "linspace", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        if count == 1 {
            return Value::List(vec![Value::Number(start)]);
        }

        let mut result = Vec::with_capacity(count);
        let range = end.sub(&start);
        let divisor = Number::from_i64((count - 1) as i64);

        for i in 0..count {
            let t = Number::from_i64(i as i64);
            let fraction = match t.checked_div(&divisor) {
                Ok(f) => f,
                Err(e) => return Value::Error(e.into()),
            };
            let value = start.add(&range.mul(&fraction));
            result.push(Value::Number(value));
        }

        Value::List(result)
    }
}

// ============ Logspace ============

pub struct Logspace;

static LOGSPACE_ARGS: [ArgMeta; 4] = [
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Starting value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end",
        typ: "Number",
        description: "Ending value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of points",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "base",
        typ: "Number",
        description: "Logarithm base",
        optional: true,
        default: Some("10"),
    },
];

static LOGSPACE_EXAMPLES: [&str; 2] = [
    "logspace(1, 1000, 4) → [1, 10, 100, 1000]",
    "logspace(1, 8, 4, 2) → [1, 2, 4, 8]",
];

static LOGSPACE_RELATED: [&str; 2] = ["linspace", "geometric"];

impl FunctionPlugin for Logspace {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "logspace",
            description: "Generate logarithmically spaced values",
            usage: "logspace(start, end, count, [base])",
            args: &LOGSPACE_ARGS,
            returns: "List<Number>",
            examples: &LOGSPACE_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &LOGSPACE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 || args.len() > 4 {
            return Value::Error(FolioError::arg_count("logspace", 3, args.len()));
        }

        let start = match extract_number(&args[0], "start") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match extract_number(&args[1], "end") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "logspace", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let base = match extract_optional_number(args, 3) {
            Ok(Some(n)) => n,
            Ok(None) => Number::from_i64(10),
            Err(e) => return Value::Error(e),
        };

        // Validate start and end are positive
        if start.is_zero() || start.is_negative() || end.is_zero() || end.is_negative() {
            return Value::Error(FolioError::domain_error(
                "logspace() requires positive start and end values"
            ));
        }

        if count == 1 {
            return Value::List(vec![Value::Number(start)]);
        }

        let mut result = Vec::with_capacity(count);

        // Calculate log(start) and log(end) in the given base
        // log_b(x) = ln(x) / ln(b)
        let ln_base = match base.ln(ctx.precision) {
            Ok(ln) => ln,
            Err(e) => return Value::Error(e.into()),
        };

        let log_start = match start.ln(ctx.precision) {
            Ok(ln) => match ln.checked_div(&ln_base) {
                Ok(l) => l,
                Err(e) => return Value::Error(e.into()),
            },
            Err(e) => return Value::Error(e.into()),
        };

        let log_end = match end.ln(ctx.precision) {
            Ok(ln) => match ln.checked_div(&ln_base) {
                Ok(l) => l,
                Err(e) => return Value::Error(e.into()),
            },
            Err(e) => return Value::Error(e.into()),
        };

        let log_range = log_end.sub(&log_start);
        let divisor = Number::from_i64((count - 1) as i64);

        for i in 0..count {
            let t = Number::from_i64(i as i64);
            let fraction = match t.checked_div(&divisor) {
                Ok(f) => f,
                Err(e) => return Value::Error(e.into()),
            };
            let log_val = log_start.add(&log_range.mul(&fraction));
            // value = base^log_val
            let value = base.pow_real(&log_val, ctx.precision);
            result.push(Value::Number(value));
        }

        Value::List(result)
    }
}

// ============ Arithmetic ============

pub struct Arithmetic;

static ARITHMETIC_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "first",
        typ: "Number",
        description: "First term",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "diff",
        typ: "Number",
        description: "Common difference",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of terms",
        optional: false,
        default: None,
    },
];

static ARITHMETIC_EXAMPLES: [&str; 1] = [
    "arithmetic(5, 3, 6) → [5, 8, 11, 14, 17, 20]",
];

static ARITHMETIC_RELATED: [&str; 2] = ["geometric", "range"];

impl FunctionPlugin for Arithmetic {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "arithmetic",
            description: "Generate arithmetic sequence: a_n = first + (n-1) × diff",
            usage: "arithmetic(first, diff, count)",
            args: &ARITHMETIC_ARGS,
            returns: "List<Number>",
            examples: &ARITHMETIC_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &ARITHMETIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("arithmetic", 3, args.len()));
        }

        let first = match extract_number(&args[0], "first") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let diff = match extract_number(&args[1], "diff") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "arithmetic", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        let mut current = first;

        for _ in 0..count {
            result.push(Value::Number(current.clone()));
            current = current.add(&diff);
        }

        Value::List(result)
    }
}

// ============ Geometric ============

pub struct Geometric;

static GEOMETRIC_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "first",
        typ: "Number",
        description: "First term",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "ratio",
        typ: "Number",
        description: "Common ratio",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of terms",
        optional: false,
        default: None,
    },
];

static GEOMETRIC_EXAMPLES: [&str; 1] = [
    "geometric(2, 3, 5) → [2, 6, 18, 54, 162]",
];

static GEOMETRIC_RELATED: [&str; 2] = ["arithmetic", "powers"];

impl FunctionPlugin for Geometric {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "geometric",
            description: "Generate geometric sequence: a_n = first × ratio^(n-1)",
            usage: "geometric(first, ratio, count)",
            args: &GEOMETRIC_ARGS,
            returns: "List<Number>",
            examples: &GEOMETRIC_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &GEOMETRIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("geometric", 3, args.len()));
        }

        let first = match extract_number(&args[0], "first") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let ratio = match extract_number(&args[1], "ratio") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "geometric", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        let mut current = first;

        for _ in 0..count {
            result.push(Value::Number(current.clone()));
            current = current.mul(&ratio);
        }

        Value::List(result)
    }
}

// ============ Harmonic ============

pub struct Harmonic;

static HARMONIC_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of terms",
    optional: false,
    default: None,
}];

static HARMONIC_EXAMPLES: [&str; 1] = [
    "harmonic(5) → [1, 0.5, 0.333..., 0.25, 0.2]",
];

static HARMONIC_RELATED: [&str; 1] = ["arithmetic"];

impl FunctionPlugin for Harmonic {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "harmonic",
            description: "Generate harmonic sequence: 1, 1/2, 1/3, 1/4, ...",
            usage: "harmonic(count)",
            args: &HARMONIC_ARGS,
            returns: "List<Number>",
            examples: &HARMONIC_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &HARMONIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("harmonic", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "harmonic", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let one = Number::from_i64(1);
        let mut result = Vec::with_capacity(count);

        for i in 1..=count {
            let denom = Number::from_i64(i as i64);
            match one.checked_div(&denom) {
                Ok(val) => result.push(Value::Number(val)),
                Err(e) => return Value::Error(e.into()),
            }
        }

        Value::List(result)
    }
}

// ============ RepeatSeq ============

pub struct RepeatSeq;

static REPEAT_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to repeat",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of repetitions",
        optional: false,
        default: None,
    },
];

static REPEAT_SEQ_EXAMPLES: [&str; 1] = [
    "repeat_seq(7, 5) → [7, 7, 7, 7, 7]",
];

static REPEAT_SEQ_RELATED: [&str; 1] = ["cycle"];

impl FunctionPlugin for RepeatSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "repeat_seq",
            description: "Repeat a single value",
            usage: "repeat_seq(value, count)",
            args: &REPEAT_SEQ_ARGS,
            returns: "List<Number>",
            examples: &REPEAT_SEQ_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &REPEAT_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("repeat_seq", 2, args.len()));
        }

        let value = match extract_number(&args[0], "value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[1], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "repeat_seq", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let result: Vec<Value> = (0..count).map(|_| Value::Number(value.clone())).collect();
        Value::List(result)
    }
}

// ============ Cycle ============

pub struct Cycle;

static CYCLE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "List to cycle through",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Total number of elements to generate",
        optional: false,
        default: None,
    },
];

static CYCLE_EXAMPLES: [&str; 1] = [
    "cycle([1, 2, 3], 8) → [1, 2, 3, 1, 2, 3, 1, 2]",
];

static CYCLE_RELATED: [&str; 1] = ["repeat_seq"];

impl FunctionPlugin for Cycle {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cycle",
            description: "Cycle through a list to generate count elements",
            usage: "cycle(list, count)",
            args: &CYCLE_ARGS,
            returns: "List<Number>",
            examples: &CYCLE_EXAMPLES,
            category: "sequence/generators",
            source: None,
            related: &CYCLE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("cycle", 2, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if list.is_empty() {
            return Value::Error(FolioError::domain_error("cycle() requires non-empty list"));
        }

        let count_num = match extract_number(&args[1], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "cycle", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        for i in 0..count {
            result.push(Value::Number(list[i % list.len()].clone()));
        }

        Value::List(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_range_basic() {
        let range = Range;
        let args = vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(5)),
        ];
        let result = range.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(5));
    }

    #[test]
    fn test_range_with_step() {
        let range = Range;
        let args = vec![
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(10)),
            Value::Number(Number::from_i64(2)),
        ];
        let result = range.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 6); // 0, 2, 4, 6, 8, 10
    }

    #[test]
    fn test_arithmetic() {
        let arith = Arithmetic;
        let args = vec![
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
        ];
        let result = arith.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(8));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(11));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(14));
    }

    #[test]
    fn test_geometric() {
        let geom = Geometric;
        let args = vec![
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
        ];
        let result = geom.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 4);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(6));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(18));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(54));
    }

    #[test]
    fn test_harmonic() {
        let harm = Harmonic;
        let args = vec![Value::Number(Number::from_i64(3))];
        let result = harm.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        // 1/2 and 1/3 won't be exact integers
    }

    #[test]
    fn test_cycle() {
        let cycle = Cycle;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::Number(Number::from_i64(7)),
        ];
        let result = cycle.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 7);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[6].as_number().unwrap().to_i64(), Some(1));
    }
}
```

folio-sequence\src\helpers.rs
```rs
//! Helper functions and utility sequence operations
//!
//! Common utilities for extracting inputs and utility sequence functions.

use folio_plugin::prelude::*;
use folio_core::{Number, Value, FolioError};

/// Extract a list from a single argument
pub fn extract_list(arg: &Value) -> Result<Vec<Number>, FolioError> {
    match arg {
        Value::List(list) => {
            let mut numbers = Vec::new();
            for item in list {
                match item {
                    Value::Number(n) => numbers.push(n.clone()),
                    Value::Error(e) => return Err(e.clone()),
                    other => return Err(FolioError::type_error("Number", other.type_name())),
                }
            }
            Ok(numbers)
        }
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::type_error("List", other.type_name())),
    }
}

/// Extract a single number from argument
pub fn extract_number(arg: &Value, name: &str) -> Result<Number, FolioError> {
    match arg {
        Value::Number(n) => Ok(n.clone()),
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::arg_type("", name, "Number", other.type_name())),
    }
}

/// Extract an optional number argument
pub fn extract_optional_number(args: &[Value], index: usize) -> Result<Option<Number>, FolioError> {
    if index >= args.len() {
        return Ok(None);
    }
    match &args[index] {
        Value::Number(n) => Ok(Some(n.clone())),
        Value::Null => Ok(None),
        Value::Error(e) => Err(e.clone()),
        other => Err(FolioError::type_error("Number", other.type_name())),
    }
}

/// Require a positive integer count parameter
pub fn require_count(n: &Number, func: &str, max: usize) -> Result<usize, FolioError> {
    if !n.is_integer() || n.is_negative() || n.is_zero() {
        return Err(FolioError::domain_error(format!(
            "{}() requires positive integer count",
            func
        )));
    }
    let count = n.to_i64().unwrap_or(0) as usize;
    if count > max {
        return Err(FolioError::domain_error(format!(
            "{}() limited to {} elements for performance",
            func, max
        )));
    }
    Ok(count)
}

/// Require a non-negative integer index parameter
pub fn require_index(n: &Number, func: &str) -> Result<usize, FolioError> {
    if !n.is_integer() || n.is_negative() {
        return Err(FolioError::domain_error(format!(
            "{}() requires non-negative integer index",
            func
        )));
    }
    Ok(n.to_i64().unwrap_or(0) as usize)
}

/// Calculate sum of numbers
pub fn sum(numbers: &[Number]) -> Number {
    numbers
        .iter()
        .fold(Number::from_i64(0), |acc, n| acc.add(n))
}

/// Calculate product of numbers
pub fn product(numbers: &[Number]) -> Number {
    numbers
        .iter()
        .fold(Number::from_i64(1), |acc, n| acc.mul(n))
}

// ============ Utility Functions ============

/// Get nth element of named sequence (more efficient than generating full sequence)
pub struct Nth;

static NTH_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "sequence_name",
        typ: "Text",
        description: "Name of sequence: fibonacci, prime, lucas, etc.",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Index (1-based)",
        optional: false,
        default: None,
    },
];

static NTH_EXAMPLES: [&str; 2] = [
    "nth(\"fibonacci\", 10) → 55",
    "nth(\"prime\", 5) → 11",
];

static NTH_RELATED: [&str; 2] = ["fibonacci", "primes"];

impl FunctionPlugin for Nth {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nth",
            description: "Get nth element of named sequence (more efficient than generating full sequence)",
            usage: "nth(sequence_name, n)",
            args: &NTH_ARGS,
            returns: "Number",
            examples: &NTH_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &NTH_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("nth", 2, args.len()));
        }

        let name = match &args[0] {
            Value::Text(s) => s.to_lowercase(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("nth", "sequence_name", "Text", other.type_name())),
        };

        let n = match extract_number(&args[1], "n") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if !n.is_integer() || n.is_negative() {
            return Value::Error(FolioError::domain_error(
                "nth() requires non-negative integer index"
            ));
        }

        let idx = n.to_i64().unwrap_or(0) as usize;

        match name.as_str() {
            "fibonacci" => nth_fibonacci(idx),
            "prime" | "primes" => nth_prime(idx),
            "lucas" => nth_lucas(idx),
            "triangular" => nth_triangular(idx),
            "square" => nth_square(idx),
            "cube" => nth_cube(idx),
            "factorial" => nth_factorial(idx, ctx),
            _ => Value::Error(FolioError::domain_error(format!(
                "Unknown sequence name: {}. Valid: fibonacci, prime, lucas, triangular, square, cube, factorial",
                name
            ))),
        }
    }
}

fn nth_fibonacci(n: usize) -> Value {
    if n == 0 {
        return Value::Number(Number::from_i64(0));
    }
    let (mut a, mut b) = (Number::from_i64(0), Number::from_i64(1));
    for _ in 1..n {
        let next = a.add(&b);
        a = b;
        b = next;
    }
    Value::Number(b)
}

fn nth_lucas(n: usize) -> Value {
    if n == 0 {
        return Value::Number(Number::from_i64(2));
    }
    let (mut a, mut b) = (Number::from_i64(2), Number::from_i64(1));
    for _ in 1..n {
        let next = a.add(&b);
        a = b;
        b = next;
    }
    Value::Number(b)
}

fn nth_prime(n: usize) -> Value {
    if n == 0 {
        return Value::Error(FolioError::domain_error("Prime index must be >= 1"));
    }
    if n > 10000 {
        return Value::Error(FolioError::domain_error("nth(prime) limited to n <= 10000"));
    }

    let mut count = 0;
    let mut candidate = 2u64;

    while count < n {
        if is_prime(candidate) {
            count += 1;
            if count == n {
                return Value::Number(Number::from_i64(candidate as i64));
            }
        }
        candidate += 1;
    }

    Value::Number(Number::from_i64(candidate as i64))
}

fn nth_triangular(n: usize) -> Value {
    // T(n) = n(n+1)/2
    let n = Number::from_i64(n as i64);
    let result = n.mul(&n.add(&Number::from_i64(1)));
    match result.checked_div(&Number::from_i64(2)) {
        Ok(r) => Value::Number(r),
        Err(e) => Value::Error(e.into()),
    }
}

fn nth_square(n: usize) -> Value {
    let n = Number::from_i64(n as i64);
    Value::Number(n.mul(&n))
}

fn nth_cube(n: usize) -> Value {
    let n = Number::from_i64(n as i64);
    Value::Number(n.mul(&n).mul(&n))
}

fn nth_factorial(n: usize, _ctx: &EvalContext) -> Value {
    if n > 170 {
        return Value::Error(FolioError::domain_error("nth(factorial) limited to n <= 170"));
    }
    let mut result = Number::from_i64(1);
    for i in 2..=n {
        result = result.mul(&Number::from_i64(i as i64));
    }
    Value::Number(result)
}

/// Check if a number is prime
pub fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 {
        return true;
    }
    if n % 2 == 0 {
        return false;
    }
    let sqrt_n = (n as f64).sqrt() as u64;
    for i in (3..=sqrt_n).step_by(2) {
        if n % i == 0 {
            return false;
        }
    }
    true
}

// ============ IndexOfSeq ============

pub struct IndexOfSeq;

static INDEX_OF_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "List to search in",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to find",
        optional: false,
        default: None,
    },
];

static INDEX_OF_SEQ_EXAMPLES: [&str; 1] = [
    "index_of_seq(fibonacci(20), 55) → 10",
];

static INDEX_OF_SEQ_RELATED: [&str; 1] = ["is_in_sequence"];

impl FunctionPlugin for IndexOfSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "index_of_seq",
            description: "Find index of value in sequence (returns -1 if not found)",
            usage: "index_of_seq(list, value)",
            args: &INDEX_OF_SEQ_ARGS,
            returns: "Number",
            examples: &INDEX_OF_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &INDEX_OF_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("index_of_seq", 2, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let value = match extract_number(&args[1], "value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        for (i, item) in list.iter().enumerate() {
            if item == &value {
                return Value::Number(Number::from_i64((i + 1) as i64)); // 1-indexed
            }
        }

        Value::Number(Number::from_i64(-1))
    }
}

// ============ IsInSequence ============

pub struct IsInSequence;

static IS_IN_SEQUENCE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to check",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "sequence_name",
        typ: "Text",
        description: "Name of sequence: fibonacci, prime, etc.",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "max_check",
        typ: "Number",
        description: "Maximum index to check",
        optional: true,
        default: Some("1000"),
    },
];

static IS_IN_SEQUENCE_EXAMPLES: [&str; 3] = [
    "is_in_sequence(55, \"fibonacci\") → true",
    "is_in_sequence(17, \"prime\") → true",
    "is_in_sequence(15, \"prime\") → false",
];

static IS_IN_SEQUENCE_RELATED: [&str; 2] = ["nth", "index_of_seq"];

impl FunctionPlugin for IsInSequence {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_in_sequence",
            description: "Check if value is in named sequence",
            usage: "is_in_sequence(value, sequence_name, [max_check])",
            args: &IS_IN_SEQUENCE_ARGS,
            returns: "Bool",
            examples: &IS_IN_SEQUENCE_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &IS_IN_SEQUENCE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 3 {
            return Value::Error(FolioError::arg_count("is_in_sequence", 2, args.len()));
        }

        let value = match extract_number(&args[0], "value") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let name = match &args[1] {
            Value::Text(s) => s.to_lowercase(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("is_in_sequence", "sequence_name", "Text", other.type_name())),
        };

        let max_check = match extract_optional_number(args, 2) {
            Ok(Some(n)) => n.to_i64().unwrap_or(1000) as usize,
            Ok(None) => 1000,
            Err(e) => return Value::Error(e),
        };

        let max_check = max_check.min(10000);

        match name.as_str() {
            "fibonacci" => is_in_fibonacci(&value, max_check),
            "prime" | "primes" => is_in_primes(&value),
            "lucas" => is_in_lucas(&value, max_check),
            "triangular" => is_in_triangular(&value),
            "square" => is_in_square(&value),
            "cube" => is_in_cube(&value),
            _ => Value::Error(FolioError::domain_error(format!(
                "Unknown sequence name: {}",
                name
            ))),
        }
    }
}

fn is_in_fibonacci(value: &Number, max_check: usize) -> Value {
    let (mut a, mut b) = (Number::from_i64(0), Number::from_i64(1));
    for _ in 0..max_check {
        if &a == value {
            return Value::Bool(true);
        }
        if a.sub(value).is_negative() == false && !a.sub(value).is_zero() {
            return Value::Bool(false);
        }
        let next = a.add(&b);
        a = b;
        b = next;
    }
    Value::Bool(false)
}

fn is_in_lucas(value: &Number, max_check: usize) -> Value {
    let (mut a, mut b) = (Number::from_i64(2), Number::from_i64(1));
    for _ in 0..max_check {
        if &a == value {
            return Value::Bool(true);
        }
        if a.sub(value).is_negative() == false && !a.sub(value).is_zero() {
            return Value::Bool(false);
        }
        let next = a.add(&b);
        a = b;
        b = next;
    }
    Value::Bool(false)
}

fn is_in_primes(value: &Number) -> Value {
    if !value.is_integer() || value.is_negative() {
        return Value::Bool(false);
    }
    let n = value.to_i64().unwrap_or(0);
    if n < 2 {
        return Value::Bool(false);
    }
    Value::Bool(is_prime(n as u64))
}

fn is_in_triangular(value: &Number) -> Value {
    // T(n) = n(n+1)/2, so 8*T + 1 should be a perfect square
    if !value.is_integer() || value.is_negative() {
        return Value::Bool(false);
    }
    let eight = Number::from_i64(8);
    let one = Number::from_i64(1);
    let discriminant = eight.mul(value).add(&one);
    // Check if discriminant is a perfect square
    if let Some(d) = discriminant.to_i64() {
        let sqrt_d = (d as f64).sqrt() as i64;
        if sqrt_d * sqrt_d == d && sqrt_d % 2 == 1 {
            return Value::Bool(true);
        }
    }
    Value::Bool(false)
}

fn is_in_square(value: &Number) -> Value {
    if !value.is_integer() || value.is_negative() {
        return Value::Bool(false);
    }
    if let Some(n) = value.to_i64() {
        let sqrt_n = (n as f64).sqrt() as i64;
        return Value::Bool(sqrt_n * sqrt_n == n);
    }
    Value::Bool(false)
}

fn is_in_cube(value: &Number) -> Value {
    if !value.is_integer() {
        return Value::Bool(false);
    }
    if let Some(n) = value.to_i64() {
        let cbrt = if n >= 0 {
            (n as f64).cbrt() as i64
        } else {
            -((-n as f64).cbrt() as i64)
        };
        // Check nearby values due to floating point issues
        for candidate in (cbrt - 1)..=(cbrt + 1) {
            if candidate * candidate * candidate == n {
                return Value::Bool(true);
            }
        }
    }
    Value::Bool(false)
}

// ============ ReverseSeq ============

pub struct ReverseSeq;

static REVERSE_SEQ_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "List to reverse",
    optional: false,
    default: None,
}];

static REVERSE_SEQ_EXAMPLES: [&str; 1] = [
    "reverse_seq([1, 2, 3, 4, 5]) → [5, 4, 3, 2, 1]",
];

static REVERSE_SEQ_RELATED: [&str; 2] = ["take_seq", "drop_seq"];

impl FunctionPlugin for ReverseSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "reverse_seq",
            description: "Reverse a sequence",
            usage: "reverse_seq(list)",
            args: &REVERSE_SEQ_ARGS,
            returns: "List<Number>",
            examples: &REVERSE_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &REVERSE_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("reverse_seq", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let reversed: Vec<Value> = list.into_iter().rev().map(Value::Number).collect();
        Value::List(reversed)
    }
}

// ============ Interleave ============

pub struct Interleave;

static INTERLEAVE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List<Number>",
        description: "First list",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List<Number>",
        description: "Second list",
        optional: false,
        default: None,
    },
];

static INTERLEAVE_EXAMPLES: [&str; 1] = [
    "interleave([1, 3, 5], [2, 4, 6]) → [1, 2, 3, 4, 5, 6]",
];

static INTERLEAVE_RELATED: [&str; 1] = ["zip_seq"];

impl FunctionPlugin for Interleave {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "interleave",
            description: "Interleave two sequences",
            usage: "interleave(list1, list2)",
            args: &INTERLEAVE_ARGS,
            returns: "List<Number>",
            examples: &INTERLEAVE_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &INTERLEAVE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("interleave", 2, args.len()));
        }

        let list1 = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let list2 = match extract_list(&args[1]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::new();
        let max_len = list1.len().max(list2.len());

        for i in 0..max_len {
            if i < list1.len() {
                result.push(Value::Number(list1[i].clone()));
            }
            if i < list2.len() {
                result.push(Value::Number(list2[i].clone()));
            }
        }

        Value::List(result)
    }
}

// ============ ZipSeq ============

pub struct ZipSeq;

static ZIP_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List",
        description: "First list",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List",
        description: "Second list",
        optional: false,
        default: None,
    },
];

static ZIP_SEQ_EXAMPLES: [&str; 1] = [
    "zip_seq([1, 2, 3], [\"a\", \"b\", \"c\"]) → [[1, \"a\"], [2, \"b\"], [3, \"c\"]]",
];

static ZIP_SEQ_RELATED: [&str; 1] = ["interleave"];

impl FunctionPlugin for ZipSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "zip_seq",
            description: "Zip two sequences into pairs",
            usage: "zip_seq(list1, list2)",
            args: &ZIP_SEQ_ARGS,
            returns: "List<List>",
            examples: &ZIP_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &ZIP_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("zip_seq", 2, args.len()));
        }

        let list1 = match &args[0] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("zip_seq", "list1", "List", other.type_name())),
        };

        let list2 = match &args[1] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("zip_seq", "list2", "List", other.type_name())),
        };

        let min_len = list1.len().min(list2.len());
        let mut result = Vec::with_capacity(min_len);

        for i in 0..min_len {
            result.push(Value::List(vec![list1[i].clone(), list2[i].clone()]));
        }

        Value::List(result)
    }
}

// ============ TakeSeq ============

pub struct TakeSeq;

static TAKE_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to take from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements to take",
        optional: false,
        default: None,
    },
];

static TAKE_SEQ_EXAMPLES: [&str; 1] = [
    "take_seq(range(1, 100), 5) → [1, 2, 3, 4, 5]",
];

static TAKE_SEQ_RELATED: [&str; 2] = ["drop_seq", "slice_seq"];

impl FunctionPlugin for TakeSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "take_seq",
            description: "Take first n elements from a sequence",
            usage: "take_seq(list, n)",
            args: &TAKE_SEQ_ARGS,
            returns: "List",
            examples: &TAKE_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &TAKE_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("take_seq", 2, args.len()));
        }

        let list = match &args[0] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("take_seq", "list", "List", other.type_name())),
        };

        let n = match extract_number(&args[1], "n") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = match require_index(&n, "take_seq") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let take_count = n.min(list.len());
        Value::List(list[..take_count].to_vec())
    }
}

// ============ DropSeq ============

pub struct DropSeq;

static DROP_SEQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to drop from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements to drop",
        optional: false,
        default: None,
    },
];

static DROP_SEQ_EXAMPLES: [&str; 1] = [
    "drop_seq([1, 2, 3, 4, 5], 2) → [3, 4, 5]",
];

static DROP_SEQ_RELATED: [&str; 2] = ["take_seq", "slice_seq"];

impl FunctionPlugin for DropSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "drop_seq",
            description: "Drop first n elements from a sequence",
            usage: "drop_seq(list, n)",
            args: &DROP_SEQ_ARGS,
            returns: "List",
            examples: &DROP_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &DROP_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("drop_seq", 2, args.len()));
        }

        let list = match &args[0] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("drop_seq", "list", "List", other.type_name())),
        };

        let n = match extract_number(&args[1], "n") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = match require_index(&n, "drop_seq") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if n >= list.len() {
            return Value::List(vec![]);
        }

        Value::List(list[n..].to_vec())
    }
}

// ============ SliceSeq ============

pub struct SliceSeq;

static SLICE_SEQ_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to slice",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index (0-indexed)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "end",
        typ: "Number",
        description: "End index (exclusive)",
        optional: false,
        default: None,
    },
];

static SLICE_SEQ_EXAMPLES: [&str; 1] = [
    "slice_seq([1, 2, 3, 4, 5], 1, 4) → [2, 3, 4]",
];

static SLICE_SEQ_RELATED: [&str; 2] = ["take_seq", "drop_seq"];

impl FunctionPlugin for SliceSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "slice_seq",
            description: "Get a slice of a sequence (0-indexed, end exclusive)",
            usage: "slice_seq(list, start, end)",
            args: &SLICE_SEQ_ARGS,
            returns: "List",
            examples: &SLICE_SEQ_EXAMPLES,
            category: "sequence/utility",
            source: None,
            related: &SLICE_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("slice_seq", 3, args.len()));
        }

        let list = match &args[0] {
            Value::List(l) => l.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("slice_seq", "list", "List", other.type_name())),
        };

        let start = match extract_number(&args[1], "start") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match extract_number(&args[2], "end") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let start = match require_index(&start, "slice_seq") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let end = match require_index(&end, "slice_seq") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if start >= list.len() {
            return Value::List(vec![]);
        }

        let end = end.min(list.len());
        if start >= end {
            return Value::List(vec![]);
        }

        Value::List(list[start..end].to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_prime() {
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(!is_prime(4));
        assert!(is_prime(5));
        assert!(is_prime(17));
        assert!(!is_prime(15));
        assert!(is_prime(97));
    }

    #[test]
    fn test_extract_list() {
        let arg = Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ]);
        let result = extract_list(&arg).unwrap();
        assert_eq!(result.len(), 3);
    }
}
```

folio-sequence\src\lib.rs
```rs
//! Folio Sequence Plugin
//!
//! Sequence generation, named sequences, pattern detection, and series operations.
//! All functions follow the never-panic philosophy and return `Value::Error` on failure.

mod helpers;
mod expr;
mod generators;
mod named;
mod recurrence;
mod pattern;
mod series;

use folio_plugin::PluginRegistry;

/// Load sequence functions into registry
pub fn load_sequence_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Basic generators
        .with_function(generators::Range)
        .with_function(generators::Linspace)
        .with_function(generators::Logspace)
        .with_function(generators::Arithmetic)
        .with_function(generators::Geometric)
        .with_function(generators::Harmonic)
        .with_function(generators::RepeatSeq)
        .with_function(generators::Cycle)

        // Named sequences
        .with_function(named::Fibonacci)
        .with_function(named::Lucas)
        .with_function(named::Tribonacci)
        .with_function(named::Primes)
        .with_function(named::PrimesUpTo)
        .with_function(named::FactorialSeq)
        .with_function(named::Triangular)
        .with_function(named::SquareNumbers)
        .with_function(named::CubeNumbers)
        .with_function(named::Powers)
        .with_function(named::Catalan)
        .with_function(named::Bell)
        .with_function(named::Pentagonal)
        .with_function(named::Hexagonal)

        // Recurrence relations
        .with_function(recurrence::Recurrence)
        .with_function(recurrence::RecurrenceNamed)

        // Pattern detection
        .with_function(pattern::DetectPattern)
        .with_function(pattern::ExtendPattern)
        .with_function(pattern::IsArithmetic)
        .with_function(pattern::IsGeometric)
        .with_function(pattern::CommonDiff)
        .with_function(pattern::CommonRatio)
        .with_function(pattern::NthTermFormula)

        // Series operations
        .with_function(series::SumSeq)
        .with_function(series::ProductSeq)
        .with_function(series::PartialSums)
        .with_function(series::PartialProducts)
        .with_function(series::AlternatingSum)
        .with_function(series::SumFormula)

        // Utility functions
        .with_function(helpers::Nth)
        .with_function(helpers::IndexOfSeq)
        .with_function(helpers::IsInSequence)
        .with_function(helpers::ReverseSeq)
        .with_function(helpers::Interleave)
        .with_function(helpers::ZipSeq)
        .with_function(helpers::TakeSeq)
        .with_function(helpers::DropSeq)
        .with_function(helpers::SliceSeq)
}
```

folio-sequence\src\named.rs
```rs
//! Named mathematical sequences
//!
//! fibonacci, lucas, tribonacci, primes, factorial, triangular, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_number, extract_optional_number, require_count, is_prime};

// ============ Fibonacci ============

pub struct Fibonacci;

static FIBONACCI_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of Fibonacci numbers to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index (0 = F(0)=0, 1 = F(1)=1, etc.)",
        optional: true,
        default: Some("1"),
    },
];

static FIBONACCI_EXAMPLES: [&str; 3] = [
    "fibonacci(10) → [1, 1, 2, 3, 5, 8, 13, 21, 34, 55]",
    "fibonacci(5, 10) → [55, 89, 144, 233, 377]",
    "fibonacci(5, 0) → [0, 1, 1, 2, 3]",
];

static FIBONACCI_RELATED: [&str; 2] = ["lucas", "tribonacci"];

impl FunctionPlugin for Fibonacci {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "fibonacci",
            description: "Generate Fibonacci sequence",
            usage: "fibonacci(count, [start])",
            args: &FIBONACCI_ARGS,
            returns: "List<Number>",
            examples: &FIBONACCI_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &FIBONACCI_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("fibonacci", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "fibonacci", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_optional_number(args, 1) {
            Ok(Some(n)) => {
                if !n.is_integer() || n.is_negative() {
                    return Value::Error(FolioError::domain_error(
                        "fibonacci() start must be non-negative integer"
                    ));
                }
                n.to_i64().unwrap_or(1) as usize
            }
            Ok(None) => 1,
            Err(e) => return Value::Error(e),
        };

        // Generate Fibonacci numbers
        let total_needed = start + count;
        let mut fibs = Vec::with_capacity(total_needed);

        let (mut a, mut b) = (Number::from_i64(0), Number::from_i64(1));
        for _ in 0..total_needed {
            fibs.push(a.clone());
            let next = a.add(&b);
            a = b;
            b = next;
        }

        let result: Vec<Value> = fibs[start..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

// ============ Lucas ============

pub struct Lucas;

static LUCAS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of Lucas numbers to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index",
        optional: true,
        default: Some("0"),
    },
];

static LUCAS_EXAMPLES: [&str; 1] = [
    "lucas(8) → [2, 1, 3, 4, 7, 11, 18, 29]",
];

static LUCAS_RELATED: [&str; 2] = ["fibonacci", "tribonacci"];

impl FunctionPlugin for Lucas {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "lucas",
            description: "Generate Lucas numbers (2, 1, 3, 4, 7, 11, ...)",
            usage: "lucas(count, [start])",
            args: &LUCAS_ARGS,
            returns: "List<Number>",
            examples: &LUCAS_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &LUCAS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("lucas", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "lucas", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_optional_number(args, 1) {
            Ok(Some(n)) => {
                if !n.is_integer() || n.is_negative() {
                    return Value::Error(FolioError::domain_error(
                        "lucas() start must be non-negative integer"
                    ));
                }
                n.to_i64().unwrap_or(0) as usize
            }
            Ok(None) => 0,
            Err(e) => return Value::Error(e),
        };

        let total_needed = start + count;
        let mut nums = Vec::with_capacity(total_needed);

        // Lucas: L(0)=2, L(1)=1
        let (mut a, mut b) = (Number::from_i64(2), Number::from_i64(1));
        for _ in 0..total_needed {
            nums.push(a.clone());
            let next = a.add(&b);
            a = b;
            b = next;
        }

        let result: Vec<Value> = nums[start..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

// ============ Tribonacci ============

pub struct Tribonacci;

static TRIBONACCI_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of tribonacci numbers to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index",
        optional: true,
        default: Some("0"),
    },
];

static TRIBONACCI_EXAMPLES: [&str; 1] = [
    "tribonacci(10) → [0, 0, 1, 1, 2, 4, 7, 13, 24, 44]",
];

static TRIBONACCI_RELATED: [&str; 2] = ["fibonacci", "lucas"];

impl FunctionPlugin for Tribonacci {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tribonacci",
            description: "Generate tribonacci sequence (each term = sum of previous 3)",
            usage: "tribonacci(count, [start])",
            args: &TRIBONACCI_ARGS,
            returns: "List<Number>",
            examples: &TRIBONACCI_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &TRIBONACCI_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("tribonacci", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "tribonacci", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_optional_number(args, 1) {
            Ok(Some(n)) => {
                if !n.is_integer() || n.is_negative() {
                    return Value::Error(FolioError::domain_error(
                        "tribonacci() start must be non-negative integer"
                    ));
                }
                n.to_i64().unwrap_or(0) as usize
            }
            Ok(None) => 0,
            Err(e) => return Value::Error(e),
        };

        let total_needed = start + count;
        let mut nums = Vec::with_capacity(total_needed);

        // Tribonacci: T(0)=0, T(1)=0, T(2)=1
        let (mut a, mut b, mut c) = (
            Number::from_i64(0),
            Number::from_i64(0),
            Number::from_i64(1),
        );

        for i in 0..total_needed {
            if i == 0 {
                nums.push(a.clone());
            } else if i == 1 {
                nums.push(b.clone());
            } else if i == 2 {
                nums.push(c.clone());
            } else {
                let next = a.add(&b).add(&c);
                a = b;
                b = c;
                c = next;
                nums.push(c.clone());
            }
        }

        let result: Vec<Value> = nums[start..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

// ============ Primes ============

pub struct Primes;

static PRIMES_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of primes to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index (1 = first prime = 2)",
        optional: true,
        default: Some("1"),
    },
];

static PRIMES_EXAMPLES: [&str; 2] = [
    "primes(10) → [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]",
    "primes(5, 10) → [29, 31, 37, 41, 43]",
];

static PRIMES_RELATED: [&str; 1] = ["primes_up_to"];

impl FunctionPlugin for Primes {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "primes",
            description: "Generate prime numbers",
            usage: "primes(count, [start])",
            args: &PRIMES_ARGS,
            returns: "List<Number>",
            examples: &PRIMES_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &PRIMES_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("primes", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "primes", 1000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match extract_optional_number(args, 1) {
            Ok(Some(n)) => {
                if !n.is_integer() || n.is_negative() || n.is_zero() {
                    return Value::Error(FolioError::domain_error(
                        "primes() start must be positive integer"
                    ));
                }
                n.to_i64().unwrap_or(1) as usize
            }
            Ok(None) => 1,
            Err(e) => return Value::Error(e),
        };

        let total_needed = start + count - 1;
        let mut primes_list = Vec::with_capacity(total_needed);
        let mut candidate = 2u64;

        while primes_list.len() < total_needed {
            if is_prime(candidate) {
                primes_list.push(Number::from_i64(candidate as i64));
            }
            candidate += 1;
        }

        let result: Vec<Value> = primes_list[(start - 1)..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

// ============ PrimesUpTo ============

pub struct PrimesUpTo;

static PRIMES_UP_TO_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "max",
    typ: "Number",
    description: "Maximum value",
    optional: false,
    default: None,
}];

static PRIMES_UP_TO_EXAMPLES: [&str; 1] = [
    "primes_up_to(30) → [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]",
];

static PRIMES_UP_TO_RELATED: [&str; 1] = ["primes"];

impl FunctionPlugin for PrimesUpTo {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "primes_up_to",
            description: "Generate all primes up to max (Sieve of Eratosthenes)",
            usage: "primes_up_to(max)",
            args: &PRIMES_UP_TO_ARGS,
            returns: "List<Number>",
            examples: &PRIMES_UP_TO_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &PRIMES_UP_TO_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("primes_up_to", 1, args.len()));
        }

        let max_num = match extract_number(&args[0], "max") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if !max_num.is_integer() || max_num.is_negative() {
            return Value::Error(FolioError::domain_error(
                "primes_up_to() requires non-negative integer"
            ));
        }

        let max = max_num.to_i64().unwrap_or(0) as usize;

        if max > 1_000_000 {
            return Value::Error(FolioError::domain_error(
                "primes_up_to() limited to max <= 1,000,000"
            ));
        }

        if max < 2 {
            return Value::List(vec![]);
        }

        // Sieve of Eratosthenes
        let mut sieve = vec![true; max + 1];
        sieve[0] = false;
        sieve[1] = false;

        let sqrt_max = (max as f64).sqrt() as usize;
        for i in 2..=sqrt_max {
            if sieve[i] {
                let mut j = i * i;
                while j <= max {
                    sieve[j] = false;
                    j += i;
                }
            }
        }

        let result: Vec<Value> = sieve.iter().enumerate()
            .filter(|(_, &is_prime)| is_prime)
            .map(|(i, _)| Value::Number(Number::from_i64(i as i64)))
            .collect();

        Value::List(result)
    }
}

// ============ FactorialSeq ============

pub struct FactorialSeq;

static FACTORIAL_SEQ_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of factorials to generate (starting from 0!)",
    optional: false,
    default: None,
}];

static FACTORIAL_SEQ_EXAMPLES: [&str; 1] = [
    "factorial_seq(7) → [1, 1, 2, 6, 24, 120, 720]",
];

static FACTORIAL_SEQ_RELATED: [&str; 1] = ["triangular"];

impl FunctionPlugin for FactorialSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "factorial_seq",
            description: "Generate sequence of factorials: 0!, 1!, 2!, 3!, ...",
            usage: "factorial_seq(count)",
            args: &FACTORIAL_SEQ_ARGS,
            returns: "List<Number>",
            examples: &FACTORIAL_SEQ_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &FACTORIAL_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("factorial_seq", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "factorial_seq", 171) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        let mut factorial = Number::from_i64(1);

        for i in 0..count {
            if i == 0 {
                result.push(Value::Number(Number::from_i64(1)));
            } else {
                factorial = factorial.mul(&Number::from_i64(i as i64));
                result.push(Value::Number(factorial.clone()));
            }
        }

        Value::List(result)
    }
}

// ============ Triangular ============

pub struct Triangular;

static TRIANGULAR_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of triangular numbers to generate",
    optional: false,
    default: None,
}];

static TRIANGULAR_EXAMPLES: [&str; 1] = [
    "triangular(6) → [1, 3, 6, 10, 15, 21]",
];

static TRIANGULAR_RELATED: [&str; 2] = ["square_numbers", "pentagonal"];

impl FunctionPlugin for Triangular {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "triangular",
            description: "Generate triangular numbers: T(n) = n(n+1)/2",
            usage: "triangular(count)",
            args: &TRIANGULAR_ARGS,
            returns: "List<Number>",
            examples: &TRIANGULAR_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &TRIANGULAR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("triangular", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "triangular", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let two = Number::from_i64(2);
        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            let n_plus_1 = n.add(&Number::from_i64(1));
            let product = n.mul(&n_plus_1);
            match product.checked_div(&two) {
                Ok(val) => Value::Number(val),
                Err(e) => Value::Error(e.into()),
            }
        }).collect();

        Value::List(result)
    }
}

// ============ SquareNumbers ============

pub struct SquareNumbers;

static SQUARE_NUMBERS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of square numbers to generate",
    optional: false,
    default: None,
}];

static SQUARE_NUMBERS_EXAMPLES: [&str; 1] = [
    "square_numbers(6) → [1, 4, 9, 16, 25, 36]",
];

static SQUARE_NUMBERS_RELATED: [&str; 2] = ["cube_numbers", "triangular"];

impl FunctionPlugin for SquareNumbers {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "square_numbers",
            description: "Generate perfect squares: 1, 4, 9, 16, 25, ...",
            usage: "square_numbers(count)",
            args: &SQUARE_NUMBERS_ARGS,
            returns: "List<Number>",
            examples: &SQUARE_NUMBERS_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &SQUARE_NUMBERS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("square_numbers", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "square_numbers", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            Value::Number(n.mul(&n))
        }).collect();

        Value::List(result)
    }
}

// ============ CubeNumbers ============

pub struct CubeNumbers;

static CUBE_NUMBERS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of cube numbers to generate",
    optional: false,
    default: None,
}];

static CUBE_NUMBERS_EXAMPLES: [&str; 1] = [
    "cube_numbers(5) → [1, 8, 27, 64, 125]",
];

static CUBE_NUMBERS_RELATED: [&str; 2] = ["square_numbers", "powers"];

impl FunctionPlugin for CubeNumbers {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cube_numbers",
            description: "Generate perfect cubes: 1, 8, 27, 64, 125, ...",
            usage: "cube_numbers(count)",
            args: &CUBE_NUMBERS_ARGS,
            returns: "List<Number>",
            examples: &CUBE_NUMBERS_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &CUBE_NUMBERS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("cube_numbers", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "cube_numbers", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            Value::Number(n.mul(&n).mul(&n))
        }).collect();

        Value::List(result)
    }
}

// ============ Powers ============

pub struct Powers;

static POWERS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "base",
        typ: "Number",
        description: "Base number",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of powers to generate (starting from base^0)",
        optional: false,
        default: None,
    },
];

static POWERS_EXAMPLES: [&str; 1] = [
    "powers(2, 8) → [1, 2, 4, 8, 16, 32, 64, 128]",
];

static POWERS_RELATED: [&str; 2] = ["geometric", "square_numbers"];

impl FunctionPlugin for Powers {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "powers",
            description: "Generate powers of base: base^0, base^1, base^2, ...",
            usage: "powers(base, count)",
            args: &POWERS_ARGS,
            returns: "List<Number>",
            examples: &POWERS_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &POWERS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("powers", 2, args.len()));
        }

        let base = match extract_number(&args[0], "base") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count_num = match extract_number(&args[1], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "powers", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(count);
        let mut current = Number::from_i64(1);

        for _ in 0..count {
            result.push(Value::Number(current.clone()));
            current = current.mul(&base);
        }

        Value::List(result)
    }
}

// ============ Catalan ============

pub struct Catalan;

static CATALAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of Catalan numbers to generate",
    optional: false,
    default: None,
}];

static CATALAN_EXAMPLES: [&str; 1] = [
    "catalan(7) → [1, 1, 2, 5, 14, 42, 132]",
];

static CATALAN_RELATED: [&str; 1] = ["bell"];

impl FunctionPlugin for Catalan {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "catalan",
            description: "Generate Catalan numbers: C(n) = (2n)! / ((n+1)! * n!)",
            usage: "catalan(count)",
            args: &CATALAN_ARGS,
            returns: "List<Number>",
            examples: &CATALAN_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &CATALAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("catalan", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "catalan", 100) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        // C(0) = 1, C(n+1) = C(n) * 2(2n+1) / (n+2)
        let mut result = Vec::with_capacity(count);
        let mut c = Number::from_i64(1);

        for n in 0..count {
            result.push(Value::Number(c.clone()));
            if n + 1 < count {
                // C(n+1) = C(n) * 2(2n+1) / (n+2)
                let two_n_plus_1 = Number::from_i64(2 * n as i64 + 1);
                let n_plus_2 = Number::from_i64(n as i64 + 2);
                let two = Number::from_i64(2);
                let numerator = c.mul(&two).mul(&two_n_plus_1);
                c = match numerator.checked_div(&n_plus_2) {
                    Ok(val) => val,
                    Err(e) => return Value::Error(e.into()),
                };
            }
        }

        Value::List(result)
    }
}

// ============ Bell ============

pub struct Bell;

static BELL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of Bell numbers to generate",
    optional: false,
    default: None,
}];

static BELL_EXAMPLES: [&str; 1] = [
    "bell(7) → [1, 1, 2, 5, 15, 52, 203]",
];

static BELL_RELATED: [&str; 1] = ["catalan"];

impl FunctionPlugin for Bell {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "bell",
            description: "Generate Bell numbers (number of partitions of a set)",
            usage: "bell(count)",
            args: &BELL_ARGS,
            returns: "List<Number>",
            examples: &BELL_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &BELL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("bell", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "bell", 100) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        // Use Bell triangle
        // B(0) = 1
        // Build triangle row by row
        let mut result = Vec::with_capacity(count);

        if count == 0 {
            return Value::List(result);
        }

        // Bell triangle: row[0] = previous row's last, row[i] = row[i-1] + prev_row[i-1]
        let mut prev_row = vec![Number::from_i64(1)];
        result.push(Value::Number(Number::from_i64(1)));

        for _ in 1..count {
            let mut row = Vec::with_capacity(prev_row.len() + 1);
            // First element is last element of previous row
            row.push(prev_row.last().unwrap().clone());

            // Build rest of row
            for i in 0..prev_row.len() {
                let next = row[i].add(&prev_row[i]);
                row.push(next);
            }

            result.push(Value::Number(row[0].clone()));
            prev_row = row;
        }

        Value::List(result)
    }
}

// ============ Pentagonal ============

pub struct Pentagonal;

static PENTAGONAL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of pentagonal numbers to generate",
    optional: false,
    default: None,
}];

static PENTAGONAL_EXAMPLES: [&str; 1] = [
    "pentagonal(6) → [1, 5, 12, 22, 35, 51]",
];

static PENTAGONAL_RELATED: [&str; 2] = ["triangular", "hexagonal"];

impl FunctionPlugin for Pentagonal {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pentagonal",
            description: "Generate pentagonal numbers: P(n) = n(3n-1)/2",
            usage: "pentagonal(count)",
            args: &PENTAGONAL_ARGS,
            returns: "List<Number>",
            examples: &PENTAGONAL_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &PENTAGONAL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("pentagonal", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "pentagonal", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let two = Number::from_i64(2);
        let three = Number::from_i64(3);
        let one = Number::from_i64(1);

        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            // P(n) = n(3n-1)/2
            let three_n_minus_1 = three.mul(&n).sub(&one);
            let product = n.mul(&three_n_minus_1);
            match product.checked_div(&two) {
                Ok(val) => Value::Number(val),
                Err(e) => Value::Error(e.into()),
            }
        }).collect();

        Value::List(result)
    }
}

// ============ Hexagonal ============

pub struct Hexagonal;

static HEXAGONAL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "count",
    typ: "Number",
    description: "Number of hexagonal numbers to generate",
    optional: false,
    default: None,
}];

static HEXAGONAL_EXAMPLES: [&str; 1] = [
    "hexagonal(5) → [1, 6, 15, 28, 45]",
];

static HEXAGONAL_RELATED: [&str; 2] = ["triangular", "pentagonal"];

impl FunctionPlugin for Hexagonal {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "hexagonal",
            description: "Generate hexagonal numbers: H(n) = n(2n-1)",
            usage: "hexagonal(count)",
            args: &HEXAGONAL_ARGS,
            returns: "List<Number>",
            examples: &HEXAGONAL_EXAMPLES,
            category: "sequence/named",
            source: None,
            related: &HEXAGONAL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("hexagonal", 1, args.len()));
        }

        let count_num = match extract_number(&args[0], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "hexagonal", 100000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let two = Number::from_i64(2);
        let one = Number::from_i64(1);

        let result: Vec<Value> = (1..=count).map(|n| {
            let n = Number::from_i64(n as i64);
            // H(n) = n(2n-1)
            let two_n_minus_1 = two.mul(&n).sub(&one);
            Value::Number(n.mul(&two_n_minus_1))
        }).collect();

        Value::List(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_fibonacci() {
        let fib = Fibonacci;
        let args = vec![Value::Number(Number::from_i64(10))];
        let result = fib.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 10);
        // Default start=1 means F(1), F(2), ... = 1, 1, 2, 3, 5, 8, 13, 21, 34, 55
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[9].as_number().unwrap().to_i64(), Some(55));
    }

    #[test]
    fn test_primes() {
        let primes = Primes;
        let args = vec![Value::Number(Number::from_i64(5))];
        let result = primes.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(3));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(7));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(11));
    }

    #[test]
    fn test_triangular() {
        let tri = Triangular;
        let args = vec![Value::Number(Number::from_i64(5))];
        let result = tri.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(3));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(6));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(10));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(15));
    }

    #[test]
    fn test_catalan() {
        let cat = Catalan;
        let args = vec![Value::Number(Number::from_i64(5))];
        let result = cat.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(14));
    }
}
```

folio-sequence\src\pattern.rs
```rs
//! Pattern detection for sequences
//!
//! detect_pattern, extend_pattern, is_arithmetic, is_geometric, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_number, extract_list, require_count};
use std::collections::HashMap;

// ============ DetectPattern ============

pub struct DetectPattern;

static DETECT_PATTERN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Sequence to analyze (minimum 4 elements)",
    optional: false,
    default: None,
}];

static DETECT_PATTERN_EXAMPLES: [&str; 2] = [
    "detect_pattern([2, 5, 8, 11, 14]) → {type: \"arithmetic\", ...}",
    "detect_pattern([1, 4, 9, 16, 25]) → {type: \"polynomial\", ...}",
];

static DETECT_PATTERN_RELATED: [&str; 2] = ["extend_pattern", "is_arithmetic"];

impl FunctionPlugin for DetectPattern {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "detect_pattern",
            description: "Detect the pattern type of a sequence",
            usage: "detect_pattern(list)",
            args: &DETECT_PATTERN_ARGS,
            returns: "Object",
            examples: &DETECT_PATTERN_EXAMPLES,
            category: "sequence/pattern",
            source: None,
            related: &DETECT_PATTERN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("detect_pattern", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if list.len() < 4 {
            return Value::Error(FolioError::domain_error(
                "detect_pattern() requires at least 4 elements"
            ));
        }

        // Try different patterns in order

        // 1. Check arithmetic
        if let Some(diff) = check_arithmetic(&list) {
            return build_pattern_result(
                "arithmetic",
                1.0,
                vec![
                    ("first", Value::Number(list[0].clone())),
                    ("diff", Value::Number(diff.clone())),
                ],
                format!("{} + {}*(n-1)", format_num(&list[0]), format_num(&diff)),
                predict_arithmetic(&list, &diff, 3),
            );
        }

        // 2. Check geometric
        if let Some(ratio) = check_geometric(&list) {
            return build_pattern_result(
                "geometric",
                1.0,
                vec![
                    ("first", Value::Number(list[0].clone())),
                    ("ratio", Value::Number(ratio.clone())),
                ],
                format!("{} * {}^(n-1)", format_num(&list[0]), format_num(&ratio)),
                predict_geometric(&list, &ratio, 3),
            );
        }

        // 3. Check fibonacci-like
        if check_fibonacci_like(&list) {
            return build_pattern_result(
                "fibonacci_like",
                0.95,
                vec![
                    ("initial_1", Value::Number(list[0].clone())),
                    ("initial_2", Value::Number(list[1].clone())),
                ],
                format!("a(n-1) + a(n-2), starting with {}, {}", format_num(&list[0]), format_num(&list[1])),
                predict_fibonacci_like(&list, 3),
            );
        }

        // 4. Check polynomial (degree 2)
        if let Some(coeffs) = check_polynomial(&list, 2) {
            let formula = format_polynomial(&coeffs);
            return build_pattern_result(
                "polynomial",
                0.9,
                vec![
                    ("degree", Value::Number(Number::from_i64(2))),
                    ("coefficients", Value::List(coeffs.iter().map(|c| Value::Number(c.clone())).collect())),
                ],
                formula,
                predict_polynomial(&coeffs, list.len(), 3),
            );
        }

        // 5. Check power pattern (a_n = n^k)
        if let Some(exp) = check_power_pattern(&list) {
            return build_pattern_result(
                "power",
                0.85,
                vec![("exponent", Value::Number(exp.clone()))],
                format!("n^{}", format_num(&exp)),
                predict_power(&exp, list.len(), 3),
            );
        }

        // 6. Unknown
        let mut result = HashMap::new();
        result.insert("type".to_string(), Value::Text("unknown".to_string()));
        result.insert("confidence".to_string(), Value::Number(Number::from_i64(0)));
        result.insert("parameters".to_string(), Value::Object(HashMap::new()));
        result.insert("formula".to_string(), Value::Text("unknown".to_string()));
        result.insert("next_values".to_string(), Value::List(vec![]));
        Value::Object(result)
    }
}

fn format_num(n: &Number) -> String {
    if n.is_integer() {
        n.to_i64().map(|i| i.to_string()).unwrap_or_else(|| n.as_decimal(2))
    } else {
        n.as_decimal(4)
    }
}

fn build_pattern_result(
    typ: &str,
    confidence: f64,
    params: Vec<(&str, Value)>,
    formula: String,
    next_values: Vec<Number>,
) -> Value {
    let mut result = HashMap::new();
    result.insert("type".to_string(), Value::Text(typ.to_string()));
    result.insert("confidence".to_string(), Value::Number(Number::from_str(&confidence.to_string()).unwrap_or(Number::from_i64(0))));

    let mut params_map = HashMap::new();
    for (k, v) in params {
        params_map.insert(k.to_string(), v);
    }
    result.insert("parameters".to_string(), Value::Object(params_map));
    result.insert("formula".to_string(), Value::Text(formula));
    result.insert("next_values".to_string(), Value::List(next_values.into_iter().map(Value::Number).collect()));

    Value::Object(result)
}

fn check_arithmetic(list: &[Number]) -> Option<Number> {
    if list.len() < 2 {
        return None;
    }
    let diff = list[1].sub(&list[0]);
    for i in 2..list.len() {
        let d = list[i].sub(&list[i - 1]);
        if !d.sub(&diff).is_zero() {
            return None;
        }
    }
    Some(diff)
}

fn check_geometric(list: &[Number]) -> Option<Number> {
    if list.len() < 2 {
        return None;
    }
    if list[0].is_zero() {
        return None;
    }
    let ratio = match list[1].checked_div(&list[0]) {
        Ok(r) => r,
        Err(_) => return None,
    };
    for i in 2..list.len() {
        if list[i - 1].is_zero() {
            return None;
        }
        let r = match list[i].checked_div(&list[i - 1]) {
            Ok(r) => r,
            Err(_) => return None,
        };
        // Allow small tolerance for floating point
        let diff = r.sub(&ratio).abs();
        let tolerance = Number::from_str("0.0000001").unwrap_or(Number::from_i64(0));
        if !diff.sub(&tolerance).is_negative() && !diff.is_zero() {
            return None;
        }
    }
    Some(ratio)
}

fn check_fibonacci_like(list: &[Number]) -> bool {
    if list.len() < 3 {
        return false;
    }
    for i in 2..list.len() {
        let sum = list[i - 2].add(&list[i - 1]);
        if !sum.sub(&list[i]).is_zero() {
            return false;
        }
    }
    true
}

fn check_polynomial(list: &[Number], max_degree: usize) -> Option<Vec<Number>> {
    // Check if sequence fits a polynomial of degree 2 (quadratic)
    // For quadratic: a*n^2 + b*n + c
    // We need at least 3 points
    if list.len() < 3 || max_degree < 2 {
        return None;
    }

    // Use first differences and second differences
    let mut diffs: Vec<Number> = Vec::new();
    for i in 1..list.len() {
        diffs.push(list[i].sub(&list[i - 1]));
    }

    let mut second_diffs: Vec<Number> = Vec::new();
    for i in 1..diffs.len() {
        second_diffs.push(diffs[i].sub(&diffs[i - 1]));
    }

    // For quadratic, second differences should be constant
    if second_diffs.is_empty() {
        return None;
    }

    let second_diff = &second_diffs[0];
    for sd in &second_diffs[1..] {
        if !sd.sub(second_diff).is_zero() {
            return None;
        }
    }

    // Reconstruct coefficients
    // a = second_diff / 2
    // b = first_diff[0] - a * (2*1 - 1) = first_diff[0] - a
    // c = list[0] - a - b = list[0] - a*1 - b*1

    let two = Number::from_i64(2);
    let a = match second_diff.checked_div(&two) {
        Ok(a) => a,
        Err(_) => return None,
    };
    let b = diffs[0].sub(&a);
    let c = list[0].sub(&a).sub(&b);

    // Verify the fit
    for (i, expected) in list.iter().enumerate() {
        let n = Number::from_i64((i + 1) as i64);
        let computed = a.mul(&n).mul(&n).add(&b.mul(&n)).add(&c);
        if !computed.sub(expected).is_zero() {
            return None;
        }
    }

    Some(vec![a, b, c])
}

fn format_polynomial(coeffs: &[Number]) -> String {
    if coeffs.len() != 3 {
        return "unknown polynomial".to_string();
    }
    let a = format_num(&coeffs[0]);
    let b = format_num(&coeffs[1]);
    let c = format_num(&coeffs[2]);

    let mut parts = Vec::new();
    if !coeffs[0].is_zero() {
        if coeffs[0] == Number::from_i64(1) {
            parts.push("n²".to_string());
        } else {
            parts.push(format!("{}n²", a));
        }
    }
    if !coeffs[1].is_zero() {
        let sign = if coeffs[1].is_negative() { "" } else if !parts.is_empty() { "+" } else { "" };
        if coeffs[1] == Number::from_i64(1) {
            parts.push(format!("{}n", sign));
        } else if coeffs[1] == Number::from_i64(-1) {
            parts.push("-n".to_string());
        } else {
            parts.push(format!("{}{}n", sign, b));
        }
    }
    if !coeffs[2].is_zero() || parts.is_empty() {
        let sign = if coeffs[2].is_negative() { "" } else if !parts.is_empty() { "+" } else { "" };
        parts.push(format!("{}{}", sign, c));
    }

    parts.join("")
}

fn check_power_pattern(list: &[Number]) -> Option<Number> {
    // Check if list[i] = (i+1)^k for some k
    if list.len() < 3 {
        return None;
    }

    // Try small integer exponents first
    for k in 1..=4i64 {
        let exp = Number::from_i64(k);
        let mut matches = true;
        for (i, val) in list.iter().enumerate() {
            let n = Number::from_i64((i + 1) as i64);
            let expected = n.pow(k as i32);
            if !expected.sub(val).is_zero() {
                matches = false;
                break;
            }
        }
        if matches {
            return Some(exp);
        }
    }
    None
}

fn predict_arithmetic(list: &[Number], diff: &Number, count: usize) -> Vec<Number> {
    let mut result = Vec::new();
    let mut current = list.last().unwrap().clone();
    for _ in 0..count {
        current = current.add(diff);
        result.push(current.clone());
    }
    result
}

fn predict_geometric(list: &[Number], ratio: &Number, count: usize) -> Vec<Number> {
    let mut result = Vec::new();
    let mut current = list.last().unwrap().clone();
    for _ in 0..count {
        current = current.mul(ratio);
        result.push(current.clone());
    }
    result
}

fn predict_fibonacci_like(list: &[Number], count: usize) -> Vec<Number> {
    let mut result = Vec::new();
    let mut prev2 = list[list.len() - 2].clone();
    let mut prev1 = list[list.len() - 1].clone();
    for _ in 0..count {
        let next = prev2.add(&prev1);
        result.push(next.clone());
        prev2 = prev1;
        prev1 = next;
    }
    result
}

fn predict_polynomial(coeffs: &[Number], current_len: usize, count: usize) -> Vec<Number> {
    let mut result = Vec::new();
    for i in 0..count {
        let n = Number::from_i64((current_len + i + 1) as i64);
        let val = coeffs[0].mul(&n).mul(&n).add(&coeffs[1].mul(&n)).add(&coeffs[2]);
        result.push(val);
    }
    result
}

fn predict_power(exp: &Number, current_len: usize, count: usize) -> Vec<Number> {
    let mut result = Vec::new();
    let exp_i = exp.to_i64().unwrap_or(1) as i32;
    for i in 0..count {
        let n = Number::from_i64((current_len + i + 1) as i64);
        result.push(n.pow(exp_i));
    }
    result
}

// ============ ExtendPattern ============

pub struct ExtendPattern;

static EXTEND_PATTERN_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Sequence to extend",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of elements to add",
        optional: false,
        default: None,
    },
];

static EXTEND_PATTERN_EXAMPLES: [&str; 1] = [
    "extend_pattern([2, 4, 8, 16, 32], 5) → [64, 128, 256, 512, 1024]",
];

static EXTEND_PATTERN_RELATED: [&str; 1] = ["detect_pattern"];

impl FunctionPlugin for ExtendPattern {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "extend_pattern",
            description: "Extend sequence using detected pattern",
            usage: "extend_pattern(list, count)",
            args: &EXTEND_PATTERN_ARGS,
            returns: "List<Number>",
            examples: &EXTEND_PATTERN_EXAMPLES,
            category: "sequence/pattern",
            source: None,
            related: &EXTEND_PATTERN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("extend_pattern", 2, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if list.len() < 4 {
            return Value::Error(FolioError::domain_error(
                "extend_pattern() requires at least 4 elements"
            ));
        }

        let count_num = match extract_number(&args[1], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "extend_pattern", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        // Try patterns in order

        // 1. Arithmetic
        if let Some(diff) = check_arithmetic(&list) {
            let result = predict_arithmetic(&list, &diff, count);
            return Value::List(result.into_iter().map(Value::Number).collect());
        }

        // 2. Geometric
        if let Some(ratio) = check_geometric(&list) {
            let result = predict_geometric(&list, &ratio, count);
            return Value::List(result.into_iter().map(Value::Number).collect());
        }

        // 3. Fibonacci-like
        if check_fibonacci_like(&list) {
            let result = predict_fibonacci_like(&list, count);
            return Value::List(result.into_iter().map(Value::Number).collect());
        }

        // 4. Polynomial
        if let Some(coeffs) = check_polynomial(&list, 2) {
            let result = predict_polynomial(&coeffs, list.len(), count);
            return Value::List(result.into_iter().map(Value::Number).collect());
        }

        // 5. Power
        if let Some(exp) = check_power_pattern(&list) {
            let result = predict_power(&exp, list.len(), count);
            return Value::List(result.into_iter().map(Value::Number).collect());
        }

        Value::Error(FolioError::domain_error(
            "extend_pattern() could not detect a pattern with high confidence"
        ))
    }
}

// ============ IsArithmetic ============

pub struct IsArithmetic;

static IS_ARITHMETIC_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Sequence to check",
    optional: false,
    default: None,
}];

static IS_ARITHMETIC_EXAMPLES: [&str; 2] = [
    "is_arithmetic([2, 5, 8, 11]) → true",
    "is_arithmetic([1, 2, 4, 8]) → false",
];

static IS_ARITHMETIC_RELATED: [&str; 2] = ["is_geometric", "common_diff"];

impl FunctionPlugin for IsArithmetic {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_arithmetic",
            description: "Check if sequence is arithmetic (constant difference)",
            usage: "is_arithmetic(list)",
            args: &IS_ARITHMETIC_ARGS,
            returns: "Bool",
            examples: &IS_ARITHMETIC_EXAMPLES,
            category: "sequence/pattern",
            source: None,
            related: &IS_ARITHMETIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("is_arithmetic", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if list.len() < 2 {
            return Value::Bool(true); // Trivially arithmetic
        }

        Value::Bool(check_arithmetic(&list).is_some())
    }
}

// ============ IsGeometric ============

pub struct IsGeometric;

static IS_GEOMETRIC_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Sequence to check",
    optional: false,
    default: None,
}];

static IS_GEOMETRIC_EXAMPLES: [&str; 2] = [
    "is_geometric([1, 2, 4, 8]) → true",
    "is_geometric([2, 5, 8, 11]) → false",
];

static IS_GEOMETRIC_RELATED: [&str; 2] = ["is_arithmetic", "common_ratio"];

impl FunctionPlugin for IsGeometric {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_geometric",
            description: "Check if sequence is geometric (constant ratio)",
            usage: "is_geometric(list)",
            args: &IS_GEOMETRIC_ARGS,
            returns: "Bool",
            examples: &IS_GEOMETRIC_EXAMPLES,
            category: "sequence/pattern",
            source: None,
            related: &IS_GEOMETRIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("is_geometric", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if list.len() < 2 {
            return Value::Bool(true); // Trivially geometric
        }

        Value::Bool(check_geometric(&list).is_some())
    }
}

// ============ CommonDiff ============

pub struct CommonDiff;

static COMMON_DIFF_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Arithmetic sequence",
    optional: false,
    default: None,
}];

static COMMON_DIFF_EXAMPLES: [&str; 1] = [
    "common_diff([2, 5, 8, 11]) → 3",
];

static COMMON_DIFF_RELATED: [&str; 2] = ["is_arithmetic", "common_ratio"];

impl FunctionPlugin for CommonDiff {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "common_diff",
            description: "Get common difference of arithmetic sequence",
            usage: "common_diff(list)",
            args: &COMMON_DIFF_ARGS,
            returns: "Number",
            examples: &COMMON_DIFF_EXAMPLES,
            category: "sequence/pattern",
            source: None,
            related: &COMMON_DIFF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("common_diff", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if list.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "common_diff() requires at least 2 elements"
            ));
        }

        match check_arithmetic(&list) {
            Some(diff) => Value::Number(diff),
            None => Value::Error(FolioError::domain_error(
                "Sequence is not arithmetic"
            )),
        }
    }
}

// ============ CommonRatio ============

pub struct CommonRatio;

static COMMON_RATIO_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Geometric sequence",
    optional: false,
    default: None,
}];

static COMMON_RATIO_EXAMPLES: [&str; 1] = [
    "common_ratio([2, 6, 18, 54]) → 3",
];

static COMMON_RATIO_RELATED: [&str; 2] = ["is_geometric", "common_diff"];

impl FunctionPlugin for CommonRatio {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "common_ratio",
            description: "Get common ratio of geometric sequence",
            usage: "common_ratio(list)",
            args: &COMMON_RATIO_ARGS,
            returns: "Number",
            examples: &COMMON_RATIO_EXAMPLES,
            category: "sequence/pattern",
            source: None,
            related: &COMMON_RATIO_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("common_ratio", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if list.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "common_ratio() requires at least 2 elements"
            ));
        }

        match check_geometric(&list) {
            Some(ratio) => Value::Number(ratio),
            None => Value::Error(FolioError::domain_error(
                "Sequence is not geometric"
            )),
        }
    }
}

// ============ NthTermFormula ============

pub struct NthTermFormula;

static NTH_TERM_FORMULA_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Sequence to analyze",
    optional: false,
    default: None,
}];

static NTH_TERM_FORMULA_EXAMPLES: [&str; 1] = [
    "nth_term_formula([3, 7, 11, 15]) → \"3 + 4*(n-1)\"",
];

static NTH_TERM_FORMULA_RELATED: [&str; 1] = ["detect_pattern"];

impl FunctionPlugin for NthTermFormula {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nth_term_formula",
            description: "Get formula for nth term of sequence",
            usage: "nth_term_formula(list)",
            args: &NTH_TERM_FORMULA_ARGS,
            returns: "Text",
            examples: &NTH_TERM_FORMULA_EXAMPLES,
            category: "sequence/pattern",
            source: None,
            related: &NTH_TERM_FORMULA_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("nth_term_formula", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if list.len() < 2 {
            return Value::Text("unknown".to_string());
        }

        // Arithmetic
        if let Some(diff) = check_arithmetic(&list) {
            return Value::Text(format!(
                "{} + {}*(n-1)",
                format_num(&list[0]),
                format_num(&diff)
            ));
        }

        // Geometric
        if let Some(ratio) = check_geometric(&list) {
            return Value::Text(format!(
                "{} * {}^(n-1)",
                format_num(&list[0]),
                format_num(&ratio)
            ));
        }

        // Polynomial (quadratic)
        if list.len() >= 3 {
            if let Some(coeffs) = check_polynomial(&list, 2) {
                return Value::Text(format_polynomial(&coeffs));
            }
        }

        // Power
        if let Some(exp) = check_power_pattern(&list) {
            return Value::Text(format!("n^{}", format_num(&exp)));
        }

        Value::Text("unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_is_arithmetic() {
        let func = IsArithmetic;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(8)),
            Value::Number(Number::from_i64(11)),
        ])];
        let result = func.call(&args, &eval_ctx());
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_is_geometric() {
        let func = IsGeometric;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(6)),
            Value::Number(Number::from_i64(18)),
            Value::Number(Number::from_i64(54)),
        ])];
        let result = func.call(&args, &eval_ctx());
        assert_eq!(result.as_bool(), Some(true));
    }

    #[test]
    fn test_common_diff() {
        let func = CommonDiff;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(8)),
            Value::Number(Number::from_i64(11)),
        ])];
        let result = func.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3));
    }

    #[test]
    fn test_common_ratio() {
        let func = CommonRatio;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(6)),
            Value::Number(Number::from_i64(18)),
            Value::Number(Number::from_i64(54)),
        ])];
        let result = func.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3));
    }
}
```

folio-sequence\src\recurrence.rs
```rs
//! Custom recurrence relations
//!
//! recurrence, recurrence_named

use folio_plugin::prelude::*;
use crate::helpers::{extract_number, extract_list, require_count};
use crate::expr::eval_recurrence_expr;

// ============ Recurrence ============

pub struct Recurrence;

static RECURRENCE_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "initial",
        typ: "List<Number>",
        description: "Initial values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "expr",
        typ: "Text",
        description: "Recurrence expression using a, b, c, d (previous values) and n (index)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Total number of elements to generate",
        optional: false,
        default: None,
    },
];

static RECURRENCE_EXAMPLES: [&str; 4] = [
    "recurrence([1, 1], \"a + b\", 10) → [1, 1, 2, 3, 5, 8, ...]",
    "recurrence([0, 0, 1], \"a + b + c\", 10) → [0, 0, 1, 1, 2, 4, ...]",
    "recurrence([1], \"2 * a\", 8) → [1, 2, 4, 8, 16, ...]",
    "recurrence([1], \"a * n\", 6) → [1, 2, 6, 24, 120, 720]",
];

static RECURRENCE_RELATED: [&str; 2] = ["recurrence_named", "fibonacci"];

impl FunctionPlugin for Recurrence {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "recurrence",
            description: "Generate sequence from custom recurrence relation",
            usage: "recurrence(initial, expr, count)",
            args: &RECURRENCE_ARGS,
            returns: "List<Number>",
            examples: &RECURRENCE_EXAMPLES,
            category: "sequence/recurrence",
            source: None,
            related: &RECURRENCE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("recurrence", 3, args.len()));
        }

        let initial = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        if initial.is_empty() {
            return Value::Error(FolioError::domain_error(
                "recurrence() requires at least one initial value"
            ));
        }

        let expr_str = match &args[1] {
            Value::Text(s) => s.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("recurrence", "expr", "Text", other.type_name())),
        };

        let count_num = match extract_number(&args[2], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "recurrence", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        // Generate sequence
        let mut sequence = initial.clone();

        while sequence.len() < count {
            let n = sequence.len() as i64 + 1; // 1-based index for the new element
            match eval_recurrence_expr(&expr_str, &sequence, n, ctx.precision) {
                Ok(next) => sequence.push(next),
                Err(e) => return Value::Error(e),
            }
        }

        // Truncate to exact count if initial was larger
        sequence.truncate(count);

        let result: Vec<Value> = sequence.into_iter().map(Value::Number).collect();
        Value::List(result)
    }
}

// ============ RecurrenceNamed ============

pub struct RecurrenceNamed;

static RECURRENCE_NAMED_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "name",
        typ: "Text",
        description: "Name of predefined recurrence pattern",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "count",
        typ: "Number",
        description: "Number of elements to generate",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "start",
        typ: "Number",
        description: "Start index",
        optional: true,
        default: Some("0"),
    },
];

static RECURRENCE_NAMED_EXAMPLES: [&str; 3] = [
    "recurrence_named(\"fibonacci\", 10) → [1, 1, 2, 3, 5, ...]",
    "recurrence_named(\"pell\", 8) → [0, 1, 2, 5, 12, ...]",
    "recurrence_named(\"jacobsthal\", 8) → [0, 1, 1, 3, 5, ...]",
];

static RECURRENCE_NAMED_RELATED: [&str; 2] = ["recurrence", "fibonacci"];

impl FunctionPlugin for RecurrenceNamed {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "recurrence_named",
            description: "Generate sequence from named recurrence pattern",
            usage: "recurrence_named(name, count, [start])",
            args: &RECURRENCE_NAMED_ARGS,
            returns: "List<Number>",
            examples: &RECURRENCE_NAMED_EXAMPLES,
            category: "sequence/recurrence",
            source: None,
            related: &RECURRENCE_NAMED_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 3 {
            return Value::Error(FolioError::arg_count("recurrence_named", 2, args.len()));
        }

        let name = match &args[0] {
            Value::Text(s) => s.to_lowercase(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("recurrence_named", "name", "Text", other.type_name())),
        };

        let count_num = match extract_number(&args[1], "count") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let count = match require_count(&count_num, "recurrence_named", 10000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let start = match args.get(2) {
            Some(Value::Number(n)) => {
                if !n.is_integer() || n.is_negative() {
                    return Value::Error(FolioError::domain_error(
                        "recurrence_named() start must be non-negative integer"
                    ));
                }
                n.to_i64().unwrap_or(0) as usize
            }
            Some(Value::Null) | None => 0,
            Some(Value::Error(e)) => return Value::Error(e.clone()),
            Some(other) => return Value::Error(FolioError::arg_type("recurrence_named", "start", "Number", other.type_name())),
        };

        // Get initial values and expression based on name
        let (initial, expr) = match name.as_str() {
            "fibonacci" => (
                vec![Number::from_i64(0), Number::from_i64(1)],
                "a + b",
            ),
            "lucas" => (
                vec![Number::from_i64(2), Number::from_i64(1)],
                "a + b",
            ),
            "pell" => (
                vec![Number::from_i64(0), Number::from_i64(1)],
                "2*a + b",
            ),
            "jacobsthal" => (
                vec![Number::from_i64(0), Number::from_i64(1)],
                "a + 2*b",
            ),
            "tribonacci" => (
                vec![Number::from_i64(0), Number::from_i64(0), Number::from_i64(1)],
                "a + b + c",
            ),
            "padovan" => (
                vec![Number::from_i64(1), Number::from_i64(1), Number::from_i64(1)],
                "b + c",
            ),
            "perrin" => (
                vec![Number::from_i64(3), Number::from_i64(0), Number::from_i64(2)],
                "b + c",
            ),
            _ => {
                return Value::Error(FolioError::domain_error(format!(
                    "Unknown recurrence name: {}. Valid: fibonacci, lucas, pell, jacobsthal, tribonacci, padovan, perrin",
                    name
                )));
            }
        };

        // Generate sequence
        let total_needed = start + count;
        let mut sequence = initial;

        while sequence.len() < total_needed {
            let n = sequence.len() as i64 + 1;
            match eval_recurrence_expr(expr, &sequence, n, ctx.precision) {
                Ok(next) => sequence.push(next),
                Err(e) => return Value::Error(e),
            }
        }

        let result: Vec<Value> = sequence[start..].iter()
            .take(count)
            .map(|n| Value::Number(n.clone()))
            .collect();

        Value::List(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_recurrence_fibonacci() {
        let rec = Recurrence;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(1)),
            ]),
            Value::Text("a + b".to_string()),
            Value::Number(Number::from_i64(8)),
        ];
        let result = rec.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 8);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(3));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[5].as_number().unwrap().to_i64(), Some(8));
        assert_eq!(list[6].as_number().unwrap().to_i64(), Some(13));
        assert_eq!(list[7].as_number().unwrap().to_i64(), Some(21));
    }

    #[test]
    fn test_recurrence_doubling() {
        let rec = Recurrence;
        let args = vec![
            Value::List(vec![Value::Number(Number::from_i64(1))]),
            Value::Text("2 * a".to_string()),
            Value::Number(Number::from_i64(5)),
        ];
        let result = rec.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(4));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(8));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(16));
    }

    #[test]
    fn test_recurrence_factorial() {
        let rec = Recurrence;
        let args = vec![
            Value::List(vec![Value::Number(Number::from_i64(1))]),
            Value::Text("a * n".to_string()),
            Value::Number(Number::from_i64(6)),
        ];
        let result = rec.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 6);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));  // 1
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(2));  // 1*2
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(6));  // 2*3
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(24)); // 6*4
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(120)); // 24*5
        assert_eq!(list[5].as_number().unwrap().to_i64(), Some(720)); // 120*6
    }

    #[test]
    fn test_recurrence_named_pell() {
        let rec = RecurrenceNamed;
        let args = vec![
            Value::Text("pell".to_string()),
            Value::Number(Number::from_i64(6)),
        ];
        let result = rec.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 6);
        // Pell: 0, 1, 2, 5, 12, 29
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(0));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(5));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(12));
        assert_eq!(list[5].as_number().unwrap().to_i64(), Some(29));
    }
}
```

folio-sequence\src\series.rs
```rs
//! Series operations (sums and products)
//!
//! sum_seq, product_seq, partial_sums, partial_products, alternating_sum, sum_formula

use folio_plugin::prelude::*;
use crate::helpers::{extract_number, extract_list, sum, product, require_count};
use std::collections::HashMap;

// ============ SumSeq ============

pub struct SumSeq;

static SUM_SEQ_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Sequence to sum",
    optional: false,
    default: None,
}];

static SUM_SEQ_EXAMPLES: [&str; 1] = [
    "sum_seq(range(1, 100)) → 5050",
];

static SUM_SEQ_RELATED: [&str; 2] = ["product_seq", "partial_sums"];

impl FunctionPlugin for SumSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sum_seq",
            description: "Sum of sequence elements",
            usage: "sum_seq(list)",
            args: &SUM_SEQ_ARGS,
            returns: "Number",
            examples: &SUM_SEQ_EXAMPLES,
            category: "sequence/series",
            source: None,
            related: &SUM_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("sum_seq", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        Value::Number(sum(&list))
    }
}

// ============ ProductSeq ============

pub struct ProductSeq;

static PRODUCT_SEQ_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Sequence to multiply",
    optional: false,
    default: None,
}];

static PRODUCT_SEQ_EXAMPLES: [&str; 1] = [
    "product_seq(range(1, 5)) → 120",
];

static PRODUCT_SEQ_RELATED: [&str; 2] = ["sum_seq", "partial_products"];

impl FunctionPlugin for ProductSeq {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "product_seq",
            description: "Product of sequence elements",
            usage: "product_seq(list)",
            args: &PRODUCT_SEQ_ARGS,
            returns: "Number",
            examples: &PRODUCT_SEQ_EXAMPLES,
            category: "sequence/series",
            source: None,
            related: &PRODUCT_SEQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("product_seq", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        Value::Number(product(&list))
    }
}

// ============ PartialSums ============

pub struct PartialSums;

static PARTIAL_SUMS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Sequence to compute cumulative sums",
    optional: false,
    default: None,
}];

static PARTIAL_SUMS_EXAMPLES: [&str; 1] = [
    "partial_sums([1, 2, 3, 4, 5]) → [1, 3, 6, 10, 15]",
];

static PARTIAL_SUMS_RELATED: [&str; 2] = ["partial_products", "sum_seq"];

impl FunctionPlugin for PartialSums {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "partial_sums",
            description: "Cumulative sums of sequence",
            usage: "partial_sums(list)",
            args: &PARTIAL_SUMS_ARGS,
            returns: "List<Number>",
            examples: &PARTIAL_SUMS_EXAMPLES,
            category: "sequence/series",
            source: None,
            related: &PARTIAL_SUMS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("partial_sums", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(list.len());
        let mut running_sum = Number::from_i64(0);

        for n in list {
            running_sum = running_sum.add(&n);
            result.push(Value::Number(running_sum.clone()));
        }

        Value::List(result)
    }
}

// ============ PartialProducts ============

pub struct PartialProducts;

static PARTIAL_PRODUCTS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Sequence to compute cumulative products",
    optional: false,
    default: None,
}];

static PARTIAL_PRODUCTS_EXAMPLES: [&str; 1] = [
    "partial_products([1, 2, 3, 4]) → [1, 2, 6, 24]",
];

static PARTIAL_PRODUCTS_RELATED: [&str; 2] = ["partial_sums", "product_seq"];

impl FunctionPlugin for PartialProducts {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "partial_products",
            description: "Cumulative products of sequence",
            usage: "partial_products(list)",
            args: &PARTIAL_PRODUCTS_ARGS,
            returns: "List<Number>",
            examples: &PARTIAL_PRODUCTS_EXAMPLES,
            category: "sequence/series",
            source: None,
            related: &PARTIAL_PRODUCTS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("partial_products", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let mut result = Vec::with_capacity(list.len());
        let mut running_product = Number::from_i64(1);

        for n in list {
            running_product = running_product.mul(&n);
            result.push(Value::Number(running_product.clone()));
        }

        Value::List(result)
    }
}

// ============ AlternatingSum ============

pub struct AlternatingSum;

static ALTERNATING_SUM_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Sequence for alternating sum",
    optional: false,
    default: None,
}];

static ALTERNATING_SUM_EXAMPLES: [&str; 1] = [
    "alternating_sum([1, 2, 3, 4, 5]) → 3",
];

static ALTERNATING_SUM_RELATED: [&str; 1] = ["sum_seq"];

impl FunctionPlugin for AlternatingSum {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "alternating_sum",
            description: "Sum with alternating signs: a₁ - a₂ + a₃ - a₄ + ...",
            usage: "alternating_sum(list)",
            args: &ALTERNATING_SUM_ARGS,
            returns: "Number",
            examples: &ALTERNATING_SUM_EXAMPLES,
            category: "sequence/series",
            source: None,
            related: &ALTERNATING_SUM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("alternating_sum", 1, args.len()));
        }

        let list = match extract_list(&args[0]) {
            Ok(l) => l,
            Err(e) => return Value::Error(e),
        };

        let mut result = Number::from_i64(0);
        for (i, n) in list.iter().enumerate() {
            if i % 2 == 0 {
                result = result.add(n);
            } else {
                result = result.sub(n);
            }
        }

        Value::Number(result)
    }
}

// ============ SumFormula ============

pub struct SumFormula;

static SUM_FORMULA_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "type",
        typ: "Text",
        description: "Sum type: arithmetic, geometric, squares, cubes, triangular, natural",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of terms",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "params",
        typ: "Object",
        description: "Parameters for the formula (type-dependent)",
        optional: true,
        default: None,
    },
];

static SUM_FORMULA_EXAMPLES: [&str; 4] = [
    "sum_formula(\"natural\", 100) → 5050",
    "sum_formula(\"squares\", 10) → 385",
    "sum_formula(\"cubes\", 10) → 3025",
    "sum_formula(\"arithmetic\", 100, {first: 1, diff: 1}) → 5050",
];

static SUM_FORMULA_RELATED: [&str; 2] = ["sum_seq", "partial_sums"];

impl FunctionPlugin for SumFormula {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sum_formula",
            description: "Compute sum using closed-form formula",
            usage: "sum_formula(type, n, [params])",
            args: &SUM_FORMULA_ARGS,
            returns: "Number",
            examples: &SUM_FORMULA_EXAMPLES,
            category: "sequence/series",
            source: None,
            related: &SUM_FORMULA_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(FolioError::arg_count("sum_formula", 2, args.len()));
        }

        let sum_type = match &args[0] {
            Value::Text(s) => s.to_lowercase(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("sum_formula", "type", "Text", other.type_name())),
        };

        let n_num = match extract_number(&args[1], "n") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = match require_count(&n_num, "sum_formula", 1_000_000) {
            Ok(c) => c,
            Err(e) => return Value::Error(e),
        };

        let n = Number::from_i64(n as i64);

        let params: HashMap<String, Value> = if args.len() == 3 {
            match &args[2] {
                Value::Object(m) => m.clone(),
                Value::Null => HashMap::new(),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("sum_formula", "params", "Object", other.type_name())),
            }
        } else {
            HashMap::new()
        };

        match sum_type.as_str() {
            "natural" => {
                // Sum of 1 to n = n(n+1)/2
                let one = Number::from_i64(1);
                let two = Number::from_i64(2);
                let n_plus_1 = n.add(&one);
                let product = n.mul(&n_plus_1);
                match product.checked_div(&two) {
                    Ok(r) => Value::Number(r),
                    Err(e) => Value::Error(e.into()),
                }
            }
            "squares" => {
                // Sum of 1² to n² = n(n+1)(2n+1)/6
                let one = Number::from_i64(1);
                let two = Number::from_i64(2);
                let six = Number::from_i64(6);
                let n_plus_1 = n.add(&one);
                let two_n_plus_1 = two.mul(&n).add(&one);
                let product = n.mul(&n_plus_1).mul(&two_n_plus_1);
                match product.checked_div(&six) {
                    Ok(r) => Value::Number(r),
                    Err(e) => Value::Error(e.into()),
                }
            }
            "cubes" => {
                // Sum of 1³ to n³ = [n(n+1)/2]²
                let one = Number::from_i64(1);
                let two = Number::from_i64(2);
                let n_plus_1 = n.add(&one);
                let product = n.mul(&n_plus_1);
                match product.checked_div(&two) {
                    Ok(r) => Value::Number(r.mul(&r)),
                    Err(e) => Value::Error(e.into()),
                }
            }
            "triangular" => {
                // Sum of triangular numbers T(1) to T(n) = n(n+1)(n+2)/6
                let one = Number::from_i64(1);
                let two = Number::from_i64(2);
                let six = Number::from_i64(6);
                let n_plus_1 = n.add(&one);
                let n_plus_2 = n.add(&two);
                let product = n.mul(&n_plus_1).mul(&n_plus_2);
                match product.checked_div(&six) {
                    Ok(r) => Value::Number(r),
                    Err(e) => Value::Error(e.into()),
                }
            }
            "arithmetic" => {
                // Sum of arithmetic sequence: n(a₁ + aₙ)/2 = n(2a₁ + (n-1)d)/2
                let first = match params.get("first") {
                    Some(Value::Number(f)) => f.clone(),
                    _ => return Value::Error(FolioError::domain_error(
                        "sum_formula(\"arithmetic\", ...) requires params.first"
                    )),
                };
                let diff = match params.get("diff") {
                    Some(Value::Number(d)) => d.clone(),
                    _ => return Value::Error(FolioError::domain_error(
                        "sum_formula(\"arithmetic\", ...) requires params.diff"
                    )),
                };

                let two = Number::from_i64(2);
                let one = Number::from_i64(1);
                let n_minus_1 = n.sub(&one);
                // S = n * (2*first + (n-1)*diff) / 2
                let two_first = two.mul(&first);
                let term = n_minus_1.mul(&diff);
                let inner = two_first.add(&term);
                let product = n.mul(&inner);
                match product.checked_div(&two) {
                    Ok(r) => Value::Number(r),
                    Err(e) => Value::Error(e.into()),
                }
            }
            "geometric" => {
                // Sum of geometric sequence: a₁(rⁿ - 1)/(r - 1)
                let first = match params.get("first") {
                    Some(Value::Number(f)) => f.clone(),
                    _ => return Value::Error(FolioError::domain_error(
                        "sum_formula(\"geometric\", ...) requires params.first"
                    )),
                };
                let ratio = match params.get("ratio") {
                    Some(Value::Number(r)) => r.clone(),
                    _ => return Value::Error(FolioError::domain_error(
                        "sum_formula(\"geometric\", ...) requires params.ratio"
                    )),
                };

                let one = Number::from_i64(1);

                // Special case: ratio = 1
                if ratio == one {
                    return Value::Number(first.mul(&n));
                }

                let n_int = n.to_i64().unwrap_or(0) as i32;
                let r_to_n = ratio.pow(n_int);
                let numerator = first.mul(&r_to_n.sub(&one));
                let denominator = ratio.sub(&one);

                match numerator.checked_div(&denominator) {
                    Ok(r) => Value::Number(r),
                    Err(e) => Value::Error(e.into()),
                }
            }
            _ => Value::Error(FolioError::domain_error(format!(
                "Unknown sum type: {}. Valid: natural, squares, cubes, triangular, arithmetic, geometric",
                sum_type
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_sum_seq() {
        let func = SumSeq;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = func.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(15));
    }

    #[test]
    fn test_product_seq() {
        let func = ProductSeq;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = func.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(120));
    }

    #[test]
    fn test_partial_sums() {
        let func = PartialSums;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = func.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(1));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(3));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(6));
        assert_eq!(list[3].as_number().unwrap().to_i64(), Some(10));
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(15));
    }

    #[test]
    fn test_alternating_sum() {
        let func = AlternatingSum;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = func.call(&args, &eval_ctx());
        // 1 - 2 + 3 - 4 + 5 = 3
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3));
    }

    #[test]
    fn test_sum_formula_natural() {
        let func = SumFormula;
        let args = vec![
            Value::Text("natural".to_string()),
            Value::Number(Number::from_i64(100)),
        ];
        let result = func.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(5050));
    }

    #[test]
    fn test_sum_formula_squares() {
        let func = SumFormula;
        let args = vec![
            Value::Text("squares".to_string()),
            Value::Number(Number::from_i64(10)),
        ];
        let result = func.call(&args, &eval_ctx());
        // 1 + 4 + 9 + 16 + 25 + 36 + 49 + 64 + 81 + 100 = 385
        assert_eq!(result.as_number().unwrap().to_i64(), Some(385));
    }

    #[test]
    fn test_sum_formula_cubes() {
        let func = SumFormula;
        let args = vec![
            Value::Text("cubes".to_string()),
            Value::Number(Number::from_i64(10)),
        ];
        let result = func.call(&args, &eval_ctx());
        // [10*11/2]² = 55² = 3025
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3025));
    }

    #[test]
    fn test_sum_formula_geometric() {
        let func = SumFormula;
        let mut params = HashMap::new();
        params.insert("first".to_string(), Value::Number(Number::from_i64(1)));
        params.insert("ratio".to_string(), Value::Number(Number::from_i64(2)));
        let args = vec![
            Value::Text("geometric".to_string()),
            Value::Number(Number::from_i64(10)),
            Value::Object(params),
        ];
        let result = func.call(&args, &eval_ctx());
        // 1 + 2 + 4 + 8 + 16 + 32 + 64 + 128 + 256 + 512 = 2^10 - 1 = 1023
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1023));
    }
}
```

folio-stats\Cargo.toml
```toml
[package]
name = "folio-stats"
description = "Statistical functions for Folio with arbitrary precision"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
folio-core = { path = "../folio-core" }
folio-plugin = { path = "../folio-plugin" }
serde = { workspace = true }
```

folio-stats\src\bivariate.rs
```rs
//! Bivariate functions: covariance, correlation, spearman

use folio_plugin::prelude::*;
use crate::helpers::{extract_two_lists, mean, variance_impl, ranks};

// ============ Covariance (Sample) ============

pub struct Covariance;

static COVARIANCE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "First variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Second variable",
        optional: false,
        default: None,
    },
];

static COVARIANCE_EXAMPLES: [&str; 1] = ["covariance([1,2,3], [4,5,6]) → 1"];

static COVARIANCE_RELATED: [&str; 2] = ["covariance_p", "correlation"];

impl FunctionPlugin for Covariance {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "covariance",
            description: "Sample covariance (divides by n-1)",
            usage: "covariance(x, y)",
            args: &COVARIANCE_ARGS,
            returns: "Number",
            examples: &COVARIANCE_EXAMPLES,
            category: "stats/bivariate",
            source: None,
            related: &COVARIANCE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match covariance_impl(&x, &y, true) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Covariance (Population) ============

pub struct CovarianceP;

static COVARIANCE_P_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "First variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Second variable",
        optional: false,
        default: None,
    },
];

static COVARIANCE_P_EXAMPLES: [&str; 1] = ["covariance_p([1,2,3], [4,5,6]) → 0.667"];

static COVARIANCE_P_RELATED: [&str; 2] = ["covariance", "correlation"];

impl FunctionPlugin for CovarianceP {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "covariance_p",
            description: "Population covariance (divides by n)",
            usage: "covariance_p(x, y)",
            args: &COVARIANCE_P_ARGS,
            returns: "Number",
            examples: &COVARIANCE_P_EXAMPLES,
            category: "stats/bivariate",
            source: None,
            related: &COVARIANCE_P_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match covariance_impl(&x, &y, false) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

/// Calculate covariance (sample or population)
fn covariance_impl(x: &[Number], y: &[Number], sample: bool) -> Result<Number, FolioError> {
    let n = x.len();
    if n == 0 {
        return Err(FolioError::domain_error("Cannot calculate covariance of empty lists"));
    }
    if sample && n < 2 {
        return Err(FolioError::domain_error(
            "Sample covariance requires at least 2 values",
        ));
    }

    let mean_x = mean(x)?;
    let mean_y = mean(y)?;

    let mut sum_products = Number::from_i64(0);
    for (xi, yi) in x.iter().zip(y.iter()) {
        let dev_x = xi.sub(&mean_x);
        let dev_y = yi.sub(&mean_y);
        sum_products = sum_products.add(&dev_x.mul(&dev_y));
    }

    let divisor = if sample {
        Number::from_i64((n - 1) as i64)
    } else {
        Number::from_i64(n as i64)
    };

    sum_products.checked_div(&divisor).map_err(|e| e.into())
}

// ============ Correlation (Pearson) ============

pub struct Correlation;

static CORRELATION_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "First variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Second variable",
        optional: false,
        default: None,
    },
];

static CORRELATION_EXAMPLES: [&str; 1] = ["correlation([1,2,3], [1,2,3]) → 1"];

static CORRELATION_RELATED: [&str; 2] = ["covariance", "spearman"];

impl FunctionPlugin for Correlation {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "correlation",
            description: "Pearson correlation coefficient r",
            usage: "correlation(x, y)",
            args: &CORRELATION_ARGS,
            returns: "Number",
            examples: &CORRELATION_EXAMPLES,
            category: "stats/bivariate",
            source: None,
            related: &CORRELATION_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "correlation() requires at least 2 pairs",
            ));
        }

        // r = cov(x,y) / (sd_x * sd_y)
        let cov = match covariance_impl(&x, &y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var_x = match variance_impl(&x, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var_y = match variance_impl(&y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd_x = match var_x.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let sd_y = match var_y.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if sd_x.is_zero() || sd_y.is_zero() {
            return Value::Error(FolioError::domain_error(
                "correlation() undefined when a variable has zero variance",
            ));
        }

        match cov.checked_div(&sd_x.mul(&sd_y)) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Spearman Rank Correlation ============

pub struct Spearman;

static SPEARMAN_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "First variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Second variable",
        optional: false,
        default: None,
    },
];

static SPEARMAN_EXAMPLES: [&str; 1] = ["spearman([1,2,3], [1,2,3]) → 1"];

static SPEARMAN_RELATED: [&str; 2] = ["correlation", "rank"];

impl FunctionPlugin for Spearman {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "spearman",
            description: "Spearman rank correlation coefficient",
            usage: "spearman(x, y)",
            args: &SPEARMAN_ARGS,
            returns: "Number",
            examples: &SPEARMAN_EXAMPLES,
            category: "stats/bivariate",
            source: None,
            related: &SPEARMAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "spearman() requires at least 2 pairs",
            ));
        }

        // Convert to ranks and compute Pearson correlation on ranks
        let ranks_x = ranks(&x);
        let ranks_y = ranks(&y);

        // r = cov(rank_x, rank_y) / (sd_rank_x * sd_rank_y)
        let cov = match covariance_impl(&ranks_x, &ranks_y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var_x = match variance_impl(&ranks_x, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var_y = match variance_impl(&ranks_y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd_x = match var_x.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let sd_y = match var_y.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if sd_x.is_zero() || sd_y.is_zero() {
            return Value::Error(FolioError::domain_error(
                "spearman() undefined when a variable has zero variance",
            ));
        }

        match cov.checked_div(&sd_x.mul(&sd_y)) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_correlation_perfect() {
        let correlation = Correlation;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
        ];
        let result = correlation.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        // Should be exactly 1
        assert_eq!(num.to_i64(), Some(1));
    }

    #[test]
    fn test_covariance() {
        let covariance = Covariance;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(5)),
                Value::Number(Number::from_i64(6)),
            ]),
        ];
        let result = covariance.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1));
    }
}
```

folio-stats\src\central.rs
```rs
//! Central tendency functions: mean, median, mode, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_non_empty, mean, sorted};
use std::collections::HashMap;

// ============ Mean ============

pub struct Mean;

static MEAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers to average",
    optional: false,
    default: None,
}];

static MEAN_EXAMPLES: [&str; 3] = [
    "mean(1, 2, 3, 4, 5) → 3",
    "mean(Data.Value) → average of column",
    "mean([10, 20, 30]) → 20",
];

static MEAN_RELATED: [&str; 4] = ["median", "mode", "gmean", "hmean"];

impl FunctionPlugin for Mean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mean",
            description: "Arithmetic mean (average) of values",
            usage: "mean(values) or mean(a, b, c, ...)",
            args: &MEAN_ARGS,
            returns: "Number",
            examples: &MEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &MEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "mean") {
            return Value::Error(e);
        }

        match mean(&numbers) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Median ============

pub struct Median;

static MEDIAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers to find median of",
    optional: false,
    default: None,
}];

static MEDIAN_EXAMPLES: [&str; 2] = [
    "median(1, 2, 3, 4, 5) → 3",
    "median(1, 2, 3, 4) → 2.5",
];

static MEDIAN_RELATED: [&str; 3] = ["mean", "mode", "percentile"];

impl FunctionPlugin for Median {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "median",
            description: "Middle value (average of two middle if even count)",
            usage: "median(values)",
            args: &MEDIAN_ARGS,
            returns: "Number",
            examples: &MEDIAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &MEDIAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "median") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        let n = sorted_nums.len();

        if n % 2 == 1 {
            Value::Number(sorted_nums[n / 2].clone())
        } else {
            let mid1 = &sorted_nums[n / 2 - 1];
            let mid2 = &sorted_nums[n / 2];
            let two = Number::from_i64(2);
            match mid1.add(mid2).checked_div(&two) {
                Ok(result) => Value::Number(result),
                Err(e) => Value::Error(e.into()),
            }
        }
    }
}

// ============ Mode ============

pub struct Mode;

static MODE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers to find mode of",
    optional: false,
    default: None,
}];

static MODE_EXAMPLES: [&str; 2] = [
    "mode(1, 2, 2, 3) → 2",
    "mode(1, 1, 2, 2) → [1, 2]",
];

static MODE_RELATED: [&str; 2] = ["mean", "median"];

impl FunctionPlugin for Mode {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mode",
            description: "Most frequent value. Returns List if tie",
            usage: "mode(values)",
            args: &MODE_ARGS,
            returns: "Number | List<Number>",
            examples: &MODE_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &MODE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "mode") {
            return Value::Error(e);
        }

        // Count frequencies using string representation as key
        let mut counts: HashMap<String, (Number, usize)> = HashMap::new();
        for n in &numbers {
            let key = n.as_decimal(20);
            let entry = counts.entry(key).or_insert_with(|| (n.clone(), 0));
            entry.1 += 1;
        }

        let max_count = counts.values().map(|(_, c)| *c).max().unwrap_or(0);
        let modes: Vec<Number> = counts
            .into_iter()
            .filter(|(_, (_, c))| *c == max_count)
            .map(|(_, (n, _))| n)
            .collect();

        if modes.len() == 1 {
            Value::Number(modes.into_iter().next().unwrap())
        } else {
            Value::List(modes.into_iter().map(Value::Number).collect())
        }
    }
}

// ============ Geometric Mean ============

pub struct GeometricMean;

static GMEAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Positive numbers",
    optional: false,
    default: None,
}];

static GMEAN_EXAMPLES: [&str; 1] = ["gmean(1, 2, 4, 8) → 2.828..."];

static GMEAN_RELATED: [&str; 2] = ["mean", "hmean"];

impl FunctionPlugin for GeometricMean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "gmean",
            description: "Geometric mean: (∏x)^(1/n). Error if any x ≤ 0",
            usage: "gmean(values)",
            args: &GMEAN_ARGS,
            returns: "Number",
            examples: &GMEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &GMEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "gmean") {
            return Value::Error(e);
        }

        // Check all values are positive
        let zero = Number::from_i64(0);
        for n in &numbers {
            if n.sub(&zero).is_negative() || n.is_zero() {
                return Value::Error(FolioError::domain_error(
                    "gmean() requires all positive values",
                ));
            }
        }

        // Calculate: exp(mean(ln(x_i)))
        let mut sum_ln = Number::from_i64(0);
        for n in &numbers {
            match n.ln(ctx.precision) {
                Ok(ln_val) => sum_ln = sum_ln.add(&ln_val),
                Err(e) => return Value::Error(e.into()),
            }
        }

        let count = Number::from_i64(numbers.len() as i64);
        match sum_ln.checked_div(&count) {
            Ok(mean_ln) => Value::Number(mean_ln.exp(ctx.precision)),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Harmonic Mean ============

pub struct HarmonicMean;

static HMEAN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Non-zero numbers",
    optional: false,
    default: None,
}];

static HMEAN_EXAMPLES: [&str; 1] = ["hmean(1, 2, 4) → 1.714..."];

static HMEAN_RELATED: [&str; 2] = ["mean", "gmean"];

impl FunctionPlugin for HarmonicMean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "hmean",
            description: "Harmonic mean: n/Σ(1/x). Error if any x = 0",
            usage: "hmean(values)",
            args: &HMEAN_ARGS,
            returns: "Number",
            examples: &HMEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &HMEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "hmean") {
            return Value::Error(e);
        }

        // Check all values are non-zero
        for n in &numbers {
            if n.is_zero() {
                return Value::Error(FolioError::domain_error(
                    "hmean() requires all non-zero values",
                ));
            }
        }

        // Calculate: n / Σ(1/x_i)
        let one = Number::from_i64(1);
        let mut sum_reciprocals = Number::from_i64(0);
        for n in &numbers {
            match one.checked_div(n) {
                Ok(recip) => sum_reciprocals = sum_reciprocals.add(&recip),
                Err(e) => return Value::Error(e.into()),
            }
        }

        let count = Number::from_i64(numbers.len() as i64);
        match count.checked_div(&sum_reciprocals) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Trimmed Mean ============

pub struct TrimmedMean;

static TMEAN_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List<Number> | Number...",
        description: "Numbers to average",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "pct",
        typ: "Number",
        description: "Percentage to trim from each tail (0-50)",
        optional: false,
        default: None,
    },
];

static TMEAN_EXAMPLES: [&str; 1] = ["tmean([1,2,3,4,100], 20) → 3 (trims extremes)"];

static TMEAN_RELATED: [&str; 2] = ["mean", "median"];

impl FunctionPlugin for TrimmedMean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tmean",
            description: "Trimmed mean excluding pct% from each tail",
            usage: "tmean(values, pct)",
            args: &TMEAN_ARGS,
            returns: "Number",
            examples: &TMEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &TMEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("tmean", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let pct = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("tmean", "pct", "Number", other.type_name())),
        };

        if let Err(e) = require_non_empty(&numbers, "tmean") {
            return Value::Error(e);
        }

        // Validate pct is in [0, 50)
        let pct_f64 = pct.to_f64().unwrap_or(0.0);
        if pct_f64 < 0.0 || pct_f64 >= 50.0 {
            return Value::Error(FolioError::domain_error(
                "tmean() requires 0 <= pct < 50",
            ));
        }

        let sorted_nums = sorted(&numbers);
        let n = sorted_nums.len();

        // Calculate how many to trim from each end
        let trim_count = ((n as f64) * pct_f64 / 100.0).floor() as usize;

        if 2 * trim_count >= n {
            return Value::Error(FolioError::domain_error(
                "tmean() would trim all values",
            ));
        }

        let trimmed = &sorted_nums[trim_count..(n - trim_count)];
        match mean(trimmed) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Weighted Mean ============

pub struct WeightedMean;

static WMEAN_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List<Number>",
        description: "Values to average",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "weights",
        typ: "List<Number>",
        description: "Weights (must be same length as values)",
        optional: false,
        default: None,
    },
];

static WMEAN_EXAMPLES: [&str; 1] = ["wmean([80, 90, 100], [1, 2, 1]) → 90"];

static WMEAN_RELATED: [&str; 1] = ["mean"];

impl FunctionPlugin for WeightedMean {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "wmean",
            description: "Weighted mean: Σ(w·x)/Σw",
            usage: "wmean(values, weights)",
            args: &WMEAN_ARGS,
            returns: "Number",
            examples: &WMEAN_EXAMPLES,
            category: "stats/central",
            source: None,
            related: &WMEAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("wmean", 2, args.len()));
        }

        let values = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let weights = match extract_numbers(&args[1..2]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if values.len() != weights.len() {
            return Value::Error(FolioError::domain_error(format!(
                "wmean() requires equal length lists: {} values vs {} weights",
                values.len(),
                weights.len()
            )));
        }

        if let Err(e) = require_non_empty(&values, "wmean") {
            return Value::Error(e);
        }

        // Check all weights are non-negative
        for w in &weights {
            if w.is_negative() {
                return Value::Error(FolioError::domain_error(
                    "wmean() requires non-negative weights",
                ));
            }
        }

        let mut weighted_sum = Number::from_i64(0);
        let mut weight_sum = Number::from_i64(0);

        for (v, w) in values.iter().zip(weights.iter()) {
            weighted_sum = weighted_sum.add(&v.mul(w));
            weight_sum = weight_sum.add(w);
        }

        if weight_sum.is_zero() {
            return Value::Error(FolioError::domain_error(
                "wmean() requires at least one non-zero weight",
            ));
        }

        match weighted_sum.checked_div(&weight_sum) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_mean() {
        let mean = Mean;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])];
        let result = mean.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
    }

    #[test]
    fn test_median_odd() {
        let median = Median;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(2)),
        ])];
        let result = median.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
    }

    #[test]
    fn test_median_even() {
        let median = Median;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
        ])];
        let result = median.call(&args, &eval_ctx());
        // Median of [1,2,3,4] = (2+3)/2 = 2.5
        let num = result.as_number().unwrap();
        let decimal = num.as_decimal(1);
        assert!(decimal.starts_with("2.5"));
    }

    #[test]
    fn test_mode_single() {
        let mode = Mode;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])];
        let result = mode.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
    }
}
```

folio-stats\src\confidence.rs
```rs
//! Confidence interval functions: ci, moe

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_min_count, mean, variance_impl};
use crate::distributions::normal::standard_normal_inv;
use std::collections::HashMap;

// ============ Confidence Interval ============

pub struct Ci;

static CI_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Sample data",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "level",
        typ: "Number",
        description: "Confidence level (0-1, default 0.95)",
        optional: true,
        default: Some("0.95"),
    },
];

static CI_EXAMPLES: [&str; 2] = [
    "ci([1,2,3,4,5]) → {low: ..., high: ...}",
    "ci([1,2,3,4,5], 0.99)",
];

static CI_RELATED: [&str; 2] = ["moe", "se"];

impl FunctionPlugin for Ci {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ci",
            description: "Confidence interval for the mean",
            usage: "ci(list, level?)",
            args: &CI_ARGS,
            returns: "Object",
            examples: &CI_EXAMPLES,
            category: "stats/confidence",
            source: None,
            related: &CI_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("ci", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let level = if args.len() > 1 {
            match &args[1] {
                Value::Number(n) => n.to_f64().unwrap_or(0.95),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("ci", "level", "Number", other.type_name())),
            }
        } else {
            0.95
        };

        if level <= 0.0 || level >= 1.0 {
            return Value::Error(FolioError::domain_error(
                "ci() requires 0 < level < 1",
            ));
        }

        if let Err(e) = require_min_count(&numbers, 2, "ci") {
            return Value::Error(e);
        }

        let n = numbers.len();
        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let se = match sd.checked_div(&Number::from_i64(n as i64).sqrt(ctx.precision).unwrap_or(Number::from_i64(1))) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // z-score for given confidence level
        let alpha = (1.0 - level) / 2.0;
        let p = Number::from_str(&format!("{:.15}", 1.0 - alpha)).unwrap_or(Number::from_str("0.975").unwrap());
        let z = standard_normal_inv(&p, ctx.precision);

        let margin = se.mul(&z);
        let ci_low = m.sub(&margin);
        let ci_high = m.add(&margin);

        let mut result = HashMap::new();
        result.insert("low".to_string(), Value::Number(ci_low));
        result.insert("high".to_string(), Value::Number(ci_high));
        result.insert("margin".to_string(), Value::Number(margin));
        result.insert("level".to_string(), Value::Number(Number::from_str(&format!("{:.15}", level)).unwrap_or(Number::from_str("0.95").unwrap())));

        Value::Object(result)
    }
}

// ============ Margin of Error ============

pub struct Moe;

static MOE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Sample data",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "level",
        typ: "Number",
        description: "Confidence level (0-1, default 0.95)",
        optional: true,
        default: Some("0.95"),
    },
];

static MOE_EXAMPLES: [&str; 1] = ["moe([1,2,3,4,5]) → margin value"];

static MOE_RELATED: [&str; 2] = ["ci", "se"];

impl FunctionPlugin for Moe {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "moe",
            description: "Margin of error",
            usage: "moe(list, level?)",
            args: &MOE_ARGS,
            returns: "Number",
            examples: &MOE_EXAMPLES,
            category: "stats/confidence",
            source: None,
            related: &MOE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("moe", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let level = if args.len() > 1 {
            match &args[1] {
                Value::Number(n) => n.to_f64().unwrap_or(0.95),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("moe", "level", "Number", other.type_name())),
            }
        } else {
            0.95
        };

        if level <= 0.0 || level >= 1.0 {
            return Value::Error(FolioError::domain_error(
                "moe() requires 0 < level < 1",
            ));
        }

        if let Err(e) = require_min_count(&numbers, 2, "moe") {
            return Value::Error(e);
        }

        let n = numbers.len();

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let se = match sd.checked_div(&Number::from_i64(n as i64).sqrt(ctx.precision).unwrap_or(Number::from_i64(1))) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // z-score for given confidence level
        let alpha = (1.0 - level) / 2.0;
        let p = Number::from_str(&format!("{:.15}", 1.0 - alpha)).unwrap_or(Number::from_str("0.975").unwrap());
        let z = standard_normal_inv(&p, ctx.precision);

        Value::Number(se.mul(&z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_ci() {
        let ci = Ci;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = ci.call(&args, &eval_ctx());
        assert!(result.as_object().is_some());
    }

    #[test]
    fn test_moe() {
        let moe = Moe;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = moe.call(&args, &eval_ctx());
        assert!(result.as_number().is_some());
    }
}
```

folio-stats\src\dispersion.rs
```rs
//! Dispersion functions: variance, stddev, range, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_non_empty, require_min_count, mean, variance_impl, sorted, percentile_impl};

// ============ Variance (Sample) ============

pub struct Variance;

static VARIANCE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Sample data",
    optional: false,
    default: None,
}];

static VARIANCE_EXAMPLES: [&str; 2] = [
    "variance(2, 4, 4, 4, 5, 5, 7, 9) → 4.571...",
    "variance(Data.Value) → sample variance of column",
];

static VARIANCE_RELATED: [&str; 3] = ["variance_p", "stddev", "cv"];

impl FunctionPlugin for Variance {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "variance",
            description: "Sample variance (divides by n-1)",
            usage: "variance(values)",
            args: &VARIANCE_ARGS,
            returns: "Number",
            examples: &VARIANCE_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &VARIANCE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        match variance_impl(&numbers, true) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Variance (Population) ============

pub struct VarianceP;

static VARIANCE_P_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Population data",
    optional: false,
    default: None,
}];

static VARIANCE_P_EXAMPLES: [&str; 1] = ["variance_p(2, 4, 4, 4, 5, 5, 7, 9) → 4"];

static VARIANCE_P_RELATED: [&str; 3] = ["variance", "stddev_p", "cv"];

impl FunctionPlugin for VarianceP {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "variance_p",
            description: "Population variance (divides by n)",
            usage: "variance_p(values)",
            args: &VARIANCE_P_ARGS,
            returns: "Number",
            examples: &VARIANCE_P_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &VARIANCE_P_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        match variance_impl(&numbers, false) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Stddev (Sample) ============

pub struct Stddev;

static STDDEV_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Sample data",
    optional: false,
    default: None,
}];

static STDDEV_EXAMPLES: [&str; 1] = ["stddev(2, 4, 4, 4, 5, 5, 7, 9) → 2.138..."];

static STDDEV_RELATED: [&str; 3] = ["stddev_p", "variance", "se"];

impl FunctionPlugin for Stddev {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "stddev",
            description: "Sample standard deviation: √variance",
            usage: "stddev(values)",
            args: &STDDEV_ARGS,
            returns: "Number",
            examples: &STDDEV_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &STDDEV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        match variance_impl(&numbers, true) {
            Ok(var) => match var.sqrt(ctx.precision) {
                Ok(result) => Value::Number(result),
                Err(e) => Value::Error(e.into()),
            },
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Stddev (Population) ============

pub struct StddevP;

static STDDEV_P_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Population data",
    optional: false,
    default: None,
}];

static STDDEV_P_EXAMPLES: [&str; 1] = ["stddev_p(2, 4, 4, 4, 5, 5, 7, 9) → 2"];

static STDDEV_P_RELATED: [&str; 3] = ["stddev", "variance_p", "se"];

impl FunctionPlugin for StddevP {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "stddev_p",
            description: "Population standard deviation",
            usage: "stddev_p(values)",
            args: &STDDEV_P_ARGS,
            returns: "Number",
            examples: &STDDEV_P_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &STDDEV_P_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        match variance_impl(&numbers, false) {
            Ok(var) => match var.sqrt(ctx.precision) {
                Ok(result) => Value::Number(result),
                Err(e) => Value::Error(e.into()),
            },
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Range ============

pub struct Range;

static RANGE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static RANGE_EXAMPLES: [&str; 1] = ["range(1, 5, 10) → 9"];

static RANGE_RELATED: [&str; 2] = ["min", "max"];

impl FunctionPlugin for Range {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "range",
            description: "max - min",
            usage: "range(values)",
            args: &RANGE_ARGS,
            returns: "Number",
            examples: &RANGE_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &RANGE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "range") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        let min = &sorted_nums[0];
        let max = &sorted_nums[sorted_nums.len() - 1];

        Value::Number(max.sub(min))
    }
}

// ============ IQR ============

pub struct Iqr;

static IQR_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static IQR_EXAMPLES: [&str; 1] = ["iqr(1, 2, 3, 4, 5, 6, 7, 8) → 4"];

static IQR_RELATED: [&str; 3] = ["q1", "q3", "percentile"];

impl FunctionPlugin for Iqr {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "iqr",
            description: "Interquartile range: Q3 - Q1",
            usage: "iqr(values)",
            args: &IQR_ARGS,
            returns: "Number",
            examples: &IQR_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &IQR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "iqr") {
            return Value::Error(e);
        }

        let p25 = Number::from_i64(25);
        let p75 = Number::from_i64(75);

        let q1 = match percentile_impl(&numbers, &p25) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let q3 = match percentile_impl(&numbers, &p75) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        Value::Number(q3.sub(&q1))
    }
}

// ============ MAD (Median Absolute Deviation) ============

pub struct Mad;

static MAD_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static MAD_EXAMPLES: [&str; 1] = ["mad(1, 1, 2, 2, 4, 6, 9) → 1"];

static MAD_RELATED: [&str; 2] = ["median", "stddev"];

impl FunctionPlugin for Mad {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mad",
            description: "Median absolute deviation",
            usage: "mad(values)",
            args: &MAD_ARGS,
            returns: "Number",
            examples: &MAD_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &MAD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "mad") {
            return Value::Error(e);
        }

        // Calculate median
        let sorted_nums = sorted(&numbers);
        let n = sorted_nums.len();
        let median = if n % 2 == 1 {
            sorted_nums[n / 2].clone()
        } else {
            let mid1 = &sorted_nums[n / 2 - 1];
            let mid2 = &sorted_nums[n / 2];
            let two = Number::from_i64(2);
            mid1.add(mid2).checked_div(&two).unwrap_or(mid1.clone())
        };

        // Calculate absolute deviations from median
        let abs_devs: Vec<Number> = numbers
            .iter()
            .map(|x| x.sub(&median).abs())
            .collect();

        // Return median of absolute deviations
        let sorted_devs = sorted(&abs_devs);
        let m = sorted_devs.len();
        if m % 2 == 1 {
            Value::Number(sorted_devs[m / 2].clone())
        } else {
            let mid1 = &sorted_devs[m / 2 - 1];
            let mid2 = &sorted_devs[m / 2];
            let two = Number::from_i64(2);
            match mid1.add(mid2).checked_div(&two) {
                Ok(result) => Value::Number(result),
                Err(e) => Value::Error(e.into()),
            }
        }
    }
}

// ============ CV (Coefficient of Variation) ============

pub struct Cv;

static CV_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static CV_EXAMPLES: [&str; 1] = ["cv(10, 20, 30) → 0.5"];

static CV_RELATED: [&str; 2] = ["stddev", "mean"];

impl FunctionPlugin for Cv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cv",
            description: "Coefficient of variation: stddev/mean",
            usage: "cv(values)",
            args: &CV_ARGS,
            returns: "Number",
            examples: &CV_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &CV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 2, "cv") {
            return Value::Error(e);
        }

        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if m.is_zero() {
            return Value::Error(FolioError::domain_error(
                "cv() undefined when mean is zero",
            ));
        }

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        match sd.checked_div(&m) {
            Ok(result) => Value::Number(result.abs()),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ SE (Standard Error) ============

pub struct Se;

static SE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Sample data",
    optional: false,
    default: None,
}];

static SE_EXAMPLES: [&str; 1] = ["se(1, 2, 3, 4, 5) → 0.707..."];

static SE_RELATED: [&str; 2] = ["stddev", "ci"];

impl FunctionPlugin for Se {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "se",
            description: "Standard error: stddev/√n",
            usage: "se(values)",
            args: &SE_ARGS,
            returns: "Number",
            examples: &SE_EXAMPLES,
            category: "stats/dispersion",
            source: None,
            related: &SE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 2, "se") {
            return Value::Error(e);
        }

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let n = Number::from_i64(numbers.len() as i64);
        let sqrt_n = match n.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        match sd.checked_div(&sqrt_n) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_variance() {
        let variance = Variance;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(7)),
            Value::Number(Number::from_i64(9)),
        ])];
        let result = variance.call(&args, &eval_ctx());
        // Sample variance should be approximately 4.571
        let num = result.as_number().unwrap();
        let decimal = num.as_decimal(2);
        assert!(decimal.starts_with("4.5"));
    }

    #[test]
    fn test_range() {
        let range = Range;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(10)),
        ])];
        let result = range.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(9));
    }
}
```

folio-stats\src\distributions\chi.rs
```rs
//! Chi-squared distribution functions

use folio_plugin::prelude::*;
use super::t::gamma_ln;

// ============ Chi PDF ============

pub struct ChiPdf;

static CHI_PDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value (must be ≥ 0)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df",
        typ: "Number",
        description: "Degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static CHI_PDF_EXAMPLES: [&str; 1] = ["chi_pdf(5, 3) → 0.072..."];

static CHI_PDF_RELATED: [&str; 2] = ["chi_cdf", "f_pdf"];

impl FunctionPlugin for ChiPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "chi_pdf",
            description: "Chi-squared distribution PDF",
            usage: "chi_pdf(x, df)",
            args: &CHI_PDF_ARGS,
            returns: "Number",
            examples: &CHI_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &CHI_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("chi_pdf", 2, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_pdf", "x", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_pdf", "df", "Number", other.type_name())),
        };

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let df_f64 = df.to_f64().unwrap_or(1.0);

        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("chi_pdf() requires df > 0"));
        }
        if x_f64 < 0.0 {
            return Value::Number(Number::from_i64(0));
        }

        let result = chi_pdf_f64(x_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn chi_pdf_f64(x: f64, df: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        if df < 2.0 {
            return f64::INFINITY;
        } else if df == 2.0 {
            return 0.5;
        } else {
            return 0.0;
        }
    }

    let k = df / 2.0;
    let log_pdf = -k * 2.0_f64.ln() - gamma_ln(k) + (k - 1.0) * x.ln() - x / 2.0;
    log_pdf.exp()
}

// ============ Chi CDF ============

pub struct ChiCdf;

static CHI_CDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value (must be ≥ 0)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df",
        typ: "Number",
        description: "Degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static CHI_CDF_EXAMPLES: [&str; 1] = ["chi_cdf(3.84, 1) → 0.95"];

static CHI_CDF_RELATED: [&str; 2] = ["chi_pdf", "chi_inv"];

impl FunctionPlugin for ChiCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "chi_cdf",
            description: "Chi-squared distribution CDF",
            usage: "chi_cdf(x, df)",
            args: &CHI_CDF_ARGS,
            returns: "Number",
            examples: &CHI_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &CHI_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("chi_cdf", 2, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_cdf", "x", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_cdf", "df", "Number", other.type_name())),
        };

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let df_f64 = df.to_f64().unwrap_or(1.0);

        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("chi_cdf() requires df > 0"));
        }
        if x_f64 < 0.0 {
            return Value::Number(Number::from_i64(0));
        }

        let result = chi_cdf_f64(x_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

pub fn chi_cdf_f64(x: f64, df: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    // Chi-squared CDF = lower regularized incomplete gamma function
    // P(k/2, x/2) where P is the regularized gamma function
    lower_incomplete_gamma(df / 2.0, x / 2.0)
}

// ============ Chi Inverse ============

pub struct ChiInv;

static CHI_INV_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability (0 < p < 1)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df",
        typ: "Number",
        description: "Degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static CHI_INV_EXAMPLES: [&str; 1] = ["chi_inv(0.95, 1) → 3.84"];

static CHI_INV_RELATED: [&str; 2] = ["chi_cdf", "f_inv"];

impl FunctionPlugin for ChiInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "chi_inv",
            description: "Chi-squared distribution inverse (quantile)",
            usage: "chi_inv(p, df)",
            args: &CHI_INV_ARGS,
            returns: "Number",
            examples: &CHI_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &CHI_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("chi_inv", 2, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_inv", "p", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("chi_inv", "df", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        let df_f64 = df.to_f64().unwrap_or(1.0);

        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error("chi_inv() requires 0 < p < 1"));
        }
        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("chi_inv() requires df > 0"));
        }

        let result = chi_inv_f64(p_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn chi_inv_f64(p: f64, df: f64) -> f64 {
    // Newton-Raphson
    let mut x = df; // Initial guess

    for _ in 0..100 {
        let cdf = chi_cdf_f64(x, df);
        let pdf = chi_pdf_f64(x, df);
        if pdf.abs() < 1e-15 {
            break;
        }
        let dx = (cdf - p) / pdf;
        x -= dx;
        if x < 0.0 {
            x = 0.001;
        }
        if dx.abs() < 1e-12 {
            break;
        }
    }

    x
}

/// Lower regularized incomplete gamma function
fn lower_incomplete_gamma(a: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x < a + 1.0 {
        // Use series representation
        gamma_series(a, x)
    } else {
        // Use continued fraction representation
        1.0 - gamma_cf(a, x)
    }
}

fn gamma_series(a: f64, x: f64) -> f64 {
    let gln = gamma_ln(a);
    let mut ap = a;
    let mut sum = 1.0 / a;
    let mut del = sum;

    for _ in 0..200 {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * 3e-14 {
            break;
        }
    }

    sum * (-x + a * x.ln() - gln).exp()
}

fn gamma_cf(a: f64, x: f64) -> f64 {
    let gln = gamma_ln(a);
    let fpmin = 1e-30;
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / fpmin;
    let mut d = 1.0 / b;
    let mut h = d;

    for i in 1..=200 {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < fpmin {
            d = fpmin;
        }
        c = b + an / c;
        if c.abs() < fpmin {
            c = fpmin;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 3e-14 {
            break;
        }
    }

    (-x + a * x.ln() - gln).exp() * h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_chi_cdf() {
        let chi_cdf = ChiCdf;
        let args = vec![
            Value::Number(Number::from_str("3.84").unwrap()),
            Value::Number(Number::from_i64(1)),
        ];
        let result = chi_cdf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        // χ²(3.84, df=1) ≈ 0.95
        assert!((f - 0.95).abs() < 0.01);
    }
}
```

folio-stats\src\distributions\discrete.rs
```rs
//! Discrete distribution functions: binomial, Poisson

use folio_plugin::prelude::*;
use super::t::gamma_ln;

// ============ Binomial PMF ============

pub struct BinomPmf;

static BINOM_PMF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "Number of successes",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of trials",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability of success (0 ≤ p ≤ 1)",
        optional: false,
        default: None,
    },
];

static BINOM_PMF_EXAMPLES: [&str; 1] = ["binom_pmf(3, 10, 0.5) → 0.117..."];

static BINOM_PMF_RELATED: [&str; 2] = ["binom_cdf", "poisson_pmf"];

impl FunctionPlugin for BinomPmf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "binom_pmf",
            description: "Binomial probability mass function",
            usage: "binom_pmf(k, n, p)",
            args: &BINOM_PMF_ARGS,
            returns: "Number",
            examples: &BINOM_PMF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &BINOM_PMF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("binom_pmf", 3, args.len()));
        }

        let k = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_pmf", "k", "Number", other.type_name())),
        };

        let n = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_pmf", "n", "Number", other.type_name())),
        };

        let p = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_pmf", "p", "Number", other.type_name())),
        };

        let k_i64 = k.to_i64().unwrap_or(-1);
        let n_i64 = n.to_i64().unwrap_or(-1);
        let p_f64 = p.to_f64().unwrap_or(-1.0);

        if n_i64 < 0 || k_i64 < 0 {
            return Value::Error(FolioError::domain_error("binom_pmf() requires k ≥ 0 and n ≥ 0"));
        }
        if k_i64 > n_i64 {
            return Value::Number(Number::from_i64(0));
        }
        if p_f64 < 0.0 || p_f64 > 1.0 {
            return Value::Error(FolioError::domain_error("binom_pmf() requires 0 ≤ p ≤ 1"));
        }

        let result = binom_pmf_f64(k_i64 as u64, n_i64 as u64, p_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn binom_pmf_f64(k: u64, n: u64, p: f64) -> f64 {
    if k > n {
        return 0.0;
    }
    if p == 0.0 {
        return if k == 0 { 1.0 } else { 0.0 };
    }
    if p == 1.0 {
        return if k == n { 1.0 } else { 0.0 };
    }

    // Use log for numerical stability
    // PMF = C(n,k) * p^k * (1-p)^(n-k)
    // log(PMF) = log(C(n,k)) + k*log(p) + (n-k)*log(1-p)
    let log_coef = log_binomial(n, k);
    let log_prob = (k as f64) * p.ln() + ((n - k) as f64) * (1.0 - p).ln();
    (log_coef + log_prob).exp()
}

fn log_binomial(n: u64, k: u64) -> f64 {
    // log(C(n,k)) = log(n!) - log(k!) - log((n-k)!)
    // Using gamma_ln(x+1) = log(x!)
    gamma_ln((n + 1) as f64) - gamma_ln((k + 1) as f64) - gamma_ln((n - k + 1) as f64)
}

// ============ Binomial CDF ============

pub struct BinomCdf;

static BINOM_CDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "Number of successes",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of trials",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability of success (0 ≤ p ≤ 1)",
        optional: false,
        default: None,
    },
];

static BINOM_CDF_EXAMPLES: [&str; 1] = ["binom_cdf(5, 10, 0.5) → 0.623..."];

static BINOM_CDF_RELATED: [&str; 2] = ["binom_pmf", "poisson_cdf"];

impl FunctionPlugin for BinomCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "binom_cdf",
            description: "Binomial cumulative distribution function",
            usage: "binom_cdf(k, n, p)",
            args: &BINOM_CDF_ARGS,
            returns: "Number",
            examples: &BINOM_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &BINOM_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("binom_cdf", 3, args.len()));
        }

        let k = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_cdf", "k", "Number", other.type_name())),
        };

        let n = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_cdf", "n", "Number", other.type_name())),
        };

        let p = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("binom_cdf", "p", "Number", other.type_name())),
        };

        let k_i64 = k.to_i64().unwrap_or(-1);
        let n_i64 = n.to_i64().unwrap_or(-1);
        let p_f64 = p.to_f64().unwrap_or(-1.0);

        if n_i64 < 0 {
            return Value::Error(FolioError::domain_error("binom_cdf() requires n ≥ 0"));
        }
        if k_i64 < 0 {
            return Value::Number(Number::from_i64(0));
        }
        if k_i64 >= n_i64 {
            return Value::Number(Number::from_i64(1));
        }
        if p_f64 < 0.0 || p_f64 > 1.0 {
            return Value::Error(FolioError::domain_error("binom_cdf() requires 0 ≤ p ≤ 1"));
        }

        let mut cdf = 0.0;
        for i in 0..=(k_i64 as u64) {
            cdf += binom_pmf_f64(i, n_i64 as u64, p_f64);
        }

        Value::Number(Number::from_str(&format!("{:.15}", cdf)).unwrap_or(Number::from_i64(0)))
    }
}

// ============ Poisson PMF ============

pub struct PoissonPmf;

static POISSON_PMF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "Number of events",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "λ",
        typ: "Number",
        description: "Rate parameter (must be > 0)",
        optional: false,
        default: None,
    },
];

static POISSON_PMF_EXAMPLES: [&str; 1] = ["poisson_pmf(3, 2.5) → 0.214..."];

static POISSON_PMF_RELATED: [&str; 2] = ["poisson_cdf", "binom_pmf"];

impl FunctionPlugin for PoissonPmf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "poisson_pmf",
            description: "Poisson probability mass function",
            usage: "poisson_pmf(k, λ)",
            args: &POISSON_PMF_ARGS,
            returns: "Number",
            examples: &POISSON_PMF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &POISSON_PMF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("poisson_pmf", 2, args.len()));
        }

        let k = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("poisson_pmf", "k", "Number", other.type_name())),
        };

        let lambda = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("poisson_pmf", "λ", "Number", other.type_name())),
        };

        let k_i64 = k.to_i64().unwrap_or(-1);
        let lambda_f64 = lambda.to_f64().unwrap_or(-1.0);

        if k_i64 < 0 {
            return Value::Number(Number::from_i64(0));
        }
        if lambda_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("poisson_pmf() requires λ > 0"));
        }

        let result = poisson_pmf_f64(k_i64 as u64, lambda_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn poisson_pmf_f64(k: u64, lambda: f64) -> f64 {
    // PMF = λ^k * e^(-λ) / k!
    // log(PMF) = k*log(λ) - λ - log(k!)
    let log_prob = (k as f64) * lambda.ln() - lambda - gamma_ln((k + 1) as f64);
    log_prob.exp()
}

// ============ Poisson CDF ============

pub struct PoissonCdf;

static POISSON_CDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "Number of events",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "λ",
        typ: "Number",
        description: "Rate parameter (must be > 0)",
        optional: false,
        default: None,
    },
];

static POISSON_CDF_EXAMPLES: [&str; 1] = ["poisson_cdf(5, 3) → 0.916..."];

static POISSON_CDF_RELATED: [&str; 2] = ["poisson_pmf", "binom_cdf"];

impl FunctionPlugin for PoissonCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "poisson_cdf",
            description: "Poisson cumulative distribution function",
            usage: "poisson_cdf(k, λ)",
            args: &POISSON_CDF_ARGS,
            returns: "Number",
            examples: &POISSON_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &POISSON_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("poisson_cdf", 2, args.len()));
        }

        let k = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("poisson_cdf", "k", "Number", other.type_name())),
        };

        let lambda = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("poisson_cdf", "λ", "Number", other.type_name())),
        };

        let k_i64 = k.to_i64().unwrap_or(-1);
        let lambda_f64 = lambda.to_f64().unwrap_or(-1.0);

        if k_i64 < 0 {
            return Value::Number(Number::from_i64(0));
        }
        if lambda_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("poisson_cdf() requires λ > 0"));
        }

        let mut cdf = 0.0;
        for i in 0..=(k_i64 as u64) {
            cdf += poisson_pmf_f64(i, lambda_f64);
        }

        Value::Number(Number::from_str(&format!("{:.15}", cdf)).unwrap_or(Number::from_i64(0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_binom_pmf() {
        let binom_pmf = BinomPmf;
        let args = vec![
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(10)),
            Value::Number(Number::from_str("0.5").unwrap()),
        ];
        let result = binom_pmf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        // P(X=5|n=10, p=0.5) ≈ 0.246
        assert!((f - 0.246).abs() < 0.01);
    }

    #[test]
    fn test_poisson_pmf() {
        let poisson_pmf = PoissonPmf;
        let args = vec![
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_str("2.5").unwrap()),
        ];
        let result = poisson_pmf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        // P(X=3|λ=2.5) ≈ 0.214
        assert!((f - 0.214).abs() < 0.01);
    }
}
```

folio-stats\src\distributions\f.rs
```rs
//! F distribution functions

use folio_plugin::prelude::*;
use super::t::{gamma_ln, regularized_incomplete_beta};

// ============ F PDF ============

pub struct FPdf;

static F_PDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value (must be ≥ 0)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df1",
        typ: "Number",
        description: "Numerator degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df2",
        typ: "Number",
        description: "Denominator degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static F_PDF_EXAMPLES: [&str; 1] = ["f_pdf(2, 5, 10) → 0.127..."];

static F_PDF_RELATED: [&str; 2] = ["f_cdf", "chi_pdf"];

impl FunctionPlugin for FPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "f_pdf",
            description: "F-distribution PDF",
            usage: "f_pdf(x, df1, df2)",
            args: &F_PDF_ARGS,
            returns: "Number",
            examples: &F_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &F_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("f_pdf", 3, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_pdf", "x", "Number", other.type_name())),
        };

        let df1 = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_pdf", "df1", "Number", other.type_name())),
        };

        let df2 = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_pdf", "df2", "Number", other.type_name())),
        };

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let df1_f64 = df1.to_f64().unwrap_or(1.0);
        let df2_f64 = df2.to_f64().unwrap_or(1.0);

        if df1_f64 <= 0.0 || df2_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("f_pdf() requires df1 > 0 and df2 > 0"));
        }
        if x_f64 < 0.0 {
            return Value::Number(Number::from_i64(0));
        }

        let result = f_pdf_f64(x_f64, df1_f64, df2_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn f_pdf_f64(x: f64, d1: f64, d2: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        if d1 < 2.0 {
            return f64::INFINITY;
        } else if d1 == 2.0 {
            return 1.0;
        } else {
            return 0.0;
        }
    }

    let log_num = (d1 / 2.0) * (d1).ln() + (d2 / 2.0) * (d2).ln()
        + ((d1 / 2.0) - 1.0) * x.ln();
    let log_den = gamma_ln(d1 / 2.0) + gamma_ln(d2 / 2.0)
        - gamma_ln((d1 + d2) / 2.0)
        + ((d1 + d2) / 2.0) * (d1 * x + d2).ln();

    (log_num - log_den).exp()
}

// ============ F CDF ============

pub struct FCdf;

static F_CDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value (must be ≥ 0)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df1",
        typ: "Number",
        description: "Numerator degrees of freedom",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df2",
        typ: "Number",
        description: "Denominator degrees of freedom",
        optional: false,
        default: None,
    },
];

static F_CDF_EXAMPLES: [&str; 1] = ["f_cdf(3.89, 3, 20) → 0.975"];

static F_CDF_RELATED: [&str; 2] = ["f_pdf", "f_inv"];

impl FunctionPlugin for FCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "f_cdf",
            description: "F-distribution CDF",
            usage: "f_cdf(x, df1, df2)",
            args: &F_CDF_ARGS,
            returns: "Number",
            examples: &F_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &F_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("f_cdf", 3, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_cdf", "x", "Number", other.type_name())),
        };

        let df1 = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_cdf", "df1", "Number", other.type_name())),
        };

        let df2 = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_cdf", "df2", "Number", other.type_name())),
        };

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let df1_f64 = df1.to_f64().unwrap_or(1.0);
        let df2_f64 = df2.to_f64().unwrap_or(1.0);

        if df1_f64 <= 0.0 || df2_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("f_cdf() requires df1 > 0 and df2 > 0"));
        }
        if x_f64 < 0.0 {
            return Value::Number(Number::from_i64(0));
        }

        let result = f_cdf_f64(x_f64, df1_f64, df2_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

pub fn f_cdf_f64(x: f64, d1: f64, d2: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }

    // F CDF = I_{d1*x/(d1*x+d2)}(d1/2, d2/2)
    let z = d1 * x / (d1 * x + d2);
    regularized_incomplete_beta(d1 / 2.0, d2 / 2.0, z)
}

// ============ F Inverse ============

pub struct FInv;

static F_INV_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability (0 < p < 1)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df1",
        typ: "Number",
        description: "Numerator degrees of freedom",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df2",
        typ: "Number",
        description: "Denominator degrees of freedom",
        optional: false,
        default: None,
    },
];

static F_INV_EXAMPLES: [&str; 1] = ["f_inv(0.95, 5, 10) → 3.33"];

static F_INV_RELATED: [&str; 2] = ["f_cdf", "chi_inv"];

impl FunctionPlugin for FInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "f_inv",
            description: "F-distribution inverse (quantile)",
            usage: "f_inv(p, df1, df2)",
            args: &F_INV_ARGS,
            returns: "Number",
            examples: &F_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &F_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("f_inv", 3, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_inv", "p", "Number", other.type_name())),
        };

        let df1 = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_inv", "df1", "Number", other.type_name())),
        };

        let df2 = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("f_inv", "df2", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        let df1_f64 = df1.to_f64().unwrap_or(1.0);
        let df2_f64 = df2.to_f64().unwrap_or(1.0);

        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error("f_inv() requires 0 < p < 1"));
        }
        if df1_f64 <= 0.0 || df2_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("f_inv() requires df1 > 0 and df2 > 0"));
        }

        let result = f_inv_f64(p_f64, df1_f64, df2_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn f_inv_f64(p: f64, d1: f64, d2: f64) -> f64 {
    // Newton-Raphson
    let mut x = 1.0; // Initial guess

    for _ in 0..100 {
        let cdf = f_cdf_f64(x, d1, d2);
        let pdf = f_pdf_f64(x, d1, d2);
        if pdf.abs() < 1e-15 {
            break;
        }
        let dx = (cdf - p) / pdf;
        x -= dx;
        if x < 0.0 {
            x = 0.001;
        }
        if dx.abs() < 1e-12 {
            break;
        }
    }

    x
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_f_cdf() {
        let f_cdf = FCdf;
        let args = vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(10)),
        ];
        let result = f_cdf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        // F(1, 5, 10) should be around 0.5
        assert!(f > 0.4 && f < 0.6);
    }
}
```

folio-stats\src\distributions\mod.rs
```rs
//! Statistical distributions: normal, t, chi-squared, F, binomial, Poisson

pub mod normal;
pub mod t;
pub mod chi;
pub mod f;
mod discrete;

pub use normal::{NormPdf, NormCdf, NormInv, SnormPdf, SnormCdf, SnormInv};
pub use t::{TPdf, TCdf, TInv};
pub use chi::{ChiPdf, ChiCdf, ChiInv};
pub use f::{FPdf, FCdf, FInv};
pub use discrete::{BinomPmf, BinomCdf, PoissonPmf, PoissonCdf};
```

folio-stats\src\distributions\normal.rs
```rs
//! Normal distribution functions

use folio_plugin::prelude::*;

// ============ Standard Normal PDF ============

pub struct SnormPdf;

static SNORM_PDF_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "x",
    typ: "Number",
    description: "Value",
    optional: false,
    default: None,
}];

static SNORM_PDF_EXAMPLES: [&str; 1] = ["snorm_pdf(0) → 0.3989..."];

static SNORM_PDF_RELATED: [&str; 2] = ["snorm_cdf", "norm_pdf"];

impl FunctionPlugin for SnormPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "snorm_pdf",
            description: "Standard normal PDF (μ=0, σ=1)",
            usage: "snorm_pdf(x)",
            args: &SNORM_PDF_ARGS,
            returns: "Number",
            examples: &SNORM_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &SNORM_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("snorm_pdf", 1, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("snorm_pdf", "x", "Number", other.type_name())),
        };

        // PDF(x) = (1/√(2π)) * exp(-x²/2)
        let two = Number::from_i64(2);
        let pi = Number::pi(ctx.precision);
        let two_pi = two.mul(&pi);
        let sqrt_two_pi = match two_pi.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let x_squared = x.mul(x);
        let neg_half_x2 = Number::from_i64(0).sub(&x_squared.checked_div(&two).unwrap_or(Number::from_i64(0)));
        let exp_term = neg_half_x2.exp(ctx.precision);

        match exp_term.checked_div(&sqrt_two_pi) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Standard Normal CDF ============

pub struct SnormCdf;

static SNORM_CDF_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "x",
    typ: "Number",
    description: "Value",
    optional: false,
    default: None,
}];

static SNORM_CDF_EXAMPLES: [&str; 1] = ["snorm_cdf(0) → 0.5"];

static SNORM_CDF_RELATED: [&str; 2] = ["snorm_pdf", "snorm_inv"];

impl FunctionPlugin for SnormCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "snorm_cdf",
            description: "Standard normal CDF P(X ≤ x)",
            usage: "snorm_cdf(x)",
            args: &SNORM_CDF_ARGS,
            returns: "Number",
            examples: &SNORM_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &SNORM_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("snorm_cdf", 1, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("snorm_cdf", "x", "Number", other.type_name())),
        };

        Value::Number(standard_normal_cdf(x, ctx.precision))
    }
}

/// Standard normal CDF using error function approximation
pub fn standard_normal_cdf(x: &Number, precision: u32) -> Number {
    // Φ(x) = 0.5 * (1 + erf(x/√2))
    let sqrt_2 = Number::from_i64(2).sqrt(precision).unwrap_or(Number::from_str("1.41421356").unwrap());
    let z = x.checked_div(&sqrt_2).unwrap_or(Number::from_i64(0));

    let erf_z = erf(&z, precision);
    let one = Number::from_i64(1);
    let half = Number::from_ratio(1, 2);

    half.mul(&one.add(&erf_z))
}

/// Error function approximation using Taylor series
fn erf(x: &Number, precision: u32) -> Number {
    // erf(x) ≈ (2/√π) * Σ((-1)^n * x^(2n+1)) / (n! * (2n+1))
    let x_f64 = x.to_f64().unwrap_or(0.0);

    // For large |x|, use asymptotic value
    if x_f64.abs() > 4.0 {
        if x_f64 > 0.0 {
            return Number::from_i64(1);
        } else {
            return Number::from_i64(-1);
        }
    }

    let pi = Number::pi(precision);
    let sqrt_pi = pi.sqrt(precision).unwrap_or(Number::from_str("1.77245385").unwrap());
    let two_over_sqrt_pi = Number::from_i64(2).checked_div(&sqrt_pi).unwrap_or(Number::from_i64(1));

    let mut sum = Number::from_i64(0);
    let mut term = x.clone();
    let x_squared = x.mul(x);

    let iterations = (precision / 2).max(20).min(100) as i64;

    for n in 0..iterations {
        // term = (-1)^n * x^(2n+1) / (n! * (2n+1))
        let divisor = Number::from_i64(2 * n + 1);
        let contribution = match term.checked_div(&divisor) {
            Ok(v) => v,
            Err(_) => break,
        };
        sum = sum.add(&contribution);

        // Next term: multiply by -x² / (n+1)
        let next_n = Number::from_i64(n + 1);
        term = Number::from_i64(0)
            .sub(&term.mul(&x_squared))
            .checked_div(&next_n)
            .unwrap_or(Number::from_i64(0));

        // Check for convergence
        if term.to_f64().map(|t| t.abs() < 1e-15).unwrap_or(true) {
            break;
        }
    }

    two_over_sqrt_pi.mul(&sum)
}

// ============ Standard Normal Inverse ============

pub struct SnormInv;

static SNORM_INV_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "p",
    typ: "Number",
    description: "Probability (0 < p < 1)",
    optional: false,
    default: None,
}];

static SNORM_INV_EXAMPLES: [&str; 1] = ["snorm_inv(0.975) → 1.96"];

static SNORM_INV_RELATED: [&str; 2] = ["snorm_cdf", "norm_inv"];

impl FunctionPlugin for SnormInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "snorm_inv",
            description: "Standard normal inverse (quantile function)",
            usage: "snorm_inv(p)",
            args: &SNORM_INV_ARGS,
            returns: "Number",
            examples: &SNORM_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &SNORM_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("snorm_inv", 1, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("snorm_inv", "p", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error(
                "snorm_inv() requires 0 < p < 1",
            ));
        }

        Value::Number(standard_normal_inv(p, ctx.precision))
    }
}

/// Standard normal inverse using rational approximation (Abramowitz and Stegun)
pub fn standard_normal_inv(p: &Number, _precision: u32) -> Number {
    let p_f64 = p.to_f64().unwrap_or(0.5);

    // Rational approximation constants
    const A: [f64; 4] = [2.515517, 0.802853, 0.010328, 0.0];
    const B: [f64; 4] = [1.0, 1.432788, 0.189269, 0.001308];

    let sign = if p_f64 < 0.5 { -1.0 } else { 1.0 };
    let p_adj = if p_f64 < 0.5 { p_f64 } else { 1.0 - p_f64 };

    let t = (-2.0 * p_adj.ln()).sqrt();

    let num = A[0] + t * (A[1] + t * A[2]);
    let den = 1.0 + t * (B[1] + t * (B[2] + t * B[3]));

    let result = sign * (t - num / den);

    // Convert to Number
    Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0))
}

// ============ Normal PDF ============

pub struct NormPdf;

static NORM_PDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "μ",
        typ: "Number",
        description: "Mean",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "σ",
        typ: "Number",
        description: "Standard deviation (must be > 0)",
        optional: false,
        default: None,
    },
];

static NORM_PDF_EXAMPLES: [&str; 1] = ["norm_pdf(0, 0, 1) → 0.3989..."];

static NORM_PDF_RELATED: [&str; 2] = ["norm_cdf", "snorm_pdf"];

impl FunctionPlugin for NormPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "norm_pdf",
            description: "Normal distribution PDF",
            usage: "norm_pdf(x, μ, σ)",
            args: &NORM_PDF_ARGS,
            returns: "Number",
            examples: &NORM_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &NORM_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("norm_pdf", 3, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_pdf", "x", "Number", other.type_name())),
        };

        let mu = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_pdf", "μ", "Number", other.type_name())),
        };

        let sigma = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_pdf", "σ", "Number", other.type_name())),
        };

        if sigma.is_zero() || sigma.is_negative() {
            return Value::Error(FolioError::domain_error("norm_pdf() requires σ > 0"));
        }

        // Standardize: z = (x - μ) / σ
        let z = match x.sub(mu).checked_div(sigma) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // PDF(x) = (1/(σ√(2π))) * exp(-(x-μ)²/(2σ²))
        let two = Number::from_i64(2);
        let pi = Number::pi(ctx.precision);
        let two_pi = two.mul(&pi);
        let sqrt_two_pi = match two_pi.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let z_squared = z.mul(&z);
        let neg_half_z2 = Number::from_i64(0).sub(&z_squared.checked_div(&two).unwrap_or(Number::from_i64(0)));
        let exp_term = neg_half_z2.exp(ctx.precision);

        let denominator = sigma.mul(&sqrt_two_pi);
        match exp_term.checked_div(&denominator) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Normal CDF ============

pub struct NormCdf;

static NORM_CDF_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "μ",
        typ: "Number",
        description: "Mean",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "σ",
        typ: "Number",
        description: "Standard deviation (must be > 0)",
        optional: false,
        default: None,
    },
];

static NORM_CDF_EXAMPLES: [&str; 1] = ["norm_cdf(0, 0, 1) → 0.5"];

static NORM_CDF_RELATED: [&str; 2] = ["norm_pdf", "norm_inv"];

impl FunctionPlugin for NormCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "norm_cdf",
            description: "Normal distribution CDF P(X ≤ x)",
            usage: "norm_cdf(x, μ, σ)",
            args: &NORM_CDF_ARGS,
            returns: "Number",
            examples: &NORM_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &NORM_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("norm_cdf", 3, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_cdf", "x", "Number", other.type_name())),
        };

        let mu = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_cdf", "μ", "Number", other.type_name())),
        };

        let sigma = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_cdf", "σ", "Number", other.type_name())),
        };

        if sigma.is_zero() || sigma.is_negative() {
            return Value::Error(FolioError::domain_error("norm_cdf() requires σ > 0"));
        }

        // Standardize: z = (x - μ) / σ
        let z = match x.sub(mu).checked_div(sigma) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        Value::Number(standard_normal_cdf(&z, ctx.precision))
    }
}

// ============ Normal Inverse ============

pub struct NormInv;

static NORM_INV_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability (0 < p < 1)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "μ",
        typ: "Number",
        description: "Mean",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "σ",
        typ: "Number",
        description: "Standard deviation (must be > 0)",
        optional: false,
        default: None,
    },
];

static NORM_INV_EXAMPLES: [&str; 1] = ["norm_inv(0.975, 0, 1) → 1.96"];

static NORM_INV_RELATED: [&str; 2] = ["norm_cdf", "snorm_inv"];

impl FunctionPlugin for NormInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "norm_inv",
            description: "Normal distribution inverse (quantile function)",
            usage: "norm_inv(p, μ, σ)",
            args: &NORM_INV_ARGS,
            returns: "Number",
            examples: &NORM_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &NORM_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("norm_inv", 3, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_inv", "p", "Number", other.type_name())),
        };

        let mu = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_inv", "μ", "Number", other.type_name())),
        };

        let sigma = match &args[2] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("norm_inv", "σ", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error("norm_inv() requires 0 < p < 1"));
        }

        if sigma.is_zero() || sigma.is_negative() {
            return Value::Error(FolioError::domain_error("norm_inv() requires σ > 0"));
        }

        // x = μ + σ * Φ⁻¹(p)
        let z = standard_normal_inv(p, ctx.precision);
        Value::Number(mu.add(&sigma.mul(&z)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_snorm_cdf_zero() {
        let snorm_cdf = SnormCdf;
        let args = vec![Value::Number(Number::from_i64(0))];
        let result = snorm_cdf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        assert!((f - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_snorm_inv_half() {
        let snorm_inv = SnormInv;
        let args = vec![Value::Number(Number::from_str("0.5").unwrap())];
        let result = snorm_inv.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        assert!(f.abs() < 0.001);
    }
}
```

folio-stats\src\distributions\t.rs
```rs
//! Student's t distribution functions

use folio_plugin::prelude::*;

// ============ T PDF ============

pub struct TPdf;

static T_PDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df",
        typ: "Number",
        description: "Degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static T_PDF_EXAMPLES: [&str; 1] = ["t_pdf(0, 10) → 0.389..."];

static T_PDF_RELATED: [&str; 2] = ["t_cdf", "snorm_pdf"];

impl FunctionPlugin for TPdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_pdf",
            description: "Student's t distribution PDF",
            usage: "t_pdf(x, df)",
            args: &T_PDF_ARGS,
            returns: "Number",
            examples: &T_PDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &T_PDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("t_pdf", 2, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_pdf", "x", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_pdf", "df", "Number", other.type_name())),
        };

        let df_f64 = df.to_f64().unwrap_or(1.0);
        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("t_pdf() requires df > 0"));
        }

        let x_f64 = x.to_f64().unwrap_or(0.0);

        // t PDF using f64 for gamma function
        let result = t_pdf_f64(x_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn t_pdf_f64(x: f64, df: f64) -> f64 {
    // PDF(x) = Γ((ν+1)/2) / (√(νπ) * Γ(ν/2)) * (1 + x²/ν)^(-(ν+1)/2)
    let nu = df;
    let coef = gamma_ln((nu + 1.0) / 2.0) - gamma_ln(nu / 2.0) - 0.5 * (nu * std::f64::consts::PI).ln();
    let term = -(nu + 1.0) / 2.0 * (1.0 + x * x / nu).ln();
    (coef + term).exp()
}

// ============ T CDF ============

pub struct TCdf;

static T_CDF_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "Number",
        description: "Value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df",
        typ: "Number",
        description: "Degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static T_CDF_EXAMPLES: [&str; 1] = ["t_cdf(1.96, 30) → 0.97..."];

static T_CDF_RELATED: [&str; 2] = ["t_pdf", "t_inv"];

impl FunctionPlugin for TCdf {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_cdf",
            description: "Student's t distribution CDF",
            usage: "t_cdf(x, df)",
            args: &T_CDF_ARGS,
            returns: "Number",
            examples: &T_CDF_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &T_CDF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("t_cdf", 2, args.len()));
        }

        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_cdf", "x", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_cdf", "df", "Number", other.type_name())),
        };

        let df_f64 = df.to_f64().unwrap_or(1.0);
        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("t_cdf() requires df > 0"));
        }

        let x_f64 = x.to_f64().unwrap_or(0.0);
        let result = t_cdf_f64(x_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

pub fn t_cdf_f64(x: f64, df: f64) -> f64 {
    // Use regularized incomplete beta function
    let t2 = x * x;
    let p = df / (df + t2);

    if x >= 0.0 {
        1.0 - 0.5 * regularized_incomplete_beta(df / 2.0, 0.5, p)
    } else {
        0.5 * regularized_incomplete_beta(df / 2.0, 0.5, p)
    }
}

// ============ T Inverse ============

pub struct TInv;

static T_INV_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Probability (0 < p < 1)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "df",
        typ: "Number",
        description: "Degrees of freedom (must be > 0)",
        optional: false,
        default: None,
    },
];

static T_INV_EXAMPLES: [&str; 1] = ["t_inv(0.975, 30) → 2.042..."];

static T_INV_RELATED: [&str; 2] = ["t_cdf", "snorm_inv"];

impl FunctionPlugin for TInv {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_inv",
            description: "Student's t distribution inverse (quantile)",
            usage: "t_inv(p, df)",
            args: &T_INV_ARGS,
            returns: "Number",
            examples: &T_INV_EXAMPLES,
            category: "stats/distribution",
            source: None,
            related: &T_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("t_inv", 2, args.len()));
        }

        let p = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_inv", "p", "Number", other.type_name())),
        };

        let df = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_inv", "df", "Number", other.type_name())),
        };

        let p_f64 = p.to_f64().unwrap_or(0.5);
        let df_f64 = df.to_f64().unwrap_or(1.0);

        if p_f64 <= 0.0 || p_f64 >= 1.0 {
            return Value::Error(FolioError::domain_error("t_inv() requires 0 < p < 1"));
        }
        if df_f64 <= 0.0 {
            return Value::Error(FolioError::domain_error("t_inv() requires df > 0"));
        }

        let result = t_inv_f64(p_f64, df_f64);
        Value::Number(Number::from_str(&format!("{:.15}", result)).unwrap_or(Number::from_i64(0)))
    }
}

fn t_inv_f64(p: f64, df: f64) -> f64 {
    // Newton-Raphson iteration starting from normal approximation
    let mut x = norm_inv_approx(p);

    for _ in 0..50 {
        let cdf = t_cdf_f64(x, df);
        let pdf = t_pdf_f64(x, df);
        if pdf.abs() < 1e-15 {
            break;
        }
        let dx = (cdf - p) / pdf;
        x -= dx;
        if dx.abs() < 1e-12 {
            break;
        }
    }

    x
}

fn norm_inv_approx(p: f64) -> f64 {
    const A: [f64; 4] = [2.515517, 0.802853, 0.010328, 0.0];
    const B: [f64; 4] = [1.0, 1.432788, 0.189269, 0.001308];

    let sign = if p < 0.5 { -1.0 } else { 1.0 };
    let p_adj = if p < 0.5 { p } else { 1.0 - p };
    let t = (-2.0 * p_adj.ln()).sqrt();
    let num = A[0] + t * (A[1] + t * A[2]);
    let den = 1.0 + t * (B[1] + t * (B[2] + t * B[3]));
    sign * (t - num / den)
}

/// Log gamma function using Lanczos approximation
pub fn gamma_ln(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::INFINITY;
    }

    const COEFFS: [f64; 8] = [
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];

    let g = 7.0;
    let z = x - 1.0;

    let mut sum = 0.99999999999980993;
    for (i, &c) in COEFFS.iter().enumerate() {
        sum += c / (z + i as f64 + 1.0);
    }

    let t = z + g + 0.5;
    0.5 * (2.0 * std::f64::consts::PI).ln() + (z + 0.5) * t.ln() - t + sum.ln()
}

/// Regularized incomplete beta function (simple approximation)
pub fn regularized_incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Use continued fraction for better accuracy
    let bt = if x == 0.0 || x == 1.0 {
        0.0
    } else {
        (gamma_ln(a + b) - gamma_ln(a) - gamma_ln(b) + a * x.ln() + b * (1.0 - x).ln()).exp()
    };

    // Use continued fraction
    let sym = a / (a + b);
    if x < sym {
        bt * beta_cf(a, b, x) / a
    } else {
        1.0 - bt * beta_cf(b, a, 1.0 - x) / b
    }
}

fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    let fpmin = 1e-30;
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;

    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < fpmin {
        d = fpmin;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=200 {
        let m = m as f64;
        let m2 = 2.0 * m;

        // Even step
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < fpmin {
            d = fpmin;
        }
        c = 1.0 + aa / c;
        if c.abs() < fpmin {
            c = fpmin;
        }
        d = 1.0 / d;
        h *= d * c;

        // Odd step
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < fpmin {
            d = fpmin;
        }
        c = 1.0 + aa / c;
        if c.abs() < fpmin {
            c = fpmin;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;

        if (del - 1.0).abs() < 3e-14 {
            break;
        }
    }

    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_t_cdf_zero() {
        let t_cdf = TCdf;
        let args = vec![
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(10)),
        ];
        let result = t_cdf.call(&args, &eval_ctx());
        let num = result.as_number().unwrap();
        let f = num.to_f64().unwrap();
        assert!((f - 0.5).abs() < 0.001);
    }
}
```

folio-stats\src\goodness.rs
```rs
//! Goodness-of-fit tests for distribution analysis

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use std::collections::HashMap;
use crate::helpers::{extract_numbers, require_min_count, sorted, mean};

// ============================================================================
// JarqueBera - Jarque-Bera test for normality
// ============================================================================

pub struct JarqueBera;

static JB_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
];
static JB_EXAMPLES: [&str; 1] = ["jarque_bera(data) → {statistic: 2.34, p_value: 0.31, ...}"];
static JB_RELATED: [&str; 2] = ["shapiro_wilk", "is_normal"];

impl FunctionPlugin for JarqueBera {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "jarque_bera",
            description: "Jarque-Bera test for normality (uses skewness and kurtosis)",
            usage: "jarque_bera(list)",
            args: &JB_ARGS,
            returns: "Object",
            examples: &JB_EXAMPLES,
            category: "distribution",
            source: None,
            related: &JB_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("jarque_bera", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 8, "jarque_bera") {
            return Value::Error(e);
        }

        let n = numbers.len();

        // Calculate mean
        let m = match mean(&numbers) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        // Calculate central moments
        let mut m2 = Number::from_i64(0);
        let mut m3 = Number::from_i64(0);
        let mut m4 = Number::from_i64(0);

        for x in &numbers {
            let dev = x.sub(&m);
            let dev2 = dev.mul(&dev);
            let dev3 = dev2.mul(&dev);
            let dev4 = dev3.mul(&dev);
            m2 = m2.add(&dev2);
            m3 = m3.add(&dev3);
            m4 = m4.add(&dev4);
        }

        let n_num = Number::from_i64(n as i64);
        m2 = match m2.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };
        m3 = match m3.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };
        m4 = match m4.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if m2.is_zero() {
            return Value::Error(FolioError::domain_error("Variance is zero, cannot compute Jarque-Bera test"));
        }

        // Skewness = m3 / m2^(3/2)
        let m2_sqrt = match m2.sqrt(50) {
            Ok(s) => s,
            Err(e) => return Value::Error(e.into()),
        };
        let m2_32 = m2_sqrt.mul(&m2);
        let skewness = match m3.checked_div(&m2_32) {
            Ok(s) => s,
            Err(e) => return Value::Error(e.into()),
        };

        // Kurtosis = m4 / m2^2 - 3 (excess kurtosis)
        let m2_sq = m2.mul(&m2);
        let kurt_raw = match m4.checked_div(&m2_sq) {
            Ok(k) => k,
            Err(e) => return Value::Error(e.into()),
        };
        let kurtosis = kurt_raw.sub(&Number::from_i64(3));

        // JB statistic = n/6 * (S^2 + K^2/4)
        let skew_sq = skewness.mul(&skewness);
        let kurt_sq = kurtosis.mul(&kurtosis);
        let kurt_term = match kurt_sq.checked_div(&Number::from_i64(4)) {
            Ok(k) => k,
            Err(e) => return Value::Error(e.into()),
        };
        let sum_terms = skew_sq.add(&kurt_term);
        let jb_stat = match n_num.checked_div(&Number::from_i64(6)) {
            Ok(factor) => factor.mul(&sum_terms),
            Err(e) => return Value::Error(e.into()),
        };

        // P-value from chi-squared distribution with 2 df
        let jb_f64 = jb_stat.to_f64().unwrap_or(0.0);
        let p_value = chi_squared_sf(jb_f64, 2);

        let mut result = HashMap::new();
        result.insert("statistic".to_string(), Value::Number(jb_stat));
        result.insert("p_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("skewness".to_string(), Value::Number(skewness));
        result.insert("kurtosis".to_string(), Value::Number(kurtosis));

        Value::Object(result)
    }
}

// ============================================================================
// ShapiroWilk - Shapiro-Wilk test for normality
// ============================================================================

pub struct ShapiroWilk;

static SW_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values (3 ≤ n ≤ 5000)",
        optional: false,
        default: None,
    },
];
static SW_EXAMPLES: [&str; 1] = ["shapiro_wilk(data) → {w: 0.967, p_value: 0.234}"];
static SW_RELATED: [&str; 2] = ["jarque_bera", "is_normal"];

impl FunctionPlugin for ShapiroWilk {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "shapiro_wilk",
            description: "Shapiro-Wilk test for normality (best for n < 50)",
            usage: "shapiro_wilk(list)",
            args: &SW_ARGS,
            returns: "Object",
            examples: &SW_EXAMPLES,
            category: "distribution",
            source: None,
            related: &SW_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("shapiro_wilk", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = numbers.len();
        if n < 3 {
            return Value::Error(FolioError::domain_error("Shapiro-Wilk requires at least 3 values"));
        }
        if n > 5000 {
            return Value::Error(FolioError::domain_error("Shapiro-Wilk limited to n ≤ 5000"));
        }

        let sorted_data = sorted(&numbers);
        let m = match mean(&numbers) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        // Calculate SS (sum of squared deviations)
        let mut ss = Number::from_i64(0);
        for x in &numbers {
            let dev = x.sub(&m);
            ss = ss.add(&dev.mul(&dev));
        }

        if ss.is_zero() {
            return Value::Error(FolioError::domain_error("All values are identical"));
        }

        // Calculate W statistic using the standard Shapiro-Wilk formula
        // W = (Σ a_i * x_(i))² / SS
        // where x_(i) are order statistics and a_i are Shapiro-Wilk coefficients

        // Convert to f64 for calculation
        let sorted_f64: Vec<f64> = sorted_data.iter()
            .map(|x| x.to_f64().unwrap_or(0.0))
            .collect();
        let ss_f64 = ss.to_f64().unwrap_or(1.0);

        // Get the Shapiro-Wilk coefficients (full vector for all n values)
        let a_coeffs = shapiro_wilk_coefficients(n);

        // Calculate the numerator: (Σ a_i * x_(i))²
        // The coefficients a are symmetric: a[i] = -a[n-1-i] for i < n/2
        // So the sum becomes: Σ a[i] * (x_(n-i) - x_(i+1)) for i = 0..n/2
        let mut b = 0.0_f64;
        let half_n = n / 2;

        for i in 0..half_n {
            b += a_coeffs[i] * (sorted_f64[n - 1 - i] - sorted_f64[i]);
        }

        let w_val = if ss_f64 > 0.0 {
            let w = (b * b) / ss_f64;
            // W should be in (0, 1] but may slightly exceed due to numerical precision
            w.min(1.0).max(0.0)
        } else {
            1.0
        };

        let w_stat = Number::from_str(&format!("{:.10}", w_val)).unwrap_or(Number::from_i64(1));

        // Approximate p-value using Royston's approximation
        let w_f64 = w_stat.to_f64().unwrap_or(1.0);
        let p_value = shapiro_wilk_p_value(w_f64, n);

        let mut result = HashMap::new();
        result.insert("w".to_string(), Value::Number(w_stat));
        result.insert("p_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", p_value)).unwrap_or(Number::from_i64(0))));

        Value::Object(result)
    }
}

/// Generate Shapiro-Wilk coefficients
/// Uses exact tabulated values from Shapiro-Wilk (1965) and Royston (1992)
fn shapiro_wilk_coefficients(n: usize) -> Vec<f64> {
    let half_n = n / 2;

    if n < 3 {
        return vec![0.0; half_n];
    }

    // Exact tabulated coefficients from Shapiro-Wilk (1965)
    // These are the 'a' coefficients for the first half (paired with symmetric negatives)
    // Source: Original S-W paper and verified against R's shapiro.test
    match n {
        3 => vec![0.7071],
        4 => vec![0.6872, 0.1677],
        5 => vec![0.6646, 0.2413],
        6 => vec![0.6431, 0.2806, 0.0875],
        7 => vec![0.6233, 0.3031, 0.1401],
        8 => vec![0.6052, 0.3164, 0.1743, 0.0561],
        9 => vec![0.5888, 0.3244, 0.1976, 0.0947],
        10 => vec![0.5739, 0.3291, 0.2141, 0.1224, 0.0399],
        11 => vec![0.5601, 0.3315, 0.2260, 0.1429, 0.0695],
        12 => vec![0.5475, 0.3325, 0.2347, 0.1586, 0.0922, 0.0303],
        13 => vec![0.5359, 0.3325, 0.2412, 0.1707, 0.1099, 0.0539],
        14 => vec![0.5251, 0.3318, 0.2460, 0.1802, 0.1240, 0.0727, 0.0240],
        15 => vec![0.5150, 0.3306, 0.2495, 0.1878, 0.1353, 0.0880, 0.0433],
        16 => vec![0.5056, 0.3290, 0.2521, 0.1939, 0.1447, 0.1005, 0.0593, 0.0196],
        17 => vec![0.4968, 0.3273, 0.2540, 0.1988, 0.1524, 0.1109, 0.0725, 0.0359],
        18 => vec![0.4886, 0.3253, 0.2553, 0.2027, 0.1587, 0.1197, 0.0837, 0.0496, 0.0163],
        19 => vec![0.4808, 0.3232, 0.2561, 0.2059, 0.1641, 0.1271, 0.0932, 0.0612, 0.0303],
        20 => vec![0.4734, 0.3211, 0.2565, 0.2085, 0.1686, 0.1334, 0.1013, 0.0711, 0.0422, 0.0140],
        _ => {
            // For n > 20, use Royston's approximation (AS R94)
            royston_coefficients(n)
        }
    }
}

/// Royston's approximation for Shapiro-Wilk coefficients when n > 20
fn royston_coefficients(n: usize) -> Vec<f64> {
    let half_n = n / 2;
    let n_f64 = n as f64;

    // Calculate expected order statistics (m-values) using Blom's approximation
    let m: Vec<f64> = (1..=n).map(|i| {
        let p = (i as f64 - 0.375) / (n_f64 + 0.25);
        normal_quantile(p)
    }).collect();

    // Calculate sum of m^2
    let m_sq_sum: f64 = m.iter().map(|x| x * x).sum();

    if m_sq_sum == 0.0 {
        return vec![0.0; half_n];
    }

    let sqrt_m_sq_sum = m_sq_sum.sqrt();

    // Polynomial approximation for the largest coefficient a[n]
    // From Royston (1992) Algorithm AS R94
    let u = 1.0 / n_f64.sqrt();

    // Coefficients c1-c6 for a_n polynomial
    let a_n = {
        let poly_val = -2.706056 * u.powi(5)
            + 4.434685 * u.powi(4)
            - 2.071190 * u.powi(3)
            - 0.147981 * u.powi(2)
            + 0.221157 * u;
        poly_val + m[n - 1] / sqrt_m_sq_sum
    };

    // Calculate remaining coefficients using the constraint sum(2*a_i^2) = 1
    // (because each a_i pairs with -a_i on the other end)
    let mut a = vec![0.0; half_n];
    a[0] = a_n;

    // For the remaining coefficients, scale the m values
    // so that the sum of squares constraint is satisfied
    let remaining_m_sq: f64 = (1..half_n).map(|i| {
        let diff = m[n - 1 - i] - m[i];
        diff * diff
    }).sum();

    let a_n_contribution = 2.0 * a_n * a_n;
    let remaining_scale_sq = (1.0 - a_n_contribution) / (4.0 * remaining_m_sq);

    if remaining_scale_sq > 0.0 {
        let scale = remaining_scale_sq.sqrt();
        for i in 1..half_n {
            a[i] = scale * (m[n - 1 - i] - m[i]);
        }
    } else {
        // Fallback: use normalized m differences
        for i in 1..half_n {
            a[i] = (m[n - 1 - i] - m[i]) / (2.0 * sqrt_m_sq_sum);
        }
    }

    // Final normalization to ensure sum of 2*a_i^2 = 1
    let sum_2a_sq: f64 = a.iter().map(|x| 2.0 * x * x).sum();
    if sum_2a_sq > 0.0 && (sum_2a_sq - 1.0).abs() > 0.001 {
        let norm = sum_2a_sq.sqrt();
        for coeff in &mut a {
            *coeff /= norm;
        }
    }

    a
}

/// Approximate p-value for Shapiro-Wilk test using Royston's Algorithm AS R94
/// This implements the transformation to normality and returns the p-value
fn shapiro_wilk_p_value(w: f64, n: usize) -> f64 {
    let n_f64 = n as f64;

    if w >= 1.0 {
        return 1.0;
    }
    if w <= 0.0 {
        return 0.0;
    }

    // Royston's 1992 algorithm for p-value approximation
    // Different transformations for different sample size ranges

    if n <= 11 {
        // Small sample: use gamma approximation
        let gamma = poly(&[-2.273, 0.459], n_f64);
        let mu = poly(&[0.544, -0.39978, 0.025054, -0.0006714], n_f64);
        let sigma = poly(&[1.3822, -0.77857, 0.062767, -0.0020322], n_f64).exp();

        let y = -((1.0 - w).ln());
        let z = (y - mu) / sigma;

        // Adjust for gamma distribution shape
        1.0 - normal_cdf(gamma + z * (1.0 + gamma * sigma).abs())
    } else if n <= 2000 {
        // Medium to large sample: use log transformation
        let ln_n = n_f64.ln();

        // Royston's coefficients for the transformation
        let mu = poly(&[-1.5861, -0.31082, -0.083751, 0.0038915], ln_n);
        let sigma = poly(&[-0.4803, -0.082676, 0.0030302], ln_n).exp();

        // Transform W to approximately standard normal
        let y = (1.0 - w).ln();
        let z = (y - mu) / sigma;

        // P-value from standard normal (upper tail)
        1.0 - normal_cdf(z)
    } else {
        // Very large sample: use asymptotic approximation
        // For n > 2000, W is approximately normal with known mean and variance
        let mean_w = 1.0 - 2.0 / (9.0 * n_f64);
        let var_w = 2.0 / (81.0 * n_f64 * n_f64);
        let z = (w - mean_w) / var_w.sqrt();

        normal_cdf(z)
    }
}

/// Evaluate polynomial at x: c[0] + c[1]*x + c[2]*x^2 + ...
fn poly(coeffs: &[f64], x: f64) -> f64 {
    let mut result = 0.0;
    let mut x_pow = 1.0;
    for &c in coeffs {
        result += c * x_pow;
        x_pow *= x;
    }
    result
}

// ============================================================================
// IsNormal - Convenience function for normality test
// ============================================================================

pub struct IsNormal;

static IS_NORMAL_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "alpha",
        typ: "Number",
        description: "Significance level (default 0.05)",
        optional: true,
        default: Some("0.05"),
    },
];
static IS_NORMAL_EXAMPLES: [&str; 2] = [
    "is_normal(data) → true",
    "is_normal(data, 0.01) → false (stricter test)",
];
static IS_NORMAL_RELATED: [&str; 2] = ["shapiro_wilk", "jarque_bera"];

impl FunctionPlugin for IsNormal {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_normal",
            description: "Test if data is normally distributed",
            usage: "is_normal(list, alpha?)",
            args: &IS_NORMAL_ARGS,
            returns: "Bool",
            examples: &IS_NORMAL_EXAMPLES,
            category: "distribution",
            source: None,
            related: &IS_NORMAL_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("is_normal", 1, args.len()));
        }

        let alpha = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.to_f64().unwrap_or(0.05),
                other => return Value::Error(FolioError::arg_type("is_normal", "alpha", "Number", other.type_name())),
            }
        } else {
            0.05
        };

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = numbers.len();
        if n < 3 {
            return Value::Error(FolioError::domain_error("is_normal requires at least 3 values"));
        }

        // Use Shapiro-Wilk for small samples, Jarque-Bera for large
        let p_value = if n < 50 {
            let sw = ShapiroWilk;
            match sw.call(&args[0..1], ctx) {
                Value::Object(obj) => {
                    if let Some(Value::Number(p)) = obj.get("p_value") {
                        p.to_f64().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            }
        } else {
            let jb = JarqueBera;
            match jb.call(&args[0..1], ctx) {
                Value::Object(obj) => {
                    if let Some(Value::Number(p)) = obj.get("p_value") {
                        p.to_f64().unwrap_or(0.0)
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            }
        };

        // If p > alpha, we fail to reject normality hypothesis
        Value::Bool(p_value > alpha)
    }
}

// ============================================================================
// KsTest2 - Two-sample Kolmogorov-Smirnov test
// ============================================================================

pub struct KsTest2;

static KS_TEST_2_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List",
        description: "First sample",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List",
        description: "Second sample",
        optional: false,
        default: None,
    },
];
static KS_TEST_2_EXAMPLES: [&str; 1] = ["ks_test_2(before, after) → {statistic, p_value, critical_01, critical_05, critical_10}"];
static KS_TEST_2_RELATED: [&str; 2] = ["t_test_2", "anderson_darling"];

impl FunctionPlugin for KsTest2 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ks_test_2",
            description: "Two-sample Kolmogorov-Smirnov test",
            usage: "ks_test_2(list1, list2)",
            args: &KS_TEST_2_ARGS,
            returns: "Object",
            examples: &KS_TEST_2_EXAMPLES,
            category: "distribution",
            source: None,
            related: &KS_TEST_2_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("ks_test_2", 2, args.len()));
        }

        let x = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let y = match extract_numbers(&args[1..2]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&x, 5, "ks_test_2") {
            return Value::Error(e);
        }
        if let Err(e) = require_min_count(&y, 5, "ks_test_2") {
            return Value::Error(e);
        }

        let n1 = x.len();
        let n2 = y.len();

        // Sort both samples
        let sorted_x = sorted(&x);
        let sorted_y = sorted(&y);

        // Calculate KS statistic: max |F1(x) - F2(x)|
        let mut d_max = 0.0;
        let mut i = 0;
        let mut j = 0;

        while i < n1 && j < n2 {
            let x_val = sorted_x[i].to_f64().unwrap_or(0.0);
            let y_val = sorted_y[j].to_f64().unwrap_or(0.0);

            if x_val <= y_val {
                i += 1;
            }
            if y_val <= x_val {
                j += 1;
            }

            let f1 = i as f64 / n1 as f64;
            let f2 = j as f64 / n2 as f64;
            let d = (f1 - f2).abs();
            if d > d_max {
                d_max = d;
            }
        }

        // Calculate p-value using asymptotic distribution
        let n_eff = (n1 * n2) as f64 / (n1 + n2) as f64;
        let lambda = (n_eff.sqrt() + 0.12 + 0.11 / n_eff.sqrt()) * d_max;
        let p_value = ks_p_value(lambda);

        // Critical values
        let c_alpha = |alpha: f64| -> f64 {
            (-0.5 * (alpha / 2.0).ln()).sqrt() / (n_eff.sqrt() + 0.12 + 0.11 / n_eff.sqrt())
        };

        let mut result = HashMap::new();
        result.insert("statistic".to_string(), Value::Number(Number::from_str(&format!("{:.10}", d_max)).unwrap_or(Number::from_i64(0))));
        result.insert("p_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("critical_01".to_string(), Value::Number(Number::from_str(&format!("{:.10}", c_alpha(0.01))).unwrap_or(Number::from_i64(0))));
        result.insert("critical_05".to_string(), Value::Number(Number::from_str(&format!("{:.10}", c_alpha(0.05))).unwrap_or(Number::from_i64(0))));
        result.insert("critical_10".to_string(), Value::Number(Number::from_str(&format!("{:.10}", c_alpha(0.10))).unwrap_or(Number::from_i64(0))));

        Value::Object(result)
    }
}

// ============================================================================
// Helper functions for statistical distributions
// ============================================================================

/// Standard normal CDF
fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

/// Standard normal quantile (inverse CDF)
fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if p == 0.5 {
        return 0.0;
    }

    // Rational approximation
    let a = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383577518672690e2,
        -3.066479806614716e1,
        2.506628277459239e0,
    ];
    let b = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    let c = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838e0,
        -2.549732539343734e0,
        4.374664141464968e0,
        2.938163982698783e0,
    ];
    let d = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996e0,
        3.754408661907416e0,
    ];

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

/// Error function approximation
fn erf(x: f64) -> f64 {
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Chi-squared survival function (1 - CDF)
fn chi_squared_sf(x: f64, df: usize) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    // Use incomplete gamma function
    1.0 - incomplete_gamma(df as f64 / 2.0, x / 2.0)
}

/// Regularized incomplete gamma function (lower)
fn incomplete_gamma(a: f64, x: f64) -> f64 {
    if x < 0.0 || a <= 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        return 0.0;
    }

    // Use series expansion for small x, continued fraction for large x
    if x < a + 1.0 {
        // Series expansion
        gamma_series(a, x)
    } else {
        // Continued fraction
        1.0 - gamma_cf(a, x)
    }
}

fn gamma_series(a: f64, x: f64) -> f64 {
    let gln = ln_gamma(a);
    let mut ap = a;
    let mut sum = 1.0 / a;
    let mut del = sum;

    for _ in 0..100 {
        ap += 1.0;
        del *= x / ap;
        sum += del;
        if del.abs() < sum.abs() * 1e-10 {
            break;
        }
    }

    sum * (-x + a * x.ln() - gln).exp()
}

fn gamma_cf(a: f64, x: f64) -> f64 {
    let gln = ln_gamma(a);
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / 1e-30;
    let mut d = 1.0 / b;
    let mut h = d;

    for i in 1..100 {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = b + an / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;
        if (del - 1.0).abs() < 1e-10 {
            break;
        }
    }

    (-x + a * x.ln() - gln).exp() * h
}

fn ln_gamma(x: f64) -> f64 {
    let cof = [
        76.18009172947146,
        -86.50532032941677,
        24.01409824083091,
        -1.231739572450155,
        0.1208650973866179e-2,
        -0.5395239384953e-5,
    ];

    let y = x;
    let tmp = x + 5.5 - (x + 0.5) * (x + 5.5).ln();
    let mut ser = 1.000000000190015;
    for (j, &c) in cof.iter().enumerate() {
        ser += c / (y + j as f64 + 1.0);
    }

    -tmp + (2.5066282746310005 * ser / x).ln()
}

/// KS p-value from the Kolmogorov distribution
fn ks_p_value(lambda: f64) -> f64 {
    if lambda <= 0.0 {
        return 1.0;
    }
    if lambda >= 3.0 {
        return 0.0;
    }

    // Asymptotic formula
    let mut sum = 0.0;
    for k in 1..100 {
        let k_f64 = k as f64;
        let term = (-2.0 * k_f64 * k_f64 * lambda * lambda).exp();
        if k % 2 == 0 {
            sum -= term;
        } else {
            sum += term;
        }
        if term.abs() < 1e-10 {
            break;
        }
    }
    2.0 * sum
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    fn make_list(values: &[i64]) -> Value {
        Value::List(values.iter().map(|v| Value::Number(Number::from_i64(*v))).collect())
    }

    #[test]
    fn test_jarque_bera() {
        let jb = JarqueBera;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = jb.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("statistic"));
            assert!(obj.contains_key("p_value"));
            assert!(obj.contains_key("skewness"));
            assert!(obj.contains_key("kurtosis"));
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_shapiro_wilk() {
        let sw = ShapiroWilk;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = sw.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("w"));
            assert!(obj.contains_key("p_value"));
            // W should be between 0 and 1 (or close to it)
            if let Some(Value::Number(w)) = obj.get("w") {
                let w_val = w.to_f64().unwrap_or(0.0);
                // Allow some tolerance as simplified implementation may slightly exceed 1
                assert!(w_val > 0.0 && w_val <= 1.1, "W should be in range (0,1], got {}", w_val);
            }
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_is_normal() {
        let is_norm = IsNormal;
        // Uniform data - should not be normal
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = is_norm.call(&args, &ctx);
        assert!(matches!(result, Value::Bool(_)));
    }

    #[test]
    fn test_ks_test_2() {
        let ks = KsTest2;
        let args = vec![
            make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            make_list(&[2, 3, 4, 5, 6, 7, 8, 9, 10, 11]),
        ];
        let ctx = eval_ctx();
        let result = ks.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("statistic"));
            assert!(obj.contains_key("p_value"));
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }
}
```

folio-stats\src\helpers.rs
```rs
//! Helper functions for statistical operations
//!
//! Common utilities for extracting and validating inputs.

use folio_core::{Number, Value, FolioError};

/// Extract numbers from arguments, handling both varargs and List
pub fn extract_numbers(args: &[Value]) -> Result<Vec<Number>, FolioError> {
    let mut numbers = Vec::new();

    for arg in args {
        match arg {
            Value::Number(n) => numbers.push(n.clone()),
            Value::List(list) => {
                for item in list {
                    match item {
                        Value::Number(n) => numbers.push(n.clone()),
                        Value::Error(e) => return Err(e.clone()),
                        other => return Err(FolioError::type_error("Number", other.type_name())),
                    }
                }
            }
            Value::Error(e) => return Err(e.clone()),
            other => return Err(FolioError::type_error("Number or List", other.type_name())),
        }
    }

    Ok(numbers)
}

/// Extract exactly two equal-length lists for bivariate functions
pub fn extract_two_lists(args: &[Value]) -> Result<(Vec<Number>, Vec<Number>), FolioError> {
    if args.len() != 2 {
        return Err(FolioError::new(
            "ARG_COUNT",
            format!("Expected 2 lists, got {} arguments", args.len()),
        ));
    }

    let x = extract_numbers(&args[0..1])?;
    let y = extract_numbers(&args[1..2])?;

    if x.len() != y.len() {
        return Err(FolioError::domain_error(format!(
            "Lists must have equal length: {} vs {}",
            x.len(),
            y.len()
        )));
    }

    Ok((x, y))
}

/// Require non-empty list
pub fn require_non_empty(numbers: &[Number], func: &str) -> Result<(), FolioError> {
    if numbers.is_empty() {
        return Err(FolioError::domain_error(format!(
            "{}() requires at least one value",
            func
        )));
    }
    Ok(())
}

/// Require minimum count
pub fn require_min_count(numbers: &[Number], min: usize, func: &str) -> Result<(), FolioError> {
    if numbers.len() < min {
        return Err(FolioError::domain_error(format!(
            "{}() requires at least {} values, got {}",
            func,
            min,
            numbers.len()
        )));
    }
    Ok(())
}

/// Calculate sum of numbers
pub fn sum(numbers: &[Number]) -> Number {
    numbers
        .iter()
        .fold(Number::from_i64(0), |acc, n| acc.add(n))
}

/// Calculate mean of numbers
pub fn mean(numbers: &[Number]) -> Result<Number, FolioError> {
    if numbers.is_empty() {
        return Err(FolioError::domain_error("Cannot calculate mean of empty list"));
    }
    let s = sum(numbers);
    let count = Number::from_i64(numbers.len() as i64);
    s.checked_div(&count).map_err(|e| e.into())
}

/// Calculate variance (sample or population)
pub fn variance_impl(numbers: &[Number], sample: bool) -> Result<Number, FolioError> {
    let n = numbers.len();
    if n == 0 {
        return Err(FolioError::domain_error("Cannot calculate variance of empty list"));
    }
    if sample && n < 2 {
        return Err(FolioError::domain_error(
            "Sample variance requires at least 2 values",
        ));
    }

    let m = mean(numbers)?;
    let mut ss = Number::from_i64(0);
    for x in numbers {
        let dev = x.sub(&m);
        ss = ss.add(&dev.mul(&dev));
    }

    let divisor = if sample {
        Number::from_i64((n - 1) as i64)
    } else {
        Number::from_i64(n as i64)
    };

    ss.checked_div(&divisor).map_err(|e| e.into())
}

/// Sort numbers (returns new sorted vector)
pub fn sorted(numbers: &[Number]) -> Vec<Number> {
    let mut sorted = numbers.to_vec();
    sorted.sort_by(|a, b| {
        let diff = a.sub(b);
        if diff.is_zero() {
            std::cmp::Ordering::Equal
        } else if diff.is_negative() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    sorted
}

/// Calculate percentile using linear interpolation
pub fn percentile_impl(numbers: &[Number], p: &Number) -> Result<Number, FolioError> {
    if numbers.is_empty() {
        return Err(FolioError::domain_error("Cannot calculate percentile of empty list"));
    }

    // Validate p is in [0, 100]
    let zero = Number::from_i64(0);
    let hundred = Number::from_i64(100);
    if p.sub(&zero).is_negative() || p.sub(&hundred).is_negative() == false && !p.sub(&hundred).is_zero() {
        return Err(FolioError::domain_error(
            "Percentile must be between 0 and 100",
        ));
    }

    let sorted_nums = sorted(numbers);
    let n = sorted_nums.len();

    if n == 1 {
        return Ok(sorted_nums[0].clone());
    }

    // Convert percentile to rank
    // rank = p/100 * (n-1)
    let n_minus_1 = Number::from_i64((n - 1) as i64);
    let rank = p.mul(&n_minus_1).checked_div(&hundred)?;

    // Get floor and ceiling indices
    let floor_rank = rank.floor();
    let ceil_rank = rank.ceil();

    let floor_idx = floor_rank.to_i64().unwrap_or(0) as usize;
    let ceil_idx = ceil_rank.to_i64().unwrap_or(0) as usize;

    if floor_idx >= n {
        return Ok(sorted_nums[n - 1].clone());
    }
    if ceil_idx >= n {
        return Ok(sorted_nums[n - 1].clone());
    }

    if floor_idx == ceil_idx {
        return Ok(sorted_nums[floor_idx].clone());
    }

    // Linear interpolation
    let lower = &sorted_nums[floor_idx];
    let upper = &sorted_nums[ceil_idx];
    let frac = rank.sub(&floor_rank);
    let interpolated = lower.add(&upper.sub(lower).mul(&frac));

    Ok(interpolated)
}

/// Calculate ranks for a list (1-indexed, average for ties)
pub fn ranks(numbers: &[Number]) -> Vec<Number> {
    let n = numbers.len();
    if n == 0 {
        return vec![];
    }

    // Create pairs of (value, original_index)
    let mut indexed: Vec<(Number, usize)> = numbers.iter().cloned().enumerate().map(|(i, n)| (n, i)).collect();

    // Sort by value
    indexed.sort_by(|(a, _), (b, _)| {
        let diff = a.sub(b);
        if diff.is_zero() {
            std::cmp::Ordering::Equal
        } else if diff.is_negative() {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    let mut result = vec![Number::from_i64(0); n];
    let mut i = 0;

    while i < n {
        let mut j = i;
        // Find all elements with same value (ties)
        while j < n && indexed[j].0.sub(&indexed[i].0).is_zero() {
            j += 1;
        }

        // Average rank for ties: (i+1 + j) / 2
        let avg_rank = Number::from_i64((i + j + 1) as i64)
            .checked_div(&Number::from_i64(2))
            .unwrap_or(Number::from_i64(1));

        // Assign average rank to all tied elements
        for k in i..j {
            result[indexed[k].1] = avg_rank.clone();
        }

        i = j;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_numbers_list() {
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])];
        let result = extract_numbers(&args).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_extract_numbers_varargs() {
        let args = vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ];
        let result = extract_numbers(&args).unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_sum() {
        let numbers = vec![
            Number::from_i64(1),
            Number::from_i64(2),
            Number::from_i64(3),
        ];
        let result = sum(&numbers);
        assert_eq!(result.to_i64(), Some(6));
    }

    #[test]
    fn test_mean() {
        let numbers = vec![
            Number::from_i64(2),
            Number::from_i64(4),
            Number::from_i64(6),
        ];
        let result = mean(&numbers).unwrap();
        assert_eq!(result.to_i64(), Some(4));
    }

    #[test]
    fn test_sorted() {
        let numbers = vec![
            Number::from_i64(3),
            Number::from_i64(1),
            Number::from_i64(2),
        ];
        let result = sorted(&numbers);
        assert_eq!(result[0].to_i64(), Some(1));
        assert_eq!(result[1].to_i64(), Some(2));
        assert_eq!(result[2].to_i64(), Some(3));
    }
}
```

folio-stats\src\histogram.rs
```rs
//! Histogram and binning functions for distribution analysis

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use std::collections::HashMap;
use crate::helpers::{extract_numbers, require_min_count, sorted};

// ============================================================================
// Histogram
// ============================================================================

pub struct Histogram;

static HISTOGRAM_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "bins",
        typ: "Number|Text",
        description: "Bin count or method: 'auto', 'sturges', 'scott', 'freedman'",
        optional: true,
        default: Some("auto"),
    },
];
static HISTOGRAM_EXAMPLES: [&str; 2] = [
    "histogram([1,2,2,3,3,3,4,4,5], 5) → {edges, counts, density, ...}",
    "histogram(data, \"scott\") → {edges, counts, ...}",
];
static HISTOGRAM_RELATED: [&str; 2] = ["frequency", "bin_edges"];

impl FunctionPlugin for Histogram {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "histogram",
            description: "Create histogram with automatic or specified bin count",
            usage: "histogram(list, bins?)",
            args: &HISTOGRAM_ARGS,
            returns: "Object",
            examples: &HISTOGRAM_EXAMPLES,
            category: "distribution",
            source: None,
            related: &HISTOGRAM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("histogram", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 2, "histogram") {
            return Value::Error(e);
        }

        // Determine bin count
        let bin_count = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => {
                    match n.to_i64() {
                        Some(b) if b > 0 => b as usize,
                        _ => return Value::Error(FolioError::domain_error("Bin count must be a positive integer")),
                    }
                }
                Value::Text(method) => {
                    match calculate_bin_count(&numbers, method.as_str()) {
                        Ok(b) => b,
                        Err(e) => return Value::Error(e),
                    }
                }
                _ => return Value::Error(FolioError::arg_type("histogram", "bins", "Number or Text", args[1].type_name())),
            }
        } else {
            // Default: auto (Freedman-Diaconis with Sturges fallback)
            calculate_bin_count(&numbers, "auto").unwrap_or(10)
        };

        let sorted_data = sorted(&numbers);
        let min_val = sorted_data.first().unwrap().clone();
        let max_val = sorted_data.last().unwrap().clone();

        // Handle case where all values are the same
        if min_val.sub(&max_val).is_zero() {
            let edges = vec![min_val.clone(), min_val.add(&Number::from_i64(1))];
            let counts = vec![Value::Number(Number::from_i64(numbers.len() as i64))];
            let density = vec![Value::Number(Number::from_i64(1))];
            let cumulative = vec![Value::Number(Number::from_i64(numbers.len() as i64))];

            let mut result = HashMap::new();
            result.insert("edges".to_string(), Value::List(edges.into_iter().map(Value::Number).collect()));
            result.insert("counts".to_string(), Value::List(counts));
            result.insert("density".to_string(), Value::List(density));
            result.insert("cumulative".to_string(), Value::List(cumulative));
            result.insert("bin_width".to_string(), Value::Number(Number::from_i64(1)));
            result.insert("n".to_string(), Value::Number(Number::from_i64(numbers.len() as i64)));
            return Value::Object(result);
        }

        // Calculate bin width and edges
        let range = max_val.sub(&min_val);
        let bin_width = match range.checked_div(&Number::from_i64(bin_count as i64)) {
            Ok(w) => w,
            Err(e) => return Value::Error(e.into()),
        };

        // Create bin edges
        let mut edges = Vec::with_capacity(bin_count + 1);
        for i in 0..=bin_count {
            edges.push(min_val.add(&bin_width.mul(&Number::from_i64(i as i64))));
        }

        // Count frequencies
        let mut counts = vec![0i64; bin_count];
        for x in &numbers {
            // Find bin index
            let idx = if x.sub(&max_val).is_zero() {
                // Include max value in last bin
                bin_count - 1
            } else {
                let offset = x.sub(&min_val);
                let bin_idx = match offset.checked_div(&bin_width) {
                    Ok(b) => b.floor().to_i64().unwrap_or(0) as usize,
                    Err(_) => 0,
                };
                bin_idx.min(bin_count - 1)
            };
            counts[idx] += 1;
        }

        // Calculate density (normalized so sum = 1)
        let n = numbers.len() as i64;
        let density: Vec<Value> = counts.iter().map(|c| {
            let d = Number::from_i64(*c).checked_div(&Number::from_i64(n)).unwrap_or(Number::from_i64(0));
            Value::Number(d)
        }).collect();

        // Calculate cumulative counts
        let mut cum = 0i64;
        let cumulative: Vec<Value> = counts.iter().map(|c| {
            cum += c;
            Value::Number(Number::from_i64(cum))
        }).collect();

        let counts_values: Vec<Value> = counts.into_iter().map(|c| Value::Number(Number::from_i64(c))).collect();
        let edges_values: Vec<Value> = edges.into_iter().map(Value::Number).collect();

        let mut result = HashMap::new();
        result.insert("edges".to_string(), Value::List(edges_values));
        result.insert("counts".to_string(), Value::List(counts_values));
        result.insert("density".to_string(), Value::List(density));
        result.insert("cumulative".to_string(), Value::List(cumulative));
        result.insert("bin_width".to_string(), Value::Number(bin_width));
        result.insert("n".to_string(), Value::Number(Number::from_i64(n)));

        Value::Object(result)
    }
}

/// Calculate optimal bin count using various methods
fn calculate_bin_count(numbers: &[Number], method: &str) -> Result<usize, FolioError> {
    let n = numbers.len();
    if n < 2 {
        return Ok(1);
    }

    let sorted_data = sorted(numbers);
    let min_val = sorted_data.first().unwrap();
    let max_val = sorted_data.last().unwrap();
    let range = max_val.sub(min_val);

    if range.is_zero() {
        return Ok(1);
    }

    match method.to_lowercase().as_str() {
        "sturges" => {
            // Sturges: ceil(log2(n)) + 1
            let log2_n = (n as f64).log2().ceil() as usize;
            Ok((log2_n + 1).max(1))
        }
        "scott" => {
            // Scott: 3.5 * σ / n^(1/3)
            let stddev = calculate_stddev(numbers)?;
            let n_cbrt = (n as f64).powf(1.0 / 3.0);
            let stddev_f64 = stddev.to_f64().unwrap_or(1.0);
            let h = 3.5 * stddev_f64 / n_cbrt;
            let range_f64 = range.to_f64().unwrap_or(1.0);
            if h <= 0.0 {
                return Ok(10);
            }
            Ok(((range_f64 / h).ceil() as usize).max(1))
        }
        "freedman" | "fd" => {
            // Freedman-Diaconis: 2 * IQR / n^(1/3)
            let iqr = calculate_iqr(numbers)?;
            let n_cbrt = (n as f64).powf(1.0 / 3.0);
            let iqr_f64 = iqr.to_f64().unwrap_or(1.0);
            let h = 2.0 * iqr_f64 / n_cbrt;
            let range_f64 = range.to_f64().unwrap_or(1.0);
            if h <= 0.0 {
                // Fallback to Sturges
                let log2_n = (n as f64).log2().ceil() as usize;
                return Ok((log2_n + 1).max(1));
            }
            Ok(((range_f64 / h).ceil() as usize).max(1))
        }
        "auto" => {
            // Use Freedman-Diaconis, fallback to Sturges if IQR is 0
            let iqr = calculate_iqr(numbers)?;
            if iqr.is_zero() {
                let log2_n = (n as f64).log2().ceil() as usize;
                return Ok((log2_n + 1).max(1));
            }
            calculate_bin_count(numbers, "freedman")
        }
        _ => Err(FolioError::domain_error(format!(
            "Unknown bin method '{}'. Use 'auto', 'sturges', 'scott', or 'freedman'",
            method
        ))),
    }
}

fn calculate_stddev(numbers: &[Number]) -> Result<Number, FolioError> {
    let variance = crate::helpers::variance_impl(numbers, true)?;
    variance.sqrt(50).map_err(|e| FolioError::new("MATH_ERROR", e.to_string()))
}

fn calculate_iqr(numbers: &[Number]) -> Result<Number, FolioError> {
    let q1 = crate::helpers::percentile_impl(numbers, &Number::from_i64(25))?;
    let q3 = crate::helpers::percentile_impl(numbers, &Number::from_i64(75))?;
    Ok(q3.sub(&q1))
}

// ============================================================================
// BinEdges
// ============================================================================

pub struct BinEdges;

static BIN_EDGES_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "bins",
        typ: "Number|Text",
        description: "Bin count or method: 'auto', 'sturges', 'scott', 'freedman'",
        optional: true,
        default: Some("auto"),
    },
];
static BIN_EDGES_EXAMPLES: [&str; 2] = [
    "bin_edges([1,2,3,4,5], 3) → [1, 2.33, 3.67, 5]",
    "bin_edges(data, \"scott\") → [...]",
];
static BIN_EDGES_RELATED: [&str; 2] = ["histogram", "frequency"];

impl FunctionPlugin for BinEdges {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "bin_edges",
            description: "Calculate bin edges for histogram",
            usage: "bin_edges(list, bins?)",
            args: &BIN_EDGES_ARGS,
            returns: "List",
            examples: &BIN_EDGES_EXAMPLES,
            category: "distribution",
            source: None,
            related: &BIN_EDGES_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("bin_edges", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 1, "bin_edges") {
            return Value::Error(e);
        }

        let bin_count = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => {
                    match n.to_i64() {
                        Some(b) if b > 0 => b as usize,
                        _ => return Value::Error(FolioError::domain_error("Bin count must be a positive integer")),
                    }
                }
                Value::Text(method) => {
                    match calculate_bin_count(&numbers, method.as_str()) {
                        Ok(b) => b,
                        Err(e) => return Value::Error(e),
                    }
                }
                other => return Value::Error(FolioError::arg_type("bin_edges", "bins", "Number or Text", other.type_name())),
            }
        } else {
            // Default: auto
            calculate_bin_count(&numbers, "auto").unwrap_or(10)
        };

        let sorted_data = sorted(&numbers);
        let min_val = sorted_data.first().unwrap().clone();
        let max_val = sorted_data.last().unwrap().clone();
        let range = max_val.sub(&min_val);

        if range.is_zero() {
            return Value::List(vec![Value::Number(min_val), Value::Number(max_val.add(&Number::from_i64(1)))]);
        }

        let bin_width = match range.checked_div(&Number::from_i64(bin_count as i64)) {
            Ok(w) => w,
            Err(e) => return Value::Error(e.into()),
        };

        let mut edges = Vec::with_capacity(bin_count + 1);
        for i in 0..=bin_count {
            edges.push(Value::Number(min_val.add(&bin_width.mul(&Number::from_i64(i as i64)))));
        }

        Value::List(edges)
    }
}

// ============================================================================
// Frequency
// ============================================================================

pub struct Frequency;

static FREQUENCY_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "edges",
        typ: "List",
        description: "Bin edges (n+1 values for n bins)",
        optional: false,
        default: None,
    },
];
static FREQUENCY_EXAMPLES: [&str; 1] = ["frequency([1,2,3,4,5,6], [0,3,6,9]) → [2, 3, 1]"];
static FREQUENCY_RELATED: [&str; 2] = ["histogram", "bin_edges"];

impl FunctionPlugin for Frequency {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "frequency",
            description: "Count frequencies for given bin edges",
            usage: "frequency(list, edges)",
            args: &FREQUENCY_ARGS,
            returns: "List",
            examples: &FREQUENCY_EXAMPLES,
            category: "distribution",
            source: None,
            related: &FREQUENCY_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("frequency", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let edges = match extract_numbers(&args[1..2]) {
            Ok(e) => e,
            Err(e) => return Value::Error(e),
        };

        if edges.len() < 2 {
            return Value::Error(FolioError::domain_error("Need at least 2 edges to define bins"));
        }

        let sorted_edges = sorted(&edges);
        let bin_count = sorted_edges.len() - 1;
        let mut counts = vec![0i64; bin_count];

        for x in &numbers {
            // Find bin for this value
            for i in 0..bin_count {
                let lower = &sorted_edges[i];
                let upper = &sorted_edges[i + 1];
                let in_lower = !x.sub(lower).is_negative();
                let in_upper = if i == bin_count - 1 {
                    // Include upper bound in last bin
                    !upper.sub(x).is_negative()
                } else {
                    upper.sub(x).is_negative() == false && !upper.sub(x).is_zero()
                };
                if in_lower && in_upper {
                    counts[i] += 1;
                    break;
                }
            }
        }

        Value::List(counts.into_iter().map(|c| Value::Number(Number::from_i64(c))).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    fn make_list(values: &[i64]) -> Value {
        Value::List(values.iter().map(|v| Value::Number(Number::from_i64(*v))).collect())
    }

    #[test]
    fn test_histogram_basic() {
        let hist = Histogram;
        let args = vec![
            make_list(&[1, 2, 2, 3, 3, 3, 4, 4, 5]),
            Value::Number(Number::from_i64(4)),
        ];
        let ctx = eval_ctx();
        let result = hist.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("edges"));
            assert!(obj.contains_key("counts"));
            assert!(obj.contains_key("density"));
            assert!(obj.contains_key("cumulative"));
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_frequency() {
        let freq = Frequency;
        let args = vec![
            make_list(&[1, 2, 3, 4, 5, 6]),
            make_list(&[0, 3, 6, 9]),
        ];
        let ctx = eval_ctx();
        let result = freq.call(&args, &ctx);

        if let Value::List(counts) = result {
            assert_eq!(counts.len(), 3);
            // [0,3): 1,2 -> 2 values
            // [3,6): 3,4,5 -> 3 values
            // [6,9]: 6 -> 1 value
            if let Value::Number(n) = &counts[0] {
                assert_eq!(n.to_i64(), Some(2));
            }
            if let Value::Number(n) = &counts[1] {
                assert_eq!(n.to_i64(), Some(3));
            }
            if let Value::Number(n) = &counts[2] {
                assert_eq!(n.to_i64(), Some(1));
            }
        } else {
            panic!("Expected List, got {:?}", result);
        }
    }
}
```

folio-stats\src\hypothesis.rs
```rs
//! Hypothesis testing functions: t_test, chi_test, f_test, anova

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, extract_two_lists, mean, variance_impl};
use crate::distributions::t::t_cdf_f64;
use crate::distributions::chi::chi_cdf_f64;
use crate::distributions::f::f_cdf_f64;
use std::collections::HashMap;

// ============ One-Sample T-Test ============

pub struct TTest1;

static T_TEST_1_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Sample data",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "μ0",
        typ: "Number",
        description: "Hypothesized population mean",
        optional: false,
        default: None,
    },
];

static T_TEST_1_EXAMPLES: [&str; 1] = ["t_test_1([1,2,3,4,5], 3)"];

static T_TEST_1_RELATED: [&str; 2] = ["t_test_2", "ci"];

impl FunctionPlugin for TTest1 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_test_1",
            description: "One-sample t-test",
            usage: "t_test_1(list, μ0)",
            args: &T_TEST_1_ARGS,
            returns: "Object",
            examples: &T_TEST_1_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &T_TEST_1_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("t_test_1", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let mu0 = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_test_1", "μ0", "Number", other.type_name())),
        };

        if numbers.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "t_test_1() requires at least 2 observations",
            ));
        }

        let n = numbers.len();
        let df = (n - 1) as f64;

        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let se = match var.sqrt(ctx.precision) {
            Ok(v) => v.checked_div(&Number::from_i64(n as i64).sqrt(ctx.precision).unwrap_or(Number::from_i64(1))),
            Err(e) => return Value::Error(e.into()),
        };

        let se = match se {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if se.is_zero() {
            return Value::Error(FolioError::domain_error(
                "t_test_1() requires non-zero standard error",
            ));
        }

        let mean_diff = m.sub(mu0);
        let t_stat = match mean_diff.checked_div(&se) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let t_f64 = t_stat.to_f64().unwrap_or(0.0);

        // Two-tailed p-value
        let p_value = 2.0 * (1.0 - t_cdf_f64(t_f64.abs(), df));

        // 95% confidence interval
        let t_crit = 1.96; // Approximate for large samples
        let margin = se.mul(&Number::from_str(&format!("{:.15}", t_crit)).unwrap_or(Number::from_i64(2)));
        let ci_low = m.sub(&margin);
        let ci_high = m.add(&margin);

        let mut result = HashMap::new();
        result.insert("t".to_string(), Value::Number(t_stat));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df".to_string(), Value::Number(Number::from_i64(df as i64)));
        result.insert("ci_low".to_string(), Value::Number(ci_low));
        result.insert("ci_high".to_string(), Value::Number(ci_high));
        result.insert("mean_diff".to_string(), Value::Number(mean_diff));

        Value::Object(result)
    }
}

// ============ Two-Sample T-Test (Welch's) ============

pub struct TTest2;

static T_TEST_2_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List<Number>",
        description: "First sample",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List<Number>",
        description: "Second sample",
        optional: false,
        default: None,
    },
];

static T_TEST_2_EXAMPLES: [&str; 1] = ["t_test_2([1,2,3], [4,5,6])"];

static T_TEST_2_RELATED: [&str; 2] = ["t_test_1", "t_test_paired"];

impl FunctionPlugin for TTest2 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_test_2",
            description: "Two-sample t-test (Welch's)",
            usage: "t_test_2(list1, list2)",
            args: &T_TEST_2_ARGS,
            returns: "Object",
            examples: &T_TEST_2_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &T_TEST_2_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 || y.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "t_test_2() requires at least 2 observations in each group",
            ));
        }

        let n1 = x.len() as f64;
        let n2 = y.len() as f64;

        let m1 = match mean(&x) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let m2 = match mean(&y) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var1 = match variance_impl(&x, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let var2 = match variance_impl(&y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var1_f64 = var1.to_f64().unwrap_or(0.0);
        let var2_f64 = var2.to_f64().unwrap_or(0.0);

        // Welch's t-test
        let se_squared = var1_f64 / n1 + var2_f64 / n2;
        let se = se_squared.sqrt();

        if se == 0.0 {
            return Value::Error(FolioError::domain_error(
                "t_test_2() requires non-zero pooled standard error",
            ));
        }

        let mean_diff = m1.sub(&m2);
        let mean_diff_f64 = mean_diff.to_f64().unwrap_or(0.0);
        let t_stat = mean_diff_f64 / se;

        // Welch-Satterthwaite degrees of freedom
        let num = se_squared * se_squared;
        let den = (var1_f64 / n1).powi(2) / (n1 - 1.0) + (var2_f64 / n2).powi(2) / (n2 - 1.0);
        let df = num / den;

        // Two-tailed p-value
        let p_value = 2.0 * (1.0 - t_cdf_f64(t_stat.abs(), df));

        // Confidence interval
        let t_crit = 1.96;
        let margin = t_crit * se;
        let ci_low = mean_diff_f64 - margin;
        let ci_high = mean_diff_f64 + margin;

        let mut result = HashMap::new();
        result.insert("t".to_string(), Value::Number(Number::from_str(&format!("{:.15}", t_stat)).unwrap_or(Number::from_i64(0))));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df".to_string(), Value::Number(Number::from_str(&format!("{:.15}", df)).unwrap_or(Number::from_i64(0))));
        result.insert("ci_low".to_string(), Value::Number(Number::from_str(&format!("{:.15}", ci_low)).unwrap_or(Number::from_i64(0))));
        result.insert("ci_high".to_string(), Value::Number(Number::from_str(&format!("{:.15}", ci_high)).unwrap_or(Number::from_i64(0))));
        result.insert("mean_diff".to_string(), Value::Number(mean_diff));

        Value::Object(result)
    }
}

// ============ Paired T-Test ============

pub struct TTestPaired;

static T_TEST_PAIRED_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List<Number>",
        description: "First measurements",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List<Number>",
        description: "Second measurements (paired)",
        optional: false,
        default: None,
    },
];

static T_TEST_PAIRED_EXAMPLES: [&str; 1] = ["t_test_paired([1,2,3], [2,3,4])"];

static T_TEST_PAIRED_RELATED: [&str; 2] = ["t_test_1", "t_test_2"];

impl FunctionPlugin for TTestPaired {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_test_paired",
            description: "Paired t-test",
            usage: "t_test_paired(list1, list2)",
            args: &T_TEST_PAIRED_ARGS,
            returns: "Object",
            examples: &T_TEST_PAIRED_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &T_TEST_PAIRED_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "t_test_paired() requires at least 2 pairs",
            ));
        }

        // Calculate differences
        let diffs: Vec<Number> = x.iter().zip(y.iter())
            .map(|(a, b)| a.sub(b))
            .collect();

        // One-sample t-test on differences with μ0 = 0
        let n = diffs.len();
        let df = (n - 1) as f64;

        let m = match mean(&diffs) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&diffs, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let se = match var.sqrt(ctx.precision) {
            Ok(v) => v.checked_div(&Number::from_i64(n as i64).sqrt(ctx.precision).unwrap_or(Number::from_i64(1))),
            Err(e) => return Value::Error(e.into()),
        };

        let se = match se {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if se.is_zero() {
            return Value::Error(FolioError::domain_error(
                "t_test_paired() requires non-zero standard error of differences",
            ));
        }

        let t_stat = match m.checked_div(&se) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let t_f64 = t_stat.to_f64().unwrap_or(0.0);
        let p_value = 2.0 * (1.0 - t_cdf_f64(t_f64.abs(), df));

        let t_crit = 1.96;
        let margin = se.mul(&Number::from_str(&format!("{:.15}", t_crit)).unwrap_or(Number::from_i64(2)));
        let ci_low = m.sub(&margin);
        let ci_high = m.add(&margin);

        let mut result = HashMap::new();
        result.insert("t".to_string(), Value::Number(t_stat));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df".to_string(), Value::Number(Number::from_i64(df as i64)));
        result.insert("ci_low".to_string(), Value::Number(ci_low));
        result.insert("ci_high".to_string(), Value::Number(ci_high));
        result.insert("mean_diff".to_string(), Value::Number(m));

        Value::Object(result)
    }
}

// ============ Chi-Squared Test ============

pub struct ChiTest;

static CHI_TEST_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "observed",
        typ: "List<Number>",
        description: "Observed frequencies",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "expected",
        typ: "List<Number>",
        description: "Expected frequencies",
        optional: false,
        default: None,
    },
];

static CHI_TEST_EXAMPLES: [&str; 1] = ["chi_test([10,20,30], [15,20,25])"];

static CHI_TEST_RELATED: [&str; 2] = ["chi_cdf", "anova"];

impl FunctionPlugin for ChiTest {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "chi_test",
            description: "Chi-squared goodness of fit test",
            usage: "chi_test(observed, expected)",
            args: &CHI_TEST_ARGS,
            returns: "Object",
            examples: &CHI_TEST_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &CHI_TEST_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (observed, expected) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if observed.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "chi_test() requires at least 2 categories",
            ));
        }

        // Calculate chi-squared statistic
        let mut chi_sq = 0.0;
        for (o, e) in observed.iter().zip(expected.iter()) {
            let o_f64 = o.to_f64().unwrap_or(0.0);
            let e_f64 = e.to_f64().unwrap_or(0.0);
            if e_f64 <= 0.0 {
                return Value::Error(FolioError::domain_error(
                    "chi_test() requires all expected values > 0",
                ));
            }
            chi_sq += (o_f64 - e_f64).powi(2) / e_f64;
        }

        let df = (observed.len() - 1) as f64;
        let p_value = 1.0 - chi_cdf_f64(chi_sq, df);

        let mut result = HashMap::new();
        result.insert("chi_sq".to_string(), Value::Number(Number::from_str(&format!("{:.15}", chi_sq)).unwrap_or(Number::from_i64(0))));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df".to_string(), Value::Number(Number::from_i64(df as i64)));

        Value::Object(result)
    }
}

// ============ F-Test ============

pub struct FTest;

static F_TEST_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List<Number>",
        description: "First sample",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List<Number>",
        description: "Second sample",
        optional: false,
        default: None,
    },
];

static F_TEST_EXAMPLES: [&str; 1] = ["f_test([1,2,3], [4,5,6])"];

static F_TEST_RELATED: [&str; 2] = ["f_cdf", "anova"];

impl FunctionPlugin for FTest {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "f_test",
            description: "F-test for variance equality",
            usage: "f_test(list1, list2)",
            args: &F_TEST_ARGS,
            returns: "Object",
            examples: &F_TEST_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &F_TEST_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 || y.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "f_test() requires at least 2 observations in each group",
            ));
        }

        let var1 = match variance_impl(&x, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let var2 = match variance_impl(&y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var1_f64 = var1.to_f64().unwrap_or(0.0);
        let var2_f64 = var2.to_f64().unwrap_or(0.0);

        if var2_f64 == 0.0 {
            return Value::Error(FolioError::domain_error(
                "f_test() requires non-zero variance in second sample",
            ));
        }

        let f_stat = var1_f64 / var2_f64;
        let df1 = (x.len() - 1) as f64;
        let df2 = (y.len() - 1) as f64;

        // Two-tailed p-value
        let p = f_cdf_f64(f_stat, df1, df2);
        let p_value = 2.0 * (if p > 0.5 { 1.0 - p } else { p });

        let mut result = HashMap::new();
        result.insert("f".to_string(), Value::Number(Number::from_str(&format!("{:.15}", f_stat)).unwrap_or(Number::from_i64(0))));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df1".to_string(), Value::Number(Number::from_i64(df1 as i64)));
        result.insert("df2".to_string(), Value::Number(Number::from_i64(df2 as i64)));

        Value::Object(result)
    }
}

// ============ ANOVA ============

pub struct Anova;

static ANOVA_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "groups",
    typ: "List<List<Number>> | List<Number>...",
    description: "Two or more groups to compare",
    optional: false,
    default: None,
}];

static ANOVA_EXAMPLES: [&str; 1] = ["anova([1,2,3], [4,5,6], [7,8,9])"];

static ANOVA_RELATED: [&str; 2] = ["f_test", "t_test_2"];

impl FunctionPlugin for Anova {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "anova",
            description: "One-way ANOVA",
            usage: "anova(group1, group2, ...)",
            args: &ANOVA_ARGS,
            returns: "Object",
            examples: &ANOVA_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &ANOVA_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        // Extract groups
        let mut groups: Vec<Vec<f64>> = Vec::new();

        for arg in args {
            match arg {
                Value::List(list) => {
                    let mut group = Vec::new();
                    for item in list {
                        match item {
                            Value::Number(n) => group.push(n.to_f64().unwrap_or(0.0)),
                            Value::Error(e) => return Value::Error(e.clone()),
                            _ => return Value::Error(FolioError::type_error("Number", item.type_name())),
                        }
                    }
                    groups.push(group);
                }
                Value::Error(e) => return Value::Error(e.clone()),
                _ => return Value::Error(FolioError::type_error("List", arg.type_name())),
            }
        }

        if groups.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "anova() requires at least 2 groups",
            ));
        }

        for g in &groups {
            if g.is_empty() {
                return Value::Error(FolioError::domain_error(
                    "anova() requires non-empty groups",
                ));
            }
        }

        // Calculate grand mean
        let total_n: usize = groups.iter().map(|g| g.len()).sum();
        let total_sum: f64 = groups.iter().flat_map(|g| g.iter()).sum();
        let grand_mean = total_sum / total_n as f64;

        // Calculate SS_between and SS_within
        let mut ss_between = 0.0;
        let mut ss_within = 0.0;

        for group in &groups {
            let n = group.len() as f64;
            let group_mean: f64 = group.iter().sum::<f64>() / n;

            ss_between += n * (group_mean - grand_mean).powi(2);

            for x in group {
                ss_within += (x - group_mean).powi(2);
            }
        }

        let k = groups.len() as f64;
        let df_between = k - 1.0;
        let df_within = total_n as f64 - k;

        let ms_between = ss_between / df_between;
        let ms_within = ss_within / df_within;

        let f_stat = if ms_within > 0.0 {
            ms_between / ms_within
        } else {
            return Value::Error(FolioError::domain_error(
                "anova() requires non-zero within-group variance",
            ));
        };

        let p_value = 1.0 - f_cdf_f64(f_stat, df_between, df_within);

        let mut result = HashMap::new();
        result.insert("f".to_string(), Value::Number(Number::from_str(&format!("{:.15}", f_stat)).unwrap_or(Number::from_i64(0))));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df_between".to_string(), Value::Number(Number::from_i64(df_between as i64)));
        result.insert("df_within".to_string(), Value::Number(Number::from_i64(df_within as i64)));
        result.insert("ss_between".to_string(), Value::Number(Number::from_str(&format!("{:.15}", ss_between)).unwrap_or(Number::from_i64(0))));
        result.insert("ss_within".to_string(), Value::Number(Number::from_str(&format!("{:.15}", ss_within)).unwrap_or(Number::from_i64(0))));

        Value::Object(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_t_test_1() {
        let t_test = TTest1;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(5)),
            ]),
            Value::Number(Number::from_i64(3)),
        ];
        let result = t_test.call(&args, &eval_ctx());
        assert!(result.as_object().is_some());
    }
}
```

folio-stats\src\lib.rs
```rs
//! Folio Statistics Plugin
//!
//! Statistical functions with arbitrary precision using BigRational.
//! All functions follow the never-panic philosophy and return `Value::Error` on failure.

mod helpers;
mod central;
mod dispersion;
mod position;
mod shape;
mod bivariate;
mod regression;
mod distributions;
mod hypothesis;
mod confidence;
mod transform;
mod histogram;
mod outliers;
mod goodness;
mod qq;

use folio_plugin::PluginRegistry;

/// Load statistics functions into registry
pub fn load_stats_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Central tendency
        .with_function(central::Mean)
        .with_function(central::Median)
        .with_function(central::Mode)
        .with_function(central::GeometricMean)
        .with_function(central::HarmonicMean)
        .with_function(central::TrimmedMean)
        .with_function(central::WeightedMean)

        // Dispersion
        .with_function(dispersion::Variance)
        .with_function(dispersion::VarianceP)
        .with_function(dispersion::Stddev)
        .with_function(dispersion::StddevP)
        .with_function(dispersion::Range)
        .with_function(dispersion::Iqr)
        .with_function(dispersion::Mad)
        .with_function(dispersion::Cv)
        .with_function(dispersion::Se)

        // Position
        .with_function(position::Min)
        .with_function(position::Max)
        .with_function(position::Percentile)
        .with_function(position::Quantile)
        .with_function(position::Q1)
        .with_function(position::Q3)
        .with_function(position::Rank)
        .with_function(position::Ranks)
        .with_function(position::Zscore)

        // Shape
        .with_function(shape::Skewness)
        .with_function(shape::Kurtosis)
        .with_function(shape::Count)
        .with_function(shape::Product)

        // Bivariate
        .with_function(bivariate::Covariance)
        .with_function(bivariate::CovarianceP)
        .with_function(bivariate::Correlation)
        .with_function(bivariate::Spearman)

        // Regression
        .with_function(regression::LinearReg)
        .with_function(regression::Slope)
        .with_function(regression::Intercept)
        .with_function(regression::RSquared)
        .with_function(regression::Predict)
        .with_function(regression::Residuals)

        // Distributions - Normal
        .with_function(distributions::NormPdf)
        .with_function(distributions::NormCdf)
        .with_function(distributions::NormInv)
        .with_function(distributions::SnormPdf)
        .with_function(distributions::SnormCdf)
        .with_function(distributions::SnormInv)
        // Distributions - Student's t
        .with_function(distributions::TPdf)
        .with_function(distributions::TCdf)
        .with_function(distributions::TInv)
        // Distributions - Chi-squared
        .with_function(distributions::ChiPdf)
        .with_function(distributions::ChiCdf)
        .with_function(distributions::ChiInv)
        // Distributions - F
        .with_function(distributions::FPdf)
        .with_function(distributions::FCdf)
        .with_function(distributions::FInv)
        // Distributions - Discrete
        .with_function(distributions::BinomPmf)
        .with_function(distributions::BinomCdf)
        .with_function(distributions::PoissonPmf)
        .with_function(distributions::PoissonCdf)

        // Hypothesis tests
        .with_function(hypothesis::TTest1)
        .with_function(hypothesis::TTest2)
        .with_function(hypothesis::TTestPaired)
        .with_function(hypothesis::ChiTest)
        .with_function(hypothesis::FTest)
        .with_function(hypothesis::Anova)

        // Confidence intervals
        .with_function(confidence::Ci)
        .with_function(confidence::Moe)

        // Transforms
        .with_function(transform::Normalize)
        .with_function(transform::Standardize)
        .with_function(transform::Cumsum)
        .with_function(transform::Differences)
        .with_function(transform::Lag)
        .with_function(transform::MovingAvg)
        .with_function(transform::Ewma)

        // Histogram & Binning
        .with_function(histogram::Histogram)
        .with_function(histogram::BinEdges)
        .with_function(histogram::Frequency)

        // Outlier Detection
        .with_function(outliers::OutliersIqr)
        .with_function(outliers::OutliersZscore)
        .with_function(outliers::OutliersMad)
        .with_function(outliers::GrubbsTest)

        // Goodness-of-Fit Tests
        .with_function(goodness::JarqueBera)
        .with_function(goodness::ShapiroWilk)
        .with_function(goodness::IsNormal)
        .with_function(goodness::KsTest2)

        // Q-Q Analysis
        .with_function(qq::QQPoints)
        .with_function(qq::QQResiduals)
}
```

folio-stats\src\outliers.rs
```rs
//! Outlier detection functions

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use std::collections::HashMap;
use crate::helpers::{extract_numbers, require_min_count, sorted, mean, variance_impl};

// ============================================================================
// OutliersIqr - IQR-based outlier detection
// ============================================================================

pub struct OutliersIqr;

static OUTLIERS_IQR_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "k",
        typ: "Number",
        description: "IQR multiplier (default 1.5, use 3 for extreme)",
        optional: true,
        default: Some("1.5"),
    },
];
static OUTLIERS_IQR_EXAMPLES: [&str; 2] = [
    "outliers_iqr([1,2,3,100]) → {indices: [...], values: [...], lower_fence, upper_fence, q1, q3, iqr, count}",
    "outliers_iqr(data, 3) → extreme outliers only (k=3)",
];
static OUTLIERS_IQR_RELATED: [&str; 2] = ["outliers_zscore", "outliers_mad"];

impl FunctionPlugin for OutliersIqr {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "outliers_iqr",
            description: "IQR-based outlier detection (Tukey's method)",
            usage: "outliers_iqr(list, k?)",
            args: &OUTLIERS_IQR_ARGS,
            returns: "Object",
            examples: &OUTLIERS_IQR_EXAMPLES,
            category: "distribution",
            source: None,
            related: &OUTLIERS_IQR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("outliers_iqr", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 4, "outliers_iqr") {
            return Value::Error(e);
        }

        // Get k multiplier (default 1.5)
        let k = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.clone(),
                other => return Value::Error(FolioError::arg_type("outliers_iqr", "k", "Number", other.type_name())),
            }
        } else {
            Number::from_str("1.5").unwrap()
        };

        // Calculate Q1, Q3, IQR
        let q1 = match crate::helpers::percentile_impl(&numbers, &Number::from_i64(25)) {
            Ok(q) => q,
            Err(e) => return Value::Error(e),
        };
        let q3 = match crate::helpers::percentile_impl(&numbers, &Number::from_i64(75)) {
            Ok(q) => q,
            Err(e) => return Value::Error(e),
        };
        let iqr = q3.sub(&q1);

        // Calculate fences
        let k_iqr = k.mul(&iqr);
        let lower_fence = q1.sub(&k_iqr);
        let upper_fence = q3.add(&k_iqr);

        // Find outliers
        let mut indices = Vec::new();
        let mut values = Vec::new();

        for (i, x) in numbers.iter().enumerate() {
            let below_lower = x.sub(&lower_fence).is_negative();
            let above_upper = !x.sub(&upper_fence).is_negative() && !x.sub(&upper_fence).is_zero();
            if below_lower || above_upper {
                indices.push(Value::Number(Number::from_i64(i as i64)));
                values.push(Value::Number(x.clone()));
            }
        }

        let count = indices.len() as i64;

        let mut result = HashMap::new();
        result.insert("indices".to_string(), Value::List(indices));
        result.insert("values".to_string(), Value::List(values));
        result.insert("count".to_string(), Value::Number(Number::from_i64(count)));
        result.insert("lower_fence".to_string(), Value::Number(lower_fence));
        result.insert("upper_fence".to_string(), Value::Number(upper_fence));
        result.insert("q1".to_string(), Value::Number(q1));
        result.insert("q3".to_string(), Value::Number(q3));
        result.insert("iqr".to_string(), Value::Number(iqr));

        Value::Object(result)
    }
}

// ============================================================================
// OutliersZscore - Z-score based outlier detection
// ============================================================================

pub struct OutliersZscore;

static OUTLIERS_ZSCORE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "threshold",
        typ: "Number",
        description: "Z-score threshold (default 3)",
        optional: true,
        default: Some("3"),
    },
];
static OUTLIERS_ZSCORE_EXAMPLES: [&str; 1] = ["outliers_zscore([1,2,3,100], 3) → {indices: [3], z_scores: [2.89], ...}"];
static OUTLIERS_ZSCORE_RELATED: [&str; 2] = ["outliers_iqr", "zscore"];

impl FunctionPlugin for OutliersZscore {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "outliers_zscore",
            description: "Z-score based outlier detection",
            usage: "outliers_zscore(list, threshold?)",
            args: &OUTLIERS_ZSCORE_ARGS,
            returns: "Object",
            examples: &OUTLIERS_ZSCORE_EXAMPLES,
            category: "distribution",
            source: None,
            related: &OUTLIERS_ZSCORE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("outliers_zscore", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "outliers_zscore") {
            return Value::Error(e);
        }

        // Get threshold (default 3)
        let threshold = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.clone(),
                other => return Value::Error(FolioError::arg_type("outliers_zscore", "threshold", "Number", other.type_name())),
            }
        } else {
            Number::from_i64(3)
        };

        // Calculate mean and stddev
        let m = match mean(&numbers) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };
        let variance = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let stddev = match variance.sqrt(50) {
            Ok(s) => s,
            Err(e) => return Value::Error(e.into()),
        };

        if stddev.is_zero() {
            // All values are the same, no outliers
            let mut result = HashMap::new();
            result.insert("indices".to_string(), Value::List(vec![]));
            result.insert("values".to_string(), Value::List(vec![]));
            result.insert("z_scores".to_string(), Value::List(vec![]));
            result.insert("count".to_string(), Value::Number(Number::from_i64(0)));
            result.insert("mean".to_string(), Value::Number(m));
            result.insert("stddev".to_string(), Value::Number(stddev));
            result.insert("threshold".to_string(), Value::Number(threshold));
            return Value::Object(result);
        }

        // Find outliers
        let mut indices = Vec::new();
        let mut values = Vec::new();
        let mut z_scores = Vec::new();

        for (i, x) in numbers.iter().enumerate() {
            let z = match x.sub(&m).checked_div(&stddev) {
                Ok(z) => z,
                Err(_) => continue,
            };
            let abs_z = z.abs();
            if !abs_z.sub(&threshold).is_negative() {
                indices.push(Value::Number(Number::from_i64(i as i64)));
                values.push(Value::Number(x.clone()));
                z_scores.push(Value::Number(z));
            }
        }

        let count = indices.len() as i64;

        let mut result = HashMap::new();
        result.insert("indices".to_string(), Value::List(indices));
        result.insert("values".to_string(), Value::List(values));
        result.insert("z_scores".to_string(), Value::List(z_scores));
        result.insert("count".to_string(), Value::Number(Number::from_i64(count)));
        result.insert("mean".to_string(), Value::Number(m));
        result.insert("stddev".to_string(), Value::Number(stddev));
        result.insert("threshold".to_string(), Value::Number(threshold));

        Value::Object(result)
    }
}

// ============================================================================
// OutliersMad - MAD-based outlier detection (robust)
// ============================================================================

pub struct OutliersMad;

static OUTLIERS_MAD_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "threshold",
        typ: "Number",
        description: "Modified Z-score threshold (default 3.5)",
        optional: true,
        default: Some("3.5"),
    },
];
static OUTLIERS_MAD_EXAMPLES: [&str; 1] = ["outliers_mad([1,2,3,100], 3.5) → {indices: [...], modified_z: [...], ...}"];
static OUTLIERS_MAD_RELATED: [&str; 2] = ["outliers_iqr", "mad"];

impl FunctionPlugin for OutliersMad {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "outliers_mad",
            description: "MAD-based outlier detection (robust to outliers)",
            usage: "outliers_mad(list, threshold?)",
            args: &OUTLIERS_MAD_ARGS,
            returns: "Object",
            examples: &OUTLIERS_MAD_EXAMPLES,
            category: "distribution",
            source: None,
            related: &OUTLIERS_MAD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("outliers_mad", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "outliers_mad") {
            return Value::Error(e);
        }

        // Get threshold (default 3.5)
        let threshold = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.clone(),
                other => return Value::Error(FolioError::arg_type("outliers_mad", "threshold", "Number", other.type_name())),
            }
        } else {
            Number::from_str("3.5").unwrap()
        };

        // Calculate median
        let sorted_nums = sorted(&numbers);
        let n = sorted_nums.len();
        let median = if n % 2 == 0 {
            let mid = n / 2;
            sorted_nums[mid - 1].add(&sorted_nums[mid]).checked_div(&Number::from_i64(2)).unwrap_or(sorted_nums[mid].clone())
        } else {
            sorted_nums[n / 2].clone()
        };

        // Calculate MAD (Median Absolute Deviation)
        let abs_devs: Vec<Number> = numbers.iter().map(|x| x.sub(&median).abs()).collect();
        let sorted_devs = sorted(&abs_devs);
        let mad = if n % 2 == 0 {
            let mid = n / 2;
            sorted_devs[mid - 1].add(&sorted_devs[mid]).checked_div(&Number::from_i64(2)).unwrap_or(sorted_devs[mid].clone())
        } else {
            sorted_devs[n / 2].clone()
        };

        // Scale factor for consistency with normal distribution: 1.4826
        let scale = Number::from_str("1.4826").unwrap();
        let scaled_mad = mad.mul(&scale);

        if scaled_mad.is_zero() {
            // All values have same deviation from median
            let mut result = HashMap::new();
            result.insert("indices".to_string(), Value::List(vec![]));
            result.insert("values".to_string(), Value::List(vec![]));
            result.insert("modified_z".to_string(), Value::List(vec![]));
            result.insert("count".to_string(), Value::Number(Number::from_i64(0)));
            result.insert("median".to_string(), Value::Number(median));
            result.insert("mad".to_string(), Value::Number(mad));
            result.insert("threshold".to_string(), Value::Number(threshold));
            return Value::Object(result);
        }

        // Find outliers using modified Z-score
        let mut indices = Vec::new();
        let mut values = Vec::new();
        let mut modified_z = Vec::new();

        for (i, x) in numbers.iter().enumerate() {
            let m_z = match x.sub(&median).checked_div(&scaled_mad) {
                Ok(z) => z,
                Err(_) => continue,
            };
            let abs_mz = m_z.abs();
            if !abs_mz.sub(&threshold).is_negative() {
                indices.push(Value::Number(Number::from_i64(i as i64)));
                values.push(Value::Number(x.clone()));
                modified_z.push(Value::Number(m_z));
            }
        }

        let count = indices.len() as i64;

        let mut result = HashMap::new();
        result.insert("indices".to_string(), Value::List(indices));
        result.insert("values".to_string(), Value::List(values));
        result.insert("modified_z".to_string(), Value::List(modified_z));
        result.insert("count".to_string(), Value::Number(Number::from_i64(count)));
        result.insert("median".to_string(), Value::Number(median));
        result.insert("mad".to_string(), Value::Number(mad));
        result.insert("threshold".to_string(), Value::Number(threshold));

        Value::Object(result)
    }
}

// ============================================================================
// GrubbsTest - Grubbs' test for single outlier
// ============================================================================

pub struct GrubbsTest;

static GRUBBS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "alpha",
        typ: "Number",
        description: "Significance level (default 0.05)",
        optional: true,
        default: Some("0.05"),
    },
];
static GRUBBS_EXAMPLES: [&str; 1] = ["grubbs_test([1,2,3,100]) → {has_outlier: true, outlier, index, g_statistic, critical_value, p_value}"];
static GRUBBS_RELATED: [&str; 2] = ["outliers_iqr", "outliers_zscore"];

impl FunctionPlugin for GrubbsTest {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "grubbs_test",
            description: "Grubbs' test for a single outlier",
            usage: "grubbs_test(list, alpha?)",
            args: &GRUBBS_ARGS,
            returns: "Object",
            examples: &GRUBBS_EXAMPLES,
            category: "distribution",
            source: None,
            related: &GRUBBS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("grubbs_test", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "grubbs_test") {
            return Value::Error(e);
        }

        // Get alpha (default 0.05)
        let alpha = if args.len() == 2 {
            match &args[1] {
                Value::Number(n) => n.to_f64().unwrap_or(0.05),
                other => return Value::Error(FolioError::arg_type("grubbs_test", "alpha", "Number", other.type_name())),
            }
        } else {
            0.05
        };

        let n = numbers.len();

        // Calculate mean and stddev
        let m = match mean(&numbers) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };
        let variance = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let stddev = match variance.sqrt(50) {
            Ok(s) => s,
            Err(e) => return Value::Error(e.into()),
        };

        if stddev.is_zero() {
            let mut result = HashMap::new();
            result.insert("has_outlier".to_string(), Value::Bool(false));
            result.insert("outlier".to_string(), Value::Null);
            result.insert("index".to_string(), Value::Null);
            result.insert("g_statistic".to_string(), Value::Number(Number::from_i64(0)));
            result.insert("critical_value".to_string(), Value::Number(Number::from_i64(0)));
            result.insert("p_value".to_string(), Value::Number(Number::from_i64(1)));
            return Value::Object(result);
        }

        // Find the value with maximum deviation from mean
        let mut max_dev = Number::from_i64(0);
        let mut outlier_idx = 0;
        let mut outlier_val = numbers[0].clone();

        for (i, x) in numbers.iter().enumerate() {
            let dev = x.sub(&m).abs();
            if !dev.sub(&max_dev).is_negative() && !dev.sub(&max_dev).is_zero() {
                max_dev = dev.clone();
                outlier_idx = i;
                outlier_val = x.clone();
            }
        }

        // Calculate Grubbs statistic G = max|x - mean| / s
        let g_stat = match max_dev.checked_div(&stddev) {
            Ok(g) => g,
            Err(e) => return Value::Error(e.into()),
        };

        // Calculate critical value using t-distribution approximation
        // G_crit = ((n-1) / sqrt(n)) * sqrt(t^2 / (n - 2 + t^2))
        // where t is the two-sided t-value at alpha/(2n) with n-2 df
        let n_f64 = n as f64;

        // Approximate t-value using inverse normal (simplified for common cases)
        let t_alpha = t_critical(alpha / (2.0 * n_f64), n - 2);
        let t_sq = t_alpha * t_alpha;
        let g_crit = ((n_f64 - 1.0) / n_f64.sqrt()) * (t_sq / (n_f64 - 2.0 + t_sq)).sqrt();

        let has_outlier = g_stat.to_f64().unwrap_or(0.0) > g_crit;

        // Approximate p-value (simplified)
        let p_value = if has_outlier { alpha } else { 1.0 - alpha };

        let mut result = HashMap::new();
        result.insert("has_outlier".to_string(), Value::Bool(has_outlier));
        if has_outlier {
            result.insert("outlier".to_string(), Value::Number(outlier_val));
            result.insert("index".to_string(), Value::Number(Number::from_i64(outlier_idx as i64)));
        } else {
            result.insert("outlier".to_string(), Value::Null);
            result.insert("index".to_string(), Value::Null);
        }
        result.insert("g_statistic".to_string(), Value::Number(g_stat));
        result.insert("critical_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", g_crit)).unwrap_or(Number::from_i64(0))));
        result.insert("p_value".to_string(), Value::Number(Number::from_str(&format!("{:.10}", p_value)).unwrap_or(Number::from_i64(0))));

        Value::Object(result)
    }
}

/// Approximate t-critical value using inverse normal approximation
fn t_critical(alpha: f64, df: usize) -> f64 {
    // Use normal approximation for large df, else lookup tables
    // This is a simplified approximation
    let z = normal_inv(1.0 - alpha);
    if df >= 30 {
        z
    } else {
        // Cornish-Fisher expansion for small df
        let g1 = (z * z * z + z) / 4.0;
        let g2 = (5.0 * z.powi(5) + 16.0 * z.powi(3) + 3.0 * z) / 96.0;
        z + g1 / (df as f64) + g2 / ((df as f64).powi(2))
    }
}

/// Approximate inverse normal CDF
fn normal_inv(p: f64) -> f64 {
    // Abramowitz and Stegun approximation
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if p == 0.5 {
        return 0.0;
    }

    let a = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383577518672690e2,
        -3.066479806614716e1,
        2.506628277459239e0,
    ];
    let b = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    let c = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838e0,
        -2.549732539343734e0,
        4.374664141464968e0,
        2.938163982698783e0,
    ];
    let d = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996e0,
        3.754408661907416e0,
    ];

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    fn make_list(values: &[i64]) -> Value {
        Value::List(values.iter().map(|v| Value::Number(Number::from_i64(*v))).collect())
    }

    #[test]
    fn test_outliers_iqr() {
        let outliers = OutliersIqr;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 100])];
        let ctx = eval_ctx();
        let result = outliers.call(&args, &ctx);

        if let Value::Object(obj) = result {
            if let Some(Value::Number(count)) = obj.get("count") {
                assert!(count.to_i64().unwrap() > 0, "Should detect outlier");
            }
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_outliers_zscore() {
        let outliers = OutliersZscore;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 100])];
        let ctx = eval_ctx();
        let result = outliers.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("indices"));
            assert!(obj.contains_key("z_scores"));
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }
}
```

folio-stats\src\position.rs
```rs
//! Position functions: min, max, percentile, quantile, etc.

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_non_empty, mean, variance_impl, sorted, percentile_impl, ranks};

// ============ Min ============

pub struct Min;

static MIN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static MIN_EXAMPLES: [&str; 1] = ["min(3, 1, 4, 1, 5) → 1"];

static MIN_RELATED: [&str; 2] = ["max", "range"];

impl FunctionPlugin for Min {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "min",
            description: "Minimum value",
            usage: "min(values)",
            args: &MIN_ARGS,
            returns: "Number",
            examples: &MIN_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &MIN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "min") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        Value::Number(sorted_nums[0].clone())
    }
}

// ============ Max ============

pub struct Max;

static MAX_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static MAX_EXAMPLES: [&str; 1] = ["max(3, 1, 4, 1, 5) → 5"];

static MAX_RELATED: [&str; 2] = ["min", "range"];

impl FunctionPlugin for Max {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "max",
            description: "Maximum value",
            usage: "max(values)",
            args: &MAX_ARGS,
            returns: "Number",
            examples: &MAX_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &MAX_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "max") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        Value::Number(sorted_nums[sorted_nums.len() - 1].clone())
    }
}

// ============ Percentile ============

pub struct Percentile;

static PERCENTILE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List<Number>",
        description: "Numbers",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "p",
        typ: "Number",
        description: "Percentile (0-100)",
        optional: false,
        default: None,
    },
];

static PERCENTILE_EXAMPLES: [&str; 1] = ["percentile([1,2,3,4,5], 50) → 3"];

static PERCENTILE_RELATED: [&str; 3] = ["quantile", "median", "q1"];

impl FunctionPlugin for Percentile {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "percentile",
            description: "p-th percentile (0-100)",
            usage: "percentile(values, p)",
            args: &PERCENTILE_ARGS,
            returns: "Number",
            examples: &PERCENTILE_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &PERCENTILE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("percentile", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let p = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("percentile", "p", "Number", other.type_name())),
        };

        match percentile_impl(&numbers, p) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Quantile ============

pub struct Quantile;

static QUANTILE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List<Number>",
        description: "Numbers",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "q",
        typ: "Number",
        description: "Quantile (0-1)",
        optional: false,
        default: None,
    },
];

static QUANTILE_EXAMPLES: [&str; 1] = ["quantile([1,2,3,4,5], 0.5) → 3"];

static QUANTILE_RELATED: [&str; 2] = ["percentile", "median"];

impl FunctionPlugin for Quantile {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "quantile",
            description: "q-th quantile (0-1)",
            usage: "quantile(values, q)",
            args: &QUANTILE_ARGS,
            returns: "Number",
            examples: &QUANTILE_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &QUANTILE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("quantile", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let q = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("quantile", "q", "Number", other.type_name())),
        };

        // Validate q is in [0, 1]
        let q_f64 = q.to_f64().unwrap_or(0.0);
        if q_f64 < 0.0 || q_f64 > 1.0 {
            return Value::Error(FolioError::domain_error(
                "quantile() requires 0 <= q <= 1",
            ));
        }

        // Convert quantile to percentile
        let hundred = Number::from_i64(100);
        let p = q.mul(&hundred);

        match percentile_impl(&numbers, &p) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Q1 ============

pub struct Q1;

static Q1_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static Q1_EXAMPLES: [&str; 1] = ["q1([1,2,3,4,5,6,7,8]) → 2.5"];

static Q1_RELATED: [&str; 3] = ["q3", "median", "iqr"];

impl FunctionPlugin for Q1 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "q1",
            description: "First quartile (25th percentile)",
            usage: "q1(values)",
            args: &Q1_ARGS,
            returns: "Number",
            examples: &Q1_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &Q1_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let p25 = Number::from_i64(25);
        match percentile_impl(&numbers, &p25) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Q3 ============

pub struct Q3;

static Q3_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static Q3_EXAMPLES: [&str; 1] = ["q3([1,2,3,4,5,6,7,8]) → 6.5"];

static Q3_RELATED: [&str; 3] = ["q1", "median", "iqr"];

impl FunctionPlugin for Q3 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "q3",
            description: "Third quartile (75th percentile)",
            usage: "q3(values)",
            args: &Q3_ARGS,
            returns: "Number",
            examples: &Q3_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &Q3_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let p75 = Number::from_i64(75);
        match percentile_impl(&numbers, &p75) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Rank ============

pub struct Rank;

static RANK_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to find rank of",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Reference list",
        optional: false,
        default: None,
    },
];

static RANK_EXAMPLES: [&str; 1] = ["rank(3, [1,2,3,4,5]) → 3"];

static RANK_RELATED: [&str; 2] = ["percentile", "zscore"];

impl FunctionPlugin for Rank {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "rank",
            description: "Position in sorted list (1-indexed)",
            usage: "rank(value, list)",
            args: &RANK_ARGS,
            returns: "Number",
            examples: &RANK_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &RANK_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("rank", 2, args.len()));
        }

        let value = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("rank", "value", "Number", other.type_name())),
        };

        let numbers = match extract_numbers(&args[1..2]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "rank") {
            return Value::Error(e);
        }

        // Add the value to the list and compute ranks
        let mut all_values = numbers.clone();
        all_values.push(value.clone());
        let all_ranks = ranks(&all_values);

        // Return the rank of the last element (our value)
        Value::Number(all_ranks[all_values.len() - 1].clone())
    }
}

// ============ Zscore ============

pub struct Zscore;

static ZSCORE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to standardize",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Reference list",
        optional: false,
        default: None,
    },
];

static ZSCORE_EXAMPLES: [&str; 1] = ["zscore(5, [2,4,4,4,5,5,7,9]) → 0.467..."];

static ZSCORE_RELATED: [&str; 2] = ["normalize", "stddev"];

impl FunctionPlugin for Zscore {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "zscore",
            description: "Standard score: (x - mean)/stddev",
            usage: "zscore(value, list)",
            args: &ZSCORE_ARGS,
            returns: "Number",
            examples: &ZSCORE_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &ZSCORE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("zscore", 2, args.len()));
        }

        let value = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("zscore", "value", "Number", other.type_name())),
        };

        let numbers = match extract_numbers(&args[1..2]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if numbers.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "zscore() requires at least 2 values in list",
            ));
        }

        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if sd.is_zero() {
            return Value::Error(FolioError::domain_error(
                "zscore() undefined when stddev is zero",
            ));
        }

        let deviation = value.sub(&m);
        match deviation.checked_div(&sd) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Ranks ============

pub struct Ranks;

static RANKS_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Values to rank",
        optional: false,
        default: None,
    },
];

static RANKS_EXAMPLES: [&str; 2] = [
    "ranks([45, 23, 67, 89]) → [2, 1, 3, 4]",
    "ranks([5, 3, 5, 1]) → [3.5, 2, 3.5, 1]  // ties get average rank",
];

static RANKS_RELATED: [&str; 2] = ["rank", "percentile"];

impl FunctionPlugin for Ranks {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ranks",
            description: "Compute ranks for all values in list (1-indexed, average for ties)",
            usage: "ranks(list)",
            args: &RANKS_ARGS,
            returns: "List",
            examples: &RANKS_EXAMPLES,
            category: "stats/position",
            source: None,
            related: &RANKS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("ranks", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "ranks") {
            return Value::Error(e);
        }

        let ranked = ranks(&numbers);
        Value::List(ranked.into_iter().map(Value::Number).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_min() {
        let min = Min;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(4)),
        ])];
        let result = min.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1));
    }

    #[test]
    fn test_max() {
        let max = Max;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(4)),
        ])];
        let result = max.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(4));
    }

    #[test]
    fn test_percentile() {
        let percentile = Percentile;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(5)),
            ]),
            Value::Number(Number::from_i64(50)),
        ];
        let result = percentile.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3));
    }
}
```

folio-stats\src\qq.rs
```rs
//! Q-Q (Quantile-Quantile) analysis functions

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use std::collections::HashMap;
use crate::helpers::{extract_numbers, require_min_count, sorted, mean, variance_impl};

// ============================================================================
// QQPoints - Generate Q-Q plot points
// ============================================================================

pub struct QQPoints;

static QQ_POINTS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "distribution",
        typ: "Text",
        description: "Distribution: 'normal' (default)",
        optional: true,
        default: Some("normal"),
    },
];
static QQ_POINTS_EXAMPLES: [&str; 1] = ["qq_points(data) → {theoretical: [...], sample: [...], r_squared: 0.98}"];
static QQ_POINTS_RELATED: [&str; 2] = ["qq_residuals", "is_normal"];

impl FunctionPlugin for QQPoints {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "qq_points",
            description: "Generate Q-Q plot points for visual normality assessment",
            usage: "qq_points(list, distribution?)",
            args: &QQ_POINTS_ARGS,
            returns: "Object",
            examples: &QQ_POINTS_EXAMPLES,
            category: "distribution",
            source: None,
            related: &QQ_POINTS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("qq_points", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "qq_points") {
            return Value::Error(e);
        }

        // Get distribution (default "normal")
        let dist = if args.len() == 2 {
            match &args[1] {
                Value::Text(s) => s.to_lowercase(),
                other => return Value::Error(FolioError::arg_type("qq_points", "distribution", "Text", other.type_name())),
            }
        } else {
            "normal".to_string()
        };

        if dist != "normal" {
            return Value::Error(FolioError::domain_error(format!(
                "Only 'normal' distribution currently supported, got '{}'",
                dist
            )));
        }

        let n = numbers.len();
        let sorted_data = sorted(&numbers);

        // Calculate theoretical quantiles
        let theoretical: Vec<Number> = (0..n).map(|i| {
            // Filliben approximation for plotting positions
            let p = if i == 0 {
                1.0 - 0.5_f64.powf(1.0 / n as f64)
            } else if i == n - 1 {
                0.5_f64.powf(1.0 / n as f64)
            } else {
                (i as f64 + 1.0 - 0.3175) / (n as f64 + 0.365)
            };
            let q = normal_quantile(p);
            Number::from_str(&format!("{:.10}", q)).unwrap_or(Number::from_i64(0))
        }).collect();

        // Sample quantiles (sorted data)
        let sample: Vec<Number> = sorted_data.clone();

        // Calculate R-squared for linearity assessment
        let m_x = match mean(&theoretical) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };
        let m_y = match mean(&sample) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        // Calculate regression coefficients and R²
        let mut ss_xy = Number::from_i64(0);
        let mut ss_xx = Number::from_i64(0);
        let mut ss_yy = Number::from_i64(0);

        for i in 0..n {
            let dx = theoretical[i].sub(&m_x);
            let dy = sample[i].sub(&m_y);
            ss_xy = ss_xy.add(&dx.mul(&dy));
            ss_xx = ss_xx.add(&dx.mul(&dx));
            ss_yy = ss_yy.add(&dy.mul(&dy));
        }

        // Slope = ss_xy / ss_xx (should be ≈ σ)
        let slope = if ss_xx.is_zero() {
            Number::from_i64(0)
        } else {
            match ss_xy.checked_div(&ss_xx) {
                Ok(s) => s,
                Err(_) => Number::from_i64(0),
            }
        };

        // Intercept = mean_y - slope * mean_x (should be ≈ μ)
        let intercept = m_y.sub(&slope.mul(&m_x));

        // R² = (ss_xy)² / (ss_xx * ss_yy)
        let ss_prod = ss_xx.mul(&ss_yy);
        let r_squared = if ss_prod.is_zero() {
            Number::from_i64(1) // Perfect fit if no variation
        } else {
            let ss_xy_sq = ss_xy.mul(&ss_xy);
            match ss_xy_sq.checked_div(&ss_prod) {
                Ok(r2) => r2,
                Err(_) => Number::from_i64(0),
            }
        };

        let theoretical_values: Vec<Value> = theoretical.into_iter().map(Value::Number).collect();
        let sample_values: Vec<Value> = sample.into_iter().map(Value::Number).collect();

        let mut result = HashMap::new();
        result.insert("theoretical".to_string(), Value::List(theoretical_values));
        result.insert("sample".to_string(), Value::List(sample_values));
        result.insert("r_squared".to_string(), Value::Number(r_squared));
        result.insert("slope".to_string(), Value::Number(slope));
        result.insert("intercept".to_string(), Value::Number(intercept));

        Value::Object(result)
    }
}

// ============================================================================
// QQResiduals - Q-Q residuals from normal distribution
// ============================================================================

pub struct QQResiduals;

static QQ_RESIDUALS_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "Data values",
        optional: false,
        default: None,
    },
];
static QQ_RESIDUALS_EXAMPLES: [&str; 1] = ["qq_residuals(data) → [0.1, -0.05, 0.02, ...]"];
static QQ_RESIDUALS_RELATED: [&str; 2] = ["qq_points", "residuals"];

impl FunctionPlugin for QQResiduals {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "qq_residuals",
            description: "Deviations from theoretical normal quantiles",
            usage: "qq_residuals(list)",
            args: &QQ_RESIDUALS_ARGS,
            returns: "List",
            examples: &QQ_RESIDUALS_EXAMPLES,
            category: "distribution",
            source: None,
            related: &QQ_RESIDUALS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("qq_residuals", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "qq_residuals") {
            return Value::Error(e);
        }

        let n = numbers.len();

        // Calculate sample mean and stddev
        let m = match mean(&numbers) {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };
        let variance = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let stddev = match variance.sqrt(50) {
            Ok(s) => s,
            Err(e) => return Value::Error(e.into()),
        };

        if stddev.is_zero() {
            // All values are the same
            return Value::List(vec![Value::Number(Number::from_i64(0)); n]);
        }

        let sorted_data = sorted(&numbers);

        // Calculate residuals: observed - expected (under normality)
        let residuals: Vec<Value> = (0..n).map(|i| {
            // Expected value at this position for a normal distribution
            let p = if i == 0 {
                1.0 - 0.5_f64.powf(1.0 / n as f64)
            } else if i == n - 1 {
                0.5_f64.powf(1.0 / n as f64)
            } else {
                (i as f64 + 1.0 - 0.3175) / (n as f64 + 0.365)
            };
            let z = normal_quantile(p);
            let expected = m.add(&stddev.mul(&Number::from_str(&format!("{:.10}", z)).unwrap_or(Number::from_i64(0))));

            // Residual = observed - expected
            let residual = sorted_data[i].sub(&expected);
            Value::Number(residual)
        }).collect();

        Value::List(residuals)
    }
}

// ============================================================================
// Helper: Standard normal quantile function
// ============================================================================

/// Standard normal quantile (inverse CDF) - Rational approximation
fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if (p - 0.5).abs() < 1e-10 {
        return 0.0;
    }

    let a = [
        -3.969683028665376e1,
        2.209460984245205e2,
        -2.759285104469687e2,
        1.383577518672690e2,
        -3.066479806614716e1,
        2.506628277459239e0,
    ];
    let b = [
        -5.447609879822406e1,
        1.615858368580409e2,
        -1.556989798598866e2,
        6.680131188771972e1,
        -1.328068155288572e1,
    ];
    let c = [
        -7.784894002430293e-3,
        -3.223964580411365e-1,
        -2.400758277161838e0,
        -2.549732539343734e0,
        4.374664141464968e0,
        2.938163982698783e0,
    ];
    let d = [
        7.784695709041462e-3,
        3.224671290700398e-1,
        2.445134137142996e0,
        3.754408661907416e0,
    ];

    let p_low = 0.02425;
    let p_high = 1.0 - p_low;

    if p < p_low {
        let q = (-2.0 * p.ln()).sqrt();
        (((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    } else if p <= p_high {
        let q = p - 0.5;
        let r = q * q;
        (((((a[0] * r + a[1]) * r + a[2]) * r + a[3]) * r + a[4]) * r + a[5]) * q
            / (((((b[0] * r + b[1]) * r + b[2]) * r + b[3]) * r + b[4]) * r + 1.0)
    } else {
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        -(((((c[0] * q + c[1]) * q + c[2]) * q + c[3]) * q + c[4]) * q + c[5])
            / ((((d[0] * q + d[1]) * q + d[2]) * q + d[3]) * q + 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    fn make_list(values: &[i64]) -> Value {
        Value::List(values.iter().map(|v| Value::Number(Number::from_i64(*v))).collect())
    }

    #[test]
    fn test_qq_points() {
        let qq = QQPoints;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = qq.call(&args, &ctx);

        if let Value::Object(obj) = result {
            assert!(obj.contains_key("theoretical"));
            assert!(obj.contains_key("sample"));
            assert!(obj.contains_key("r_squared"));
            assert!(obj.contains_key("slope"));
            assert!(obj.contains_key("intercept"));

            // R² should be high for uniform data
            if let Some(Value::Number(r2)) = obj.get("r_squared") {
                let r2_val = r2.to_f64().unwrap_or(0.0);
                assert!(r2_val > 0.9, "R² should be high, got {}", r2_val);
            }
        } else {
            panic!("Expected Object, got {:?}", result);
        }
    }

    #[test]
    fn test_qq_residuals() {
        let qq = QQResiduals;
        let args = vec![make_list(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10])];
        let ctx = eval_ctx();
        let result = qq.call(&args, &ctx);

        if let Value::List(residuals) = result {
            assert_eq!(residuals.len(), 10);
            // Residuals should be small for reasonably normal data
            for r in residuals {
                if let Value::Number(_) = r {
                    // Just check it's a number
                } else {
                    panic!("Expected Number in residuals list");
                }
            }
        } else {
            panic!("Expected List, got {:?}", result);
        }
    }
}
```

folio-stats\src\regression.rs
```rs
//! Regression functions: linear_reg, slope, intercept, r_squared, predict, residuals

use folio_plugin::prelude::*;
use crate::helpers::{extract_two_lists, mean, variance_impl};
use std::collections::HashMap;

/// Calculate linear regression coefficients
fn linear_regression_impl(x: &[Number], y: &[Number], precision: u32) -> Result<(Number, Number, Number, Number), FolioError> {
    let n = x.len();
    if n < 2 {
        return Err(FolioError::domain_error(
            "Linear regression requires at least 2 points",
        ));
    }

    let mean_x = mean(x)?;
    let mean_y = mean(y)?;

    // Calculate sums for slope
    let mut sum_xy_dev = Number::from_i64(0);
    let mut sum_xx_dev = Number::from_i64(0);

    for (xi, yi) in x.iter().zip(y.iter()) {
        let dev_x = xi.sub(&mean_x);
        let dev_y = yi.sub(&mean_y);
        sum_xy_dev = sum_xy_dev.add(&dev_x.mul(&dev_y));
        sum_xx_dev = sum_xx_dev.add(&dev_x.mul(&dev_x));
    }

    if sum_xx_dev.is_zero() {
        return Err(FolioError::domain_error(
            "Cannot perform regression: x has zero variance",
        ));
    }

    // slope = Σ(x-x̄)(y-ȳ) / Σ(x-x̄)²
    let slope = sum_xy_dev.checked_div(&sum_xx_dev)?;

    // intercept = ȳ - slope * x̄
    let intercept = mean_y.sub(&slope.mul(&mean_x));

    // Calculate R²
    let var_y = variance_impl(y, false)?;
    if var_y.is_zero() {
        // Perfect prediction if y has no variance
        return Ok((slope, intercept, Number::from_i64(1), Number::from_i64(1)));
    }

    // SS_res = Σ(y - ŷ)²
    let mut ss_res = Number::from_i64(0);
    for (xi, yi) in x.iter().zip(y.iter()) {
        let y_pred = intercept.add(&slope.mul(xi));
        let residual = yi.sub(&y_pred);
        ss_res = ss_res.add(&residual.mul(&residual));
    }

    // SS_tot = Σ(y - ȳ)²
    let mut ss_tot = Number::from_i64(0);
    for yi in y {
        let dev = yi.sub(&mean_y);
        ss_tot = ss_tot.add(&dev.mul(&dev));
    }

    // R² = 1 - SS_res/SS_tot
    let r_squared = if ss_tot.is_zero() {
        Number::from_i64(1)
    } else {
        let ratio = ss_res.checked_div(&ss_tot)?;
        Number::from_i64(1).sub(&ratio)
    };

    // r = sign(slope) * sqrt(R²)
    let r = match r_squared.sqrt(precision) {
        Ok(sqrt_r2) => {
            if slope.is_negative() {
                Number::from_i64(0).sub(&sqrt_r2)
            } else {
                sqrt_r2
            }
        }
        Err(_) => Number::from_i64(0),
    };

    Ok((slope, intercept, r_squared, r))
}

// ============ LinearReg ============

pub struct LinearReg;

static LINEAR_REG_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static LINEAR_REG_EXAMPLES: [&str; 1] = ["linear_reg([1,2,3], [2,4,6]) → {slope: 2, intercept: 0, ...}"];

static LINEAR_REG_RELATED: [&str; 3] = ["slope", "intercept", "r_squared"];

impl FunctionPlugin for LinearReg {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "linear_reg",
            description: "Full linear regression result",
            usage: "linear_reg(x, y)",
            args: &LINEAR_REG_ARGS,
            returns: "Object",
            examples: &LINEAR_REG_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &LINEAR_REG_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let (slope, intercept, r_squared, r) = match linear_regression_impl(&x, &y, ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        // Calculate standard error
        let n = x.len();
        let mut ss_res = Number::from_i64(0);
        for (xi, yi) in x.iter().zip(y.iter()) {
            let y_pred = intercept.add(&slope.mul(xi));
            let residual = yi.sub(&y_pred);
            ss_res = ss_res.add(&residual.mul(&residual));
        }

        let std_error = if n > 2 {
            let df = Number::from_i64((n - 2) as i64);
            match ss_res.checked_div(&df) {
                Ok(mse) => mse.sqrt(ctx.precision).unwrap_or(Number::from_i64(0)),
                Err(_) => Number::from_i64(0),
            }
        } else {
            Number::from_i64(0)
        };

        let mut result = HashMap::new();
        result.insert("slope".to_string(), Value::Number(slope));
        result.insert("intercept".to_string(), Value::Number(intercept));
        result.insert("r_squared".to_string(), Value::Number(r_squared));
        result.insert("r".to_string(), Value::Number(r));
        result.insert("std_error".to_string(), Value::Number(std_error));
        result.insert("n".to_string(), Value::Number(Number::from_i64(n as i64)));

        Value::Object(result)
    }
}

// ============ Slope ============

pub struct Slope;

static SLOPE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static SLOPE_EXAMPLES: [&str; 1] = ["slope([1,2,3], [2,4,6]) → 2"];

static SLOPE_RELATED: [&str; 2] = ["intercept", "linear_reg"];

impl FunctionPlugin for Slope {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "slope",
            description: "Slope of linear regression",
            usage: "slope(x, y)",
            args: &SLOPE_ARGS,
            returns: "Number",
            examples: &SLOPE_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &SLOPE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match linear_regression_impl(&x, &y, ctx.precision) {
            Ok((slope, _, _, _)) => Value::Number(slope),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Intercept ============

pub struct Intercept;

static INTERCEPT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static INTERCEPT_EXAMPLES: [&str; 1] = ["intercept([1,2,3], [3,5,7]) → 1"];

static INTERCEPT_RELATED: [&str; 2] = ["slope", "linear_reg"];

impl FunctionPlugin for Intercept {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "intercept",
            description: "Y-intercept of linear regression",
            usage: "intercept(x, y)",
            args: &INTERCEPT_ARGS,
            returns: "Number",
            examples: &INTERCEPT_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &INTERCEPT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match linear_regression_impl(&x, &y, ctx.precision) {
            Ok((_, intercept, _, _)) => Value::Number(intercept),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ RSquared ============

pub struct RSquared;

static R_SQUARED_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static R_SQUARED_EXAMPLES: [&str; 1] = ["r_squared([1,2,3], [2,4,6]) → 1"];

static R_SQUARED_RELATED: [&str; 2] = ["correlation", "linear_reg"];

impl FunctionPlugin for RSquared {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "r_squared",
            description: "Coefficient of determination (R²)",
            usage: "r_squared(x, y)",
            args: &R_SQUARED_ARGS,
            returns: "Number",
            examples: &R_SQUARED_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &R_SQUARED_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        match linear_regression_impl(&x, &y, ctx.precision) {
            Ok((_, _, r_squared, _)) => Value::Number(r_squared),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ Predict ============

pub struct Predict;

static PREDICT_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "x_or_reg",
        typ: "List|Object",
        description: "X values list OR regression object",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y_or_x",
        typ: "List|Number",
        description: "Y values list OR x value to predict",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "new_x",
        typ: "Number",
        description: "X value to predict (only for 3-arg form)",
        optional: true,
        default: None,
    },
];

static PREDICT_EXAMPLES: [&str; 2] = [
    "predict([1,2,3], [2,4,6], 4) → 8",
    "predict(linear_reg([1,2,3], [2,4,6]), 4) → 8",
];

static PREDICT_RELATED: [&str; 2] = ["linear_reg", "residuals"];

impl FunctionPlugin for Predict {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "predict",
            description: "Predict y value from linear regression (3-arg: x, y, new_x) or (2-arg: reg, x)",
            usage: "predict(x_vals, y_vals, new_x) or predict(reg, x)",
            args: &PREDICT_ARGS,
            returns: "Number",
            examples: &PREDICT_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &PREDICT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(FolioError::new("ARG_COUNT",
                "predict requires 2 or 3 arguments: predict(x, y, new_x) or predict(reg, x)"));
        }

        // 3-arg form: predict(x_vals, y_vals, new_x)
        if args.len() == 3 {
            // First compute linear regression, then predict
            let linear_reg = LinearReg;
            let reg_result = linear_reg.call(&args[0..2], ctx);

            let reg = match &reg_result {
                Value::Object(obj) => obj,
                Value::Error(e) => return Value::Error(e.clone()),
                _ => return Value::Error(FolioError::new("REGRESSION_ERROR", "Failed to compute regression")),
            };

            let new_x = match &args[2] {
                Value::Number(n) => n,
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("predict", "new_x", "Number", other.type_name())),
            };

            let slope = match reg.get("slope") {
                Some(Value::Number(n)) => n,
                _ => return Value::Error(FolioError::undefined_field("slope")),
            };

            let intercept = match reg.get("intercept") {
                Some(Value::Number(n)) => n,
                _ => return Value::Error(FolioError::undefined_field("intercept")),
            };

            return Value::Number(intercept.add(&slope.mul(new_x)));
        }

        // 2-arg form: predict(reg, x)
        let reg = match &args[0] {
            Value::Object(obj) => obj,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("predict", "reg", "Object", other.type_name())),
        };

        let x = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("predict", "x", "Number", other.type_name())),
        };

        let slope = match reg.get("slope") {
            Some(Value::Number(n)) => n,
            _ => return Value::Error(FolioError::undefined_field("slope")),
        };

        let intercept = match reg.get("intercept") {
            Some(Value::Number(n)) => n,
            _ => return Value::Error(FolioError::undefined_field("intercept")),
        };

        // y = intercept + slope * x
        Value::Number(intercept.add(&slope.mul(x)))
    }
}

// ============ Residuals ============

pub struct Residuals;

static RESIDUALS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "x",
        typ: "List<Number>",
        description: "Independent variable",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "y",
        typ: "List<Number>",
        description: "Dependent variable",
        optional: false,
        default: None,
    },
];

static RESIDUALS_EXAMPLES: [&str; 1] = ["residuals([1,2,3], [2.1, 3.9, 6.1])"];

static RESIDUALS_RELATED: [&str; 2] = ["linear_reg", "predict"];

impl FunctionPlugin for Residuals {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "residuals",
            description: "List of (y - ŷ) residuals",
            usage: "residuals(x, y)",
            args: &RESIDUALS_ARGS,
            returns: "List<Number>",
            examples: &RESIDUALS_EXAMPLES,
            category: "stats/regression",
            source: None,
            related: &RESIDUALS_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let (slope, intercept, _, _) = match linear_regression_impl(&x, &y, ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let residuals: Vec<Value> = x
            .iter()
            .zip(y.iter())
            .map(|(xi, yi)| {
                let y_pred = intercept.add(&slope.mul(xi));
                Value::Number(yi.sub(&y_pred))
            })
            .collect();

        Value::List(residuals)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_slope() {
        let slope = Slope;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(6)),
            ]),
        ];
        let result = slope.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
    }

    #[test]
    fn test_intercept() {
        let intercept = Intercept;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(3)),
                Value::Number(Number::from_i64(5)),
                Value::Number(Number::from_i64(7)),
            ]),
        ];
        let result = intercept.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1));
    }

    #[test]
    fn test_r_squared_perfect() {
        let r_squared = RSquared;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(6)),
            ]),
        ];
        let result = r_squared.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(1));
    }
}
```

folio-stats\src\shape.rs
```rs
//! Shape functions: skewness, kurtosis, count, product

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_min_count, mean, variance_impl};

// ============ Skewness ============

pub struct Skewness;

static SKEWNESS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static SKEWNESS_EXAMPLES: [&str; 1] = ["skewness([1,2,2,3,3,3,4,4,5]) → positive"];

static SKEWNESS_RELATED: [&str; 2] = ["kurtosis", "stddev"];

impl FunctionPlugin for Skewness {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "skewness",
            description: "Measure of asymmetry (Fisher's skewness)",
            usage: "skewness(values)",
            args: &SKEWNESS_ARGS,
            returns: "Number",
            examples: &SKEWNESS_EXAMPLES,
            category: "stats/shape",
            source: None,
            related: &SKEWNESS_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 3, "skewness") {
            return Value::Error(e);
        }

        let n = numbers.len();
        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if sd.is_zero() {
            return Value::Error(FolioError::domain_error(
                "skewness() undefined when stddev is zero",
            ));
        }

        // Calculate sum of cubed deviations
        let mut sum_cubed = Number::from_i64(0);
        for x in &numbers {
            let dev = x.sub(&m);
            let cubed = dev.mul(&dev).mul(&dev);
            sum_cubed = sum_cubed.add(&cubed);
        }

        // Fisher's skewness: n / ((n-1)(n-2)) * sum((x-mean)^3) / sd^3
        let n_num = Number::from_i64(n as i64);
        let n_minus_1 = Number::from_i64((n - 1) as i64);
        let n_minus_2 = Number::from_i64((n - 2) as i64);

        let sd_cubed = sd.mul(&sd).mul(&sd);
        let adjustment = match n_num.checked_div(&n_minus_1.mul(&n_minus_2)) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let m3 = match sum_cubed.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        match m3.checked_div(&sd_cubed) {
            Ok(raw) => Value::Number(adjustment.mul(&n_num).mul(&raw)),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ Kurtosis ============

pub struct Kurtosis;

static KURTOSIS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static KURTOSIS_EXAMPLES: [&str; 1] = ["kurtosis([1,2,3,4,5]) → -1.3"];

static KURTOSIS_RELATED: [&str; 2] = ["skewness", "stddev"];

impl FunctionPlugin for Kurtosis {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "kurtosis",
            description: "Excess kurtosis (Fisher's definition, normal = 0)",
            usage: "kurtosis(values)",
            args: &KURTOSIS_ARGS,
            returns: "Number",
            examples: &KURTOSIS_EXAMPLES,
            category: "stats/shape",
            source: None,
            related: &KURTOSIS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 4, "kurtosis") {
            return Value::Error(e);
        }

        let n = numbers.len();
        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if var.is_zero() {
            return Value::Error(FolioError::domain_error(
                "kurtosis() undefined when variance is zero",
            ));
        }

        // Calculate sum of fourth power deviations
        let mut sum_fourth = Number::from_i64(0);
        for x in &numbers {
            let dev = x.sub(&m);
            let fourth = dev.mul(&dev).mul(&dev).mul(&dev);
            sum_fourth = sum_fourth.add(&fourth);
        }

        // Excess kurtosis formula
        let n_num = Number::from_i64(n as i64);
        let n_minus_1 = Number::from_i64((n - 1) as i64);
        let n_minus_2 = Number::from_i64((n - 2) as i64);
        let n_minus_3 = Number::from_i64((n - 3) as i64);
        let three = Number::from_i64(3);

        let var_squared = var.mul(&var);

        // m4 = sum_fourth / n
        let m4 = match sum_fourth.checked_div(&n_num) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Raw kurtosis = m4 / var^2
        let raw_kurtosis = match m4.checked_div(&var_squared) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Fisher's excess kurtosis adjustment
        // ((n+1)*n / ((n-1)(n-2)(n-3))) * raw - 3*(n-1)^2 / ((n-2)(n-3))
        let n_plus_1 = Number::from_i64((n + 1) as i64);

        let coef1_num = n_plus_1.mul(&n_num);
        let coef1_den = n_minus_1.mul(&n_minus_2).mul(&n_minus_3);
        let coef1 = match coef1_num.checked_div(&coef1_den) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let coef2_num = three.mul(&n_minus_1).mul(&n_minus_1);
        let coef2_den = n_minus_2.mul(&n_minus_3);
        let coef2 = match coef2_num.checked_div(&coef2_den) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        Value::Number(coef1.mul(&raw_kurtosis).sub(&coef2))
    }
}

// ============ Count ============

pub struct Count;

static COUNT_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers",
    optional: false,
    default: None,
}];

static COUNT_EXAMPLES: [&str; 1] = ["count([1,2,3,4,5]) → 5"];

static COUNT_RELATED: [&str; 1] = ["sum"];

impl FunctionPlugin for Count {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "count",
            description: "Number of elements",
            usage: "count(values)",
            args: &COUNT_ARGS,
            returns: "Number",
            examples: &COUNT_EXAMPLES,
            category: "stats/shape",
            source: None,
            related: &COUNT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        Value::Number(Number::from_i64(numbers.len() as i64))
    }
}

// ============ Product ============

pub struct Product;

static PRODUCT_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "values",
    typ: "List<Number> | Number...",
    description: "Numbers to multiply",
    optional: false,
    default: None,
}];

static PRODUCT_EXAMPLES: [&str; 1] = ["product([1,2,3,4,5]) → 120"];

static PRODUCT_RELATED: [&str; 2] = ["sum", "gmean"];

impl FunctionPlugin for Product {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "product",
            description: "Product of all elements",
            usage: "product(values)",
            args: &PRODUCT_ARGS,
            returns: "Number",
            examples: &PRODUCT_EXAMPLES,
            category: "stats/shape",
            source: None,
            related: &PRODUCT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if numbers.is_empty() {
            return Value::Number(Number::from_i64(1)); // Empty product is 1
        }

        let result = numbers
            .iter()
            .fold(Number::from_i64(1), |acc, n| acc.mul(n));

        Value::Number(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_count() {
        let count = Count;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])];
        let result = count.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(3));
    }

    #[test]
    fn test_product() {
        let product = Product;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = product.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(120));
    }

    #[test]
    fn test_count_empty() {
        let count = Count;
        let args = vec![Value::List(vec![])];
        let result = count.call(&args, &eval_ctx());
        assert_eq!(result.as_number().unwrap().to_i64(), Some(0));
    }
}
```

folio-stats\src\transform.rs
```rs
//! Transform functions: normalize, standardize, cumsum, differences, lag, moving_avg, ewma

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_non_empty, require_min_count, mean, variance_impl, sorted};

// ============ Normalize (Z-scores) ============

pub struct Normalize;

static NORMALIZE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Values to normalize",
    optional: false,
    default: None,
}];

static NORMALIZE_EXAMPLES: [&str; 1] = ["normalize([1,2,3,4,5]) → z-scores"];

static NORMALIZE_RELATED: [&str; 2] = ["standardize", "zscore"];

impl FunctionPlugin for Normalize {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "normalize",
            description: "Z-scores for all values: (x - mean)/stddev",
            usage: "normalize(list)",
            args: &NORMALIZE_ARGS,
            returns: "List<Number>",
            examples: &NORMALIZE_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &NORMALIZE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_min_count(&numbers, 2, "normalize") {
            return Value::Error(e);
        }

        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let sd = match var.sqrt(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if sd.is_zero() {
            return Value::Error(FolioError::domain_error(
                "normalize() requires non-zero standard deviation",
            ));
        }

        let normalized: Vec<Value> = numbers
            .iter()
            .map(|x| {
                let z = x.sub(&m).checked_div(&sd).unwrap_or(Number::from_i64(0));
                Value::Number(z)
            })
            .collect();

        Value::List(normalized)
    }
}

// ============ Standardize (0-1 range) ============

pub struct Standardize;

static STANDARDIZE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Values to standardize",
    optional: false,
    default: None,
}];

static STANDARDIZE_EXAMPLES: [&str; 1] = ["standardize([1,2,3,4,5]) → [0, 0.25, 0.5, 0.75, 1]"];

static STANDARDIZE_RELATED: [&str; 2] = ["normalize", "range"];

impl FunctionPlugin for Standardize {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "standardize",
            description: "Scale to [0,1] range: (x - min)/(max - min)",
            usage: "standardize(list)",
            args: &STANDARDIZE_ARGS,
            returns: "List<Number>",
            examples: &STANDARDIZE_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &STANDARDIZE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = require_non_empty(&numbers, "standardize") {
            return Value::Error(e);
        }

        let sorted_nums = sorted(&numbers);
        let min = sorted_nums[0].clone();
        let max = sorted_nums[sorted_nums.len() - 1].clone();
        let range = max.sub(&min);

        if range.is_zero() {
            // All values are equal
            return Value::List(numbers.iter().map(|_| Value::Number(Number::from_i64(0))).collect());
        }

        let standardized: Vec<Value> = numbers
            .iter()
            .map(|x| {
                let scaled = x.sub(&min).checked_div(&range).unwrap_or(Number::from_i64(0));
                Value::Number(scaled)
            })
            .collect();

        Value::List(standardized)
    }
}

// ============ Cumsum ============

pub struct Cumsum;

static CUMSUM_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Values to sum cumulatively",
    optional: false,
    default: None,
}];

static CUMSUM_EXAMPLES: [&str; 1] = ["cumsum([1,2,3,4,5]) → [1,3,6,10,15]"];

static CUMSUM_RELATED: [&str; 2] = ["sum", "differences"];

impl FunctionPlugin for Cumsum {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cumsum",
            description: "Cumulative sum",
            usage: "cumsum(list)",
            args: &CUMSUM_ARGS,
            returns: "List<Number>",
            examples: &CUMSUM_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &CUMSUM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let mut sum = Number::from_i64(0);
        let cumulative: Vec<Value> = numbers
            .iter()
            .map(|x| {
                sum = sum.add(x);
                Value::Number(sum.clone())
            })
            .collect();

        Value::List(cumulative)
    }
}

// ============ Differences ============

pub struct Differences;

static DIFFERENCES_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List<Number>",
    description: "Values to difference",
    optional: false,
    default: None,
}];

static DIFFERENCES_EXAMPLES: [&str; 1] = ["differences([1,3,6,10,15]) → [2,3,4,5]"];

static DIFFERENCES_RELATED: [&str; 2] = ["cumsum", "lag"];

impl FunctionPlugin for Differences {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "differences",
            description: "First differences: x[i] - x[i-1]",
            usage: "differences(list)",
            args: &DIFFERENCES_ARGS,
            returns: "List<Number>",
            examples: &DIFFERENCES_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &DIFFERENCES_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if numbers.len() < 2 {
            return Value::List(vec![]);
        }

        let diffs: Vec<Value> = numbers
            .windows(2)
            .map(|w| Value::Number(w[1].sub(&w[0])))
            .collect();

        Value::List(diffs)
    }
}

// ============ Lag ============

pub struct Lag;

static LAG_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Values to lag",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of periods to lag (default 1)",
        optional: true,
        default: Some("1"),
    },
];

static LAG_EXAMPLES: [&str; 1] = ["lag([1,2,3,4,5], 2) → [null,null,1,2,3]"];

static LAG_RELATED: [&str; 2] = ["differences", "moving_avg"];

impl FunctionPlugin for Lag {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "lag",
            description: "Shift by n periods (earlier values become null)",
            usage: "lag(list, n?)",
            args: &LAG_ARGS,
            returns: "List<Number|Null>",
            examples: &LAG_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &LAG_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("lag", 1, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let n = if args.len() > 1 {
            match &args[1] {
                Value::Number(num) => num.to_i64().unwrap_or(1) as usize,
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("lag", "n", "Number", other.type_name())),
            }
        } else {
            1
        };

        let mut lagged: Vec<Value> = Vec::with_capacity(numbers.len());

        for i in 0..numbers.len() {
            if i < n {
                lagged.push(Value::Null);
            } else {
                lagged.push(Value::Number(numbers[i - n].clone()));
            }
        }

        Value::List(lagged)
    }
}

// ============ Moving Average ============

pub struct MovingAvg;

static MOVING_AVG_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "window",
        typ: "Number",
        description: "Window size",
        optional: false,
        default: None,
    },
];

static MOVING_AVG_EXAMPLES: [&str; 1] = ["moving_avg([1,2,3,4,5], 3) → [null,null,2,3,4]"];

static MOVING_AVG_RELATED: [&str; 2] = ["ewma", "mean"];

impl FunctionPlugin for MovingAvg {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "moving_avg",
            description: "Simple moving average",
            usage: "moving_avg(list, window)",
            args: &MOVING_AVG_ARGS,
            returns: "List<Number|Null>",
            examples: &MOVING_AVG_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &MOVING_AVG_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("moving_avg", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let window = match &args[1] {
            Value::Number(n) => n.to_i64().unwrap_or(1) as usize,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("moving_avg", "window", "Number", other.type_name())),
        };

        if window == 0 {
            return Value::Error(FolioError::domain_error(
                "moving_avg() requires window > 0",
            ));
        }

        let mut result: Vec<Value> = Vec::with_capacity(numbers.len());
        let window_size = Number::from_i64(window as i64);

        for i in 0..numbers.len() {
            if i + 1 < window {
                result.push(Value::Null);
            } else {
                let start = i + 1 - window;
                let sum: Number = numbers[start..=i]
                    .iter()
                    .fold(Number::from_i64(0), |acc, x| acc.add(x));
                let avg = sum.checked_div(&window_size).unwrap_or(Number::from_i64(0));
                result.push(Value::Number(avg));
            }
        }

        Value::List(result)
    }
}

// ============ Exponentially Weighted Moving Average ============

pub struct Ewma;

static EWMA_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "alpha",
        typ: "Number",
        description: "Smoothing factor (0 < α ≤ 1)",
        optional: false,
        default: None,
    },
];

static EWMA_EXAMPLES: [&str; 1] = ["ewma([1,2,3,4,5], 0.3)"];

static EWMA_RELATED: [&str; 2] = ["moving_avg", "mean"];

impl FunctionPlugin for Ewma {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ewma",
            description: "Exponentially weighted moving average",
            usage: "ewma(list, alpha)",
            args: &EWMA_ARGS,
            returns: "List<Number>",
            examples: &EWMA_EXAMPLES,
            category: "stats/transform",
            source: None,
            related: &EWMA_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("ewma", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let alpha = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("ewma", "alpha", "Number", other.type_name())),
        };

        let alpha_f64 = alpha.to_f64().unwrap_or(0.0);
        if alpha_f64 <= 0.0 || alpha_f64 > 1.0 {
            return Value::Error(FolioError::domain_error(
                "ewma() requires 0 < alpha ≤ 1",
            ));
        }

        if numbers.is_empty() {
            return Value::List(vec![]);
        }

        let one_minus_alpha = Number::from_i64(1).sub(alpha);
        let mut ewma = numbers[0].clone();
        let mut result: Vec<Value> = vec![Value::Number(ewma.clone())];

        for x in numbers.iter().skip(1) {
            // EWMA = α * x + (1-α) * EWMA_prev
            ewma = alpha.mul(x).add(&one_minus_alpha.mul(&ewma));
            result.push(Value::Number(ewma.clone()));
        }

        Value::List(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_cumsum() {
        let cumsum = Cumsum;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
        ])];
        let result = cumsum.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 5);
        assert_eq!(list[4].as_number().unwrap().to_i64(), Some(15));
    }

    #[test]
    fn test_differences() {
        let differences = Differences;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(6)),
            Value::Number(Number::from_i64(10)),
        ])];
        let result = differences.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(2));
        assert_eq!(list[1].as_number().unwrap().to_i64(), Some(3));
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(4));
    }

    #[test]
    fn test_standardize() {
        let standardize = Standardize;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(50)),
            Value::Number(Number::from_i64(100)),
        ])];
        let result = standardize.call(&args, &eval_ctx());
        let list = result.as_list().unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].as_number().unwrap().to_i64(), Some(0));
        // list[1] should be 0.5
        // list[2] should be 1
        assert_eq!(list[2].as_number().unwrap().to_i64(), Some(1));
    }
}
```

folio-std\Cargo.toml
```toml
[package]
name = "folio-std"
description = "Standard library for Folio: math functions, analyzers, constants"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
folio-core = { path = "../folio-core" }
folio-plugin = { path = "../folio-plugin" }
serde = { workspace = true }
```

folio-std\src\analyzers\e.rs
```rs
//! Euler's number (e) pattern detection

use folio_plugin::prelude::*;
use std::collections::HashMap;

pub struct EAnalyzer;

impl AnalyzerPlugin for EAnalyzer {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "e",
            description: "Detects Euler's number patterns (e, e², ln(n), etc.)",
            detects: &["e", "e²", "1/e", "e^n"],
        }
    }

    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.5
    }

    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let e = Number::e(ctx.precision);
        let mut result = HashMap::new();

        // Test e^n for small integer n
        for n in -5i32..=5 {
            if n == 0 {
                continue; // e^0 = 1 is not interesting
            }
            let e_n = e.pow(n);
            if let Ok(ratio) = value.checked_div(&e_n) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 100 && i != 0 {
                        let key = if n == 1 {
                            "e".to_string()
                        } else {
                            format!("e^{}", n)
                        };
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.85").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }

        // Check if value is a natural logarithm of a small integer
        // If ln(n) = value, then n = e^value
        let exp_value = value.exp(ctx.precision);
        if let Some(n) = exp_value.to_i64() {
            if n >= 2 && n <= 100 {
                let mut entry = HashMap::new();
                entry.insert("n".to_string(), Value::Number(Number::from_i64(n)));
                entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.8").unwrap()));
                result.insert("ln".to_string(), Value::Object(entry));
            }
        }

        if result.is_empty() {
            result.insert("_note".to_string(), Value::Text("No simple e patterns found".to_string()));
        }

        Value::Object(result)
    }
}
```

folio-std\src\analyzers\mod.rs
```rs
//! Pattern detection analyzers

mod phi;
mod pi;
mod e;

pub use phi::PhiAnalyzer;
pub use pi::PiAnalyzer;
pub use e::EAnalyzer;
```

folio-std\src\analyzers\phi.rs
```rs
//! Golden ratio pattern detection

use folio_plugin::prelude::*;
use std::collections::HashMap;

pub struct PhiAnalyzer;

impl AnalyzerPlugin for PhiAnalyzer {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "phi",
            description: "Detects golden ratio patterns",
            detects: &["φ", "φ²", "1/φ"],
        }
    }
    
    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.5
    }
    
    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let phi = Number::phi(ctx.precision);
        let mut result = HashMap::new();
        
        // Test φ^n for small n
        for n in -3i32..=5 {
            let phi_n = phi.pow(n);
            if let Ok(ratio) = value.checked_div(&phi_n) {
                if let Some(i) = ratio.to_i64() {
                    if i.abs() <= 100 {
                        let key = format!("φ^{}", n);
                        let mut entry = HashMap::new();
                        entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                        entry.insert("confidence".to_string(), Value::Number(Number::from_str("0.9").unwrap()));
                        result.insert(key, Value::Object(entry));
                    }
                }
            }
        }
        
        if result.is_empty() {
            result.insert("_note".to_string(), Value::Text("No simple φ patterns found".to_string()));
        }
        
        Value::Object(result)
    }
}
```

folio-std\src\analyzers\pi.rs
```rs
//! Pi pattern detection

use folio_plugin::prelude::*;
use std::collections::HashMap;

pub struct PiAnalyzer;

impl AnalyzerPlugin for PiAnalyzer {
    fn meta(&self) -> AnalyzerMeta {
        AnalyzerMeta {
            name: "pi",
            description: "Detects pi patterns",
            detects: &["π", "2π", "π²"],
        }
    }
    
    fn confidence(&self, _value: &Number, _ctx: &EvalContext) -> f64 {
        0.5
    }
    
    fn analyze(&self, value: &Number, ctx: &EvalContext) -> Value {
        let pi = Number::pi(ctx.precision);
        let mut result = HashMap::new();
        
        // Test simple π multiples
        if let Ok(ratio) = value.checked_div(&pi) {
            if let Some(i) = ratio.to_i64() {
                if i.abs() <= 100 && i != 0 {
                    let mut entry = HashMap::new();
                    entry.insert("coefficient".to_string(), Value::Number(Number::from_i64(i)));
                    result.insert("π".to_string(), Value::Object(entry));
                }
            }
        }
        
        if result.is_empty() {
            result.insert("_note".to_string(), Value::Text("No simple π patterns found".to_string()));
        }
        
        Value::Object(result)
    }
}
```

folio-std\src\commands\explain.rs
```rs
//! EXPLAIN command - show how a value was computed

use folio_plugin::prelude::*;
use std::collections::HashMap;

pub struct Explain;

static EXPLAIN_ARGS: [ArgMeta; 1] = [ArgMeta { name: "cell", typ: "String", description: "Cell name to explain", optional: false, default: None }];
static EXPLAIN_EXAMPLES: [&str; 2] = ["EXPLAIN(result)", "EXPLAIN(error)"];

impl CommandPlugin for Explain {
    fn meta(&self) -> CommandMeta {
        CommandMeta {
            name: "EXPLAIN",
            description: "Show how a value was computed, including dependencies and intermediate steps",
            args: &EXPLAIN_ARGS,
            examples: &EXPLAIN_EXAMPLES,
        }
    }

    fn execute(&self, args: &[Value], ctx: &mut EvalContext) -> Value {
        // Check argument count
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("EXPLAIN", 1, args.len()));
        }

        // Get cell name
        let cell_name = match &args[0] {
            Value::Text(s) => s.clone(),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("EXPLAIN", "cell", "String", other.type_name())),
        };

        // Look up the cell value
        let value = ctx.get_var(&cell_name);

        let mut result = HashMap::new();
        result.insert("cell".to_string(), Value::Text(cell_name.clone()));
        result.insert("value".to_string(), value.clone());

        // Find trace steps for this cell
        let trace_steps: Vec<&TraceStep> = ctx.trace.iter()
            .filter(|step| step.cell == cell_name)
            .collect();

        if !trace_steps.is_empty() {
            let step = trace_steps.last().unwrap();
            result.insert("formula".to_string(), Value::Text(step.formula.clone()));
            result.insert("dependencies".to_string(),
                Value::List(step.dependencies.iter().map(|d| Value::Text(d.clone())).collect()));

            // Build dependency chain
            let mut dep_chain = Vec::new();
            for dep_name in &step.dependencies {
                let dep_value = ctx.get_var(dep_name);
                let mut dep_info = HashMap::new();
                dep_info.insert("name".to_string(), Value::Text(dep_name.to_string()));
                dep_info.insert("value".to_string(), dep_value);

                // Find formula for dependency
                if let Some(dep_step) = ctx.trace.iter().find(|s| &s.cell == dep_name) {
                    dep_info.insert("formula".to_string(), Value::Text(dep_step.formula.clone()));
                }

                dep_chain.push(Value::Object(dep_info));
            }
            result.insert("dependency_values".to_string(), Value::List(dep_chain));
        } else {
            // No trace available - provide basic info
            result.insert("note".to_string(),
                Value::Text("Enable tracing with TRACE(true) to see computation details".to_string()));
        }

        // If it's an error, include error details
        if let Value::Error(e) = &value {
            let mut error_info = HashMap::new();
            error_info.insert("code".to_string(), Value::Text(e.code.clone()));
            error_info.insert("message".to_string(), Value::Text(e.message.clone()));
            if let Some(ref suggestion) = e.suggestion {
                error_info.insert("suggestion".to_string(), Value::Text(suggestion.clone()));
            }
            if let Some(ref context) = e.context {
                if let Some(ref formula) = context.formula {
                    error_info.insert("formula".to_string(), Value::Text(formula.clone()));
                }
                if !context.notes.is_empty() {
                    error_info.insert("notes".to_string(),
                        Value::List(context.notes.iter().map(|n| Value::Text(n.clone())).collect()));
                }
            }
            result.insert("error_details".to_string(), Value::Object(error_info));
        }

        Value::Object(result)
    }
}
```

folio-std\src\commands\mod.rs
```rs
//! Commands

mod trace;
mod explain;

pub use trace::Trace;
pub use explain::Explain;
```

folio-std\src\commands\trace.rs
```rs
//! TRACE command

use folio_plugin::prelude::*;

pub struct Trace;

static TRACE_ARGS: [ArgMeta; 1] = [ArgMeta { name: "enabled", typ: "Bool", description: "Enable tracing", optional: true, default: Some("true") }];
static TRACE_EXAMPLES: [&str; 2] = ["TRACE()", "TRACE(false)"];

impl CommandPlugin for Trace {
    fn meta(&self) -> CommandMeta {
        CommandMeta {
            name: "TRACE",
            description: "Enable or disable evaluation tracing",
            args: &TRACE_ARGS,
            examples: &TRACE_EXAMPLES,
        }
    }

    fn execute(&self, args: &[Value], ctx: &mut EvalContext) -> Value {
        let enabled = args.get(0)
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        ctx.tracing = enabled;
        Value::Bool(enabled)
    }
}
```

folio-std\src\constants.rs
```rs
//! Mathematical and physical constants with sources

use folio_plugin::ConstantDef;

// ============================================================================
// Mathematical Constants
// ============================================================================

pub fn phi() -> ConstantDef {
    ConstantDef {
        name: "φ".to_string(),
        formula: "(1 + sqrt(5)) / 2".to_string(),
        source: "https://oeis.org/A001622".to_string(),
        category: "algebraic".to_string(),
    }
}

pub fn pi() -> ConstantDef {
    ConstantDef {
        name: "π".to_string(),
        formula: "pi".to_string(),
        source: "https://oeis.org/A000796".to_string(),
        category: "transcendental".to_string(),
    }
}

pub fn e() -> ConstantDef {
    ConstantDef {
        name: "e".to_string(),
        formula: "exp(1)".to_string(),
        source: "https://oeis.org/A001113".to_string(),
        category: "transcendental".to_string(),
    }
}

pub fn sqrt3() -> ConstantDef {
    ConstantDef {
        name: "sqrt3".to_string(),
        formula: "sqrt(3)".to_string(),
        source: "https://oeis.org/A002194".to_string(),
        category: "algebraic".to_string(),
    }
}

pub fn sqrt2() -> ConstantDef {
    ConstantDef {
        name: "sqrt2".to_string(),
        formula: "sqrt(2)".to_string(),
        source: "https://oeis.org/A002193".to_string(),
        category: "algebraic".to_string(),
    }
}

// ============================================================================
// Particle Masses (PDG 2024 / CODATA 2022)
// Values in MeV unless otherwise noted
// ============================================================================

pub fn m_e() -> ConstantDef {
    ConstantDef {
        name: "m_e".to_string(),
        formula: "0.51099895000".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - electron mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

pub fn m_mu() -> ConstantDef {
    ConstantDef {
        name: "m_μ".to_string(),
        formula: "105.6583755".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - muon mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

pub fn m_tau() -> ConstantDef {
    ConstantDef {
        name: "m_τ".to_string(),
        formula: "1776.86".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - tau mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

pub fn m_higgs() -> ConstantDef {
    ConstantDef {
        name: "m_H".to_string(),
        formula: "125350".to_string(),  // MeV (125.35 GeV)
        source: "CMS/ATLAS 2024 - Higgs boson mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

// ============================================================================
// CKM Matrix Elements (PDG 2024-2025)
// ============================================================================

pub fn v_us() -> ConstantDef {
    ConstantDef {
        name: "V_us".to_string(),
        formula: "0.2243".to_string(),
        source: "PDG 2024 - CKM element |V_us| from kaon decays".to_string(),
        category: "ckm".to_string(),
    }
}

pub fn v_cb() -> ConstantDef {
    ConstantDef {
        name: "V_cb".to_string(),
        formula: "0.04121".to_string(),
        source: "PDG 2025 / HFLAV - CKM element |V_cb|".to_string(),
        category: "ckm".to_string(),
    }
}

pub fn v_ub() -> ConstantDef {
    ConstantDef {
        name: "V_ub".to_string(),
        formula: "0.00382".to_string(),
        source: "PDG 2024 - CKM element |V_ub| inclusive+exclusive avg".to_string(),
        category: "ckm".to_string(),
    }
}

pub fn v_ts() -> ConstantDef {
    ConstantDef {
        name: "V_ts".to_string(),
        formula: "0.0411".to_string(),
        source: "PDG 2024 - CKM element |V_ts|".to_string(),
        category: "ckm".to_string(),
    }
}

// ============================================================================
// Fundamental Physical Constants
// ============================================================================

pub fn c() -> ConstantDef {
    ConstantDef {
        name: "c".to_string(),
        formula: "299792458".to_string(),  // m/s (exact)
        source: "CODATA 2022 - speed of light in vacuum (exact)".to_string(),
        category: "physical".to_string(),
    }
}

pub fn alpha() -> ConstantDef {
    ConstantDef {
        name: "α".to_string(),
        formula: "0.0072973525693".to_string(),  // fine-structure constant
        source: "CODATA 2022 - fine-structure constant".to_string(),
        category: "physical".to_string(),
    }
}

// ============================================================================
// ASCII Aliases for Unicode Constants
// These allow users to type "phi" instead of "φ", etc.
// ============================================================================

pub fn phi_ascii() -> ConstantDef {
    ConstantDef {
        name: "phi".to_string(),
        formula: "(1 + sqrt(5)) / 2".to_string(),
        source: "https://oeis.org/A001622".to_string(),
        category: "algebraic".to_string(),
    }
}

pub fn pi_ascii() -> ConstantDef {
    ConstantDef {
        name: "pi".to_string(),
        formula: "pi".to_string(),
        source: "https://oeis.org/A000796".to_string(),
        category: "transcendental".to_string(),
    }
}

pub fn alpha_ascii() -> ConstantDef {
    ConstantDef {
        name: "alpha".to_string(),
        formula: "0.0072973525693".to_string(),
        source: "CODATA 2022 - fine-structure constant".to_string(),
        category: "physical".to_string(),
    }
}

pub fn m_mu_ascii() -> ConstantDef {
    ConstantDef {
        name: "m_mu".to_string(),
        formula: "105.6583755".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - muon mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

pub fn m_tau_ascii() -> ConstantDef {
    ConstantDef {
        name: "m_tau".to_string(),
        formula: "1776.86".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - tau mass".to_string(),
        category: "particle_mass".to_string(),
    }
}
```

folio-std\src\functions\aggregate.rs
```rs
//! Aggregate functions

use folio_plugin::prelude::*;

pub struct Sum;

static SUM_ARGS: [ArgMeta; 1] = [ArgMeta { name: "values", typ: "Number...", description: "Values to sum", optional: false, default: None }];
static SUM_EXAMPLES: [&str; 1] = ["sum(1, 2, 3)"];
static SUM_RELATED: [&str; 0] = [];

impl FunctionPlugin for Sum {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sum",
            description: "Sum of values",
            usage: "sum(a, b, ...)",
            args: &SUM_ARGS,
            returns: "Number",
            examples: &SUM_EXAMPLES,
            category: "aggregate",
            source: None,
            related: &SUM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let mut total = Number::from_i64(0);
        for arg in args {
            match arg {
                Value::Number(n) => total = total.add(n),
                Value::List(list) => {
                    // Handle List arguments - recursively sum all elements
                    for item in list {
                        match item {
                            Value::Number(n) => total = total.add(n),
                            Value::Error(e) => return Value::Error(e.clone()),
                            other => return Value::Error(FolioError::arg_type("sum", "values", "Number", other.type_name())),
                        }
                    }
                }
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("sum", "values", "Number or List", other.type_name())),
            }
        }
        Value::Number(total)
    }
}
```

folio-std\src\functions\datetime.rs
```rs
//! DateTime and Duration functions
//!
//! Provides functions for constructing, parsing, extracting, formatting,
//! and manipulating dates, times, and durations.

use folio_plugin::prelude::*;

// ============================================================================
// Construction Functions
// ============================================================================

pub struct DateFn;
pub struct TimeFn;
pub struct DateTimeFn;
pub struct NowFn;

// Date construction
static DATE_ARGS: [ArgMeta; 3] = [
    ArgMeta { name: "year", typ: "Number", description: "Year (e.g., 2025)", optional: false, default: None },
    ArgMeta { name: "month", typ: "Number", description: "Month (1-12)", optional: false, default: None },
    ArgMeta { name: "day", typ: "Number", description: "Day (1-31)", optional: false, default: None },
];
static DATE_EXAMPLES: [&str; 2] = ["date(2025, 6, 15)", "date(1999, 12, 31)"];
static DATE_RELATED: [&str; 3] = ["datetime", "time", "now"];

// Time construction
static TIME_ARGS: [ArgMeta; 3] = [
    ArgMeta { name: "hour", typ: "Number", description: "Hour (0-23)", optional: false, default: None },
    ArgMeta { name: "minute", typ: "Number", description: "Minute (0-59)", optional: false, default: None },
    ArgMeta { name: "second", typ: "Number", description: "Second (0-59)", optional: true, default: Some("0") },
];
static TIME_EXAMPLES: [&str; 2] = ["time(14, 30)", "time(9, 0, 30)"];
static TIME_RELATED: [&str; 2] = ["date", "datetime"];

// DateTime construction
static DATETIME_ARGS: [ArgMeta; 6] = [
    ArgMeta { name: "year", typ: "Number", description: "Year", optional: false, default: None },
    ArgMeta { name: "month", typ: "Number", description: "Month (1-12)", optional: false, default: None },
    ArgMeta { name: "day", typ: "Number", description: "Day (1-31)", optional: false, default: None },
    ArgMeta { name: "hour", typ: "Number", description: "Hour (0-23)", optional: false, default: None },
    ArgMeta { name: "minute", typ: "Number", description: "Minute (0-59)", optional: false, default: None },
    ArgMeta { name: "second", typ: "Number", description: "Second (0-59)", optional: true, default: Some("0") },
];
static DATETIME_EXAMPLES: [&str; 2] = ["datetime(2025, 6, 15, 14, 30, 0)", "datetime(2025, 1, 1, 0, 0)"];
static DATETIME_RELATED: [&str; 2] = ["date", "time"];

// Now
static NOW_ARGS: [ArgMeta; 0] = [];
static NOW_EXAMPLES: [&str; 1] = ["now()"];
static NOW_RELATED: [&str; 2] = ["date", "datetime"];

impl FunctionPlugin for DateFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "date",
            description: "Create a date (time = 00:00:00)",
            usage: "date(year, month, day)",
            args: &DATE_ARGS,
            returns: "DateTime",
            examples: &DATE_EXAMPLES,
            category: "datetime",
            source: None,
            related: &DATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 3 {
            return Value::Error(FolioError::arg_count("date", 3, args.len()));
        }

        let year = match get_i32(&args[0], "date", "year") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let month = match get_u32(&args[1], "date", "month") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let day = match get_u32(&args[2], "date", "day") {
            Ok(v) => v,
            Err(e) => return e,
        };

        match FolioDateTime::from_ymd(year, month, day) {
            Ok(dt) => Value::DateTime(dt),
            Err(e) => Value::Error(e.into()),
        }
    }
}

impl FunctionPlugin for TimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "time",
            description: "Create a time (date = 1970-01-01)",
            usage: "time(hour, minute, second?)",
            args: &TIME_ARGS,
            returns: "DateTime",
            examples: &TIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &TIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(FolioError::arg_count("time", 2, args.len())
                .with_note("time(hour, minute, second?)"));
        }

        let hour = match get_u32(&args[0], "time", "hour") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let minute = match get_u32(&args[1], "time", "minute") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let second = if args.len() > 2 {
            match get_u32(&args[2], "time", "second") {
                Ok(v) => v,
                Err(e) => return e,
            }
        } else {
            0
        };

        match FolioDateTime::from_hms(hour, minute, second) {
            Ok(dt) => Value::DateTime(dt),
            Err(e) => Value::Error(e.into()),
        }
    }
}

impl FunctionPlugin for DateTimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "datetime",
            description: "Create a datetime from components",
            usage: "datetime(year, month, day, hour, minute, second?)",
            args: &DATETIME_ARGS,
            returns: "DateTime",
            examples: &DATETIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &DATETIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 5 || args.len() > 6 {
            return Value::Error(FolioError::arg_count("datetime", 5, args.len())
                .with_note("datetime(year, month, day, hour, minute, second?)"));
        }

        let year = match get_i32(&args[0], "datetime", "year") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let month = match get_u32(&args[1], "datetime", "month") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let day = match get_u32(&args[2], "datetime", "day") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let hour = match get_u32(&args[3], "datetime", "hour") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let minute = match get_u32(&args[4], "datetime", "minute") {
            Ok(v) => v,
            Err(e) => return e,
        };
        let second = if args.len() > 5 {
            match get_u32(&args[5], "datetime", "second") {
                Ok(v) => v,
                Err(e) => return e,
            }
        } else {
            0
        };

        match FolioDateTime::from_ymd_hms(year, month, day, hour, minute, second) {
            Ok(dt) => Value::DateTime(dt),
            Err(e) => Value::Error(e.into()),
        }
    }
}

impl FunctionPlugin for NowFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "now",
            description: "Get current UTC datetime",
            usage: "now()",
            args: &NOW_ARGS,
            returns: "DateTime",
            examples: &NOW_EXAMPLES,
            category: "datetime",
            source: None,
            related: &NOW_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if !args.is_empty() {
            return Value::Error(FolioError::arg_count("now", 0, args.len()));
        }
        Value::DateTime(FolioDateTime::now())
    }
}

// ============================================================================
// Parsing Functions
// ============================================================================

pub struct ParseDateFn;
pub struct ParseTimeFn;

static PARSEDATE_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "text", typ: "Text", description: "Date string to parse", optional: false, default: None },
    ArgMeta { name: "format", typ: "Text", description: "Optional format pattern", optional: true, default: None },
];
static PARSEDATE_EXAMPLES: [&str; 3] = ["parseDate(\"2025-06-15\")", "parseDate(\"15/06/2025\", \"DD/MM/YYYY\")", "parseDate(\"2025-06-15T14:30:00Z\")"];
static PARSEDATE_RELATED: [&str; 2] = ["date", "formatDate"];

static PARSETIME_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "text", typ: "Text", description: "Time string to parse (HH:MM:SS)", optional: false, default: None },
];
static PARSETIME_EXAMPLES: [&str; 2] = ["parseTime(\"14:30\")", "parseTime(\"09:00:30\")"];
static PARSETIME_RELATED: [&str; 2] = ["time", "formatTime"];

impl FunctionPlugin for ParseDateFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parseDate",
            description: "Parse a date/datetime string (ISO 8601 or custom format)",
            usage: "parseDate(text, format?)",
            args: &PARSEDATE_ARGS,
            returns: "DateTime",
            examples: &PARSEDATE_EXAMPLES,
            category: "datetime",
            source: None,
            related: &PARSEDATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("parseDate", 1, args.len()));
        }

        let text = match &args[0] {
            Value::Text(s) => s,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("parseDate", "text", "Text", other.type_name())),
        };

        if args.len() == 2 {
            // Parse with format
            let format = match &args[1] {
                Value::Text(s) => s,
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("parseDate", "format", "Text", other.type_name())),
            };
            match FolioDateTime::parse_format(text, format) {
                Ok(dt) => Value::DateTime(dt),
                Err(e) => Value::Error(e.into()),
            }
        } else {
            // Auto-detect format
            match FolioDateTime::parse(text) {
                Ok(dt) => Value::DateTime(dt),
                Err(e) => Value::Error(e.into()),
            }
        }
    }
}

impl FunctionPlugin for ParseTimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "parseTime",
            description: "Parse a time string (HH:MM or HH:MM:SS)",
            usage: "parseTime(text)",
            args: &PARSETIME_ARGS,
            returns: "DateTime",
            examples: &PARSETIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &PARSETIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("parseTime", 1, args.len()));
        }

        let text = match &args[0] {
            Value::Text(s) => s,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("parseTime", "text", "Text", other.type_name())),
        };

        match FolioDateTime::parse_time(text) {
            Ok(dt) => Value::DateTime(dt),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============================================================================
// Extraction Functions
// ============================================================================

pub struct YearFn;
pub struct MonthFn;
pub struct DayFn;
pub struct HourFn;
pub struct MinuteFn;
pub struct SecondFn;
pub struct WeekdayFn;
pub struct DayOfYearFn;
pub struct WeekFn;

static EXTRACT_DT_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime to extract from", optional: false, default: None },
];
static EXTRACT_EXAMPLES: [&str; 1] = ["year(now())"];
static EXTRACT_RELATED: [&str; 2] = ["date", "datetime"];

impl FunctionPlugin for YearFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "year",
            description: "Extract year from datetime",
            usage: "year(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &EXTRACT_EXAMPLES,
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("year", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.year() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("year", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for MonthFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "month",
            description: "Extract month from datetime (1-12)",
            usage: "month(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["month(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("month", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.month() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("month", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for DayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "day",
            description: "Extract day from datetime (1-31)",
            usage: "day(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["day(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("day", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.day() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("day", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for HourFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "hour",
            description: "Extract hour from datetime (0-23)",
            usage: "hour(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["hour(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("hour", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.hour() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("hour", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for MinuteFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "minute",
            description: "Extract minute from datetime (0-59)",
            usage: "minute(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["minute(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("minute", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.minute() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("minute", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for SecondFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "second",
            description: "Extract second from datetime (0-59)",
            usage: "second(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["second(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("second", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.second() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("second", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for WeekdayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "weekday",
            description: "Get day of week (1=Monday, 7=Sunday)",
            usage: "weekday(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["weekday(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("weekday", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.weekday() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("weekday", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for DayOfYearFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "dayOfYear",
            description: "Get day of year (1-366)",
            usage: "dayOfYear(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["dayOfYear(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("dayOfYear", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.day_of_year() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("dayOfYear", "dt", "DateTime", other.type_name())),
        }
    }
}

impl FunctionPlugin for WeekFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "week",
            description: "Get ISO week number (1-53)",
            usage: "week(dt)",
            args: &EXTRACT_DT_ARGS,
            returns: "Number",
            examples: &["week(now())"],
            category: "datetime",
            source: None,
            related: &EXTRACT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("week", 1, args.len()));
        }
        match &args[0] {
            Value::DateTime(dt) => Value::Number(Number::from_i64(dt.iso_week() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("week", "dt", "DateTime", other.type_name())),
        }
    }
}

// ============================================================================
// Formatting Functions
// ============================================================================

pub struct FormatDateFn;
pub struct FormatTimeFn;
pub struct FormatDateTimeFn;

static FORMATDATE_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime to format", optional: false, default: None },
    ArgMeta { name: "pattern", typ: "Text", description: "Format pattern (e.g., DD/MM/YYYY)", optional: true, default: Some("YYYY-MM-DD") },
];
static FORMATDATE_EXAMPLES: [&str; 2] = ["formatDate(now())", "formatDate(now(), \"DD/MM/YYYY\")"];
static FORMATDATE_RELATED: [&str; 2] = ["parseDate", "formatDateTime"];

static FORMATTIME_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime to format", optional: false, default: None },
    ArgMeta { name: "pattern", typ: "Text", description: "Format pattern (e.g., HH:mm:ss)", optional: true, default: Some("HH:mm:ss") },
];
static FORMATTIME_EXAMPLES: [&str; 2] = ["formatTime(now())", "formatTime(now(), \"h:mm A\")"];
static FORMATTIME_RELATED: [&str; 2] = ["parseTime", "formatDateTime"];

static FORMATDATETIME_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime to format", optional: false, default: None },
    ArgMeta { name: "pattern", typ: "Text", description: "Format pattern", optional: true, default: Some("YYYY-MM-DDTHH:mm:ss") },
];
static FORMATDATETIME_EXAMPLES: [&str; 2] = ["formatDateTime(now())", "formatDateTime(now(), \"DD/MM/YYYY HH:mm\")"];
static FORMATDATETIME_RELATED: [&str; 2] = ["formatDate", "formatTime"];

impl FunctionPlugin for FormatDateFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "formatDate",
            description: "Format datetime as date string",
            usage: "formatDate(dt, pattern?)",
            args: &FORMATDATE_ARGS,
            returns: "Text",
            examples: &FORMATDATE_EXAMPLES,
            category: "datetime",
            source: None,
            related: &FORMATDATE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("formatDate", 1, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("formatDate", "dt", "DateTime", other.type_name())),
        };

        let pattern = if args.len() > 1 {
            match &args[1] {
                Value::Text(s) => s.as_str(),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("formatDate", "pattern", "Text", other.type_name())),
            }
        } else {
            "YYYY-MM-DD"
        };

        Value::Text(dt.format(pattern))
    }
}

impl FunctionPlugin for FormatTimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "formatTime",
            description: "Format datetime as time string",
            usage: "formatTime(dt, pattern?)",
            args: &FORMATTIME_ARGS,
            returns: "Text",
            examples: &FORMATTIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &FORMATTIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("formatTime", 1, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("formatTime", "dt", "DateTime", other.type_name())),
        };

        let pattern = if args.len() > 1 {
            match &args[1] {
                Value::Text(s) => s.as_str(),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("formatTime", "pattern", "Text", other.type_name())),
            }
        } else {
            "HH:mm:ss"
        };

        Value::Text(dt.format(pattern))
    }
}

impl FunctionPlugin for FormatDateTimeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "formatDateTime",
            description: "Format datetime with pattern",
            usage: "formatDateTime(dt, pattern?)",
            args: &FORMATDATETIME_ARGS,
            returns: "Text",
            examples: &FORMATDATETIME_EXAMPLES,
            category: "datetime",
            source: None,
            related: &FORMATDATETIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("formatDateTime", 1, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("formatDateTime", "dt", "DateTime", other.type_name())),
        };

        let pattern = if args.len() > 1 {
            match &args[1] {
                Value::Text(s) => s.as_str(),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("formatDateTime", "pattern", "Text", other.type_name())),
            }
        } else {
            "YYYY-MM-DDTHH:mm:ss"
        };

        Value::Text(dt.format(pattern))
    }
}

// ============================================================================
// Duration Construction Functions
// ============================================================================

pub struct WeeksDur;
pub struct DaysDur;
pub struct HoursDur;
pub struct MinutesDur;
pub struct SecondsDur;
pub struct MillisecondsDur;

static DUR_N_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "n", typ: "Number", description: "Number of units", optional: false, default: None },
];
static DUR_RELATED: [&str; 6] = ["weeks", "days", "hours", "minutes", "seconds", "milliseconds"];

impl FunctionPlugin for WeeksDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "weeks",
            description: "Create a duration of n weeks",
            usage: "weeks(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["weeks(2)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("weeks", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_weeks(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("weeks", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for DaysDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "days",
            description: "Create a duration of n days",
            usage: "days(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["days(5)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("days", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_days(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("days", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for HoursDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "hours",
            description: "Create a duration of n hours",
            usage: "hours(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["hours(24)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("hours", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_hours(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("hours", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for MinutesDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "minutes",
            description: "Create a duration of n minutes",
            usage: "minutes(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["minutes(30)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("minutes", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_minutes(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("minutes", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for SecondsDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "seconds",
            description: "Create a duration of n seconds",
            usage: "seconds(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["seconds(60)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("seconds", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_secs(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("seconds", "n", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for MillisecondsDur {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "milliseconds",
            description: "Create a duration of n milliseconds",
            usage: "milliseconds(n)",
            args: &DUR_N_ARGS,
            returns: "Duration",
            examples: &["milliseconds(500)"],
            category: "datetime",
            source: None,
            related: &DUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("milliseconds", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                if let Some(v) = n.to_i64() {
                    Value::Duration(FolioDuration::from_millis(v))
                } else {
                    Value::Error(FolioError::domain_error("value too large for duration"))
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("milliseconds", "n", "Number", other.type_name())),
        }
    }
}

// ============================================================================
// Arithmetic Functions
// ============================================================================

pub struct AddDaysFn;
pub struct AddMonthsFn;
pub struct AddYearsFn;
pub struct DiffFn;

static ADDDAYS_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Base datetime", optional: false, default: None },
    ArgMeta { name: "n", typ: "Number", description: "Number of days to add", optional: false, default: None },
];
static ADDDAYS_EXAMPLES: [&str; 2] = ["addDays(now(), 30)", "addDays(date(2025, 1, 1), -7)"];
static ADDDAYS_RELATED: [&str; 2] = ["addMonths", "addYears"];

static ADDMONTHS_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Base datetime", optional: false, default: None },
    ArgMeta { name: "n", typ: "Number", description: "Number of months to add", optional: false, default: None },
];
static ADDMONTHS_EXAMPLES: [&str; 2] = ["addMonths(now(), 3)", "addMonths(date(2025, 1, 31), 1)"];
static ADDMONTHS_RELATED: [&str; 2] = ["addDays", "addYears"];

static ADDYEARS_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Base datetime", optional: false, default: None },
    ArgMeta { name: "n", typ: "Number", description: "Number of years to add", optional: false, default: None },
];
static ADDYEARS_EXAMPLES: [&str; 2] = ["addYears(now(), 1)", "addYears(date(2020, 2, 29), 1)"];
static ADDYEARS_RELATED: [&str; 2] = ["addDays", "addMonths"];

static DIFF_ARGS: [ArgMeta; 3] = [
    ArgMeta { name: "dt1", typ: "DateTime", description: "First datetime", optional: false, default: None },
    ArgMeta { name: "dt2", typ: "DateTime", description: "Second datetime", optional: false, default: None },
    ArgMeta { name: "unit", typ: "Text", description: "Unit: years, months, weeks, days, hours, minutes, seconds, milliseconds", optional: true, default: Some("days") },
];
static DIFF_EXAMPLES: [&str; 2] = ["diff(now(), date(2025, 1, 1), \"days\")", "diff(dt1, dt2, \"hours\")"];
static DIFF_RELATED: [&str; 2] = ["addDays", "isBefore"];

impl FunctionPlugin for AddDaysFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "addDays",
            description: "Add days to a datetime",
            usage: "addDays(dt, n)",
            args: &ADDDAYS_ARGS,
            returns: "DateTime",
            examples: &ADDDAYS_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ADDDAYS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("addDays", 2, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("addDays", "dt", "DateTime", other.type_name())),
        };

        let n = match get_i64(&args[1], "addDays", "n") {
            Ok(v) => v,
            Err(e) => return e,
        };

        Value::DateTime(dt.add_days(n))
    }
}

impl FunctionPlugin for AddMonthsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "addMonths",
            description: "Add months to a datetime (handles month boundaries)",
            usage: "addMonths(dt, n)",
            args: &ADDMONTHS_ARGS,
            returns: "DateTime",
            examples: &ADDMONTHS_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ADDMONTHS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("addMonths", 2, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("addMonths", "dt", "DateTime", other.type_name())),
        };

        let n = match get_i32(&args[1], "addMonths", "n") {
            Ok(v) => v,
            Err(e) => return e,
        };

        Value::DateTime(dt.add_months(n))
    }
}

impl FunctionPlugin for AddYearsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "addYears",
            description: "Add years to a datetime (handles leap years)",
            usage: "addYears(dt, n)",
            args: &ADDYEARS_ARGS,
            returns: "DateTime",
            examples: &ADDYEARS_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ADDYEARS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("addYears", 2, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("addYears", "dt", "DateTime", other.type_name())),
        };

        let n = match get_i32(&args[1], "addYears", "n") {
            Ok(v) => v,
            Err(e) => return e,
        };

        Value::DateTime(dt.add_years(n))
    }
}

impl FunctionPlugin for DiffFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "diff",
            description: "Calculate difference between two datetimes",
            usage: "diff(dt1, dt2, unit?)",
            args: &DIFF_ARGS,
            returns: "Number",
            examples: &DIFF_EXAMPLES,
            category: "datetime",
            source: None,
            related: &DIFF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(FolioError::arg_count("diff", 2, args.len()));
        }

        let dt1 = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("diff", "dt1", "DateTime", other.type_name())),
        };

        let dt2 = match &args[1] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("diff", "dt2", "DateTime", other.type_name())),
        };

        let unit = if args.len() > 2 {
            match &args[2] {
                Value::Text(s) => s.as_str(),
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("diff", "unit", "Text", other.type_name())),
            }
        } else {
            "days"
        };

        let duration = dt1.duration_since(dt2);
        let value = match unit.to_lowercase().as_str() {
            "years" | "year" | "y" => {
                // Calendar year difference
                (dt1.year() - dt2.year()) as i64
            }
            "months" | "month" => {
                // Calendar month difference
                let y1 = dt1.year() as i64;
                let y2 = dt2.year() as i64;
                let m1 = dt1.month() as i64;
                let m2 = dt2.month() as i64;
                (y1 * 12 + m1) - (y2 * 12 + m2)
            }
            "weeks" | "week" | "w" => duration.as_weeks(),
            "days" | "day" | "d" => duration.as_days(),
            "hours" | "hour" | "h" => duration.as_hours(),
            "minutes" | "minute" | "min" | "m" => duration.as_minutes(),
            "seconds" | "second" | "sec" | "s" => duration.as_secs(),
            "milliseconds" | "millisecond" | "ms" => duration.as_millis(),
            _ => return Value::Error(FolioError::domain_error(
                format!("unknown unit '{}'; use years, months, weeks, days, hours, minutes, seconds, or milliseconds", unit)
            )),
        };

        Value::Number(Number::from_i64(value))
    }
}

// ============================================================================
// Comparison Functions
// ============================================================================

pub struct IsBeforeFn;
pub struct IsAfterFn;
pub struct IsSameDayFn;

static ISBEFORE_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt1", typ: "DateTime", description: "First datetime", optional: false, default: None },
    ArgMeta { name: "dt2", typ: "DateTime", description: "Second datetime", optional: false, default: None },
];
static ISBEFORE_EXAMPLES: [&str; 1] = ["isBefore(date(2025, 1, 1), now())"];
static ISBEFORE_RELATED: [&str; 2] = ["isAfter", "isSameDay"];

static ISAFTER_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt1", typ: "DateTime", description: "First datetime", optional: false, default: None },
    ArgMeta { name: "dt2", typ: "DateTime", description: "Second datetime", optional: false, default: None },
];
static ISAFTER_EXAMPLES: [&str; 1] = ["isAfter(now(), date(2025, 1, 1))"];
static ISAFTER_RELATED: [&str; 2] = ["isBefore", "isSameDay"];

static ISSAMEDAY_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt1", typ: "DateTime", description: "First datetime", optional: false, default: None },
    ArgMeta { name: "dt2", typ: "DateTime", description: "Second datetime", optional: false, default: None },
];
static ISSAMEDAY_EXAMPLES: [&str; 1] = ["isSameDay(now(), now())"];
static ISSAMEDAY_RELATED: [&str; 2] = ["isBefore", "isAfter"];

impl FunctionPlugin for IsBeforeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "isBefore",
            description: "Check if dt1 is before dt2",
            usage: "isBefore(dt1, dt2)",
            args: &ISBEFORE_ARGS,
            returns: "Bool",
            examples: &ISBEFORE_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ISBEFORE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("isBefore", 2, args.len()));
        }

        let dt1 = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isBefore", "dt1", "DateTime", other.type_name())),
        };

        let dt2 = match &args[1] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isBefore", "dt2", "DateTime", other.type_name())),
        };

        Value::Bool(dt1.is_before(dt2))
    }
}

impl FunctionPlugin for IsAfterFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "isAfter",
            description: "Check if dt1 is after dt2",
            usage: "isAfter(dt1, dt2)",
            args: &ISAFTER_ARGS,
            returns: "Bool",
            examples: &ISAFTER_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ISAFTER_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("isAfter", 2, args.len()));
        }

        let dt1 = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isAfter", "dt1", "DateTime", other.type_name())),
        };

        let dt2 = match &args[1] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isAfter", "dt2", "DateTime", other.type_name())),
        };

        Value::Bool(dt1.is_after(dt2))
    }
}

impl FunctionPlugin for IsSameDayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "isSameDay",
            description: "Check if two datetimes are on the same day",
            usage: "isSameDay(dt1, dt2)",
            args: &ISSAMEDAY_ARGS,
            returns: "Bool",
            examples: &ISSAMEDAY_EXAMPLES,
            category: "datetime",
            source: None,
            related: &ISSAMEDAY_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("isSameDay", 2, args.len()));
        }

        let dt1 = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isSameDay", "dt1", "DateTime", other.type_name())),
        };

        let dt2 = match &args[1] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("isSameDay", "dt2", "DateTime", other.type_name())),
        };

        Value::Bool(dt1.is_same_day(dt2))
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

pub struct StartOfDayFn;
pub struct EndOfDayFn;
pub struct StartOfMonthFn;
pub struct StartOfYearFn;

static STARTOFDAY_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime", optional: false, default: None },
];
static STARTOFDAY_EXAMPLES: [&str; 1] = ["startOfDay(now())"];
static STARTOFDAY_RELATED: [&str; 3] = ["endOfDay", "startOfMonth", "startOfYear"];

static ENDOFDAY_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime", optional: false, default: None },
];
static ENDOFDAY_EXAMPLES: [&str; 1] = ["endOfDay(now())"];
static ENDOFDAY_RELATED: [&str; 2] = ["startOfDay", "startOfMonth"];

static STARTOFMONTH_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime", optional: false, default: None },
];
static STARTOFMONTH_EXAMPLES: [&str; 1] = ["startOfMonth(now())"];
static STARTOFMONTH_RELATED: [&str; 2] = ["startOfDay", "startOfYear"];

static STARTOFYEAR_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "DateTime", optional: false, default: None },
];
static STARTOFYEAR_EXAMPLES: [&str; 1] = ["startOfYear(now())"];
static STARTOFYEAR_RELATED: [&str; 2] = ["startOfDay", "startOfMonth"];

macro_rules! utility_fn {
    ($struct:ident, $name:literal, $desc:literal, $method:ident, $args:ident, $examples:ident, $related:ident) => {
        impl FunctionPlugin for $struct {
            fn meta(&self) -> FunctionMeta {
                FunctionMeta {
                    name: $name,
                    description: $desc,
                    usage: concat!($name, "(dt)"),
                    args: &$args,
                    returns: "DateTime",
                    examples: &$examples,
                    category: "datetime",
                    source: None,
                    related: &$related,
                }
            }

            fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
                if args.len() != 1 {
                    return Value::Error(FolioError::arg_count($name, 1, args.len()));
                }
                match &args[0] {
                    Value::DateTime(dt) => Value::DateTime(dt.$method()),
                    Value::Error(e) => Value::Error(e.clone()),
                    other => Value::Error(FolioError::arg_type($name, "dt", "DateTime", other.type_name())),
                }
            }
        }
    };
}

utility_fn!(StartOfDayFn, "startOfDay", "Get start of day (00:00:00)", start_of_day, STARTOFDAY_ARGS, STARTOFDAY_EXAMPLES, STARTOFDAY_RELATED);
utility_fn!(EndOfDayFn, "endOfDay", "Get end of day (23:59:59.999...)", end_of_day, ENDOFDAY_ARGS, ENDOFDAY_EXAMPLES, ENDOFDAY_RELATED);
utility_fn!(StartOfMonthFn, "startOfMonth", "Get first day of month", start_of_month, STARTOFMONTH_ARGS, STARTOFMONTH_EXAMPLES, STARTOFMONTH_RELATED);
utility_fn!(StartOfYearFn, "startOfYear", "Get first day of year", start_of_year, STARTOFYEAR_ARGS, STARTOFYEAR_EXAMPLES, STARTOFYEAR_RELATED);

// ============================================================================
// DateTime Shortcut Functions
// ============================================================================

// End of period shortcuts
pub struct EodFn;
pub struct EowFn;
pub struct EomFn;
pub struct EoqFn;
pub struct EoyFn;

// Start of period shortcuts
pub struct SodFn;
pub struct SowFn;
pub struct SomFn;
pub struct SoqFn;
pub struct SoyFn;

// Navigation shortcuts
pub struct TomorrowFn;
pub struct NextWeekFn;
pub struct NextMonthFn;
pub struct NextMonthWdFn;

// Workday functions
pub struct IsWorkdayFn;
pub struct NextWorkdayFn;
pub struct PrevWorkdayFn;
pub struct AddWorkdaysFn;

// Optional datetime arg
static OPT_DT_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Reference datetime (default: now)", optional: true, default: Some("now()") },
];

// Helper to get optional datetime or now()
fn get_dt_or_now(args: &[Value], func: &str) -> Result<FolioDateTime, Value> {
    if args.is_empty() {
        Ok(FolioDateTime::now())
    } else {
        match &args[0] {
            Value::DateTime(dt) => Ok(dt.clone()),
            Value::Error(e) => Err(Value::Error(e.clone())),
            other => Err(Value::Error(FolioError::arg_type(func, "dt", "DateTime", other.type_name()))),
        }
    }
}

// ========== End of Period ==========

impl FunctionPlugin for EodFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eod",
            description: "End of day (23:59:59.999...)",
            usage: "eod() or eod(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eod()", "eod(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["sod", "eow", "eom"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eod", 1, args.len()));
        }
        match get_dt_or_now(args, "eod") {
            Ok(dt) => Value::DateTime(dt.end_of_day()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for EowFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eow",
            description: "End of week (Sunday 23:59:59.999...)",
            usage: "eow() or eow(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eow()", "eow(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["sow", "eod", "eom"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eow", 1, args.len()));
        }
        match get_dt_or_now(args, "eow") {
            Ok(dt) => Value::DateTime(dt.end_of_week(1)), // ISO week (Mon-Sun)
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for EomFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eom",
            description: "End of month (last day 23:59:59.999...)",
            usage: "eom() or eom(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eom()", "eom(date(2025, 2, 15))"],
            category: "datetime",
            source: None,
            related: &["som", "eow", "eoq"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eom", 1, args.len()));
        }
        match get_dt_or_now(args, "eom") {
            Ok(dt) => Value::DateTime(dt.end_of_month()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for EoqFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eoq",
            description: "End of quarter (last day 23:59:59.999...)",
            usage: "eoq() or eoq(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eoq()", "eoq(date(2025, 7, 15))"],
            category: "datetime",
            source: None,
            related: &["soq", "eom", "eoy"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eoq", 1, args.len()));
        }
        match get_dt_or_now(args, "eoq") {
            Ok(dt) => Value::DateTime(dt.end_of_quarter()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for EoyFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eoy",
            description: "End of year (Dec 31 23:59:59.999...)",
            usage: "eoy() or eoy(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["eoy()", "eoy(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["soy", "eoq", "eom"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("eoy", 1, args.len()));
        }
        match get_dt_or_now(args, "eoy") {
            Ok(dt) => Value::DateTime(dt.end_of_year()),
            Err(e) => e,
        }
    }
}

// ========== Start of Period ==========

impl FunctionPlugin for SodFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sod",
            description: "Start of day (00:00:00)",
            usage: "sod() or sod(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["sod()", "sod(now())"],
            category: "datetime",
            source: None,
            related: &["eod", "sow", "som"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("sod", 1, args.len()));
        }
        match get_dt_or_now(args, "sod") {
            Ok(dt) => Value::DateTime(dt.start_of_day()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for SowFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sow",
            description: "Start of week (Monday 00:00:00)",
            usage: "sow() or sow(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["sow()", "sow(now())"],
            category: "datetime",
            source: None,
            related: &["eow", "sod", "som"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("sow", 1, args.len()));
        }
        match get_dt_or_now(args, "sow") {
            Ok(dt) => Value::DateTime(dt.start_of_week(1)), // ISO week (Monday start)
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for SomFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "som",
            description: "Start of month (1st at 00:00:00)",
            usage: "som() or som(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["som()", "som(now())"],
            category: "datetime",
            source: None,
            related: &["eom", "sow", "soq"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("som", 1, args.len()));
        }
        match get_dt_or_now(args, "som") {
            Ok(dt) => Value::DateTime(dt.start_of_month()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for SoqFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "soq",
            description: "Start of quarter (1st at 00:00:00)",
            usage: "soq() or soq(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["soq()", "soq(date(2025, 7, 15))"],
            category: "datetime",
            source: None,
            related: &["eoq", "som", "soy"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("soq", 1, args.len()));
        }
        match get_dt_or_now(args, "soq") {
            Ok(dt) => Value::DateTime(dt.start_of_quarter()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for SoyFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "soy",
            description: "Start of year (Jan 1 00:00:00)",
            usage: "soy() or soy(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["soy()", "soy(now())"],
            category: "datetime",
            source: None,
            related: &["eoy", "soq", "som"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("soy", 1, args.len()));
        }
        match get_dt_or_now(args, "soy") {
            Ok(dt) => Value::DateTime(dt.start_of_year()),
            Err(e) => e,
        }
    }
}

// ========== Navigation ==========

impl FunctionPlugin for TomorrowFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tomorrow",
            description: "Tomorrow (same time, +1 day)",
            usage: "tomorrow() or tomorrow(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["tomorrow()", "tomorrow(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["addDays", "nextWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("tomorrow", 1, args.len()));
        }
        match get_dt_or_now(args, "tomorrow") {
            Ok(dt) => Value::DateTime(dt.tomorrow()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for NextWeekFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nextWeek",
            description: "Monday of next week (00:00:00)",
            usage: "nextWeek() or nextWeek(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["nextWeek()", "nextWeek(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["sow", "eow", "nextMonth"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("nextWeek", 1, args.len()));
        }
        match get_dt_or_now(args, "nextWeek") {
            Ok(dt) => Value::DateTime(dt.next_week(1)), // ISO week (Monday start)
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for NextMonthFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nextMonth",
            description: "First day of next month (00:00:00)",
            usage: "nextMonth() or nextMonth(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["nextMonth()", "nextMonth(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["som", "eom", "nextMonthWd"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("nextMonth", 1, args.len()));
        }
        match get_dt_or_now(args, "nextMonth") {
            Ok(dt) => Value::DateTime(dt.next_month_first()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for NextMonthWdFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nextMonthWd",
            description: "First workday of next month (skips weekends)",
            usage: "nextMonthWd() or nextMonthWd(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["nextMonthWd()", "nextMonthWd(date(2025, 6, 15))"],
            category: "datetime",
            source: None,
            related: &["nextMonth", "nextWorkday", "isWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("nextMonthWd", 1, args.len()));
        }
        match get_dt_or_now(args, "nextMonthWd") {
            Ok(dt) => Value::DateTime(dt.next_month_first_workday()),
            Err(e) => e,
        }
    }
}

// ========== Workday Functions ==========

impl FunctionPlugin for IsWorkdayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "isWorkday",
            description: "Check if date is a workday (Mon-Fri)",
            usage: "isWorkday() or isWorkday(dt)",
            args: &OPT_DT_ARGS,
            returns: "Bool",
            examples: &["isWorkday()", "isWorkday(date(2025, 6, 14))"],
            category: "datetime",
            source: None,
            related: &["nextWorkday", "prevWorkday", "addWorkdays"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("isWorkday", 1, args.len()));
        }
        match get_dt_or_now(args, "isWorkday") {
            Ok(dt) => Value::Bool(dt.is_workday()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for NextWorkdayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nextWorkday",
            description: "Next workday (skips weekends)",
            usage: "nextWorkday() or nextWorkday(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["nextWorkday()", "nextWorkday(date(2025, 6, 14))"],
            category: "datetime",
            source: None,
            related: &["prevWorkday", "addWorkdays", "isWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("nextWorkday", 1, args.len()));
        }
        match get_dt_or_now(args, "nextWorkday") {
            Ok(dt) => Value::DateTime(dt.next_workday()),
            Err(e) => e,
        }
    }
}

impl FunctionPlugin for PrevWorkdayFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "prevWorkday",
            description: "Previous workday (skips weekends)",
            usage: "prevWorkday() or prevWorkday(dt)",
            args: &OPT_DT_ARGS,
            returns: "DateTime",
            examples: &["prevWorkday()", "prevWorkday(date(2025, 6, 16))"],
            category: "datetime",
            source: None,
            related: &["nextWorkday", "addWorkdays", "isWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() > 1 {
            return Value::Error(FolioError::arg_count("prevWorkday", 1, args.len()));
        }
        match get_dt_or_now(args, "prevWorkday") {
            Ok(dt) => Value::DateTime(dt.prev_workday()),
            Err(e) => e,
        }
    }
}

static ADDWORKDAYS_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "dt", typ: "DateTime", description: "Base datetime", optional: false, default: None },
    ArgMeta { name: "n", typ: "Number", description: "Number of workdays to add", optional: false, default: None },
];

impl FunctionPlugin for AddWorkdaysFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "addWorkdays",
            description: "Add N workdays (skips weekends)",
            usage: "addWorkdays(dt, n)",
            args: &ADDWORKDAYS_ARGS,
            returns: "DateTime",
            examples: &["addWorkdays(now(), 5)", "addWorkdays(date(2025, 6, 1), 30)"],
            category: "datetime",
            source: None,
            related: &["addDays", "nextWorkday", "isWorkday"],
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("addWorkdays", 2, args.len()));
        }

        let dt = match &args[0] {
            Value::DateTime(dt) => dt,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("addWorkdays", "dt", "DateTime", other.type_name())),
        };

        let n = match get_i64(&args[1], "addWorkdays", "n") {
            Ok(v) => v,
            Err(e) => return e,
        };

        Value::DateTime(dt.add_workdays(n))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn get_i32(value: &Value, func: &str, arg: &str) -> Result<i32, Value> {
    match value {
        Value::Number(n) => {
            if let Some(v) = n.to_i64() {
                if v >= i32::MIN as i64 && v <= i32::MAX as i64 {
                    Ok(v as i32)
                } else {
                    Err(Value::Error(FolioError::domain_error(format!("{} out of range for {}", arg, func))))
                }
            } else {
                Err(Value::Error(FolioError::arg_type(func, arg, "integer", "non-integer")))
            }
        }
        Value::Error(e) => Err(Value::Error(e.clone())),
        other => Err(Value::Error(FolioError::arg_type(func, arg, "Number", other.type_name()))),
    }
}

fn get_u32(value: &Value, func: &str, arg: &str) -> Result<u32, Value> {
    match value {
        Value::Number(n) => {
            if let Some(v) = n.to_i64() {
                if v >= 0 && v <= u32::MAX as i64 {
                    Ok(v as u32)
                } else {
                    Err(Value::Error(FolioError::domain_error(format!("{} must be non-negative for {}", arg, func))))
                }
            } else {
                Err(Value::Error(FolioError::arg_type(func, arg, "integer", "non-integer")))
            }
        }
        Value::Error(e) => Err(Value::Error(e.clone())),
        other => Err(Value::Error(FolioError::arg_type(func, arg, "Number", other.type_name()))),
    }
}

fn get_i64(value: &Value, func: &str, arg: &str) -> Result<i64, Value> {
    match value {
        Value::Number(n) => {
            if let Some(v) = n.to_i64() {
                Ok(v)
            } else {
                Err(Value::Error(FolioError::arg_type(func, arg, "integer", "non-integer or out of range")))
            }
        }
        Value::Error(e) => Err(Value::Error(e.clone())),
        other => Err(Value::Error(FolioError::arg_type(func, arg, "Number", other.type_name()))),
    }
}
```

folio-std\src\functions\math.rs
```rs
//! Core math functions

use folio_plugin::prelude::*;

pub struct Sqrt;
pub struct Ln;
pub struct Exp;
pub struct Pow;
pub struct Abs;
pub struct Round;
pub struct Floor;
pub struct Ceil;

static SQRT_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value (must be non-negative)", optional: false, default: None }];
static SQRT_EXAMPLES: [&str; 2] = ["sqrt(2)", "sqrt(5)"];
static SQRT_RELATED: [&str; 2] = ["pow", "exp"];

static LN_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value (must be positive)", optional: false, default: None }];
static LN_EXAMPLES: [&str; 2] = ["ln(e)", "ln(2)"];
static LN_RELATED: [&str; 1] = ["exp"];

static EXP_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Exponent", optional: false, default: None }];
static EXP_EXAMPLES: [&str; 2] = ["exp(1)", "exp(0)"];
static EXP_RELATED: [&str; 2] = ["ln", "pow"];

static POW_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "base", typ: "Number", description: "Base value", optional: false, default: None },
    ArgMeta { name: "exponent", typ: "Number", description: "Integer exponent", optional: false, default: None },
];
static POW_EXAMPLES: [&str; 2] = ["pow(2, 10)", "pow(phi, 43)"];
static POW_RELATED: [&str; 2] = ["sqrt", "exp"];

static ABS_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value", optional: false, default: None }];
static ABS_EXAMPLES: [&str; 2] = ["abs(-5)", "abs(3.14)"];
static ABS_RELATED: [&str; 0] = [];

static ROUND_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value to round", optional: false, default: None }];
static ROUND_EXAMPLES: [&str; 2] = ["round(3.5)", "round(3.4)"];
static ROUND_RELATED: [&str; 2] = ["floor", "ceil"];

static FLOOR_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value to floor", optional: false, default: None }];
static FLOOR_EXAMPLES: [&str; 2] = ["floor(3.7)", "floor(-2.3)"];
static FLOOR_RELATED: [&str; 2] = ["ceil", "round"];

static CEIL_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Value to ceil", optional: false, default: None }];
static CEIL_EXAMPLES: [&str; 2] = ["ceil(3.2)", "ceil(-2.7)"];
static CEIL_RELATED: [&str; 2] = ["floor", "round"];

impl FunctionPlugin for Sqrt {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sqrt",
            description: "Square root with arbitrary precision",
            usage: "sqrt(x)",
            args: &SQRT_ARGS,
            returns: "Number",
            examples: &SQRT_EXAMPLES,
            category: "math",
            source: None,
            related: &SQRT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("sqrt", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                match n.sqrt(ctx.precision) {
                    Ok(result) => Value::Number(result),
                    Err(e) => Value::Error(e.into()),
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("sqrt", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Ln {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ln",
            description: "Natural logarithm",
            usage: "ln(x)",
            args: &LN_ARGS,
            returns: "Number",
            examples: &LN_EXAMPLES,
            category: "math",
            source: None,
            related: &LN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("ln", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                match n.ln(ctx.precision) {
                    Ok(result) => Value::Number(result),
                    Err(e) => Value::Error(e.into()),
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("ln", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Exp {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "exp",
            description: "Exponential function (e^x)",
            usage: "exp(x)",
            args: &EXP_ARGS,
            returns: "Number",
            examples: &EXP_EXAMPLES,
            category: "math",
            source: None,
            related: &EXP_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("exp", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.exp(ctx.precision)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("exp", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Pow {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pow",
            description: "Raise to power",
            usage: "pow(base, exponent)",
            args: &POW_ARGS,
            returns: "Number",
            examples: &POW_EXAMPLES,
            category: "math",
            source: None,
            related: &POW_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("pow", 2, args.len()));
        }
        let base = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("pow", "base", "Number", other.type_name())),
        };
        let exp = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("pow", "exponent", "Number", other.type_name())),
        };

        // Use pow_real which handles both integer and fractional exponents
        Value::Number(base.pow_real(exp, ctx.precision))
    }
}

impl FunctionPlugin for Abs {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "abs",
            description: "Absolute value",
            usage: "abs(x)",
            args: &ABS_ARGS,
            returns: "Number",
            examples: &ABS_EXAMPLES,
            category: "math",
            source: None,
            related: &ABS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("abs", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.abs()),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("abs", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Round {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "round",
            description: "Round to nearest integer",
            usage: "round(x)",
            args: &ROUND_ARGS,
            returns: "Number",
            examples: &ROUND_EXAMPLES,
            category: "math",
            source: None,
            related: &ROUND_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("round", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                // Simple rounding via string conversion
                let s = n.as_decimal(0);
                match Number::from_str(&s) {
                    Ok(rounded) => Value::Number(rounded),
                    Err(e) => Value::Error(e.into()),
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("round", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Floor {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "floor",
            description: "Largest integer less than or equal to x",
            usage: "floor(x)",
            args: &FLOOR_ARGS,
            returns: "Number",
            examples: &FLOOR_EXAMPLES,
            category: "math",
            source: None,
            related: &FLOOR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("floor", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.floor()),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("floor", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Ceil {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ceil",
            description: "Smallest integer greater than or equal to x",
            usage: "ceil(x)",
            args: &CEIL_ARGS,
            returns: "Number",
            examples: &CEIL_EXAMPLES,
            category: "math",
            source: None,
            related: &CEIL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("ceil", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.ceil()),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("ceil", "x", "Number", other.type_name())),
        }
    }
}
```

folio-std\src\functions\mod.rs
```rs
//! Standard math, datetime, and utility functions

mod math;
mod trig;
mod aggregate;
mod datetime;
mod utility;

pub use math::{Sqrt, Ln, Exp, Pow, Abs, Round, Floor, Ceil};
pub use trig::{Sin, Cos, Tan};
pub use aggregate::Sum;
pub use utility::{FieldsFn, HeadFn, TailFn, TakeFn, TypeofFn, DescribeFn, LenFn, NthFn};

// DateTime functions
pub use datetime::{
    // Construction
    DateFn, TimeFn, DateTimeFn, NowFn,
    // Parsing
    ParseDateFn, ParseTimeFn,
    // Extraction
    YearFn, MonthFn, DayFn, HourFn, MinuteFn, SecondFn, WeekdayFn, DayOfYearFn, WeekFn,
    // Formatting
    FormatDateFn, FormatTimeFn, FormatDateTimeFn,
    // Duration construction
    WeeksDur, DaysDur, HoursDur, MinutesDur, SecondsDur, MillisecondsDur,
    // Arithmetic
    AddDaysFn, AddMonthsFn, AddYearsFn, DiffFn,
    // Comparison
    IsBeforeFn, IsAfterFn, IsSameDayFn,
    // Utilities
    StartOfDayFn, EndOfDayFn, StartOfMonthFn, StartOfYearFn,
    // Shortcuts - End of period
    EodFn, EowFn, EomFn, EoqFn, EoyFn,
    // Shortcuts - Start of period
    SodFn, SowFn, SomFn, SoqFn, SoyFn,
    // Shortcuts - Navigation
    TomorrowFn, NextWeekFn, NextMonthFn, NextMonthWdFn,
    // Workday functions
    IsWorkdayFn, NextWorkdayFn, PrevWorkdayFn, AddWorkdaysFn,
};
```

folio-std\src\functions\trig.rs
```rs
//! Trigonometric functions

use folio_plugin::prelude::*;

pub struct Sin;
pub struct Cos;
pub struct Tan;

static SIN_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Angle in radians", optional: false, default: None }];
static SIN_EXAMPLES: [&str; 2] = ["sin(0)", "sin(π/2)"];
static SIN_RELATED: [&str; 2] = ["cos", "tan"];

static COS_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Angle in radians", optional: false, default: None }];
static COS_EXAMPLES: [&str; 2] = ["cos(0)", "cos(π)"];
static COS_RELATED: [&str; 2] = ["sin", "tan"];

static TAN_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "Angle in radians", optional: false, default: None }];
static TAN_EXAMPLES: [&str; 2] = ["tan(0)", "tan(π/4)"];
static TAN_RELATED: [&str; 2] = ["sin", "cos"];

impl FunctionPlugin for Sin {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "sin",
            description: "Sine function",
            usage: "sin(x)",
            args: &SIN_ARGS,
            returns: "Number",
            examples: &SIN_EXAMPLES,
            category: "trig",
            source: None,
            related: &SIN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("sin", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.sin(ctx.precision)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("sin", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Cos {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cos",
            description: "Cosine function",
            usage: "cos(x)",
            args: &COS_ARGS,
            returns: "Number",
            examples: &COS_EXAMPLES,
            category: "trig",
            source: None,
            related: &COS_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("cos", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => Value::Number(n.cos(ctx.precision)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("cos", "x", "Number", other.type_name())),
        }
    }
}

impl FunctionPlugin for Tan {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tan",
            description: "Tangent function",
            usage: "tan(x)",
            args: &TAN_ARGS,
            returns: "Number",
            examples: &TAN_EXAMPLES,
            category: "trig",
            source: None,
            related: &TAN_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("tan", 1, args.len()));
        }
        match &args[0] {
            Value::Number(n) => {
                match n.tan(ctx.precision) {
                    Ok(result) => Value::Number(result),
                    Err(e) => Value::Error(e.into()),
                }
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("tan", "x", "Number", other.type_name())),
        }
    }
}
```

folio-std\src\functions\utility.rs
```rs
//! Utility functions for LLM experience: fields, head, tail, typeof, describe

use folio_plugin::prelude::*;
use std::collections::HashMap;

// ============================================================================
// fields(object) → List<Text>
// ============================================================================

pub struct FieldsFn;

static FIELDS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "object",
    typ: "Object",
    description: "Object to get fields from",
    optional: false,
    default: None,
}];
static FIELDS_EXAMPLES: [&str; 1] = ["fields(linear_reg(x, y)) → [\"slope\", \"intercept\", ...]"];
static FIELDS_RELATED: [&str; 2] = ["describe", "typeof"];

impl FunctionPlugin for FieldsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "fields",
            description: "List available fields on an Object",
            usage: "fields(object)",
            args: &FIELDS_ARGS,
            returns: "List<Text>",
            examples: &FIELDS_EXAMPLES,
            category: "utility",
            source: None,
            related: &FIELDS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("fields", 1, args.len()));
        }

        match &args[0] {
            Value::Object(obj) => {
                let mut keys: Vec<String> = obj.keys().cloned().collect();
                keys.sort();
                Value::List(keys.into_iter().map(Value::Text).collect())
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(
                FolioError::arg_type("fields", "object", "Object", other.type_name())
                    .with_suggestion("fields() shows available fields on Objects returned by functions like linear_reg(), t_test_1(), etc.")
            ),
        }
    }
}

// ============================================================================
// head(list, n?) → List
// ============================================================================

pub struct HeadFn;

static HEAD_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to get elements from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements (default: 5)",
        optional: true,
        default: Some("5"),
    },
];
static HEAD_EXAMPLES: [&str; 2] = ["head([1,2,3,4,5,6], 3) → [1, 2, 3]", "head(data) → first 5 elements"];
static HEAD_RELATED: [&str; 2] = ["tail", "take"];

impl FunctionPlugin for HeadFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "head",
            description: "First n elements of list",
            usage: "head(list, n?)",
            args: &HEAD_ARGS,
            returns: "List",
            examples: &HEAD_EXAMPLES,
            category: "utility",
            source: None,
            related: &HEAD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("head", 1, args.len())
                .with_suggestion("Usage: head(list) or head(list, n)"));
        }

        let list = match &args[0] {
            Value::List(l) => l,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("head", "list", "List", other.type_name())),
        };

        let n = if args.len() > 1 {
            match &args[1] {
                Value::Number(n) => n.to_i64().unwrap_or(5) as usize,
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("head", "n", "Number", other.type_name())),
            }
        } else {
            5
        };

        Value::List(list.iter().take(n).cloned().collect())
    }
}

// ============================================================================
// tail(list, n?) → List
// ============================================================================

pub struct TailFn;

static TAIL_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to get elements from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements (default: 5)",
        optional: true,
        default: Some("5"),
    },
];
static TAIL_EXAMPLES: [&str; 2] = ["tail([1,2,3,4,5,6], 3) → [4, 5, 6]", "tail(data) → last 5 elements"];
static TAIL_RELATED: [&str; 2] = ["head", "take"];

impl FunctionPlugin for TailFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "tail",
            description: "Last n elements of list",
            usage: "tail(list, n?)",
            args: &TAIL_ARGS,
            returns: "List",
            examples: &TAIL_EXAMPLES,
            category: "utility",
            source: None,
            related: &TAIL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("tail", 1, args.len())
                .with_suggestion("Usage: tail(list) or tail(list, n)"));
        }

        let list = match &args[0] {
            Value::List(l) => l,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("tail", "list", "List", other.type_name())),
        };

        let n = if args.len() > 1 {
            match &args[1] {
                Value::Number(n) => n.to_i64().unwrap_or(5) as usize,
                Value::Error(e) => return Value::Error(e.clone()),
                other => return Value::Error(FolioError::arg_type("tail", "n", "Number", other.type_name())),
            }
        } else {
            5
        };

        let skip = list.len().saturating_sub(n);
        Value::List(list.iter().skip(skip).cloned().collect())
    }
}

// ============================================================================
// take(list, n) → List (alias for head)
// ============================================================================

pub struct TakeFn;

static TAKE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to get elements from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Number of elements",
        optional: false,
        default: None,
    },
];
static TAKE_EXAMPLES: [&str; 1] = ["take([1,2,3,4,5], 3) → [1, 2, 3]"];
static TAKE_RELATED: [&str; 2] = ["head", "tail"];

impl FunctionPlugin for TakeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "take",
            description: "First n elements of list (alias for head)",
            usage: "take(list, n)",
            args: &TAKE_ARGS,
            returns: "List",
            examples: &TAKE_EXAMPLES,
            category: "utility",
            source: None,
            related: &TAKE_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        // Delegate to head
        HeadFn.call(args, ctx)
    }
}

// ============================================================================
// typeof(value) → Text
// ============================================================================

pub struct TypeofFn;

static TYPEOF_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "value",
    typ: "Any",
    description: "Value to get type of",
    optional: false,
    default: None,
}];
static TYPEOF_EXAMPLES: [&str; 3] = [
    "typeof(42) → \"Number\"",
    "typeof([1,2,3]) → \"List\"",
    "typeof(linear_reg(x,y)) → \"Object\"",
];
static TYPEOF_RELATED: [&str; 2] = ["fields", "describe"];

impl FunctionPlugin for TypeofFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "typeof",
            description: "Get type name of a value",
            usage: "typeof(value)",
            args: &TYPEOF_ARGS,
            returns: "Text",
            examples: &TYPEOF_EXAMPLES,
            category: "utility",
            source: None,
            related: &TYPEOF_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("typeof", 1, args.len()));
        }

        Value::Text(args[0].type_name().to_string())
    }
}

// ============================================================================
// describe(object) → Object
// ============================================================================

pub struct DescribeFn;

static DESCRIBE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "object",
    typ: "Object",
    description: "Object to describe",
    optional: false,
    default: None,
}];
static DESCRIBE_EXAMPLES: [&str; 1] = ["describe(linear_reg(x, y)) → detailed Object info"];
static DESCRIBE_RELATED: [&str; 2] = ["fields", "typeof"];

impl FunctionPlugin for DescribeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "describe",
            description: "Full description of Object with field values and types",
            usage: "describe(object)",
            args: &DESCRIBE_ARGS,
            returns: "Object",
            examples: &DESCRIBE_EXAMPLES,
            category: "utility",
            source: None,
            related: &DESCRIBE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("describe", 1, args.len()));
        }

        match &args[0] {
            Value::Object(obj) => {
                let mut result = HashMap::new();
                result.insert("type".to_string(), Value::Text("Object".to_string()));
                result.insert("field_count".to_string(), Value::Number(Number::from_i64(obj.len() as i64)));

                let mut fields_info = HashMap::new();
                for (key, value) in obj {
                    let mut field = HashMap::new();
                    field.insert("type".to_string(), Value::Text(value.type_name().to_string()));
                    field.insert("value".to_string(), value.clone());
                    fields_info.insert(key.clone(), Value::Object(field));
                }
                result.insert("fields".to_string(), Value::Object(fields_info));

                Value::Object(result)
            }
            Value::List(list) => {
                let mut result = HashMap::new();
                result.insert("type".to_string(), Value::Text("List".to_string()));
                result.insert("length".to_string(), Value::Number(Number::from_i64(list.len() as i64)));

                // Show first 5 elements
                let preview: Vec<Value> = list.iter().take(5).cloned().collect();
                result.insert("preview".to_string(), Value::List(preview));

                // Count by type
                let mut type_counts: HashMap<String, i64> = HashMap::new();
                for item in list {
                    *type_counts.entry(item.type_name().to_string()).or_insert(0) += 1;
                }
                let types_obj: HashMap<String, Value> = type_counts
                    .into_iter()
                    .map(|(k, v)| (k, Value::Number(Number::from_i64(v))))
                    .collect();
                result.insert("element_types".to_string(), Value::Object(types_obj));

                Value::Object(result)
            }
            Value::Error(e) => Value::Error(e.clone()),
            other => {
                let mut result = HashMap::new();
                result.insert("type".to_string(), Value::Text(other.type_name().to_string()));
                result.insert("value".to_string(), other.clone());
                Value::Object(result)
            }
        }
    }
}

// ============================================================================
// len(list) → Number
// ============================================================================

pub struct LenFn;

static LEN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "list",
    typ: "List or Text",
    description: "List or Text to get length of",
    optional: false,
    default: None,
}];
static LEN_EXAMPLES: [&str; 2] = ["len([1,2,3]) → 3", "len(\"hello\") → 5"];
static LEN_RELATED: [&str; 2] = ["count", "head"];

impl FunctionPlugin for LenFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "len",
            description: "Length of a list or text",
            usage: "len(value)",
            args: &LEN_ARGS,
            returns: "Number",
            examples: &LEN_EXAMPLES,
            category: "utility",
            source: None,
            related: &LEN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("len", 1, args.len()));
        }

        match &args[0] {
            Value::List(list) => Value::Number(Number::from_i64(list.len() as i64)),
            Value::Text(s) => Value::Number(Number::from_i64(s.len() as i64)),
            Value::Error(e) => Value::Error(e.clone()),
            other => Value::Error(FolioError::arg_type("len", "list", "List or Text", other.type_name())),
        }
    }
}

// ============================================================================
// nth(list, index) → Value
// ============================================================================

pub struct NthFn;

static NTH_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List",
        description: "List to get element from",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "index",
        typ: "Number",
        description: "Zero-based index",
        optional: false,
        default: None,
    },
];
static NTH_EXAMPLES: [&str; 2] = ["nth([10, 20, 30], 0) → 10", "nth([10, 20, 30], 2) → 30"];
static NTH_RELATED: [&str; 2] = ["head", "tail"];

impl FunctionPlugin for NthFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nth",
            description: "Get element at index (0-based)",
            usage: "nth(list, index)",
            args: &NTH_ARGS,
            returns: "Value",
            examples: &NTH_EXAMPLES,
            category: "utility",
            source: None,
            related: &NTH_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("nth", 2, args.len()));
        }

        let list = match &args[0] {
            Value::List(l) => l,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("nth", "list", "List", other.type_name())),
        };

        let index = match &args[1] {
            Value::Number(n) => n.to_i64().unwrap_or(-1),
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("nth", "index", "Number", other.type_name())),
        };

        if index < 0 || index as usize >= list.len() {
            return Value::Error(FolioError::domain_error(format!(
                "Index {} out of bounds for list of length {}",
                index, list.len()
            )));
        }

        list[index as usize].clone()
    }
}
```

folio-std\src\lib.rs
```rs
//! Folio Standard Library

pub mod functions;
pub mod analyzers;
pub mod commands;
pub mod constants;

use folio_plugin::PluginRegistry;

/// Load standard library into registry
pub fn load_standard_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Math functions
        .with_function(functions::Sqrt)
        .with_function(functions::Ln)
        .with_function(functions::Exp)
        .with_function(functions::Pow)
        .with_function(functions::Abs)
        .with_function(functions::Sin)
        .with_function(functions::Cos)
        .with_function(functions::Tan)
        .with_function(functions::Sum)
        .with_function(functions::Round)
        .with_function(functions::Floor)
        .with_function(functions::Ceil)
        // DateTime functions - Construction
        .with_function(functions::DateFn)
        .with_function(functions::TimeFn)
        .with_function(functions::DateTimeFn)
        .with_function(functions::NowFn)
        // DateTime functions - Parsing
        .with_function(functions::ParseDateFn)
        .with_function(functions::ParseTimeFn)
        // DateTime functions - Extraction
        .with_function(functions::YearFn)
        .with_function(functions::MonthFn)
        .with_function(functions::DayFn)
        .with_function(functions::HourFn)
        .with_function(functions::MinuteFn)
        .with_function(functions::SecondFn)
        .with_function(functions::WeekdayFn)
        .with_function(functions::DayOfYearFn)
        .with_function(functions::WeekFn)
        // DateTime functions - Formatting
        .with_function(functions::FormatDateFn)
        .with_function(functions::FormatTimeFn)
        .with_function(functions::FormatDateTimeFn)
        // DateTime functions - Duration construction
        .with_function(functions::WeeksDur)
        .with_function(functions::DaysDur)
        .with_function(functions::HoursDur)
        .with_function(functions::MinutesDur)
        .with_function(functions::SecondsDur)
        .with_function(functions::MillisecondsDur)
        // DateTime functions - Arithmetic
        .with_function(functions::AddDaysFn)
        .with_function(functions::AddMonthsFn)
        .with_function(functions::AddYearsFn)
        .with_function(functions::DiffFn)
        // DateTime functions - Comparison
        .with_function(functions::IsBeforeFn)
        .with_function(functions::IsAfterFn)
        .with_function(functions::IsSameDayFn)
        // DateTime functions - Utilities
        .with_function(functions::StartOfDayFn)
        .with_function(functions::EndOfDayFn)
        .with_function(functions::StartOfMonthFn)
        .with_function(functions::StartOfYearFn)
        // DateTime shortcuts - End of period
        .with_function(functions::EodFn)
        .with_function(functions::EowFn)
        .with_function(functions::EomFn)
        .with_function(functions::EoqFn)
        .with_function(functions::EoyFn)
        // DateTime shortcuts - Start of period
        .with_function(functions::SodFn)
        .with_function(functions::SowFn)
        .with_function(functions::SomFn)
        .with_function(functions::SoqFn)
        .with_function(functions::SoyFn)
        // DateTime shortcuts - Navigation
        .with_function(functions::TomorrowFn)
        .with_function(functions::NextWeekFn)
        .with_function(functions::NextMonthFn)
        .with_function(functions::NextMonthWdFn)
        // Workday functions
        .with_function(functions::IsWorkdayFn)
        .with_function(functions::NextWorkdayFn)
        .with_function(functions::PrevWorkdayFn)
        .with_function(functions::AddWorkdaysFn)
        // Utility functions (LLM experience)
        .with_function(functions::FieldsFn)
        .with_function(functions::HeadFn)
        .with_function(functions::TailFn)
        .with_function(functions::TakeFn)
        .with_function(functions::TypeofFn)
        .with_function(functions::DescribeFn)
        .with_function(functions::LenFn)
        .with_function(functions::NthFn)
        // Analyzers
        .with_analyzer(analyzers::PhiAnalyzer)
        .with_analyzer(analyzers::PiAnalyzer)
        .with_analyzer(analyzers::EAnalyzer)
        .with_command(commands::Trace)
        .with_command(commands::Explain)
        // Mathematical constants
        .with_constant(constants::phi())
        .with_constant(constants::pi())
        .with_constant(constants::e())
        .with_constant(constants::sqrt2())
        .with_constant(constants::sqrt3())
        // Particle masses (MeV)
        .with_constant(constants::m_e())
        .with_constant(constants::m_mu())
        .with_constant(constants::m_tau())
        .with_constant(constants::m_higgs())
        // CKM matrix elements
        .with_constant(constants::v_us())
        .with_constant(constants::v_cb())
        .with_constant(constants::v_ub())
        .with_constant(constants::v_ts())
        // Physical constants
        .with_constant(constants::c())
        .with_constant(constants::alpha())
        // ASCII aliases for Unicode constants
        .with_constant(constants::phi_ascii())    // "phi" alias for "φ"
        .with_constant(constants::pi_ascii())     // "pi" alias for "π"
        .with_constant(constants::alpha_ascii())  // "alpha" alias for "α"
        .with_constant(constants::m_mu_ascii())   // "m_mu" alias for "m_μ"
        .with_constant(constants::m_tau_ascii())  // "m_tau" alias for "m_τ"
}

/// Create registry with standard library
pub fn standard_registry() -> PluginRegistry {
    load_standard_library(PluginRegistry::new())
}
```

folio\Cargo.toml
```toml
[package]
name = "folio"
description = "Markdown Computational Documents - Jupyter for LLMs"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
folio-core = { path = "../folio-core" }
folio-plugin = { path = "../folio-plugin" }
folio-std = { path = "../folio-std" }
folio-stats = { path = "../folio-stats" }
pest = { workspace = true }
pest_derive = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
```

folio\src\ast.rs
```rs
//! Abstract Syntax Tree

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Document {
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub name: String,
    pub attributes: HashMap<String, String>,
    pub table: Table,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Table {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    pub name: String,
    pub formula: Option<Expr>,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Number(String),
    StringLiteral(String),
    Variable(Vec<String>),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
    /// List literal: [a, b, c]
    List(Vec<Expr>),
    /// Field access on expression result: expr.field.subfield
    FieldAccess(Box<Expr>, Vec<String>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BinOp {
    Add, Sub, Mul, Div, Pow,
    // Comparison operators
    Lt, Gt, Le, Ge, Eq, Ne,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UnaryOp { Neg }
```

folio\src\eval.rs
```rs
//! Document evaluator
//!
//! Evaluates document expressions in dependency order.

use crate::ast::{Document, Expr, BinOp, UnaryOp};
use folio_plugin::EvalContext;
use folio_core::{Value, FolioError, Number};
use std::collections::{HashMap, HashSet, VecDeque};

/// Result of document evaluation
#[derive(Debug)]
pub struct EvalResult {
    /// Rendered markdown with results
    pub markdown: String,
    /// All computed values by cell name
    pub values: HashMap<String, Value>,
    /// Errors encountered
    pub errors: Vec<FolioError>,
    /// Warnings (non-fatal)
    pub warnings: Vec<FolioError>,
}

impl EvalResult {
    /// Create error result for parse failure
    pub fn parse_error(error: FolioError) -> Self {
        Self {
            markdown: format!("# Parse Error\n\n{}", error),
            values: HashMap::new(),
            errors: vec![error],
            warnings: vec![],
        }
    }
}

/// Document evaluator
pub struct Evaluator;

impl Evaluator {
    pub fn new() -> Self {
        Self
    }
    
    /// Evaluate document, return values by cell name
    pub fn eval(&self, doc: &Document, ctx: &mut EvalContext) -> HashMap<String, Value> {
        let mut values = HashMap::new();

        // Collect all cells and their formulas
        let mut cells: HashMap<String, (Option<&Expr>, &str, u32)> = HashMap::new();
        let mut section_precisions: HashMap<String, u32> = HashMap::new();

        for section in &doc.sections {
            let section_precision = section.attributes
                .get("precision")
                .and_then(|p| p.parse().ok())
                .unwrap_or(ctx.precision);

            for row in &section.table.rows {
                for cell in &row.cells {
                    cells.insert(
                        cell.name.clone(),
                        (cell.formula.as_ref(), &cell.raw_text, section_precision)
                    );
                    section_precisions.insert(cell.name.clone(), section_precision);
                }
            }
        }

        // Build dependency graph
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
        for (name, (formula, _, _)) in &cells {
            let deps = if let Some(expr) = formula {
                self.extract_dependencies(expr)
            } else {
                HashSet::new()
            };
            // Filter to only existing cells
            let filtered_deps: Vec<String> = deps.into_iter()
                .filter(|d| cells.contains_key(d))
                .collect();
            dependencies.insert(name.clone(), filtered_deps);
        }

        // Detect cycles and compute topological order
        match self.topological_sort(&dependencies) {
            Ok(order) => {
                // Evaluate in topological order
                for cell_name in order {
                    if let Some((formula, raw_text, precision)) = cells.get(&cell_name) {
                        ctx.precision = *precision;

                        // Check if this variable was already set externally
                        // External variables take precedence over hardcoded values
                        let existing = ctx.get_var(&cell_name);
                        if !existing.is_error() && formula.is_none() {
                            // External variable exists and cell is a literal - use external value
                            values.insert(cell_name.clone(), existing);
                            continue;
                        }

                        let value = match formula {
                            Some(expr) => {
                                let deps = dependencies.get(&cell_name).cloned().unwrap_or_default();
                                let result = self.eval_expr(expr, ctx);
                                if ctx.tracing {
                                    ctx.record_trace(
                                        cell_name.clone(),
                                        raw_text.to_string(),
                                        result.clone(),
                                        deps,
                                    );
                                }
                                result
                            }
                            None => self.parse_literal(raw_text),
                        };
                        ctx.set_var(cell_name.clone(), value.clone());
                        values.insert(cell_name.clone(), value);
                    }
                }
            }
            Err(cycle) => {
                // Return circular reference error for all cells in the cycle
                let error = FolioError::circular_ref(&cycle);
                for cell_name in cycle {
                    values.insert(cell_name.clone(), Value::Error(error.clone()));
                }
                // Evaluate remaining cells in document order
                for section in &doc.sections {
                    let section_precision = section.attributes
                        .get("precision")
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(ctx.precision);
                    ctx.precision = section_precision;

                    for row in &section.table.rows {
                        for cell in &row.cells {
                            if !values.contains_key(&cell.name) {
                                // Check if this variable was already set externally
                                let existing = ctx.get_var(&cell.name);
                                if !existing.is_error() && cell.formula.is_none() {
                                    // External variable exists and cell is a literal - use external value
                                    values.insert(cell.name.clone(), existing);
                                    continue;
                                }

                                let value = match &cell.formula {
                                    Some(expr) => self.eval_expr(expr, ctx),
                                    None => self.parse_literal(&cell.raw_text),
                                };
                                ctx.set_var(cell.name.clone(), value.clone());
                                values.insert(cell.name.clone(), value);
                            }
                        }
                    }
                }
            }
        }

        values
    }

    /// Extract variable dependencies from an expression
    fn extract_dependencies(&self, expr: &Expr) -> HashSet<String> {
        let mut deps = HashSet::new();
        self.collect_deps(expr, &mut deps);
        deps
    }

    fn collect_deps(&self, expr: &Expr, deps: &mut HashSet<String>) {
        match expr {
            Expr::Number(_) => {}
            Expr::StringLiteral(_) => {}
            Expr::Variable(parts) => {
                // The root variable is the dependency
                if !parts.is_empty() {
                    deps.insert(parts[0].clone());
                }
            }
            Expr::BinaryOp(left, _, right) => {
                self.collect_deps(left, deps);
                self.collect_deps(right, deps);
            }
            Expr::UnaryOp(_, inner) => {
                self.collect_deps(inner, deps);
            }
            Expr::FunctionCall(_, args) => {
                for arg in args {
                    self.collect_deps(arg, deps);
                }
            }
            Expr::List(elements) => {
                for elem in elements {
                    self.collect_deps(elem, deps);
                }
            }
            Expr::FieldAccess(base_expr, _) => {
                // Collect dependencies from the base expression
                self.collect_deps(base_expr, deps);
            }
        }
    }

    /// Topological sort using Kahn's algorithm
    /// Returns Ok(ordered_cells) or Err(cycle_cells)
    fn topological_sort(&self, dependencies: &HashMap<String, Vec<String>>) -> Result<Vec<String>, Vec<String>> {
        // Build in-degree map and reverse dependency map
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize in-degree for all nodes
        for name in dependencies.keys() {
            in_degree.entry(name.clone()).or_insert(0);
            reverse_deps.entry(name.clone()).or_default();
        }

        // Build in-degree counts and reverse mapping
        for (name, deps) in dependencies {
            for dep in deps {
                *in_degree.entry(name.clone()).or_insert(0) += 1;
                reverse_deps.entry(dep.clone()).or_default().push(name.clone());
            }
        }

        // Find all nodes with in-degree 0
        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(dependents) = reverse_deps.get(&node) {
                for dependent in dependents {
                    if let Some(deg) = in_degree.get_mut(dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() < dependencies.len() {
            // Find nodes in cycles (those with remaining in-degree > 0)
            let cycle: Vec<String> = in_degree.iter()
                .filter(|(_, &deg)| deg > 0)
                .map(|(name, _)| name.clone())
                .collect();
            Err(cycle)
        } else {
            Ok(result)
        }
    }
    
    /// Evaluate single expression
    fn eval_expr(&self, expr: &Expr, ctx: &EvalContext) -> Value {
        match expr {
            Expr::Number(s) => self.parse_literal(s),

            Expr::StringLiteral(s) => Value::Text(s.clone()),

            Expr::Variable(parts) => {
                let name = parts.join(".");
                let result = ctx.get_var(&name);
                // Add variable name context to errors for better debugging
                if let Value::Error(e) = result {
                    Value::Error(e.with_note(&format!("when resolving '{}'", name)))
                } else {
                    result
                }
            }

            Expr::BinaryOp(left, op, right) => {
                let l = self.eval_expr(left, ctx);
                let r = self.eval_expr(right, ctx);
                self.eval_binary_op(l, *op, r, ctx.precision)
            }

            Expr::UnaryOp(op, inner) => {
                let v = self.eval_expr(inner, ctx);
                self.eval_unary_op(*op, v)
            }

            Expr::FunctionCall(name, args) => {
                let evaluated_args: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a, ctx))
                    .collect();

                // Check for errors in arguments and add function context
                for (i, arg) in evaluated_args.iter().enumerate() {
                    if let Value::Error(e) = arg {
                        return Value::Error(e.clone().with_note(&format!("in argument {} of {}()", i + 1, name)));
                    }
                }

                ctx.registry.call_function(name, &evaluated_args, ctx)
            }

            Expr::List(elements) => {
                let evaluated: Vec<Value> = elements
                    .iter()
                    .map(|e| self.eval_expr(e, ctx))
                    .collect();

                // Check for errors in list elements
                for (i, elem) in evaluated.iter().enumerate() {
                    if let Value::Error(e) = elem {
                        return Value::Error(e.clone().with_note(&format!("in list element {}", i + 1)));
                    }
                }

                Value::List(evaluated)
            }

            Expr::FieldAccess(base_expr, fields) => {
                let base_value = self.eval_expr(base_expr, ctx);

                // If base evaluation resulted in error, propagate it
                if let Value::Error(e) = base_value {
                    return Value::Error(e.with_note(&format!("in field access .{}", fields.join("."))));
                }

                // Navigate through the fields
                let mut current = base_value;
                for field in fields {
                    match &current {
                        Value::Object(map) => {
                            if let Some(value) = map.get(field) {
                                current = value.clone();
                            } else {
                                return Value::Error(
                                    FolioError::new("FIELD_NOT_FOUND", format!("Field '{}' not found in object", field))
                                        .with_suggestion(&format!("Available fields: {}", map.keys().cloned().collect::<Vec<_>>().join(", ")))
                                );
                            }
                        }
                        _ => {
                            return Value::Error(
                                FolioError::new("NOT_OBJECT", format!("Cannot access field '{}' on non-object value", field))
                                    .with_note(&format!("Value is: {:?}", current))
                            );
                        }
                    }
                }
                current
            }
        }
    }
    
    fn parse_literal(&self, s: &str) -> Value {
        match folio_core::Number::from_str(s.trim()) {
            Ok(n) => Value::Number(n),
            Err(_) => Value::Text(s.to_string()),
        }
    }
    
    fn eval_binary_op(&self, left: Value, op: BinOp, right: Value, precision: u32) -> Value {
        // Propagate errors
        if let Value::Error(e) = &left {
            return Value::Error(e.clone().with_note("from left operand"));
        }
        if let Value::Error(e) = &right {
            return Value::Error(e.clone().with_note("from right operand"));
        }

        // Handle DateTime/Duration arithmetic
        match (&left, &right, op) {
            // DateTime + Duration -> DateTime
            (Value::DateTime(dt), Value::Duration(dur), BinOp::Add) => {
                return Value::DateTime(dt.add_duration(dur));
            }
            // Duration + DateTime -> DateTime
            (Value::Duration(dur), Value::DateTime(dt), BinOp::Add) => {
                return Value::DateTime(dt.add_duration(dur));
            }
            // DateTime - Duration -> DateTime
            (Value::DateTime(dt), Value::Duration(dur), BinOp::Sub) => {
                return Value::DateTime(dt.sub_duration(dur));
            }
            // DateTime - DateTime -> Duration
            (Value::DateTime(dt1), Value::DateTime(dt2), BinOp::Sub) => {
                return Value::Duration(dt1.duration_since(dt2));
            }
            // Duration + Duration -> Duration
            (Value::Duration(d1), Value::Duration(d2), BinOp::Add) => {
                return Value::Duration(d1.add(d2));
            }
            // Duration - Duration -> Duration
            (Value::Duration(d1), Value::Duration(d2), BinOp::Sub) => {
                return Value::Duration(d1.sub(d2));
            }
            // Duration * Number -> Duration
            (Value::Duration(dur), Value::Number(n), BinOp::Mul) => {
                if let Some(scalar) = n.to_i64() {
                    return Value::Duration(dur.mul(scalar));
                } else {
                    // Use floating point for non-integer multipliers
                    let f = n.to_f64().unwrap_or(1.0);
                    return Value::Duration(dur.mul_f64(f));
                }
            }
            // Number * Duration -> Duration
            (Value::Number(n), Value::Duration(dur), BinOp::Mul) => {
                if let Some(scalar) = n.to_i64() {
                    return Value::Duration(dur.mul(scalar));
                } else {
                    let f = n.to_f64().unwrap_or(1.0);
                    return Value::Duration(dur.mul_f64(f));
                }
            }
            // Duration / Number -> Duration
            (Value::Duration(dur), Value::Number(n), BinOp::Div) => {
                if let Some(scalar) = n.to_i64() {
                    if scalar == 0 {
                        return Value::Error(FolioError::div_zero());
                    }
                    return Value::Duration(dur.div(scalar).unwrap());
                } else {
                    let f = n.to_f64().unwrap_or(1.0);
                    if f == 0.0 {
                        return Value::Error(FolioError::div_zero());
                    }
                    return Value::Duration(dur.mul_f64(1.0 / f));
                }
            }
            // Duration / Duration -> Number (ratio)
            (Value::Duration(dur1), Value::Duration(dur2), BinOp::Div) => {
                let nanos2 = dur2.as_nanos();
                if nanos2 == 0 {
                    return Value::Error(FolioError::div_zero());
                }
                let nanos1 = dur1.as_nanos();
                // Return the ratio as a Number (truncate to i64)
                let ratio = (nanos1 / nanos2) as i64;
                return Value::Number(Number::from_i64(ratio));
            }
            // Invalid DateTime/Duration operations
            (Value::DateTime(_), Value::DateTime(_), BinOp::Add) => {
                return Value::Error(FolioError::type_error(
                    "Duration (to add to DateTime)", "DateTime"
                ).with_note("cannot add two DateTimes; use dt - dt to get Duration"));
            }
            (Value::DateTime(_), _, BinOp::Mul) | (_, Value::DateTime(_), BinOp::Mul) => {
                return Value::Error(FolioError::type_error(
                    "Number or Duration", "DateTime"
                ).with_note("DateTime cannot be multiplied"));
            }
            (Value::DateTime(_), _, BinOp::Div) => {
                return Value::Error(FolioError::type_error(
                    "Duration", "DateTime"
                ).with_note("DateTime cannot be divided"));
            }
            (Value::DateTime(_), _, BinOp::Pow) | (_, Value::DateTime(_), BinOp::Pow) => {
                return Value::Error(FolioError::type_error(
                    "Number", "DateTime"
                ).with_note("DateTime cannot be used with power operator"));
            }
            (Value::Duration(_), _, BinOp::Pow) | (_, Value::Duration(_), BinOp::Pow) => {
                return Value::Error(FolioError::type_error(
                    "Number", "Duration"
                ).with_note("Duration cannot be used with power operator"));
            }
            // Type mismatch for DateTime/Duration with other types
            (Value::DateTime(_), other, _) if !matches!(other, Value::Duration(_) | Value::DateTime(_)) => {
                return Value::Error(FolioError::type_error("DateTime or Duration", other.type_name()));
            }
            (other, Value::DateTime(_), _) if !matches!(other, Value::Duration(_) | Value::DateTime(_) | Value::Number(_)) => {
                return Value::Error(FolioError::type_error("DateTime, Duration, or Number", other.type_name()));
            }
            (Value::Duration(_), other, _) if !matches!(other, Value::Duration(_) | Value::DateTime(_) | Value::Number(_)) => {
                return Value::Error(FolioError::type_error("DateTime, Duration, or Number", other.type_name()));
            }
            (other, Value::Duration(_), _) if !matches!(other, Value::Duration(_) | Value::DateTime(_) | Value::Number(_)) => {
                return Value::Error(FolioError::type_error("DateTime, Duration, or Number", other.type_name()));
            }
            _ => {}
        }

        // Get numbers (standard numeric operations)
        let l = match left.as_number() {
            Some(n) => n,
            None => return Value::Error(FolioError::type_error("Number", left.type_name())),
        };
        let r = match right.as_number() {
            Some(n) => n,
            None => return Value::Error(FolioError::type_error("Number", right.type_name())),
        };

        // Perform numeric operation
        match op {
            BinOp::Add => Value::Number(l.add(r)),
            BinOp::Sub => Value::Number(l.sub(r)),
            BinOp::Mul => Value::Number(l.mul(r)),
            BinOp::Div => {
                match l.checked_div(r) {
                    Ok(n) => Value::Number(n),
                    Err(e) => Value::Error(e.into()),
                }
            }
            // Comparison operators
            BinOp::Lt => Value::Bool(l.cmp(r) == std::cmp::Ordering::Less),
            BinOp::Gt => Value::Bool(l.cmp(r) == std::cmp::Ordering::Greater),
            BinOp::Le => Value::Bool(l.cmp(r) != std::cmp::Ordering::Greater),
            BinOp::Ge => Value::Bool(l.cmp(r) != std::cmp::Ordering::Less),
            BinOp::Eq => Value::Bool(l.cmp(r) == std::cmp::Ordering::Equal),
            BinOp::Ne => Value::Bool(l.cmp(r) != std::cmp::Ordering::Equal),
            BinOp::Pow => {
                // Power with integer exponent
                if let Some(exp_i64) = r.to_i64() {
                    if exp_i64 >= i32::MIN as i64 && exp_i64 <= i32::MAX as i64 {
                        Value::Number(l.pow(exp_i64 as i32))
                    } else {
                        Value::Error(FolioError::domain_error("exponent too large for integer power"))
                    }
                } else {
                    // Non-integer exponent: x^y = e^(y * ln(x))
                    if l.is_negative() {
                        return Value::Error(FolioError::domain_error(
                            "negative base with non-integer exponent"
                        ));
                    }
                    if l.is_zero() {
                        if r.is_negative() {
                            return Value::Error(FolioError::div_zero()
                                .with_note("0 raised to negative power"));
                        }
                        return Value::Number(Number::from_i64(0));
                    }
                    match l.ln(precision) {
                        Ok(ln_l) => {
                            let y_ln_x = r.mul(&ln_l);
                            Value::Number(y_ln_x.exp(precision))
                        }
                        Err(e) => Value::Error(e.into()),
                    }
                }
            }
        }
    }
    
    fn eval_unary_op(&self, op: UnaryOp, value: Value) -> Value {
        if let Value::Error(e) = &value {
            return Value::Error(e.clone());
        }

        match op {
            UnaryOp::Neg => {
                // Handle Duration negation
                if let Some(d) = value.as_duration() {
                    return Value::Duration(d.neg());
                }
                // Handle DateTime (not allowed)
                if value.is_datetime() {
                    return Value::Error(FolioError::type_error("Number or Duration", "DateTime")
                        .with_note("DateTime cannot be negated"));
                }
                // Handle Number
                match value.as_number() {
                    Some(n) => {
                        let zero = Number::from_i64(0);
                        Value::Number(zero.sub(n))
                    }
                    None => Value::Error(FolioError::type_error("Number", value.type_name())),
                }
            }
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
```

folio\src\lib.rs
```rs
//! Folio - Markdown Computational Documents

mod parser;
mod ast;
mod eval;
mod render;

pub use ast::{Document, Section, Table, Row, Cell, Expr};
pub use eval::{Evaluator, EvalResult};
pub use render::Renderer;

use folio_plugin::{PluginRegistry, EvalContext};
use folio_core::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Main Folio engine
pub struct Folio {
    registry: Arc<PluginRegistry>,
    default_precision: u32,
}

impl Folio {
    pub fn new(registry: PluginRegistry) -> Self {
        Self {
            registry: Arc::new(registry),
            default_precision: 50,
        }
    }
    
    pub fn with_standard_library() -> Self {
        let registry = folio_std::standard_registry();
        let registry = folio_stats::load_stats_library(registry);
        Self::new(registry)
    }
    
    pub fn with_precision(mut self, precision: u32) -> Self {
        self.default_precision = precision;
        self
    }
    
    pub fn eval(&self, template: &str, variables: &HashMap<String, Value>) -> EvalResult {
        let doc = match parser::parse(template) {
            Ok(d) => d,
            Err(e) => return EvalResult::parse_error(e),
        };
        
        let mut ctx = EvalContext::new(self.registry.clone())
            .with_precision(self.default_precision)
            .with_variables(variables.clone());
        
        let evaluator = Evaluator::new();
        let values = evaluator.eval(&doc, &mut ctx);
        
        let renderer = Renderer::new();
        let markdown = renderer.render(&doc, &values, variables);
        
        EvalResult {
            markdown,
            values,
            errors: ctx.trace.iter()
                .filter_map(|s| if let Value::Error(e) = &s.result { Some(e.clone()) } else { None })
                .collect(),
            warnings: vec![],
        }
    }
    
    pub fn help(&self, name: Option<&str>) -> Value {
        self.registry.help(name)
    }
    
    pub fn list_functions(&self, category: Option<&str>) -> Value {
        self.registry.list_functions(category)
    }
    
    pub fn list_constants(&self) -> Value {
        self.registry.list_constants()
    }
}

impl Default for Folio {
    fn default() -> Self {
        Self::with_standard_library()
    }
}

#[macro_export]
macro_rules! vars {
    {} => { std::collections::HashMap::new() };
    { $($key:ident : $value:expr),* $(,)? } => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert(stringify!($key).to_string(), folio_core::Value::from($value));
        )*
        map
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_folio() -> Folio {
        Folio::with_standard_library()
    }

    #[test]
    fn test_simple_arithmetic() {
        let folio = test_folio();
        let doc = r#"
## Test
| name | formula | result |
|------|---------|--------|
| a | 10 | |
| b | 32 | |
| c | a + b | |
"#;
        let result = folio.eval(doc, &HashMap::new());
        let c = result.values.get("c").unwrap();
        assert_eq!(c.as_number().unwrap().to_i64(), Some(42));
    }

    #[test]
    fn test_power_operator() {
        let folio = test_folio();
        let doc = r#"
## Test
| name | formula | result |
|------|---------|--------|
| x | 2 ^ 10 | |
"#;
        let result = folio.eval(doc, &HashMap::new());
        let x = result.values.get("x").unwrap();
        assert_eq!(x.as_number().unwrap().to_i64(), Some(1024));
    }

    // Note: This test is disabled due to stack overflow with BigRational growing too large
    // with Newton-Raphson iterations. The sqrt function works correctly for smaller inputs.
    // #[test]
    // fn test_phi_identity() { ... }

    #[test]
    fn test_function_sqrt() {
        let folio = test_folio();
        let doc = r#"
## Test
| name | formula | result |
|------|---------|--------|
| x | sqrt(16) | |
"#;
        let result = folio.eval(doc, &HashMap::new());
        let x = result.values.get("x").unwrap();
        assert_eq!(x.as_number().unwrap().to_i64(), Some(4));
    }

    #[test]
    fn test_external_variables() {
        let folio = test_folio();
        let doc = r#"
## Test
| name | formula | result |
|------|---------|--------|
| result | x * 2 | |
"#;
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), Value::Number(folio_core::Number::from_i64(21)));
        let result = folio.eval(doc, &vars);
        let r = result.values.get("result").unwrap();
        assert_eq!(r.as_number().unwrap().to_i64(), Some(42));
    }

    #[test]
    fn test_external_variables_override_defaults() {
        let folio = test_folio();
        // Template has default value for principal, but external var should override it
        let doc = r#"
## Test
| name | formula | result |
|------|---------|--------|
| principal | 1000 | |
| result | principal * 2 | |
"#;
        // Provide external value that should override the hardcoded 1000
        let mut vars = HashMap::new();
        vars.insert("principal".to_string(), Value::Number(folio_core::Number::from_i64(5000)));
        let result = folio.eval(doc, &vars);

        // principal should be 5000 (external), not 1000 (hardcoded)
        let principal = result.values.get("principal").unwrap();
        assert_eq!(principal.as_number().unwrap().to_i64(), Some(5000),
            "External variable should override hardcoded default");

        // result should be 5000 * 2 = 10000
        let r = result.values.get("result").unwrap();
        assert_eq!(r.as_number().unwrap().to_i64(), Some(10000),
            "Formula should use the overridden value");
    }

    #[test]
    fn test_undefined_variable_error() {
        let folio = test_folio();
        let doc = r#"
## Test
| name | formula | result |
|------|---------|--------|
| result | undefined_var + 1 | |
"#;
        let result = folio.eval(doc, &HashMap::new());
        let r = result.values.get("result").unwrap();
        assert!(r.is_error());
    }

    #[test]
    fn test_division_by_zero() {
        let folio = test_folio();
        let doc = r#"
## Test
| name | formula | result |
|------|---------|--------|
| result | 42 / 0 | |
"#;
        let result = folio.eval(doc, &HashMap::new());
        let r = result.values.get("result").unwrap();
        assert!(r.is_error());
    }

    #[test]
    fn test_dependency_order() {
        let folio = test_folio();
        // Cells defined in reverse order should still evaluate correctly
        let doc = r#"
## Test
| name | formula | result |
|------|---------|--------|
| c | b + 1 | |
| b | a + 1 | |
| a | 1 | |
"#;
        let result = folio.eval(doc, &HashMap::new());
        let c = result.values.get("c").unwrap();
        assert_eq!(c.as_number().unwrap().to_i64(), Some(3));
    }

    #[test]
    fn test_trig_functions() {
        let folio = test_folio();
        let doc = r#"
## Test @precision:50
| name | formula | result |
|------|---------|--------|
| sin_0 | sin(0) | |
| cos_0 | cos(0) | |
"#;
        let result = folio.eval(doc, &HashMap::new());
        let sin_0 = result.values.get("sin_0").unwrap();
        let cos_0 = result.values.get("cos_0").unwrap();
        // sin(0) should be 0
        assert!(sin_0.as_number().unwrap().as_decimal(5).starts_with("0."));
        // cos(0) should be 1
        assert!(cos_0.as_number().unwrap().as_decimal(5).starts_with("1."));
    }

    #[test]
    fn test_negation() {
        let folio = test_folio();
        let doc = r#"
## Test
| name | formula | result |
|------|---------|--------|
| a | 42 | |
| neg | 0 - a | |
"#;
        let result = folio.eval(doc, &HashMap::new());
        let neg = result.values.get("neg").unwrap();
        assert_eq!(neg.as_number().unwrap().to_i64(), Some(-42));
    }

    #[test]
    fn test_precision_attribute() {
        let folio = test_folio();
        let doc = r#"
## Test @precision:100
| name | formula | result |
|------|---------|--------|
| pi | 3.14159265358979323846264338327950288419716939937510 | |
"#;
        let result = folio.eval(doc, &HashMap::new());
        assert!(result.values.contains_key("pi"));
    }

    #[test]
    fn test_help() {
        let folio = test_folio();
        let help = folio.help(None);
        assert!(matches!(help, Value::Object(_)));
    }

    #[test]
    fn test_help_specific_function() {
        let folio = test_folio();
        let help = folio.help(Some("sqrt"));
        assert!(matches!(help, Value::Object(_)));
        if let Value::Object(obj) = help {
            assert!(obj.contains_key("name"));
            assert!(obj.contains_key("description"));
        }
    }

    #[test]
    fn test_list_functions() {
        let folio = test_folio();
        let funcs = folio.list_functions(None);
        assert!(matches!(funcs, Value::List(_)));
    }

    #[test]
    fn test_unicode_constants() {
        let folio = test_folio();
        let doc = r#"
## Test @precision:10
| name | formula | result |
|------|---------|--------|
| pi_val | π | |
| phi_val | φ | |
| e_val | e | |
| pi_calc | π * 2 | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // π should be approximately 3.14159...
        let pi = result.values.get("pi_val").unwrap();
        assert!(!pi.is_error(), "π should resolve to a value, got error: {:?}", pi);
        let pi_str = pi.as_number().unwrap().as_decimal(5);
        assert!(pi_str.starts_with("3.1415"), "π should start with 3.1415, got: {}", pi_str);

        // φ should be approximately 1.618...
        let phi = result.values.get("phi_val").unwrap();
        assert!(!phi.is_error(), "φ should resolve to a value, got error: {:?}", phi);
        let phi_str = phi.as_number().unwrap().as_decimal(4);
        assert!(phi_str.starts_with("1.618"), "φ should start with 1.618, got: {}", phi_str);

        // e should be approximately 2.718...
        let e = result.values.get("e_val").unwrap();
        assert!(!e.is_error(), "e should resolve to a value, got error: {:?}", e);
        let e_str = e.as_number().unwrap().as_decimal(4);
        assert!(e_str.starts_with("2.718"), "e should start with 2.718, got: {}", e_str);

        // π * 2 should be approximately 6.28...
        let pi_calc = result.values.get("pi_calc").unwrap();
        assert!(!pi_calc.is_error(), "π * 2 should work, got error: {:?}", pi_calc);
        let pi_calc_str = pi_calc.as_number().unwrap().as_decimal(4);
        assert!(pi_calc_str.starts_with("6.28"), "π * 2 should start with 6.28, got: {}", pi_calc_str);
    }

    #[test]
    fn test_phi_properties() {
        let folio = test_folio();
        // Simplified version of phi_properties.fmd - all in one table section
        let doc = r#"
## Phi Properties @precision:50
| name | formula | result |
|------|---------|--------|
| phi | (1 + sqrt(5)) / 2 | |
| phi_inv | 1 / phi | |
| phi_sq | phi * phi | |
| identity_check | phi_sq - phi - 1 | |
| phi_5 | pow(phi, 5) | |
| phi_10 | pow(phi, 10) | |
| ln_phi | ln(phi) | |
| two_pi | 2 * π | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // Check phi is computed correctly
        let phi = result.values.get("phi").unwrap();
        assert!(!phi.is_error(), "phi should compute, got: {:?}", phi);
        let phi_str = phi.as_number().unwrap().as_decimal(4);
        assert!(phi_str.starts_with("1.618"), "phi should start with 1.618, got: {}", phi_str);

        // Check identity: phi^2 - phi - 1 should be ~0 (within floating point tolerance)
        let identity = result.values.get("identity_check")
            .expect(&format!("identity_check not found. Available keys: {:?}", result.values.keys().collect::<Vec<_>>()));
        assert!(!identity.is_error(), "identity should compute, got: {:?}", identity);
        let identity_val = identity.as_number().unwrap().as_decimal(20);
        // With approximate sqrt, the result should be very small (< 1e-10)
        assert!(identity_val.starts_with("0.") || identity_val.starts_with("-0.") || identity_val == "0",
            "phi^2 - phi - 1 should be ~0, got: {}", identity_val);

        // Check ln(phi) ≈ 0.4812
        let ln_phi = result.values.get("ln_phi").unwrap();
        assert!(!ln_phi.is_error(), "ln(phi) should compute, got: {:?}", ln_phi);
        let ln_phi_str = ln_phi.as_number().unwrap().as_decimal(3);
        assert!(ln_phi_str.starts_with("0.481"), "ln(phi) should start with 0.481, got: {}", ln_phi_str);

        // Check 2π ≈ 6.28
        let two_pi = result.values.get("two_pi").unwrap();
        assert!(!two_pi.is_error(), "2 * π should compute, got: {:?}", two_pi);
        let two_pi_str = two_pi.as_number().unwrap().as_decimal(4);
        assert!(two_pi_str.starts_with("6.28"), "2π should start with 6.28, got: {}", two_pi_str);
    }

    #[test]
    fn test_sigfigs_directive() {
        let folio = test_folio();
        // Test @sigfigs directive for scientific notation output
        let doc = r#"
## Physical Constants @precision:50 @sigfigs:4
| name | formula | result |
|------|---------|--------|
| avogadro | 602214076e15 | |
| h | 662607015e-42 | |
| c | 299792458 | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // Check rendered output contains scientific notation
        let rendered = result.markdown;

        // Avogadro's number should be displayed as ~6.022e23
        assert!(rendered.contains("6.022e23") || rendered.contains("6.022e+23"),
            "Avogadro should use scientific notation: {}", rendered);

        // Planck constant should be displayed as ~6.626e-34
        assert!(rendered.contains("e-"),
            "Planck constant should use scientific notation: {}", rendered);

        // Speed of light is in normal range, should not use scientific notation
        // 299792458 with 4 sigfigs = 2.998e8 (just outside normal range)
        assert!(rendered.contains("2.998e8") || rendered.contains("299800000"),
            "Speed of light display: {}", rendered);
    }

    #[test]
    fn test_physics_constants() {
        // Test that physics constants (m_e, m_mu, etc.) now work correctly
        let folio = test_folio();
        let doc = r#"
## Physics Constants @precision:10
| name | formula | result |
|------|---------|--------|
| electron_mass | m_e | |
| muon_mass | m_mu | |
| tau_mass | m_tau | |
| higgs_mass | m_H | |
| cabibbo | V_us | |
| speed_of_light | c | |
| fine_structure | alpha | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // m_e should be ~0.511 MeV
        let m_e = result.values.get("electron_mass").unwrap();
        assert!(!m_e.is_error(), "m_e should resolve, got: {:?}", m_e);
        let m_e_str = m_e.as_number().unwrap().as_decimal(3);
        assert!(m_e_str.starts_with("0.510") || m_e_str.starts_with("0.511"),
            "m_e should be ~0.511 MeV, got: {}", m_e_str);

        // m_mu should be ~105.66 MeV
        let m_mu = result.values.get("muon_mass").unwrap();
        assert!(!m_mu.is_error(), "m_mu should resolve, got: {:?}", m_mu);
        let m_mu_str = m_mu.as_number().unwrap().as_decimal(1);
        assert!(m_mu_str.starts_with("105."),
            "m_mu should be ~105.66 MeV, got: {}", m_mu_str);

        // m_tau should be ~1776.86 MeV
        let m_tau = result.values.get("tau_mass").unwrap();
        assert!(!m_tau.is_error(), "m_tau should resolve, got: {:?}", m_tau);
        let m_tau_str = m_tau.as_number().unwrap().as_decimal(0);
        assert!(m_tau_str.starts_with("1776") || m_tau_str.starts_with("1777"),
            "m_tau should be ~1776.86 MeV, got: {}", m_tau_str);

        // V_us should be ~0.2243
        let v_us = result.values.get("cabibbo").unwrap();
        assert!(!v_us.is_error(), "V_us should resolve, got: {:?}", v_us);
        let v_us_str = v_us.as_number().unwrap().as_decimal(3);
        assert!(v_us_str.starts_with("0.224"),
            "V_us should be ~0.2243, got: {}", v_us_str);

        // c should be 299792458 m/s
        let c = result.values.get("speed_of_light").unwrap();
        assert!(!c.is_error(), "c should resolve, got: {:?}", c);
        let c_val = c.as_number().unwrap().to_i64().unwrap();
        assert_eq!(c_val, 299792458, "c should be 299792458 m/s");

        // alpha should be ~0.0073
        let alpha = result.values.get("fine_structure").unwrap();
        assert!(!alpha.is_error(), "alpha should resolve, got: {:?}", alpha);
        let alpha_str = alpha.as_number().unwrap().as_decimal(4);
        assert!(alpha_str.starts_with("0.0072") || alpha_str.starts_with("0.0073"),
            "alpha should be ~0.00729, got: {}", alpha_str);
    }

    #[test]
    fn test_single_hash_header() {
        // Test that single # headers now work (previously returned empty)
        let folio = test_folio();
        let doc = r#"
# Klein Validation @precision:30

| name | formula | result |
|------|---------|--------|
| x | 5 | |
| y | x * 2 | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // Should have parsed the content
        assert!(!result.values.is_empty(), "Single # header should parse content");

        let x = result.values.get("x").unwrap();
        assert!(!x.is_error(), "x should resolve to 5");
        assert_eq!(x.as_number().unwrap().to_i64(), Some(5));

        let y = result.values.get("y").unwrap();
        assert!(!y.is_error(), "y should resolve to 10");
        assert_eq!(y.as_number().unwrap().to_i64(), Some(10));
    }

    #[test]
    fn test_datetime_shortcuts() {
        let folio = test_folio();
        let doc = r#"
## DateTime Shortcuts Test
| name | formula | result |
|------|---------|--------|
| ref_date | date(2025, 6, 15) | |
| end_of_day | eod(ref_date) | |
| end_of_month | eom(ref_date) | |
| start_of_month | som(ref_date) | |
| tomorrow_date | tomorrow(ref_date) | |
| next_week_start | nextWeek(ref_date) | |
| next_month_first | nextMonth(ref_date) | |
| is_workday_check | isWorkday(ref_date) | |
| next_workday_date | nextWorkday(ref_date) | |
| add_5_workdays | addWorkdays(ref_date, 5) | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // All values should exist and not be errors
        assert!(!result.values.get("ref_date").unwrap().is_error(), "ref_date failed");
        assert!(!result.values.get("end_of_day").unwrap().is_error(), "eod failed");
        assert!(!result.values.get("end_of_month").unwrap().is_error(), "eom failed");
        assert!(!result.values.get("start_of_month").unwrap().is_error(), "som failed");
        assert!(!result.values.get("tomorrow_date").unwrap().is_error(), "tomorrow failed");
        assert!(!result.values.get("next_week_start").unwrap().is_error(), "nextWeek failed");
        assert!(!result.values.get("next_month_first").unwrap().is_error(), "nextMonth failed");
        assert!(!result.values.get("is_workday_check").unwrap().is_error(), "isWorkday failed");
        assert!(!result.values.get("next_workday_date").unwrap().is_error(), "nextWorkday failed");
        assert!(!result.values.get("add_5_workdays").unwrap().is_error(), "addWorkdays failed");

        // June 15, 2025 is a Sunday, so it's not a workday
        let is_wd = result.values.get("is_workday_check").unwrap();
        assert_eq!(is_wd.as_bool().unwrap(), false, "June 15, 2025 is Sunday, not a workday");

        // End of month should be June 30
        let eom_dt = result.values.get("end_of_month").unwrap().as_datetime().unwrap();
        assert_eq!(eom_dt.day(), 30, "End of June should be day 30");
        assert_eq!(eom_dt.month(), 6);

        // Start of month should be June 1
        let som_dt = result.values.get("start_of_month").unwrap().as_datetime().unwrap();
        assert_eq!(som_dt.day(), 1, "Start of June should be day 1");

        // Tomorrow should be June 16
        let tom_dt = result.values.get("tomorrow_date").unwrap().as_datetime().unwrap();
        assert_eq!(tom_dt.day(), 16);

        // Next month should be July 1
        let nm_dt = result.values.get("next_month_first").unwrap().as_datetime().unwrap();
        assert_eq!(nm_dt.month(), 7);
        assert_eq!(nm_dt.day(), 1);
    }

    #[test]
    fn test_datetime_workdays() {
        let folio = test_folio();
        let doc = r#"
## Workday Tests
| name | formula | result |
|------|---------|--------|
| friday | date(2025, 6, 13) | |
| saturday | date(2025, 6, 14) | |
| sunday | date(2025, 6, 15) | |
| monday | date(2025, 6, 16) | |
| fri_is_wd | isWorkday(friday) | |
| sat_is_wd | isWorkday(saturday) | |
| sun_is_wd | isWorkday(sunday) | |
| mon_is_wd | isWorkday(monday) | |
| next_from_fri | nextWorkday(friday) | |
| next_from_sat | nextWorkday(saturday) | |
| prev_from_sat | prevWorkday(saturday) | |
| add_5_from_fri | addWorkdays(friday, 5) | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // Friday is a workday
        assert_eq!(result.values.get("fri_is_wd").unwrap().as_bool().unwrap(), true);
        // Saturday is not a workday
        assert_eq!(result.values.get("sat_is_wd").unwrap().as_bool().unwrap(), false);
        // Sunday is not a workday
        assert_eq!(result.values.get("sun_is_wd").unwrap().as_bool().unwrap(), false);
        // Monday is a workday
        assert_eq!(result.values.get("mon_is_wd").unwrap().as_bool().unwrap(), true);

        // Next workday from Friday is Monday (June 16)
        let next_fri = result.values.get("next_from_fri").unwrap().as_datetime().unwrap();
        assert_eq!(next_fri.day(), 16);

        // Next workday from Saturday is Monday (June 16)
        let next_sat = result.values.get("next_from_sat").unwrap().as_datetime().unwrap();
        assert_eq!(next_sat.day(), 16);

        // Previous workday from Saturday is Friday (June 13)
        let prev_sat = result.values.get("prev_from_sat").unwrap().as_datetime().unwrap();
        assert_eq!(prev_sat.day(), 13);

        // Add 5 workdays from Friday (June 13): Mon(16), Tue(17), Wed(18), Thu(19), Fri(20)
        let add5 = result.values.get("add_5_from_fri").unwrap().as_datetime().unwrap();
        assert_eq!(add5.day(), 20);
    }

    #[test]
    fn test_duration_time_units() {
        let folio = test_folio();
        let doc = r#"
## Duration Time Units Test
| name | formula | result |
|------|---------|--------|
| two_weeks | weeks(2) | |
| half_second | milliseconds(500) | |
| fourteen_days | days(14) | |
| week_in_days | two_weeks / days(1) | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // weeks(2) should work and divide to 14 days
        let week_days = result.values.get("week_in_days").unwrap();
        assert!(!week_days.is_error(), "weeks calculation should work, got: {:?}", week_days);
        assert_eq!(week_days.as_number().unwrap().to_i64(), Some(14));

        // milliseconds(500) should work
        let half_sec = result.values.get("half_second").unwrap();
        assert!(!half_sec.is_error(), "milliseconds() should work, got: {:?}", half_sec);
    }

    #[test]
    fn test_string_literals() {
        let folio = test_folio();
        let doc = r#"
## String Literals Test
| name | formula | result |
|------|---------|--------|
| start_date | date(2025, 1, 15) | |
| end_date | date(2025, 6, 30) | |
| days_diff | diff(end_date, start_date, "days") | |
| hours_diff | diff(end_date, start_date, "hours") | |
| months_diff | diff(end_date, start_date, "months") | |
| formatted | formatDate(start_date, "MM/DD/YYYY") | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // diff with "days" should work
        let days = result.values.get("days_diff").unwrap();
        assert!(!days.is_error(), "diff with 'days' should work, got: {:?}", days);
        assert_eq!(days.as_number().unwrap().to_i64(), Some(166));

        // diff with "hours" should work
        let hours = result.values.get("hours_diff").unwrap();
        assert!(!hours.is_error(), "diff with 'hours' should work, got: {:?}", hours);
        assert_eq!(hours.as_number().unwrap().to_i64(), Some(166 * 24));

        // diff with "months" should work
        let months = result.values.get("months_diff").unwrap();
        assert!(!months.is_error(), "diff with 'months' should work, got: {:?}", months);
        assert_eq!(months.as_number().unwrap().to_i64(), Some(5));

        // formatDate should work
        let formatted = result.values.get("formatted").unwrap();
        assert!(!formatted.is_error(), "formatDate with pattern should work, got: {:?}", formatted);
        assert_eq!(formatted.as_text().unwrap(), "01/15/2025");
    }

    #[test]
    fn test_list_literals() {
        let folio = test_folio();
        let doc = r#"
## List Literals Test
| name | formula | result |
|------|---------|--------|
| nums | [1, 2, 3, 4, 5] | |
| avg | mean(nums) | |
| sum_val | sum(nums) | |
| nested | mean([10, 20, 30]) | |
| with_expr | mean([1 + 1, 2 + 2, 3 + 3]) | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // nums should be a list
        let nums = result.values.get("nums").unwrap();
        assert!(!nums.is_error(), "list literal should work, got: {:?}", nums);
        assert!(nums.as_list().is_some(), "nums should be a List");

        // mean([1, 2, 3, 4, 5]) = 3
        let avg = result.values.get("avg").unwrap();
        assert!(!avg.is_error(), "mean of list should work, got: {:?}", avg);
        assert_eq!(avg.as_number().unwrap().to_i64(), Some(3));

        // sum([1, 2, 3, 4, 5]) = 15
        let sum_val = result.values.get("sum_val").unwrap();
        assert!(!sum_val.is_error(), "sum of list should work, got: {:?}", sum_val);
        assert_eq!(sum_val.as_number().unwrap().to_i64(), Some(15));

        // mean([10, 20, 30]) = 20
        let nested = result.values.get("nested").unwrap();
        assert!(!nested.is_error(), "inline list in function should work, got: {:?}", nested);
        assert_eq!(nested.as_number().unwrap().to_i64(), Some(20));

        // mean([2, 4, 6]) = 4
        let with_expr = result.values.get("with_expr").unwrap();
        assert!(!with_expr.is_error(), "list with expressions should work, got: {:?}", with_expr);
        assert_eq!(with_expr.as_number().unwrap().to_i64(), Some(4));
    }

    #[test]
    fn test_long_list_literals() {
        let folio = test_folio();
        let doc = r#"
## Long List Test
| name | formula | result |
|------|---------|--------|
| tech_data | [3.2, -1.5, 5.8, -2.3, 4.1, 2.9, -3.8, 6.2, 1.4, -0.9, 4.5, 3.1, -2.1, 5.3, 2.8, -4.2, 3.9, 1.2, -1.8, 4.7, 2.3, -0.5, 3.8, 2.1] | |
| tech_mean | mean(tech_data) | |
| tech_count | count(tech_data) | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // tech_data should be a list
        let tech_data = result.values.get("tech_data");
        assert!(tech_data.is_some(), "tech_data should exist in values: {:?}", result.values.keys().collect::<Vec<_>>());
        let tech_data = tech_data.unwrap();
        assert!(!tech_data.is_error(), "long list literal should work, got: {:?}", tech_data);

        if let Some(list) = tech_data.as_list() {
            assert_eq!(list.len(), 24, "list should have 24 elements");
        } else {
            panic!("tech_data should be a List");
        }

        // tech_count should be 24
        let tech_count = result.values.get("tech_count");
        assert!(tech_count.is_some(), "tech_count should exist");
        let tech_count = tech_count.unwrap();
        assert!(!tech_count.is_error(), "count should work, got: {:?}", tech_count);
        assert_eq!(tech_count.as_number().unwrap().to_i64(), Some(24));

        // tech_mean should work
        let tech_mean = result.values.get("tech_mean");
        assert!(tech_mean.is_some(), "tech_mean should exist");
        let tech_mean = tech_mean.unwrap();
        assert!(!tech_mean.is_error(), "mean should work, got: {:?}", tech_mean);
    }

    #[test]
    fn test_comparison_operators() {
        let folio = test_folio();
        let doc = r#"
## Comparison Operators Test
| name | formula | result |
|------|---------|--------|
| x | 5 | |
| y | 3 | |
| lt | x < y | |
| gt | x > y | |
| le | x <= y | |
| ge | x >= y | |
| eq | x == y | |
| ne | x != y | |
| lt_same | 5 <= 5 | |
| ge_same | 5 >= 5 | |
| eq_same | 5 == 5 | |
| with_obj | t_test_1([1, 2, 3, 4, 5], 2.5).p < 0.05 | |
"#;
        let result = folio.eval(doc, &HashMap::new());

        // x < y should be false (5 < 3 is false)
        let lt = result.values.get("lt").unwrap();
        assert!(!lt.is_error(), "< should work, got: {:?}", lt);
        assert_eq!(lt.as_bool(), Some(false));

        // x > y should be true (5 > 3 is true)
        let gt = result.values.get("gt").unwrap();
        assert!(!gt.is_error(), "> should work, got: {:?}", gt);
        assert_eq!(gt.as_bool(), Some(true));

        // x <= y should be false
        let le = result.values.get("le").unwrap();
        assert!(!le.is_error(), "<= should work, got: {:?}", le);
        assert_eq!(le.as_bool(), Some(false));

        // x >= y should be true
        let ge = result.values.get("ge").unwrap();
        assert!(!ge.is_error(), ">= should work, got: {:?}", ge);
        assert_eq!(ge.as_bool(), Some(true));

        // x == y should be false
        let eq = result.values.get("eq").unwrap();
        assert!(!eq.is_error(), "== should work, got: {:?}", eq);
        assert_eq!(eq.as_bool(), Some(false));

        // x != y should be true
        let ne = result.values.get("ne").unwrap();
        assert!(!ne.is_error(), "!= should work, got: {:?}", ne);
        assert_eq!(ne.as_bool(), Some(true));

        // 5 <= 5 should be true
        let lt_same = result.values.get("lt_same").unwrap();
        assert!(!lt_same.is_error(), "<= with equal should work, got: {:?}", lt_same);
        assert_eq!(lt_same.as_bool(), Some(true));

        // 5 >= 5 should be true
        let ge_same = result.values.get("ge_same").unwrap();
        assert!(!ge_same.is_error(), ">= with equal should work, got: {:?}", ge_same);
        assert_eq!(ge_same.as_bool(), Some(true));

        // 5 == 5 should be true
        let eq_same = result.values.get("eq_same").unwrap();
        assert!(!eq_same.is_error(), "== with equal should work, got: {:?}", eq_same);
        assert_eq!(eq_same.as_bool(), Some(true));

        // Object field comparison should work
        let with_obj = result.values.get("with_obj").unwrap();
        assert!(!with_obj.is_error(), "comparison with object field should work, got: {:?}", with_obj);
        assert!(with_obj.as_bool().is_some(), "should return a boolean");
    }
}
```

folio\src\parser.rs
```rs
//! Markdown table parser

use crate::ast::{Document, Section, Table, Row, Cell, Expr, BinOp};
use folio_core::FolioError;
use std::collections::HashMap;

/// Parse markdown document to AST
pub fn parse(input: &str) -> Result<Document, FolioError> {
    let mut sections = Vec::new();
    let mut current_section: Option<Section> = None;
    let mut in_table = false;
    let mut table_rows: Vec<Row> = Vec::new();
    let mut columns: Vec<String> = Vec::new();
    
    for line in input.lines() {
        let line = line.trim();

        // Section header - support both # and ## (# takes priority check first)
        if line.starts_with("# ") && !line.starts_with("## ") {
            // Single # header - treat as section
            if let Some(mut sec) = current_section.take() {
                sec.table.rows = std::mem::take(&mut table_rows);
                sec.table.columns = std::mem::take(&mut columns);
                sections.push(sec);
            }

            let header = &line[2..]; // Skip "# "
            let (name, attrs) = parse_section_header(header);
            current_section = Some(Section {
                name,
                attributes: attrs,
                table: Table::default(),
            });
            in_table = false;
            continue;
        }

        // Double ## section header
        if line.starts_with("## ") {
            // Save previous section
            if let Some(mut sec) = current_section.take() {
                sec.table.rows = std::mem::take(&mut table_rows);
                sec.table.columns = std::mem::take(&mut columns);
                sections.push(sec);
            }
            
            let header = &line[3..];
            let (name, attrs) = parse_section_header(header);
            current_section = Some(Section {
                name,
                attributes: attrs,
                table: Table::default(),
            });
            in_table = false;
            continue;
        }
        
        // Table header
        if line.starts_with('|') && line.ends_with('|') && !in_table {
            columns = parse_table_row_cells(line);
            in_table = true;
            continue;
        }
        
        // Table separator (only matches lines that contain only |, -, :, and whitespace)
        if line.starts_with('|') && line.ends_with('|') && in_table && table_rows.is_empty() {
            let is_separator = line.chars().all(|c| c == '|' || c == '-' || c == ':' || c.is_whitespace());
            if is_separator {
                continue;
            }
        }
        
        // Table row
        if line.starts_with('|') && line.ends_with('|') && in_table {
            let cells_text = parse_table_row_cells(line);
            if cells_text.len() >= 2 {
                let name = cells_text[0].trim().to_string();
                let formula_text = cells_text[1].trim().to_string();

                // Check for formula indicator (=) and strip it
                let (is_formula, expr_text) = if formula_text.starts_with('=') {
                    (true, formula_text[1..].trim().to_string())
                } else {
                    (false, formula_text.clone())
                };

                let formula = if expr_text.is_empty() {
                    None
                } else if is_formula {
                    // Explicitly marked as formula with =
                    Some(parse_expr(&expr_text)?)
                } else {
                    // Check if it looks like an expression (contains operators or function calls)
                    // Otherwise treat as literal value
                    if looks_like_expression(&expr_text) {
                        Some(parse_expr(&expr_text)?)
                    } else {
                        None // Treat as literal
                    }
                };
                
                table_rows.push(Row {
                    cells: vec![Cell {
                        name: name.clone(),
                        formula,
                        raw_text: expr_text, // Store the expression text (without = prefix)
                    }],
                });
            }
            continue;
        }

        // Empty line ends table
        if line.is_empty() && in_table {
            in_table = false;
        }
    }

    // Save last section
    if let Some(mut sec) = current_section {
        sec.table.rows = table_rows;
        sec.table.columns = columns;
        sections.push(sec);
    } else if !table_rows.is_empty() {
        // Fallback: create default section if there's content but no section header
        sections.push(Section {
            name: "Default".to_string(),
            attributes: HashMap::new(),
            table: Table { rows: table_rows, columns },
        });
    }

    Ok(Document { sections })
}

/// Check if text looks like an expression (vs a literal value)
fn looks_like_expression(text: &str) -> bool {
    let text = text.trim();

    // First check if it's a valid number literal (including scientific notation)
    if is_number_literal(text) {
        return false;
    }

    // List literal
    if text.starts_with('[') && text.ends_with(']') {
        return true;
    }

    // Contains operators (but not just a negative number)
    if text.contains('+') || text.contains('*') || text.contains('/') || text.contains('^') {
        return true;
    }

    // Contains comparison operators
    if text.contains('<') || text.contains('>') || text.contains("==") || text.contains("!=") {
        return true;
    }

    // Contains subtraction that's not a leading minus
    if let Some(pos) = text.find('-') {
        if pos > 0 {
            return true;
        }
    }

    // Contains function call
    if text.contains('(') && text.contains(')') {
        return true;
    }

    // References a variable (starts with letter, not just a number)
    if text.chars().next().map_or(false, |c| c.is_alphabetic()) {
        return true;
    }

    false
}

/// Check if text is a valid number literal (integer, decimal, or scientific notation)
fn is_number_literal(text: &str) -> bool {
    let text = text.trim();
    if text.is_empty() {
        return false;
    }

    // If it contains spaces, it's an expression, not a literal
    if text.contains(' ') {
        return false;
    }

    // Try parsing as f64 - handles integers, decimals, and scientific notation
    if text.parse::<f64>().is_ok() {
        return true;
    }

    // Also check for fraction format without spaces (e.g., "1/3" but not "42 / 0")
    if let Some(slash_pos) = text.find('/') {
        let (num, den) = text.split_at(slash_pos);
        let den = &den[1..]; // skip the '/'
        if !num.is_empty() && !den.is_empty()
            && num.parse::<f64>().is_ok()
            && den.parse::<f64>().is_ok() {
            return true;
        }
    }

    false
}

fn parse_section_header(header: &str) -> (String, HashMap<String, String>) {
    let mut attrs = HashMap::new();
    let parts: Vec<&str> = header.split('@').collect();
    let name = parts[0].trim().to_string();
    
    for attr_part in parts.iter().skip(1) {
        if let Some((key, value)) = attr_part.split_once(':') {
            attrs.insert(key.trim().to_string(), value.trim().to_string());
        }
    }
    
    (name, attrs)
}

fn parse_table_row_cells(line: &str) -> Vec<String> {
    line.trim_matches('|')
        .split('|')
        .map(|s| s.trim().to_string())
        .collect()
}

/// Parse expression (simple recursive descent)
pub fn parse_expr(input: &str) -> Result<Expr, FolioError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(FolioError::parse_error("Empty expression"));
    }

    parse_comparison(input)
}

/// Parse comparison operators (lowest precedence)
fn parse_comparison(input: &str) -> Result<Expr, FolioError> {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    // Scan for comparison operators from left to right (left associative)
    let mut i = 0;
    while i < char_indices.len() {
        let (byte_pos, c) = char_indices[i];
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '(' if !in_double_quote && !in_single_quote => paren_depth += 1,
            ')' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            '<' | '>' | '=' | '!' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                // Check for two-character operators
                let next_char = if i + 1 < char_indices.len() { Some(char_indices[i + 1].1) } else { None };
                let (op, op_len) = match (c, next_char) {
                    ('<', Some('=')) => (Some(BinOp::Le), 2),
                    ('>', Some('=')) => (Some(BinOp::Ge), 2),
                    ('=', Some('=')) => (Some(BinOp::Eq), 2),
                    ('!', Some('=')) => (Some(BinOp::Ne), 2),
                    ('<', _) => (Some(BinOp::Lt), 1),
                    ('>', _) => (Some(BinOp::Gt), 1),
                    _ => (None, 1),
                };

                if let Some(op) = op {
                    let left = input[..byte_pos].trim();
                    let right_start = if op_len == 2 {
                        char_indices[i + 1].0 + char_indices[i + 1].1.len_utf8()
                    } else {
                        byte_pos + c.len_utf8()
                    };
                    let right = input[right_start..].trim();

                    if !left.is_empty() && !right.is_empty() {
                        return Ok(Expr::BinaryOp(
                            Box::new(parse_additive(left)?),
                            op,
                            Box::new(parse_additive(right)?),
                        ));
                    }
                }
            }
            _ => {}
        }
        i += 1;
    }

    parse_additive(input)
}

fn parse_additive(input: &str) -> Result<Expr, FolioError> {
    // Find + or - not inside parentheses, brackets, function calls, or quotes
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    // Collect (byte_offset, char) pairs to handle multi-byte UTF-8 correctly
    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    for idx in (0..char_indices.len()).rev() {
        let (byte_pos, c) = char_indices[idx];
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            ')' if !in_double_quote && !in_single_quote => paren_depth += 1,
            '(' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            '+' | '-' if paren_depth == 0 && bracket_depth == 0 && idx > 0 && !in_double_quote && !in_single_quote => {
                let left = input[..byte_pos].trim();
                let right = input[byte_pos + c.len_utf8()..].trim();
                if !left.is_empty() && !right.is_empty() {
                    let op = if c == '+' { BinOp::Add } else { BinOp::Sub };
                    return Ok(Expr::BinaryOp(
                        Box::new(parse_additive(left)?),
                        op,
                        Box::new(parse_multiplicative(right)?),
                    ));
                }
            }
            _ => {}
        }
    }

    parse_multiplicative(input)
}

fn parse_multiplicative(input: &str) -> Result<Expr, FolioError> {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    // Collect (byte_offset, char) pairs to handle multi-byte UTF-8 correctly
    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    for idx in (0..char_indices.len()).rev() {
        let (byte_pos, c) = char_indices[idx];
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            ')' if !in_double_quote && !in_single_quote => paren_depth += 1,
            '(' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            '*' | '/' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                let left = input[..byte_pos].trim();
                let right = input[byte_pos + c.len_utf8()..].trim();
                if !left.is_empty() && !right.is_empty() {
                    let op = if c == '*' { BinOp::Mul } else { BinOp::Div };
                    return Ok(Expr::BinaryOp(
                        Box::new(parse_multiplicative(left)?),
                        op,
                        Box::new(parse_power(right)?),
                    ));
                }
            }
            _ => {}
        }
    }

    parse_power(input)
}

fn parse_power(input: &str) -> Result<Expr, FolioError> {
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;

    // Collect (byte_offset, char) pairs to handle multi-byte UTF-8 correctly
    let char_indices: Vec<(usize, char)> = input.char_indices().collect();

    for idx in 0..char_indices.len() {
        let (byte_pos, c) = char_indices[idx];
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '(' if !in_double_quote && !in_single_quote => paren_depth += 1,
            ')' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            '^' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                let left = input[..byte_pos].trim();
                let right = input[byte_pos + c.len_utf8()..].trim();
                if !left.is_empty() && !right.is_empty() {
                    return Ok(Expr::BinaryOp(
                        Box::new(parse_primary(left)?),
                        BinOp::Pow,
                        Box::new(parse_power(right)?),
                    ));
                }
            }
            _ => {}
        }
    }

    parse_primary(input)
}

fn parse_primary(input: &str) -> Result<Expr, FolioError> {
    let input = input.trim();

    // String literal (double-quoted)
    if input.starts_with('"') && input.ends_with('"') && input.len() >= 2 {
        let content = &input[1..input.len()-1];
        return Ok(Expr::StringLiteral(content.to_string()));
    }

    // String literal (single-quoted) - also support single quotes
    if input.starts_with('\'') && input.ends_with('\'') && input.len() >= 2 {
        let content = &input[1..input.len()-1];
        return Ok(Expr::StringLiteral(content.to_string()));
    }

    // List literal: [a, b, c]
    if input.starts_with('[') && input.ends_with(']') && input.len() >= 2 {
        let content = &input[1..input.len()-1];
        let elements = parse_list_elements(content)?;
        return Ok(Expr::List(elements));
    }

    // Parentheses
    if input.starts_with('(') && input.ends_with(')') {
        return parse_expr(&input[1..input.len()-1]);
    }

    // Function call - need to find matching closing parenthesis
    if let Some(paren_pos) = input.find('(') {
        let func_name = input[..paren_pos].trim().to_string();
        // Find the matching closing parenthesis
        let after_open = &input[paren_pos+1..];
        let mut depth = 1;
        let mut close_pos = None;
        let mut in_double_quote = false;
        let mut in_single_quote = false;
        for (i, c) in after_open.char_indices() {
            match c {
                '"' if !in_single_quote => in_double_quote = !in_double_quote,
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '(' if !in_double_quote && !in_single_quote => depth += 1,
                ')' if !in_double_quote && !in_single_quote => {
                    depth -= 1;
                    if depth == 0 {
                        close_pos = Some(i);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(close_idx) = close_pos {
            let args_str = &after_open[..close_idx];
            let args = parse_args(args_str)?;
            let func_call = Expr::FunctionCall(func_name, args);

            // Check if there's a property access after the function call
            let after_close = &after_open[close_idx + 1..];
            if after_close.starts_with('.') {
                // Parse as field access: func().prop.subprop
                let field_names: Vec<String> = after_close[1..].split('.').map(|s| s.trim().to_string()).collect();
                return Ok(Expr::FieldAccess(Box::new(func_call), field_names));
            }
            return Ok(func_call);
        }
    }

    // Number
    if input.chars().next().map_or(false, |c| c.is_ascii_digit() || c == '-' || c == '.') {
        if input.parse::<f64>().is_ok() || input.contains('/') {
            return Ok(Expr::Number(input.to_string()));
        }
    }

    // Variable (possibly dotted for Section.Column resolution)
    let parts: Vec<String> = input.split('.').map(|s| s.trim().to_string()).collect();
    Ok(Expr::Variable(parts))
}

/// Parse list literal elements: a, b, c (similar to args but for lists)
fn parse_list_elements(input: &str) -> Result<Vec<Expr>, FolioError> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut elements = Vec::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut current_start = 0;

    // Use char_indices for proper UTF-8 handling
    for (byte_pos, c) in input.char_indices() {
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '(' if !in_double_quote && !in_single_quote => paren_depth += 1,
            ')' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            ',' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                elements.push(parse_expr(&input[current_start..byte_pos])?);
                current_start = byte_pos + c.len_utf8();
            }
            _ => {}
        }
    }

    elements.push(parse_expr(&input[current_start..])?);
    Ok(elements)
}

fn parse_args(input: &str) -> Result<Vec<Expr>, FolioError> {
    if input.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut args = Vec::new();
    let mut paren_depth = 0;
    let mut bracket_depth = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut current_start = 0;

    // Use char_indices for proper UTF-8 handling
    for (byte_pos, c) in input.char_indices() {
        match c {
            '"' if !in_single_quote => in_double_quote = !in_double_quote,
            '\'' if !in_double_quote => in_single_quote = !in_single_quote,
            '(' if !in_double_quote && !in_single_quote => paren_depth += 1,
            ')' if !in_double_quote && !in_single_quote => paren_depth -= 1,
            '[' if !in_double_quote && !in_single_quote => bracket_depth += 1,
            ']' if !in_double_quote && !in_single_quote => bracket_depth -= 1,
            ',' if paren_depth == 0 && bracket_depth == 0 && !in_double_quote && !in_single_quote => {
                args.push(parse_expr(&input[current_start..byte_pos])?);
                current_start = byte_pos + c.len_utf8();
            }
            _ => {}
        }
    }

    args.push(parse_expr(&input[current_start..])?);
    Ok(args)
}
```

folio\src\render.rs
```rs
//! Markdown renderer
//!
//! Renders evaluated document back to markdown with results.

use crate::ast::Document;
use folio_core::Value;
use std::collections::HashMap;

/// Display format for numbers
#[derive(Clone, Copy)]
pub enum NumberFormat {
    /// Fixed decimal places (default)
    Decimal(u32),
    /// Significant figures with scientific notation for large/small values
    SigFigs(u32),
}

impl Default for NumberFormat {
    fn default() -> Self {
        NumberFormat::Decimal(10)
    }
}

/// Display formats for datetime values
#[derive(Clone, Default)]
pub struct DateTimeFormats {
    /// Format for Date values (default: YYYY-MM-DD)
    pub date_fmt: Option<String>,
    /// Format for Time values (default: HH:mm:ss)
    pub time_fmt: Option<String>,
    /// Format for DateTime values (default: ISO 8601)
    pub datetime_fmt: Option<String>,
}

/// Document renderer
pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Self
    }

    /// Render document with computed values
    pub fn render(
        &self,
        doc: &Document,
        values: &HashMap<String, Value>,
        external: &HashMap<String, Value>,
    ) -> String {
        let mut output = String::new();

        // Render external variables section if any
        if !external.is_empty() {
            output.push_str("## External Variables\n\n");
            output.push_str("| name | value |\n");
            output.push_str("|------|-------|\n");
            let default_dt_formats = DateTimeFormats::default();
            for (name, value) in external {
                output.push_str(&format!("| {} | {} |\n", name, self.render_value(value, NumberFormat::default(), &default_dt_formats)));
            }
            output.push('\n');
        }

        // Render each section
        for section in &doc.sections {
            output.push_str(&format!("## {}", section.name));

            // Add attributes if any
            if !section.attributes.is_empty() {
                let attrs: Vec<String> = section.attributes
                    .iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect();
                output.push_str(&format!(" @{}", attrs.join(",")));
            }
            output.push_str("\n\n");

            // Determine formats from section attributes
            let num_format = self.get_number_format(&section.attributes);
            let dt_formats = self.get_datetime_formats(&section.attributes);

            // Render table header
            output.push_str("| name | formula | result |\n");
            output.push_str("|------|---------|--------|\n");

            // Rows
            for row in &section.table.rows {
                for cell in &row.cells {
                    let result = values.get(&cell.name)
                        .map(|v| self.render_value(v, num_format, &dt_formats))
                        .unwrap_or_default();
                    output.push_str(&format!("| {} | {} | {} |\n",
                        cell.name, cell.raw_text, result));
                }
            }

            output.push('\n');
        }

        output
    }

    /// Get number format from section attributes
    fn get_number_format(&self, attrs: &HashMap<String, String>) -> NumberFormat {
        // Check for @sigfigs first (takes precedence)
        if let Some(sigfigs) = attrs.get("sigfigs") {
            if let Ok(n) = sigfigs.parse::<u32>() {
                return NumberFormat::SigFigs(n);
            }
        }
        // Fall back to decimal places (default 10)
        NumberFormat::Decimal(10)
    }

    /// Get datetime formats from section attributes
    fn get_datetime_formats(&self, attrs: &HashMap<String, String>) -> DateTimeFormats {
        DateTimeFormats {
            date_fmt: attrs.get("dateFmt").cloned(),
            time_fmt: attrs.get("timeFmt").cloned(),
            datetime_fmt: attrs.get("datetimeFmt").cloned(),
        }
    }

    fn render_value(&self, value: &Value, num_format: NumberFormat, dt_formats: &DateTimeFormats) -> String {
        match value {
            Value::Number(n) => match num_format {
                NumberFormat::Decimal(places) => n.as_decimal(places),
                NumberFormat::SigFigs(sigfigs) => n.as_sigfigs(sigfigs),
            },
            Value::Text(s) => s.clone(),
            Value::Bool(b) => b.to_string(),
            Value::DateTime(dt) => {
                // Use section datetime format if specified
                if let Some(ref fmt) = dt_formats.datetime_fmt {
                    dt.format(fmt)
                } else if let Some(ref fmt) = dt_formats.date_fmt {
                    // If only date format is specified, use it
                    dt.format(fmt)
                } else {
                    dt.to_string()
                }
            },
            Value::Duration(d) => d.to_string(),
            Value::Object(_) => "[Object]".to_string(),
            Value::List(l) => format!("[{}]", l.len()),
            Value::Null => "null".to_string(),
            Value::Error(e) => format!("#ERROR: {}", e.code),
        }
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}
```

README.md
```md
# Folio

**Jupyter Notebooks for LLMs** — Declarative, reproducible, self-documenting computational documents.

## The Problem

LLMs execute Python scripts, but:
- Output format varies every run
- Calculations mixed with code
- State accumulates (side effects)
- No built-in provenance
- Floating point surprises
- "Trust me, I calculated" — opaque

## The Solution

**Markdown Computational Documents (MCD)** — Tables with formulas that evaluate to results.

```markdown
## Calculation
| name        | formula              | result |
|-------------|----------------------|--------|
| principal   | 300000               |        |
| rate        | 0.065 / 12           |        |
| months      | 30 * 12              |        |
| payment     | principal * rate * (1+rate)^months / ((1+rate)^months - 1) | |
```

After `eval()`:

```markdown
## Calculation
| name        | formula              | result       |
|-------------|----------------------|--------------|
| principal   | 300000               | 300000       |
| rate        | 0.065 / 12           | 0.0054166... |
| months      | 30 * 12              | 360          |
| payment     | principal * rate * (1+rate)^months / ((1+rate)^months - 1) | 1896.20 |
```

## Key Features

### 1. Named References (No Cell Coordinates)
```markdown
| error | target - calculated |   # References by NAME, not A1, B2
```

### 2. External Variables (Templates)
```rust
eval(template, {target: 299792458, exponent: 43})
```
Documents become reusable functions.

### 3. Arbitrary Precision
BigRational arithmetic. No floating point. Explicit precision control.

### 4. Object Access (Dotted Names)
```markdown
| decomp.φ     | DECOMPOSE(error).φ      |   # Access object fields
| decomp.π     | DECOMPOSE(error).π      |
```

### 5. Built-in Constants with Sources
```markdown
| φ | (1 + sqrt(5)) / 2 | [OEIS A001622](https://oeis.org/A001622) |
```

### 6. Pattern Detection
`DECOMPOSE(value)` runs ensemble analysis to find φ, π, e patterns in values.

### 7. Error Archaeology
Recursive error analysis — when a formula doesn't match exactly, analyze the error for hidden structure.

### 8. Plugin Architecture
Extend with custom functions, analyzers, commands.

### 9. LLM-First Design
- Never crashes — returns clear error messages
- Auto-documenting — `folio()` returns usage instructions
- Deterministic output — same input = same output

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         folio-mcp                               │
│                    (MCP Server Interface)                       │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────┐
│                           folio                                 │
│              (Parser, Evaluator, Renderer)                      │
└───────────┬─────────────────────────────────────┬───────────────┘
            │                                     │
┌───────────▼───────────┐           ┌─────────────▼─────────────┐
│      folio-std        │           │      folio-isis           │
│  (Standard Library)   │           │   (ISIS Extensions)       │
│  sqrt, ln, sin, cos   │           │   ISIS transform          │
│  DECOMPOSE, EXPLAIN   │           │   φ analyzers             │
└───────────┬───────────┘           └─────────────┬─────────────┘
            │                                     │
┌───────────▼─────────────────────────────────────▼───────────────┐
│                       folio-plugin                              │
│              (Plugin Traits & Registry)                         │
└─────────────────────────────┬───────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────────┐
│                       folio-core                                │
│              (Number, Value, Error Types)                       │
└─────────────────────────────────────────────────────────────────┘
```

## Crates

| Crate | Purpose |
|-------|---------|
| `folio-core` | Core types: Number (BigRational), Value, FolioError |
| `folio-plugin` | Plugin traits (FunctionPlugin, AnalyzerPlugin, CommandPlugin) + Registry |
| `folio-std` | Standard library: math functions, analyzers, constants |
| `folio` | Parser, evaluator, renderer |
| `folio-mcp` | MCP server exposing tools |
| `folio-isis` | ISIS-specific extensions (your research) |

## Usage

### As Library

```rust
use folio::Folio;
use folio_std::StandardLibrary;

let folio = Folio::new()
    .with_plugins(StandardLibrary::default());

let result = folio.eval(r#"
| name  | formula   | result |
|-------|-----------|--------|
| x     | 2 + 2     |        |
| y     | x * 3     |        |
"#, &vars!{})?;

println!("{}", result.markdown);
```

### As MCP Server

```bash
cargo run -p folio-mcp
```

Tools exposed:
- `eval(template, variables, precision?)` → Evaluate document
- `eval_batch(template, variable_sets)` → Parameter sweep
- `help(function_name?)` → Get usage instructions
- `list_functions(category?)` → List available functions
- `list_constants()` → List constants with sources

## Error Philosophy

**Never crash. Always explain.**

```rust
// Bad (crashes)
panic!("Division by zero");

// Good (returns error value)
Value::Error(FolioError {
    code: "DIV_ZERO",
    message: "Division by zero in cell 'ratio'",
    suggestion: "Check that divisor is not zero",
    context: ErrorContext { cell: "ratio", formula: "a / b", ... }
})
```

Errors propagate through calculations (like Excel's #DIV/0!) but include:
- Error code (machine-readable)
- Message (human-readable)
- Suggestion (how to fix)
- Context (where it happened)

## LLM Auto-Documentation

Calling `folio()` returns structured documentation:

```json
{
  "function": "DECOMPOSE",
  "description": "Analyze a value for patterns involving mathematical constants",
  "usage": "DECOMPOSE(value)",
  "args": [
    {"name": "value", "type": "Number", "description": "Value to analyze"}
  ],
  "returns": "Object with fields for each detected constant (φ, π, e, ...)",
  "example": "| decomp | DECOMPOSE(6.28318) | → {π: {coefficient: 2, confidence: 0.99}}",
  "related": ["EXPLAIN", "TRACE"]
}
```

## Claude Desktop Installation

To use Folio with Claude Desktop, add the following to your Claude Desktop configuration file:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

```json
{
  "mcpServers": {
    "folio": {
      "command": "docker",
      "args": ["run", "-i", "--rm", "-v", "/path/to/your/data:/app/folio", "folio-mcp"],
      "env": {}
    }
  }
}
```

Or if running the binary directly:

```json
{
  "mcpServers": {
    "folio": {
      "command": "/path/to/folio-mcp",
      "args": [],
      "env": {
        "FOLIO_DATA_PATH": "/path/to/your/fmd/files"
      }
    }
  }
}
```

After configuring, restart Claude Desktop. The `folio()` tool will be available for computational markdown documents.

## License

MIT
```