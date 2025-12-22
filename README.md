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
