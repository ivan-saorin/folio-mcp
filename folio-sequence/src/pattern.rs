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
