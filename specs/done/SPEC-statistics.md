# Folio Statistics Extension Specification

## Overview

Statistical functions for Folio using BigRational precision. All functions follow the never-panic philosophy and return `Value::Error` on failure.

## Column Access Syntax

### Section.Column Reference

Access all values from a column as an implicit list:

```markdown
## Data

| Item | Value | Price |
|------|-------|-------|
| a    | 10    | 100   |
| b    | 20    | 200   |
| c    | 30    | 300   |

## Stats

| Metric | Formula              | Result |
|--------|----------------------|--------|
| avg    | mean(Data.Value)     |        |
| total  | sum(Data.Price)      |        |
| corr   | correlation(Data.Value, Data.Price) | |
```

**Implementation:** When evaluator encounters `Section.Column`, it collects all values from that column into a `Value::List`.

### Optional @bind Attribute

Shorthand for repeated column references:

```markdown
## Data @bind:x=Value,y=Price

| Item | Value | Price |
|------|-------|-------|
| a    | 10    | 100   |
| b    | 20    | 200   |

## Stats

| Metric | Formula         | Result |
|--------|-----------------|--------|
| avg_x  | mean(x)         |        |
| corr   | correlation(x,y)|        |
```

**Scope:** Bindings are local to the section where `@bind` is declared.

### Inline List Literal

For ad-hoc calculations:

```markdown
| avg | mean([10, 20, 30, 25, 15]) |  |
```

**Grammar addition:**
```pest
list_literal = { "[" ~ expression ~ ("," ~ expression)* ~ "]" }
```

---

## Variance Convention

| Function | Divisor | Use Case |
|----------|---------|----------|
| `variance(list)` | n-1 | Sample variance (default) |
| `variance_p(list)` | n | Population variance |
| `stddev(list)` | n-1 | Sample standard deviation |
| `stddev_p(list)` | n | Population standard deviation |
| `covariance(x,y)` | n-1 | Sample covariance |
| `covariance_p(x,y)` | n | Population covariance |

Matches Excel/Google Sheets conventions.

---

## Function Categories

### Category: `stats/central`

| Function | Signature | Description |
|----------|-----------|-------------|
| `mean` | `mean(list)` | Arithmetic mean: Σx/n |
| `median` | `median(list)` | Middle value (average of two middle if even count) |
| `mode` | `mode(list)` | Most frequent value. Returns List if tie |
| `gmean` | `gmean(list)` | Geometric mean: (∏x)^(1/n). Error if any x ≤ 0 |
| `hmean` | `hmean(list)` | Harmonic mean: n/Σ(1/x). Error if any x = 0 |
| `tmean` | `tmean(list, pct)` | Trimmed mean excluding pct% from each tail |
| `wmean` | `wmean(values, weights)` | Weighted mean: Σ(w·x)/Σw |

### Category: `stats/dispersion`

| Function | Signature | Description |
|----------|-----------|-------------|
| `variance` | `variance(list)` | Sample variance: Σ(x-x̄)²/(n-1) |
| `variance_p` | `variance_p(list)` | Population variance: Σ(x-μ)²/n |
| `stddev` | `stddev(list)` | Sample standard deviation: √variance |
| `stddev_p` | `stddev_p(list)` | Population standard deviation |
| `range` | `range(list)` | max - min |
| `iqr` | `iqr(list)` | Interquartile range: Q3 - Q1 |
| `mad` | `mad(list)` | Median absolute deviation |
| `cv` | `cv(list)` | Coefficient of variation: stddev/mean |
| `se` | `se(list)` | Standard error: stddev/√n |

### Category: `stats/position`

| Function | Signature | Description |
|----------|-----------|-------------|
| `min` | `min(list)` | Minimum value |
| `max` | `max(list)` | Maximum value |
| `percentile` | `percentile(list, p)` | p-th percentile (0-100) |
| `quantile` | `quantile(list, q)` | q-th quantile (0-1) |
| `q1` | `q1(list)` | First quartile (25th percentile) |
| `q3` | `q3(list)` | Third quartile (75th percentile) |
| `rank` | `rank(value, list)` | Position in sorted list (1-indexed) |
| `zscore` | `zscore(value, list)` | Standard score: (x - mean)/stddev |

### Category: `stats/shape`

| Function | Signature | Description |
|----------|-----------|-------------|
| `skewness` | `skewness(list)` | Measure of asymmetry |
| `kurtosis` | `kurtosis(list)` | Measure of tail heaviness (excess kurtosis) |
| `count` | `count(list)` | Number of elements |
| `sum` | `sum(list)` | Sum of all elements (already exists) |
| `product` | `product(list)` | Product of all elements |

