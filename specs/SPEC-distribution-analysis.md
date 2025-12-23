# Folio Distribution Analysis Specification

## Overview

Extension to `folio-stats` for detecting data distributions, goodness-of-fit testing, outlier detection, and histogram generation. Uses maximum likelihood estimation (MLE) for comprehensive distribution fitting.

---

## Module Structure

```
folio-stats/src/
├── distributions/
│   ├── mod.rs           # (existing)
│   ├── fitting/
│   │   ├── mod.rs
│   │   ├── mle.rs       # Maximum likelihood estimation
│   │   ├── moments.rs   # Method of moments (fallback)
│   │   └── candidates.rs
│   ├── goodness.rs      # Goodness-of-fit tests
│   ├── outliers.rs      # Outlier detection
│   └── histogram.rs     # Binning and histograms
```

---

## Supported Distributions

### Continuous

| Distribution | Parameters | MLE Estimators |
|--------------|------------|----------------|
| Normal | μ, σ | x̄, s |
| Lognormal | μ, σ (of log) | mean(ln x), std(ln x) |
| Exponential | λ | 1/x̄ |
| Uniform | a, b | min, max |
| Gamma | α, β | MLE via Newton-Raphson |
| Beta | α, β | MLE via Newton-Raphson |
| Weibull | k, λ | MLE via Newton-Raphson |
| Pareto | α, xₘ | MLE, min(x) |
| Cauchy | x₀, γ | Median, IQR/2 |
| Laplace | μ, b | Median, MAD |

### Discrete

| Distribution | Parameters | MLE Estimators |
|--------------|------------|----------------|
| Poisson | λ | x̄ |
| Binomial | n, p | Known n, p = x̄/n |
| Geometric | p | 1/x̄ |
| Negative Binomial | r, p | MLE |

---

## Functions

### Distribution Fitting

#### `fit_distribution(list, [candidates])`

Fits data to best-matching distribution from candidates.

```markdown
| data     | Data.Values                    |                |
| fit      | fit_distribution(data)         |                |
| best     | fit.distribution               | "normal"       |
| mu       | fit.parameters.mu              | 50.23          |
| sigma    | fit.parameters.sigma           | 12.45          |
| p_value  | fit.p_value                    | 0.342          |
```

**Returns Object:**
```json
{
  "distribution": "normal",
  "parameters": {
    "mu": Number,
    "sigma": Number
  },
  "log_likelihood": Number,
  "aic": Number,
  "bic": Number,
  "ks_statistic": Number,
  "ks_p_value": Number,
  "anderson_darling": Number,
  "candidates": [
    {
      "distribution": "normal",
      "parameters": {...},
      "aic": 234.5,
      "ks_p_value": 0.342
    },
    {
      "distribution": "lognormal",
      "parameters": {...},
      "aic": 238.2,
      "ks_p_value": 0.287
    }
  ]
}
```

**Candidates parameter:**
```markdown
| fit | fit_distribution(data, ["normal", "exponential", "gamma"]) |
```

Default candidates: `["normal", "lognormal", "exponential", "uniform", "gamma", "weibull"]`

#### `fit_normal(list)`

Fit specifically to normal distribution.

```markdown
| fit   | fit_normal(Data.Values)  |        |
| mu    | fit.mu                   | 50.23  |
| sigma | fit.sigma                | 12.45  |
```

**Returns:**
```json
{
  "mu": Number,
  "sigma": Number,
  "log_likelihood": Number,
  "ks_statistic": Number,
  "ks_p_value": Number
}
```

#### Specific Fitters

| Function | Parameters Returned |
|----------|---------------------|
| `fit_normal(list)` | mu, sigma |
| `fit_lognormal(list)` | mu, sigma (of log) |
| `fit_exponential(list)` | lambda |
| `fit_uniform(list)` | a, b |
| `fit_gamma(list)` | alpha, beta |
| `fit_beta(list)` | alpha, beta (requires data in [0,1]) |
| `fit_weibull(list)` | k, lambda |
| `fit_pareto(list)` | alpha, x_min |
| `fit_poisson(list)` | lambda |

