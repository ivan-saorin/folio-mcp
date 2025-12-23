# Folio LLM Experience Specification

## Overview

Improvements to make Folio maximally usable by LLMs with minimal trial-and-error. Every error becomes a learning opportunity. Every tool call returns maximum useful context.

---

## Design Principles

### 1. Fail Forward
Every error message teaches the correct usage. Never just say "wrong" - say "wrong, here's right."

### 2. Self-Documenting
The system can fully explain itself. An LLM with zero prior knowledge can become productive in one tool call.

### 3. Modular Documentation
Each crate owns its documentation. The main doc is assembled from parts.

### 4. Token-Conscious
Provide compact and verbose options. Don't waste context window on repeated help calls.

---

## Documentation Architecture

### Crate-Level Documentation

Each crate contains its own `FOLIO.md`:

```
folio-core/FOLIO.md       → Types, operators, error codes
folio-std/FOLIO.md        → Math, trig, aggregate functions
folio-stats/FOLIO.md      → Statistical functions
folio-finance/FOLIO.md    → Financial functions
folio-text/FOLIO.md       → Text functions
folio-matrix/FOLIO.md     → Matrix functions
folio-units/FOLIO.md      → Unit functions
folio-sequence/FOLIO.md   → Sequence functions
folio-isis/FOLIO.md       → ISIS functions
```

### Assembled Documentation

```rust
pub fn generate_full_docs() -> String {
    let prefix = include_str!("docs/PREFIX.md");
    let suffix = include_str!("docs/SUFFIX.md");
    
    let crate_docs: Vec<&str> = vec![
        include_str!("../folio-core/FOLIO.md"),
        include_str!("../folio-std/FOLIO.md"),
        include_str!("../folio-stats/FOLIO.md"),
        // ... enabled crates only
    ];
    
    format!("{}\n\n{}\n\n{}", prefix, crate_docs.join("\n\n---\n\n"), suffix)
}
```

### PREFIX.md

```markdown
# Folio - Markdown Computational Documents

Arbitrary precision arithmetic for LLMs. All calculations use exact rational arithmetic.

## Document Format

```markdown
## Section Name @precision:50

| name | formula | result |
|------|---------|--------|
| x | 10 | |
| y | x * 2 | |
| z | sqrt(y) | |
```

## Operators

| Operator | Description | Example | Status |
|----------|-------------|---------|--------|
| `+` | Addition | `a + b` | ✅ |
| `-` | Subtraction | `a - b` | ✅ |
| `*` | Multiplication | `a * b` | ✅ |
| `/` | Division | `a / b` | ✅ |
| `^` | Power | `a ^ b` | ✅ |
| `()` | Grouping | `(a + b) * c` | ✅ |
| `<` `>` `<=` `>=` | Comparison | - | ❌ Not implemented |
| `==` `!=` | Equality | - | ❌ Not implemented |
| `&&` `\|\|` `!` | Logical | - | ❌ Not implemented |

## Directives

| Directive | Description | Example |
|-----------|-------------|---------|
| `@precision:N` | Decimal precision | `@precision:100` |
| `@sigfigs:N` | Significant figures | `@sigfigs:6` |

## Value Types

| Type | Example | Display |
|------|---------|---------|
| Number | `42`, `3.14159` | `42.0000000000` |
| List | `[1, 2, 3]` | `[3]` (count) |
| Object | `linear_reg(x, y)` | `[Object]` |
| DateTime | `date(2024, 1, 15)` | `2024-01-15T00:00:00` |
| Duration | `days(5)` | `5d` |
| Error | - | `#ERROR: CODE` |
```

### SUFFIX.md

```markdown
## Error Codes

| Code | Meaning | Example Fix |
|------|---------|-------------|
| `UNDEFINED_VAR` | Variable not defined | Check spelling, define earlier |
| `UNDEFINED_FUNC` | Function not found | Check spelling, see function list |
| `UNDEFINED_FIELD` | Object field not found | Use `fields(obj)` to list available |
| `DIV_ZERO` | Division by zero | Check divisor |
| `DOMAIN_ERROR` | Invalid input domain | e.g., sqrt(-1), log(0) |
| `ARG_COUNT` | Wrong argument count | Check function signature |
| `TYPE_ERROR` | Wrong argument type | Check expected types |
| `PARSE_ERROR` | Formula syntax error | Check syntax |

## Tips for LLMs

1. Use `fields(obj)` to discover Object structure
2. Use `head(list, 5)` to peek at list contents
3. Functions accept both `(a, b, c)` and `([a, b, c])` for lists
4. Results are exact rationals, displayed as decimals at render
5. Reference other cells by name: `y | x * 2`
```

---

## Crate FOLIO.md Format

### Standard Structure

```markdown
# {Crate Name}

