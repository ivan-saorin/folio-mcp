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