---

### Goodness-of-Fit Tests

#### `ks_test(list, distribution, parameters)`

Kolmogorov-Smirnov test against a specific distribution.

```markdown
| ks      | ks_test(data, "normal", {mu: 50, sigma: 10}) |     |
| stat    | ks.statistic                                  | 0.087 |
| p       | ks.p_value                                    | 0.342 |
| reject  | ks.p_value < 0.05                             | false |
```

**Returns:**
```json
{
  "statistic": Number,
  "p_value": Number,
  "critical_01": Number,
  "critical_05": Number,
  "critical_10": Number
}
```

#### `ks_test_2(list1, list2)`

Two-sample KS test: do two samples come from same distribution?

```markdown
| ks   | ks_test_2(before, after) |       |
| same | ks.p_value > 0.05        | true  |
```

#### `anderson_darling(list, distribution)`

Anderson-Darling test (more sensitive to tails than KS).

```markdown
| ad   | anderson_darling(data, "normal") |       |
| stat | ad.statistic                     | 0.543 |
```

**Returns:**
```json
{
  "statistic": Number,
  "critical_values": {
    "15": Number,  // 15% significance
    "10": Number,
    "5": Number,
    "2.5": Number,
    "1": Number
  },
  "significance_level": Number  // Estimated p-value
}
```

#### `shapiro_wilk(list)`

Shapiro-Wilk test for normality (best for n < 50).

```markdown
| sw   | shapiro_wilk(Data.Values) |       |
| w    | sw.w                      | 0.967 |
| p    | sw.p_value                | 0.234 |
```

**Note:** Limited to n ≤ 5000 due to computational complexity.

#### `jarque_bera(list)`

Jarque-Bera test for normality (uses skewness and kurtosis).

```markdown
| jb   | jarque_bera(Data.Values) |       |
| stat | jb.statistic             | 2.34  |
| p    | jb.p_value               | 0.310 |
```

**Returns:**
```json
{
  "statistic": Number,
  "p_value": Number,
  "skewness": Number,
  "kurtosis": Number
}
```

#### `is_normal(list, [alpha])`

Convenience function: returns boolean.

```markdown
| normal | is_normal(Data.Values)       | true  |
| strict | is_normal(Data.Values, 0.01) | false |
```

Default alpha = 0.05. Uses Shapiro-Wilk for n < 50, Jarque-Bera otherwise.

---

### Histogram & Binning

#### `histogram(list, [bins])`

Create histogram with automatic or specified bin count.

```markdown
| hist    | histogram(Data.Values, 10)  |        |
| edges   | hist.edges                  | [0, 10, 20, ...] |
| counts  | hist.counts                 | [3, 7, 12, ...] |
| density | hist.density                | [0.03, 0.07, ...] |
```

**Returns:**
```json
{
  "edges": [Number],      // n+1 edges for n bins
  "counts": [Number],     // frequency in each bin
  "density": [Number],    // normalized (sum = 1)
  "cumulative": [Number], // cumulative counts
  "bin_width": Number,
  "n": Number
}
```

**Bin count methods:**
- Integer: exact count → `histogram(data, 10)`
- "sturges": ⌈log₂(n)⌉ + 1
- "scott": 3.5σ/n^(1/3)
- "freedman": 2·IQR/n^(1/3)
- "auto" (default): Freedman-Diaconis with Sturges fallback

```markdown
| hist | histogram(data, "scott") |  |
```

#### `bin_edges(list, bins, [method])`

Just get bin edges without counting.

```markdown
| edges | bin_edges(data, 10) | [0, 10, 20, ...] |
```

#### `frequency(list, edges)`

Count frequencies for given edges.

