# Folio Implementation Prompt for Claude Code

## Project Overview

Folio is "Jupyter Notebooks for LLMs" - a markdown computational document system with:
- Arbitrary precision arithmetic (BigRational via `rug` crate)
- Plugin architecture for extensibility
- LLM-first design (never crashes, auto-documents)
- MCP server interface (protocol 2025-11-25)
- Docker deployment
- `.fmd` file format for documents

## Directory Structure

```
folio/
├── Cargo.toml           # Workspace root
├── Dockerfile           # Multi-stage build
├── docker-compose.yml   # Development compose
├── README.md            # User documentation
├── ARCHITECTURE.md      # Technical design
├── data/
│   └── examples/        # Example .fmd files
├── folio-core/          # Number, Value, FolioError types
├── folio-plugin/        # Plugin traits and registry
├── folio-std/           # Standard library (functions, analyzers)
├── folio/               # Parser, evaluator, renderer
├── folio-mcp/           # MCP server (runs in Docker)
└── folio-isis/          # ISIS formula extensions
```

## Implementation Order

### Phase 1: Core Types (folio-core)

1. **number.rs** - Implement `Number` type wrapping `rug::Rational`:
   ```rust
   impl Number {
       pub fn from_str(s: &str) -> Result&lt;Self, NumberError&gt;;
       pub fn from_i64(n: i64) -> Self;
       pub fn from_ratio(num: i64, den: i64) -> Self;
       
       // Constants (computed to precision)
       pub fn phi(precision: u32) -> Self;
       pub fn pi(precision: u32) -> Self;
       pub fn e(precision: u32) -> Self;
       
       // Arithmetic (div returns Result)
       pub fn add(&amp;self, other: &amp;Self) -> Self;
       pub fn sub(&amp;self, other: &amp;Self) -> Self;
       pub fn mul(&amp;self, other: &amp;Self) -> Self;
       pub fn checked_div(&amp;self, other: &amp;Self) -> Result&lt;Self, NumberError&gt;;
       pub fn pow(&amp;self, exp: i32) -> Self;
       
       // Transcendental
       pub fn sqrt(&amp;self, precision: u32) -> Result&lt;Self, NumberError&gt;;
       pub fn ln(&amp;self, precision: u32) -> Result&lt;Self, NumberError&gt;;
       pub fn exp(&amp;self, precision: u32) -> Self;
       
       pub fn as_decimal(&amp;self, places: u32) -> String;
   }
   ```

   **Key**: Use `rug::Float` for transcendental functions, convert back to `Rational`.

2. **value.rs** - Already scaffolded. Add `From` implementations.

3. **error.rs** - Already complete.

### Phase 2: Plugin System (folio-plugin)

Already scaffolded. Verify compilation and test registry.

### Phase 3: Standard Library (folio-std)

1. **functions/math.rs** - Implement each function:
   ```rust
   impl FunctionPlugin for Sqrt {
       fn call(&amp;self, args: &amp;[Value], ctx: &amp;EvalContext) -> Value {
           // 1. Check arg count
           if args.len() != 1 {
               return Value::Error(FolioError::arg_count("sqrt", 1, args.len()));
           }
           // 2. Propagate errors
           let x = match &amp;args[0] {
               Value::Number(n) => n,
               Value::Error(e) => return Value::Error(e.clone()),
               other => return Value::Error(FolioError::arg_type(...)),
           };
           // 3. Domain check
           if x.is_negative() {
               return Value::Error(FolioError::domain_error(...));
           }
           // 4. Compute (never panic!)
           match x.sqrt(ctx.precision) {
               Ok(result) => Value::Number(result),
               Err(e) => Value::Error(e.into()),
           }
       }
   }
   ```

2. **analyzers/phi.rs** - Implement φ pattern detection.

3. **constants.rs** - Define all constants with OEIS sources.

### Phase 4: Parser (folio/src/parser.rs)

Create `folio/src/grammar.pest`:
```pest
document = { SOI ~ (section | NEWLINE)* ~ EOI }
section = { section_header ~ table? }
section_header = { "##" ~ " "* ~ (!NEWLINE ~ ANY)* ~ NEWLINE }
table = { table_header ~ table_separator ~ table_row+ }
table_row = { "|" ~ cell+ ~ NEWLINE }
// ... full grammar in ARCHITECTURE.md
```

### Phase 5: Evaluator (folio/src/eval.rs)

1. Build dependency graph
2. Detect cycles → return error
3. Topological sort
4. Evaluate in order

### Phase 6: MCP Server (folio-mcp)