{One-line description}

## Functions

### Category: {category_name}

#### `function_name(arg1, arg2, [optional]) → ReturnType`

{Description}

**Arguments:**
| Arg | Type | Description |
|-----|------|-------------|
| arg1 | Number | ... |
| arg2 | List<Number> | ... |
| optional | Number | Default: 0 |

**Returns:** `ReturnType`
| Field | Type | Description |
|-------|------|-------------|
| .field1 | Number | ... |
| .field2 | Number | ... |

**Examples:**
```
function_name(1, 2) → 3
function_name([1,2,3]) → 6
```

**Errors:**
- `DOMAIN_ERROR`: When arg1 < 0
- `ARG_COUNT`: Requires 2-3 arguments
```

### Example: folio-stats/FOLIO.md (partial)

```markdown
# Statistics

Statistical functions with arbitrary precision.

## Functions

### Category: stats/hypothesis

#### `t_test_1(list, μ0) → Object`

One-sample t-test. Tests if sample mean differs from hypothesized mean.

**Arguments:**
| Arg | Type | Description |
|-----|------|-------------|
| list | List<Number> | Sample data |
| μ0 | Number | Hypothesized population mean |

**Returns:** `Object`
| Field | Type | Description |
|-------|------|-------------|
| .t | Number | t-statistic |
| .p | Number | Two-tailed p-value |
| .df | Number | Degrees of freedom (n-1) |
| .ci_low | Number | Lower 95% confidence bound |
| .ci_high | Number | Upper 95% confidence bound |
| .mean_diff | Number | Sample mean - μ0 |

**Examples:**
```
t_test_1([23, 25, 28, 22, 27], 25) → Object
  .t = 0.378
  .p = 0.721
  .df = 4
```

**Errors:**
- `DOMAIN_ERROR`: Requires at least 2 values
- `ARG_COUNT`: Requires exactly 2 arguments

---

#### `chi_test(observed, expected) → Object`

Chi-squared goodness of fit test.

**Returns:** `Object`
| Field | Type | Description |
|-------|------|-------------|
| .chi_sq | Number | Chi-squared statistic |
| .p | Number | p-value |
| .df | Number | Degrees of freedom |