```markdown
| edges  | [0, 25, 50, 75, 100]       |                |
| counts | frequency(Data.Values, edges) | [12, 34, 28, 16] |
```

#### `density(list, [bandwidth])`

Kernel density estimation (Gaussian kernel).

```markdown
| kde     | density(Data.Values)        |                |
| at_50   | kde.evaluate(50)            | 0.023          |
| points  | kde.x                       | [0, 1, 2, ...] |
| values  | kde.y                       | [0.001, ...] |
```

**Returns:**
```json
{
  "bandwidth": Number,
  "x": [Number],       // evaluation points
  "y": [Number],       // density values
  "evaluate": Function // evaluate at any point
}
```

---

### Outlier Detection

#### `outliers_iqr(list, [k])`

IQR-based outlier detection.

```markdown
| out     | outliers_iqr(Data.Values)      |                |
| indices | out.indices                    | [3, 17, 42]    |
| values  | out.values                     | [150, -20, 200]|
| lower   | out.lower_fence                | 10             |
| upper   | out.upper_fence                | 90             |
```

Default k = 1.5 (standard). Use k = 3 for extreme outliers.

**Returns:**
```json
{
  "indices": [Number],   // 0-indexed positions
  "values": [Number],    // actual outlier values
  "count": Number,
  "lower_fence": Number, // Q1 - k*IQR
  "upper_fence": Number, // Q3 + k*IQR
  "q1": Number,
  "q3": Number,
  "iqr": Number
}
```

#### `outliers_zscore(list, [threshold])`

Z-score based detection.

```markdown
| out | outliers_zscore(Data.Values, 3) | |
```

Default threshold = 3.

**Returns:**
```json
{
  "indices": [Number],
  "values": [Number],
  "z_scores": [Number],  // z-scores of outliers
  "count": Number,
  "mean": Number,
  "stddev": Number,
  "threshold": Number
}
```

#### `outliers_mad(list, [threshold])`

Median Absolute Deviation based (robust to outliers themselves).

```markdown
| out | outliers_mad(Data.Values, 3.5) | |
```

Default threshold = 3.5 (≈ 3σ for normal data).

**Returns:**
```json
{
  "indices": [Number],
  "values": [Number],
  "modified_z": [Number],
  "count": Number,
  "median": Number,
  "mad": Number,
  "threshold": Number
}
```

#### `grubbs_test(list, [alpha])`

Grubbs' test for single outlier (iterative).

```markdown
| grubbs   | grubbs_test(Data.Values)  |        |
| outlier  | grubbs.outlier            | 150    |
| index    | grubbs.index              | 17     |
| g        | grubbs.g_statistic        | 3.24   |
| p        | grubbs.p_value            | 0.012  |
```

**Returns:**
```json
{
  "has_outlier": Boolean,
  "outlier": Number | null,
  "index": Number | null,
  "g_statistic": Number,
  "critical_value": Number,
  "p_value": Number
}
```

#### `outliers(list, [method], [params])`

Unified outlier detection.

```markdown
| out | outliers(data, "iqr", {k: 1.5})        | |
| out | outliers(data, "zscore", {threshold: 3}) | |
| out | outliers(data, "mad")                  | |
```

Default: "iqr" with k=1.5.

---

### Q-Q Analysis

#### `qq_points(list, distribution, [parameters])`

Generate Q-Q plot points.

```markdown
| qq       | qq_points(Data.Values, "normal")  |           |
| theory   | qq.theoretical                    | [-2.3, ...] |
| sample   | qq.sample                         | [-2.1, ...] |
| r_sq     | qq.r_squared                      | 0.987     |
```

**Returns:**
```json
{
  "theoretical": [Number],  // expected quantiles
  "sample": [Number],       // observed quantiles
  "r_squared": Number,      // linearity measure
  "slope": Number,          // should be ≈ σ
  "intercept": Number       // should be ≈ μ
}
```

#### `qq_residuals(list)`

