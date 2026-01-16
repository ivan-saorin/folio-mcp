# Folio

**Notebooks for LLMs** — Declarative, reproducible, self-documenting computational documents with 200+ functions.

![Folio Cover](https://raw.githubusercontent.com/ivan-saorin/folio-mcp/refs/heads/master/imgs/folio.png)

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
## Mortgage Calculator @precision:20
| name           | formula                                                                | result         |
|----------------|------------------------------------------------------------------------|----------------|
| principal      | 300000                                                                 | 300000         |
| rate           | 0.065 / 12                                                             | 0.0054166667   |
| months         | 30 * 12                                                                | 360            |
| payment        | principal * rate * pow(1 + rate, months) / (pow(1 + rate, months) - 1) | 1896.2040705   |
| total_paid     | payment * months                                                       | 682633.4654    |
| total_interest | total_paid - principal                                                 | 382633.4654    |
```

## Key Features

### 1. Named References (No Cell Coordinates)
```markdown
| error | target - calculated |   # References by NAME, not A1, B2
```

### 2. External Variables (Templates)
```rust
eval(template, {principal: 500000, rate: 0.07})
```
Documents become reusable functions.

### 3. Arbitrary Precision
Pure Rust `dashu` library. No floating point. Explicit precision control up to 100+ digits.

### 4. Object Access (Dotted Names)
```markdown
| reg       | linear_reg(x, y)  |              |
| slope     | reg.slope         | 2.2399       |
| r_squared | reg.r_squared     | 0.7361       |
```

### 5. Built-in Constants with Sources
```markdown
| φ | (1 + sqrt(5)) / 2 | 1.6180339887... |
| π | π                 | 3.1415926535... |
| e | e                 | 2.7182818284... |
```

### 6. 200+ Functions Across 12 Modules
Statistics, finance, matrices, sequences, text, units, and even cooking.

### 7. Plugin Architecture
Extend with custom functions via `FunctionPlugin`, `AnalyzerPlugin`, `CommandPlugin` traits.

### 8. LLM-First Design
- Never crashes — returns clear error messages with suggestions
- Auto-documenting — `folio()` returns usage instructions
- Deterministic output — same input = same output

## Architecture

![Folio Cover](https://raw.githubusercontent.com/ivan-saorin/folio-mcp/refs/heads/master/imgs/Architecture2.png)

![Folio Cover](https://raw.githubusercontent.com/ivan-saorin/folio-mcp/refs/heads/master/imgs/Architecture3.png)




## Crates

| Crate | Purpose |
|-------|---------|
| `folio-core` | Core types: Number (arbitrary precision), Value, FolioError |
| `folio-plugin` | Plugin traits (FunctionPlugin, AnalyzerPlugin, CommandPlugin) + Registry |
| `folio-std` | Standard library: math (sqrt, ln, exp, pow), trig (sin, cos, tan), aggregates |
| `folio-stats` | Statistics: descriptive stats, regression, hypothesis testing, distributions |
| `folio-finance` | Finance: TVM (NPV, IRR, MIRR), bonds, depreciation, amortization, returns |
| `folio-matrix` | Linear algebra: matrix ops, decomposition (LU, QR, Cholesky, SVD), eigenvalues |
| `folio-sequence` | Sequences: Fibonacci, primes, factorials, arithmetic/geometric progressions |
| `folio-text` | Text: string manipulation, parsing, validation, formatting |
| `folio-units` | Units: physical unit conversions with dimensional analysis |
| `folio-kitchen` | Kitchen: recipe scaling, cups↔grams, altitude/convection adjustments |
| `folio` | Parser, evaluator, renderer |
| `folio-mcp` | MCP server exposing tools for Claude Desktop |

## Function Categories

### Statistics (`folio-stats`)
- **Central tendency:** mean, median, mode, gmean, hmean, tmean, wmean
- **Dispersion:** variance, stddev, range, iqr, mad, cv, se
- **Regression:** linear_reg, slope, intercept, r_squared, predict, residuals
- **Hypothesis testing:** t_test_1, t_test_2, t_test_paired, f_test, chi_test, anova
- **Distributions:** norm_pdf/cdf/inv, t_pdf/cdf/inv, chi_pdf/cdf/inv, f_pdf/cdf/inv, binom, poisson
- **Confidence:** ci, moe

### Finance (`folio-finance`)
- **Time value of money:** pv, fv, npv, irr, mirr, xnpv, xirr
- **Loans:** pmt, ipmt, ppmt, nper, rate, cumipmt, cumprinc, amortization
- **Bonds:** bond_price, bond_yield, duration, mduration, convexity, accrint
- **Depreciation:** sln, ddb, syd, vdb, depreciation_schedule
- **Returns:** roi, cagr, sharpe, sortino, treynor, calmar, max_drawdown, alpha, beta

### Matrix (`folio-matrix`)
- **Construction:** matrix, vector, identity, zeros, ones, diagonal
- **Operations:** matmul, transpose, inverse, determinant, trace, rank
- **Decomposition:** lu, qr, cholesky, svd, eigen, schur
- **Solving:** solve, lstsq, nullspace, columnspace

### Sequences (`folio-sequence`)
- **Named:** fibonacci, lucas, primes, factorial_seq, catalan, tribonacci, triangular
- **Generators:** range, linspace, logspace, arithmetic, geometric, harmonic
- **Operations:** sum_seq, product_seq, partial_sums, extend_pattern, detect_pattern

### Text (`folio-text`)
- **Transform:** upper, lower, trim, capitalize, title_case, reverse
- **Search:** contains, starts_with, ends_with, index_of, matches
- **Modify:** replace, substring, split, join, concat, format, template
- **Validate:** is_email, is_url, is_numeric, is_uuid

### Units (`folio-units`)
- **Conversion:** convert, in_units, to_base
- **Analysis:** dimensions, compatible, is_dimensionless

### Kitchen (`folio-kitchen`)
- **Scaling:** scale_recipe, pan_scale
- **Conversions:** cups_to_grams, grams_to_cups, oven_temp, gas_mark
- **Adjustments:** convection_temp, altitude_time

## Usage

### As MCP Server (Recommended)

```bash
cargo run -p folio-mcp
```

MCP Tools exposed:
- `eval(template, variables?, precision?)` → Evaluate document
- `eval_file(name, variables?, precision?)` → Evaluate .fmd file from data directory
- `eval_batch(template, variable_sets)` → Parameter sweep
- `folio(name?, compact?)` → Get documentation (compact mode ~400 tokens)
- `quick()` → Quick reference (~400 tokens)
- `list_functions(category?)` → List available functions
- `list_constants()` → List constants with sources
- `decompose(value)` → Analyze value for mathematical patterns

#### Loading .fmd Files

**Important:** When using `eval_file`, pass the file name **without extension**:

```python
# ✅ Correct - name only, no extension
eval_file("mortgage")
eval_file("datetime_shortcuts")

# ❌ Wrong - don't include .fmd extension
eval_file("mortgage.fmd")           # Won't work
eval_file("data/examples/mortgage")  # Won't work - no paths
```

Files are loaded from the `FOLIO_DATA_PATH` directory (default: `/app/folio` in Docker).

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

## Claude Desktop Installation

Add to your Claude Desktop configuration:

**macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`  
**Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

### Using Docker (Recommended)

```json
{
  "mcpServers": {
    "folio": {
      "command": "docker",
      "args": [
        "run", "-i", "--rm",
        "-v", "/path/to/your/data:/app/folio",
        "folio-mcp"
      ],
      "env": {}
    }
  }
}
```

### Using Binary

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

## Examples

See `data/examples/` for sample .fmd files:
- `mortgage.fmd` — Mortgage payment calculator
- `phi_properties.fmd` — Golden ratio identities
- `portfolio-risk-analysis.fmd` — Full statistical analysis
- `compound_interest.fmd` — Investment calculations
- `unit_conversions.fmd` — Physical unit conversions

## Building

```bash
# Build all crates
cargo build --release

# Run tests
cargo test

# Build Docker image
docker build -t folio-mcp .
```

## License

MIT