...
```

---

## Enhanced Error Messages

### UNDEFINED_FIELD with Suggestions

```rust
fn get_field(obj: &Object, field: &str) -> Value {
    match obj.get(field) {
        Some(v) => v.clone(),
        None => {
            let available: Vec<&str> = obj.keys().collect();
            let suggestion = find_similar(field, &available);
            
            Value::Error(FolioError {
                code: "UNDEFINED_FIELD".into(),
                message: format!("Field '{}' not found", field),
                suggestion: Some(format!(
                    "Available fields: {}{}",
                    available.join(", "),
                    suggestion.map(|s| format!(". Did you mean '{}'?", s)).unwrap_or_default()
                )),
                context: None,
                severity: Severity::Error,
            })
        }
    }
}
```

**Output:**
```
#ERROR: UNDEFINED_FIELD 'statistic' not found. Available: chi_sq, p, df. Did you mean 'chi_sq'?
```

### UNDEFINED_FUNC with Suggestions

```rust
fn call_function(name: &str, args: &[Value]) -> Value {
    match registry.get(name) {
        Some(f) => f.call(args),
        None => {
            let similar = find_similar_functions(name);
            let by_category = group_by_category(&similar);
            
            Value::Error(FolioError {
                code: "UNDEFINED_FUNC".into(),
                message: format!("Function '{}' not found", name),
                suggestion: Some(format!(
                    "Similar: {}. Use folio:folio for full list.",
                    similar.iter().take(5).join(", ")
                )),
                context: None,
                severity: Severity::Error,
            })
        }
    }
}
```

**Output:**
```
#ERROR: UNDEFINED_FUNC 'stdev' not found. Similar: stddev, stddev_p. Use folio:folio for full list.
```

### ARG_COUNT with Signature

```rust
fn check_args(meta: &FunctionMeta, args: &[Value]) -> Result<(), FolioError> {
    let (min, max) = meta.arg_range();
    if args.len() < min || args.len() > max {
        return Err(FolioError {
            code: "ARG_COUNT".into(),
            message: format!(
                "{}() expects {}-{} arguments, got {}",
                meta.name, min, max, args.len()
            ),
            suggestion: Some(format!("Usage: {}", meta.usage)),
            context: None,
            severity: Severity::Error,
        });
    }
    Ok(())
}
```

**Output:**
```
#ERROR: ARG_COUNT t_test_1() expects 2 arguments, got 1. Usage: t_test_1(list, μ0)
```

---

## New Utility Functions

### `fields(object) → List<Text>`

List available fields on an Object.

```markdown
| reg | linear_reg(x, y) | [Object] |
| f | fields(reg) | ["slope", "intercept", "r_squared", "r", "std_error", "n"] |
```

### `describe(object) → Object`

Full description of Object with field values.

```markdown
| reg | linear_reg(x, y) | [Object] |
| desc | describe(reg) | [Object] |
```

Returns:
```json
{
  "type": "LinearRegression",
  "fields": {
    "slope": {"value": 1.99, "type": "Number"},
    "intercept": {"value": 0.05, "type": "Number"},
    ...
  }
}
```

### `head(list, n) → List`

First n elements of list.

```markdown
| data | [1,2,3,4,5,6,7,8,9,10] | [10] |
| peek | head(data, 3) | [1, 2, 3] |
```

### `tail(list, n) → List`

Last n elements of list.

```markdown
| data | [1,2,3,4,5,6,7,8,9,10] | [10] |
| last | tail(data, 3) | [8, 9, 10] |
```

### `take(list, n) → List`

Alias for `head()`.

### `typeof(value) → Text`

Get type name.

```markdown
| t1 | typeof(42) | "Number" |
| t2 | typeof([1,2,3]) | "List" |
| t3 | typeof(linear_reg(x,y)) | "Object" |
```

### `signature(func_name) → Text`

Get function signature.

```markdown
| sig | signature("t_test_1") | "t_test_1(list: List<Number>, μ0: Number) → Object {t, p, df, ci_low, ci_high, mean_diff}" |
```

---

## MCP Tool Improvements

### `folio:eval` - First Error Returns Docs

```rust
pub async fn eval(&self, template: &str, variables: &Object) -> Value {
    let result = self.folio.eval(template, variables);
    
    // Check if this is first call (no prior successful evals in session)
    // AND result has errors
    if self.is_first_eval() && result.has_errors() {
        return json!({
            "result": result.to_markdown(),
            "errors": result.errors,
            "help": self.generate_full_docs(),  // Include complete docs
            "hint": "First eval had errors. Full documentation included above."
        });
    }
    
    result.to_value()
}
```

### `folio:folio` - Compact Mode

```rust
pub async fn folio(&self, name: Option<&str>, compact: Option<bool>) -> Value {
    match (name, compact.unwrap_or(false)) {
        // Specific function - always verbose
        (Some(name), _) => self.function_help(name),
        
        // Compact listing
        (None, true) => self.compact_listing(),
        
        // Full listing (current behavior)
        (None, false) => self.full_listing(),
    }
}
```

### `folio:quick` - New Compact Reference

```markdown
# Folio Quick Reference

## Operators: + - * / ^ ()

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

## math
abs, ceil, floor, round, sqrt, pow, exp, ln, sin, cos, tan

## aggregate
sum, product, count, min, max

## datetime
now, date, time, datetime, parseDate, formatDate, diff, addDays, ...

## utility
fields, describe, head, tail, take, typeof, signature
```

**Token count:** ~400 (vs ~3000 for full listing)

### `folio:list_functions` - With Output Fields

```rust
pub async fn list_functions(&self, category: Option<&str>) -> Value {
    let functions = match category {
        Some(cat) => self.registry.functions_in_category(cat),
        None => self.registry.all_functions(),
    };
    
    json!(functions.iter().map(|f| {
        json!({
            "name": f.meta().name,
            "signature": f.signature_string(),
            "returns": f.return_type_string(),  // NEW: includes Object fields
            "category": f.meta().category,
        })
    }).collect::<Vec<_>>())
}
```

**Example output:**
```json
[
  {
    "name": "t_test_1",
    "signature": "t_test_1(list, μ0)",
    "returns": "Object {t, p, df, ci_low, ci_high, mean_diff}",
    "category": "stats/hypothesis"
  },
  ...
]
```

---

## List Display Improvements

### Smart Display

Small lists (≤5 elements) display inline, large lists show count:

```rust
impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::List(items) if items.len() <= 5 => {
                // Show actual values
                write!(f, "[{}]", items.iter().map(|v| v.to_string()).join(", "))
            }
            Value::List(items) => {
                // Show count
                write!(f, "[{}]", items.len())
            }
            // ...
        }
    }
}
```

**Output:**
```markdown
| small | [1, 2, 3] | [1, 2, 3] |
| large | range(1, 100) | [100] |
```

### Peek in Error Context

When a list operation fails, show first few elements:

```rust
Value::Error(FolioError {
    code: "DOMAIN_ERROR".into(),
    message: "gmean() requires all positive values",
    suggestion: Some(format!(
        "List contains non-positive values. First 5 elements: {:?}",
        list.iter().take(5).collect::<Vec<_>>()
    )),
    ...
})
```

**Output:**
```
#ERROR: DOMAIN_ERROR gmean() requires all positive values. First 5 elements: [2, -1, 3, 4, 5]
```

---

## Type Signature Format

### Standard Format

```
function_name(required_arg: Type, [optional_arg: Type = default]) → ReturnType
```

### Examples

```
mean(values: List<Number>) → Number
mean(a: Number, b: Number, ...) → Number