Q-Q residuals from normal distribution.

```markdown
| resid | qq_residuals(Data.Values) | [0.1, -0.05, ...] |
```

Returns list of deviations from theoretical quantiles.

---

## Implementation Notes

### MLE Algorithms

```rust
/// Maximum likelihood estimation trait
pub trait MleEstimator {
    /// Initial parameter guess (often method of moments)
    fn initial_guess(data: &[Number]) -> Self::Params;
    
    /// Log-likelihood function
    fn log_likelihood(data: &[Number], params: &Self::Params) -> Number;
    
    /// Gradient of log-likelihood (for Newton-Raphson)
    fn gradient(data: &[Number], params: &Self::Params) -> Vec<Number>;
    
    /// Fit parameters to data
    fn fit(data: &[Number], max_iter: usize, tol: &Number) -> Result<Self::Params, FolioError>;
}
```

For distributions without closed-form MLE (Gamma, Beta, Weibull), use Newton-Raphson with:
- Max iterations: 100
- Tolerance: 1e-10
- Fallback to method of moments if non-convergence

### Precision Considerations

- All calculations use `Number` (BigRational)
- CDF/PDF evaluation may need `Float` conversion for transcendentals
- Convert back to `Rational` for final results
- KS critical values: precomputed table + interpolation

### Error Handling

```rust
// Domain errors
if data.iter().any(|x| x <= &Number::zero()) {
    return Value::Error(FolioError::domain_error(
        "fit_lognormal requires all positive values"
    ));
}

// Insufficient data
if data.len() < min_samples {
    return Value::Error(FolioError::domain_error(format!(
        "{} requires at least {} samples, got {}",
        func_name, min_samples, data.len()
    )));
}
```

Minimum sample sizes:
- General fitting: 10
- Shapiro-Wilk: 3 (max 5000)
- Grubbs: 3
- KS test: 5

---

## Examples

### Full Distribution Analysis

```markdown
## Sample Data

| i  | value |
|----|-------|
| 1  | 45.2  |
| 2  | 52.1  |
| 3  | 48.7  |
...

## Analysis

| Metric       | Formula                              | Result   |
|--------------|--------------------------------------|----------|
| data         | Sample.value                         |          |
| fit          | fit_distribution(data)               |          |
| dist         | fit.distribution                     | normal   |
| mu           | fit.parameters.mu                    | 50.23    |
| sigma        | fit.parameters.sigma                 | 8.45     |
| is_normal    | is_normal(data)                      | true     |
| outliers     | outliers_iqr(data)                   |          |
| n_outliers   | outliers.count                       | 2        |
| hist         | histogram(data, "auto")              |          |
| n_bins       | count(hist.counts)                   | 8        |
```

### Comparing Two Groups

```markdown
## Group Comparison

| Metric    | Formula                          | Result  |
|-----------|----------------------------------|---------|
| ks        | ks_test_2(Group1.value, Group2.value) |    |
| same_dist | ks.p_value > 0.05                | true    |
| fit1      | fit_distribution(Group1.value)   |         |
| fit2      | fit_distribution(Group2.value)   |         |
```

---

## Function Summary

| Category | Functions |
|----------|-----------|
| **Fitting** | `fit_distribution`, `fit_normal`, `fit_lognormal`, `fit_exponential`, `fit_uniform`, `fit_gamma`, `fit_beta`, `fit_weibull`, `fit_pareto`, `fit_poisson` |
| **Goodness-of-Fit** | `ks_test`, `ks_test_2`, `anderson_darling`, `shapiro_wilk`, `jarque_bera`, `is_normal` |
| **Histogram** | `histogram`, `bin_edges`, `frequency`, `density` |
| **Outliers** | `outliers`, `outliers_iqr`, `outliers_zscore`, `outliers_mad`, `grubbs_test` |
| **Q-Q** | `qq_points`, `qq_residuals` |

Total: 24 functions