Already scaffolded with:
- MCP protocol 2025-11-25
- Tools: eval, eval_file, eval_batch, help, list_functions, list_constants, decompose
- Resources: folio://documents, folio://documents/{name}
- Prompts: mortgage_calculator, compound_interest, isis_analysis
- File loading from FOLIO_DATA_PATH

## Critical Requirements

### 1. Never Panic

```rust
// WRONG
fn divide(a: Number, b: Number) -> Number {
    a / b  // Panics on division by zero!
}

// CORRECT
fn divide(a: &amp;Number, b: &amp;Number) -> Value {
    match a.checked_div(b) {
        Ok(n) => Value::Number(n),
        Err(e) => Value::Error(e.into()),
    }
}
```

### 2. Error Propagation

```rust
fn eval_binary_op(&amp;self, left: Value, op: BinOp, right: Value) -> Value {
    // Check for errors FIRST
    if let Value::Error(e) = &amp;left {
        return Value::Error(e.clone().with_note("from left operand"));
    }
    // ... then proceed
}
```

### 3. Rich Error Context

```rust
Value::Error(FolioError::undefined_var("x")
    .in_cell("result")
    .with_formula("x + y")
    .with_suggestion("Define 'x' or check spelling"))
```

### 4. LLM-Friendly Documentation

Every plugin must provide complete FunctionMeta:
```rust
FunctionMeta {
    name: "sqrt",
    description: "Square root with arbitrary precision",
    usage: "sqrt(x)",
    args: &amp;[ArgMeta::required("x", "Number", "Value (must be positive)")],
    returns: "Number",
    examples: &amp;["sqrt(2)", "sqrt(phi)"],
    category: "math",
    source: None,
    related: &amp;["pow", "exp"],
}
```

## Docker Deployment

The MCP server runs in Docker:

```dockerfile
FROM rust:1.83-slim-bookworm AS builder
# ... build with GMP

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/folio-mcp /usr/local/bin/
ENV FOLIO_DATA_PATH=/app/folio
ENTRYPOINT ["/usr/local/bin/folio-mcp"]
```

MCP config:
```json
{
  "folio": {
    "command": "docker",
    "args": [
      "run", "--rm", "-i",
      "--name", "folio-mcp-claude",
      "-e", "FOLIO_DATA_PATH=/app/folio",
      "-v", "D:/projects2/projects/folio/data:/app/folio:ro",
      "folio:latest"
    ]
  }
}
```

## .fmd File Format

Folio Markdown Documents:
- First line: `&lt;!-- Description --&gt;` or `# Title`
- Sections: `## Name @precision:N`
- Tables: `| name | formula | result |`
- Formulas reference other cells by name

Example files in `data/examples/`:
- `mortgage.fmd` - Mortgage calculator
- `phi_properties.fmd` - φ identities
- `isis_analysis.fmd` - ISIS transform
- `compound_interest.fmd` - Investment math
- `trig_identities.fmd` - Trig verification

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_sqrt_positive() {
    let ctx = test_context();
    let result = Sqrt.call(&amp;[Value::Number(Number::from_i64(4))], &amp;ctx);
    assert_eq!(result.as_number().unwrap().to_i64(), Some(2));
}
```

### Integration Tests
```rust
#[test]
fn test_eval_file() {
    let folio = Folio::default();
    let template = fs::read_to_string("data/examples/mortgage.fmd").unwrap();
    let result = folio.eval(&amp;template, &amp;HashMap::new());
    assert!(!result.errors.is_empty() || result.values.contains_key("payment"));
}
```

### Docker Test
```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
  docker run --rm -i folio:latest
```

## Dependencies

### rug crate (GMP)

Windows: Use MSYS2
```bash
pacman -S mingw-w64-x86_64-gmp
```

Alternative: Use `num-bigint` + `num-rational` if GMP is problematic.

## Getting Started

1. `cd folio`
2. `cargo build` - Will fail on incomplete code
3. Start with `folio-core/src/number.rs`
4. Work through each TODO
5. `cargo test` frequently
6. `docker build -t folio:latest .` to test container

## Success Criteria

1. `cargo build` succeeds
2. `cargo test` passes
3. Docker builds and runs
4. This document evaluates correctly:
   ```markdown
   ## Test @precision:50
   | name | formula | result |
   |------|---------|--------|
   | phi  | (1 + sqrt(5)) / 2 | |
   | x    | phi^2 | |
   | y    | phi + 1 | |
   | check | x - y | |  # Should be 0
   ```
5. MCP server responds to `tools/list` and `folio()`
6. `.fmd` files load via `eval_file`
7. Errors never crash, always return `Value::Error`
