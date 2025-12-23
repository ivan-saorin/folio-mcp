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