t_test_1(list: List<Number>, μ0: Number) → Object {t, p, df, ci_low, ci_high, mean_diff}

linear_reg(x: List<Number>, y: List<Number>) → Object {slope, intercept, r_squared, r, std_error, n}

percentile(values: List<Number>, p: Number[0-100]) → Number

ci(list: List<Number>, [level: Number = 0.95]) → Object {low, high, margin, level}
```

### In Code

```rust
pub struct FunctionMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub usage: &'static str,
    pub args: &'static [ArgMeta],
    pub returns: ReturnMeta,  // NEW
    pub examples: &'static [&'static str],
    pub category: &'static str,
    pub source: Option<&'static str>,
    pub related: &'static [&'static str],
}

pub struct ReturnMeta {
    pub typ: &'static str,  // "Number", "List<Number>", "Object"
    pub fields: Option<&'static [FieldMeta]>,  // For Objects
}

pub struct FieldMeta {
    pub name: &'static str,
    pub typ: &'static str,
    pub description: &'static str,
}
```

---

## Implementation Plan

### Phase 1: Error Messages (High Impact, Low Effort)

- [ ] UNDEFINED_FIELD shows available fields
- [ ] UNDEFINED_FUNC shows similar functions
- [ ] ARG_COUNT shows usage/signature
- [ ] DOMAIN_ERROR shows problematic values (first 5)

### Phase 2: New Utility Functions

- [ ] `fields(obj)` - list Object fields
- [ ] `head(list, n)` / `tail(list, n)` / `take(list, n)`
- [ ] `typeof(value)`
- [ ] `describe(obj)` - full Object description
- [ ] `signature(func_name)`

### Phase 3: Documentation Architecture

- [ ] Create FOLIO.md in each crate
- [ ] Create PREFIX.md and SUFFIX.md
- [ ] Implement `generate_full_docs()`
- [ ] Add return type documentation to all functions

### Phase 4: MCP Tool Improvements

- [ ] `folio:eval` returns docs on first error
- [ ] `folio:folio compact=true` option
- [ ] `folio:quick` new compact reference
- [ ] `folio:list_functions` includes return fields

### Phase 5: Display Improvements

- [ ] Smart list display (≤5 inline, else count)
- [ ] Object display shows type name
- [ ] Peek at list contents in errors

---

## Token Budget Analysis

| Tool Call | Current | Improved |
|-----------|---------|----------|
| `folio:folio` (full) | ~3000 | ~3000 |
| `folio:folio` (compact) | - | ~400 |
| `folio:quick` | - | ~400 |
| `folio:folio name="t_test_1"` | ~50 | ~150 (with fields) |
| Error message | ~20 | ~50 (with suggestions) |
| First eval with errors | ~100 | ~3500 (includes docs) |

**Net effect:** LLM needs fewer exploratory calls. One `folio:quick` + targeted `folio:folio name="X"` calls is more efficient than repeated guessing.

---

## Success Metrics

1. **Zero-Knowledge Productivity:** LLM can use folio correctly with only the first-error docs
2. **Field Discovery:** Never need to guess Object field names
3. **Error Recovery:** Every error message provides actionable fix
4. **Token Efficiency:** Typical workflow uses <1000 tokens on help

---

## Example: Ideal LLM Workflow

### Before (Current)

```
1. LLM: folio:eval with formula          → Error
2. LLM: folio:folio                       → 3000 tokens, still confused about Object fields
3. LLM: folio:eval with guess at field   → Error  
4. LLM: folio:eval with another guess    → Error
5. LLM: folio:folio name="function"      → Still no field info
6. LLM: Trial and error...               → Eventually works
```

### After (Improved)

```
1. LLM: folio:eval with formula          → Error + full docs (first time only)
2. LLM: Reads docs, understands          → Knows field names from docs
3. LLM: folio:eval with correct fields   → Success
```

Or:

```
1. LLM: folio:quick                      → 400 tokens, sees "t_test_1→{t,p,df,...}"
2. LLM: folio:eval with correct fields   → Success
```