### Category: `stats/bivariate`

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `covariance` | `covariance(x, y)` | Number | Sample covariance |
| `covariance_p` | `covariance_p(x, y)` | Number | Population covariance |
| `correlation` | `correlation(x, y)` | Number | Pearson correlation coefficient r |
| `spearman` | `spearman(x, y)` | Number | Spearman rank correlation |

### Category: `stats/regression`

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `linear_reg` | `linear_reg(x, y)` | Object | Full regression result |
| `slope` | `slope(x, y)` | Number | Just the slope |
| `intercept` | `intercept(x, y)` | Number | Just the intercept |
| `r_squared` | `r_squared(x, y)` | Number | Coefficient of determination |
| `predict` | `predict(reg, x)` | Number | Predict y from regression object |
| `residuals` | `residuals(x, y)` | List | List of (y - ŷ) |

**linear_reg returns:**
```json
{
  "slope": Number,
  "intercept": Number,
  "r_squared": Number,
  "r": Number,
  "std_error": Number,
  "n": Number
}
```

### Category: `stats/distribution`

#### Normal Distribution

| Function | Signature | Description |
|----------|-----------|-------------|
| `norm_pdf` | `norm_pdf(x, μ, σ)` | Probability density |
| `norm_cdf` | `norm_cdf(x, μ, σ)` | Cumulative P(X ≤ x) |
| `norm_inv` | `norm_inv(p, μ, σ)` | Inverse CDF (quantile function) |
| `snorm_pdf` | `snorm_pdf(x)` | Standard normal PDF (μ=0, σ=1) |
| `snorm_cdf` | `snorm_cdf(x)` | Standard normal CDF |
| `snorm_inv` | `snorm_inv(p)` | Standard normal inverse |

#### Student's t Distribution

| Function | Signature | Description |
|----------|-----------|-------------|
| `t_pdf` | `t_pdf(x, df)` | t-distribution PDF |
| `t_cdf` | `t_cdf(x, df)` | t-distribution CDF |
| `t_inv` | `t_inv(p, df)` | t-distribution inverse |

#### Chi-Squared Distribution

| Function | Signature | Description |
|----------|-----------|-------------|
| `chi_pdf` | `chi_pdf(x, df)` | χ² PDF |
| `chi_cdf` | `chi_cdf(x, df)` | χ² CDF |
| `chi_inv` | `chi_inv(p, df)` | χ² inverse |

#### F Distribution

| Function | Signature | Description |
|----------|-----------|-------------|
| `f_pdf` | `f_pdf(x, df1, df2)` | F-distribution PDF |
| `f_cdf` | `f_cdf(x, df1, df2)` | F-distribution CDF |
| `f_inv` | `f_inv(p, df1, df2)` | F-distribution inverse |

#### Discrete Distributions

| Function | Signature | Description |
|----------|-----------|-------------|
| `binom_pmf` | `binom_pmf(k, n, p)` | Binomial probability mass |
| `binom_cdf` | `binom_cdf(k, n, p)` | Binomial CDF |
| `poisson_pmf` | `poisson_pmf(k, λ)` | Poisson probability mass |
| `poisson_cdf` | `poisson_cdf(k, λ)` | Poisson CDF |

### Category: `stats/hypothesis`

All hypothesis tests return an Object:

| Function | Signature | Description |
|----------|-----------|-------------|
| `t_test_1` | `t_test_1(list, μ0)` | One-sample t-test |
| `t_test_2` | `t_test_2(list1, list2)` | Two-sample t-test (Welch's) |
| `t_test_paired` | `t_test_paired(list1, list2)` | Paired t-test |
| `chi_test` | `chi_test(observed, expected)` | Chi-squared goodness of fit |
| `f_test` | `f_test(list1, list2)` | F-test for variance ratio |
| `anova` | `anova(list1, list2, ...)` | One-way ANOVA |

**t_test returns:**
```json
{
  "t": Number,
  "p": Number,
  "df": Number,
  "ci_low": Number,
  "ci_high": Number,
  "mean_diff": Number
}
```

**anova returns:**
```json
{
  "f": Number,
  "p": Number,
  "df_between": Number,
  "df_within": Number,
  "ss_between": Number,
  "ss_within": Number
}
```

### Category: `stats/confidence`

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `ci` | `ci(list, level)` | Object | Confidence interval |
| `moe` | `moe(list, level)` | Number | Margin of error |

**ci returns:**
```json
{
  "low": Number,
  "high": Number,
  "margin": Number,
  "level": Number
}
```

Default `level` = 0.95 (95% confidence)

### Category: `stats/transform`

| Function | Signature | Returns | Description |
|----------|-----------|---------|-------------|
| `normalize` | `normalize(list)` | List | Z-scores for all values |
| `standardize` | `standardize(list)` | List | Scale to [0,1] range |
| `cumsum` | `cumsum(list)` | List | Cumulative sum |
| `diff` | `diff(list)` | List | First differences |
| `lag` | `lag(list, n)` | List | Shift by n periods |
| `moving_avg` | `moving_avg(list, window)` | List | Simple moving average |
| `ewma` | `ewma(list, alpha)` | List | Exponentially weighted moving average |

---

## Implementation Structure

### New Crate: `folio-stats`

```
folio-stats/
├── Cargo.toml
└── src/
    ├── lib.rs              # Register all stats functions
    ├── helpers.rs          # extract_numbers, extract_two_lists, etc.
    ├── central.rs          # mean, median, mode, gmean, hmean, tmean, wmean
    ├── dispersion.rs       # variance, stddev, range, iqr, mad, cv, se
    ├── position.rs         # min, max, percentile, quantile, q1, q3, rank, zscore
    ├── shape.rs            # skewness, kurtosis, count, product
    ├── bivariate.rs        # covariance, correlation, spearman
    ├── regression.rs       # linear_reg, slope, intercept, r_squared, predict, residuals
    ├── distributions/
    │   ├── mod.rs
    │   ├── normal.rs       # norm_*, snorm_*
    │   ├── t.rs            # t_*
    │   ├── chi.rs          # chi_*
    │   ├── f.rs            # f_*
    │   └── discrete.rs     # binom_*, poisson_*
    ├── hypothesis.rs       # t_test_*, chi_test, f_test, anova
    ├── confidence.rs       # ci, moe
    └── transform.rs        # normalize, standardize, cumsum, diff, lag, moving_avg, ewma
```

### Cargo.toml

```toml
[package]
name = "folio-stats"
version.workspace = true
edition.workspace = true

[dependencies]
folio-core = { path = "../folio-core" }
folio-plugin = { path = "../folio-plugin" }
dashu.workspace = true
```

### Helper Functions (helpers.rs)

```rust
use folio_core::{Number, Value, FolioError};

/// Extract numbers from args, handling both varargs and List
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
        return Err(FolioError::arg_count(2, args.len()));
    }
    
    let x = extract_numbers(&args[0..1])?;
    let y = extract_numbers(&args[1..2])?;
    
    if x.len() != y.len() {
        return Err(FolioError::domain_error(format!(
            "Lists must have equal length: {} vs {}",
            x.len(), y.len()
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
            func, min, numbers.len()
        )));
    }
    Ok(())
}
```

### Example Implementation (central.rs)

```rust
use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, require_non_empty};

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
        
        let sum: Number = numbers.iter().fold(Number::zero(), |a, b| a.add(b));
        let count = Number::from_i64(numbers.len() as i64);
        
        match sum.div(&count) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

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

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let numbers = match extract_numbers(args) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        
        // Need at least 2 values for sample variance
        if numbers.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "variance() requires at least 2 values for sample variance"
            ));
        }
        
        // Calculate mean
        let n = numbers.len();
        let sum: Number = numbers.iter().fold(Number::zero(), |a, b| a.add(b));
        let mean = match sum.div(&Number::from_i64(n as i64)) {
            Ok(m) => m,
            Err(e) => return Value::Error(e.into()),
        };
        
        // Sum of squared deviations
        let mut ss = Number::zero();
        for x in &numbers {
            let dev = x.sub(&mean);
            ss = ss.add(&dev.mul(&dev));
        }
        
        // Divide by n-1 (sample variance)
        match ss.div(&Number::from_i64((n - 1) as i64)) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}
```

### Registration (lib.rs)

```rust
//! Folio Statistics Plugin
//!
//! Statistical functions with BigRational precision.

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

use folio_plugin::PluginRegistry;

pub fn register(registry: PluginRegistry) -> PluginRegistry {
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
        
        // Distributions
        .with_function(distributions::NormPdf)
        .with_function(distributions::NormCdf)
        .with_function(distributions::NormInv)
        .with_function(distributions::SnormPdf)
        .with_function(distributions::SnormCdf)
        .with_function(distributions::SnormInv)
        .with_function(distributions::TPdf)
        .with_function(distributions::TCdf)
        .with_function(distributions::TInv)
        .with_function(distributions::ChiPdf)
        .with_function(distributions::ChiCdf)
        .with_function(distributions::ChiInv)
        .with_function(distributions::FPdf)
        .with_function(distributions::FCdf)
        .with_function(distributions::FInv)
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
        .with_function(transform::Diff)
        .with_function(transform::Lag)
        .with_function(transform::MovingAvg)
        .with_function(transform::Ewma)
}
```

---

## Parser Changes Required

### 1. Add List Literal to Grammar

```pest
// In folio/src/folio.pest

list_literal = { "[" ~ ws* ~ (expression ~ (ws* ~ "," ~ ws* ~ expression)*)? ~ ws* ~ "]" }
factor = { function_call | list_literal | number | variable | "(" ~ ws* ~ expression ~ ws* ~ ")" }
```

### 2. Add List to AST

```rust
// In folio/src/ast.rs

pub enum Expr {
    Number(String),
    Variable(Vec<String>),
    BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    UnaryOp(UnaryOp, Box<Expr>),
    FunctionCall(String, Vec<Expr>),
    List(Vec<Expr>),  // NEW
}
```

### 3. Evaluator: Section.Column Resolution

```rust
// In folio/src/eval.rs

fn resolve_variable(&self, path: &[String], ctx: &EvalContext) -> Value {
    if path.len() == 2 {
        // Could be Section.Column reference
        let section_name = &path[0];
        let column_name = &path[1];
        
        if let Some(column_values) = self.get_column_values(section_name, column_name, ctx) {
            return Value::List(column_values);
        }
    }
    
    // Fall back to normal variable resolution
    ctx.get_variable(path)
}

fn get_column_values(&self, section: &str, column: &str, ctx: &EvalContext) -> Option<Vec<Value>> {
    let doc = ctx.document?;
    let section = doc.sections.iter().find(|s| s.name == section)?;
    let col_idx = section.table.columns.iter().position(|c| c == column)?;
    
    Some(section.table.rows.iter()
        .map(|row| row.cells.get(col_idx).map(|c| c.value.clone()).unwrap_or(Value::Null))
        .collect())
}
```

---

## MCP Tool Updates

Add to `folio-mcp`:

```rust
/// List statistical functions
async fn list_stats_functions(&self) -> Value {
    self.folio.registry().list_functions(Some("stats"))
}
```

---

## Testing Strategy

### Unit Tests per Module

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mean_basic() {
        let mean = Mean;
        let args = vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ];
        let result = mean.call(&args, &EvalContext::default());
        assert_eq!(result.as_number().unwrap(), &Number::from_i64(2));
    }
    
    #[test]
    fn test_mean_with_list() {
        let mean = Mean;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(10)),
            Value::Number(Number::from_i64(20)),
            Value::Number(Number::from_i64(30)),
        ])];
        let result = mean.call(&args, &EvalContext::default());
        assert_eq!(result.as_number().unwrap(), &Number::from_i64(20));
    }
    
    #[test]
    fn test_mean_empty_list() {
        let mean = Mean;
        let args = vec![Value::List(vec![])];
        let result = mean.call(&args, &EvalContext::default());
        assert!(result.is_error());
    }
}
```

### Integration Tests with Markdown

```rust
#[test]
fn test_stats_in_document() {
    let doc = r#"
## Data

| Item | Value |
|------|-------|
| a    | 10    |
| b    | 20    |
| c    | 30    |

## Stats

| Metric | Formula          | Result |
|--------|------------------|--------|
| avg    | mean(Data.Value) |        |
| std    | stddev(Data.Value)|       |
"#;
    
    let result = Folio::new().eval(doc, &HashMap::new());
    assert_eq!(result.values["avg"].as_number().unwrap(), &Number::from_i64(20));
}
```

---

## Implementation Priority

### Phase 1: Core (MVP)
- [ ] helpers.rs
- [ ] central.rs (mean, median, mode)
- [ ] dispersion.rs (variance, stddev, range)
- [ ] position.rs (min, max, percentile, quantile)
- [ ] Parser: list literal support
- [ ] Evaluator: Section.Column resolution

### Phase 2: Extended Descriptive
- [ ] central.rs (gmean, hmean, tmean, wmean)
- [ ] dispersion.rs (iqr, mad, cv, se)
- [ ] shape.rs (skewness, kurtosis, count, product)
- [ ] position.rs (q1, q3, rank, zscore)

### Phase 3: Bivariate & Regression
- [ ] bivariate.rs
- [ ] regression.rs

### Phase 4: Distributions
- [ ] distributions/normal.rs
- [ ] distributions/t.rs
- [ ] distributions/chi.rs
- [ ] distributions/f.rs
- [ ] distributions/discrete.rs

### Phase 5: Hypothesis Testing
- [ ] hypothesis.rs
- [ ] confidence.rs

### Phase 6: Transforms
- [ ] transform.rs

---

## Notes

1. **BigRational Precision**: All intermediate calculations use `Number` (dashu Rational). Transcendental functions (PDF, CDF) may need conversion to Float for computation, then back to Rational for result.

2. **Error Handling**: Every function returns `Value::Error` on failure, never panics. Errors propagate through dependent cells.

3. **List Handling**: Functions accept both `(a, b, c)` varargs and `(list)` single argument. Mixed `(list, a, b)` is NOT supported.

4. **Empty List**: Most functions return error on empty input. `count([])` returns 0, `sum([])` returns 0.

5. **NaN/Inf**: Not representable in BigRational. Operations that would produce these return `Value::Error` with appropriate domain error.
