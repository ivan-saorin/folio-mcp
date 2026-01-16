//! Matrix and vector norm functions

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use crate::types::{Matrix, Vector};
use crate::helpers::{extract_matrix, extract_vector};

// ============================================================================
// NORM - Compute matrix or vector norm
// ============================================================================

pub struct NormFn;

static NORM_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "matrix_or_vector",
        typ: "Matrix | Vector",
        description: "Matrix or vector",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "ord",
        typ: "Number | Text",
        description: "Norm type: 1, 2, 'fro', 'inf'",
        optional: true,
        default: Some("2 for vectors, 'fro' for matrices"),
    },
];
static NORM_EXAMPLES: [&str; 3] = [
    "norm(Vector(3, 4)) → 5",
    "norm(Matrix(2, 2, 1, 2, 3, 4), \"fro\") → Frobenius norm",
    "norm(Vector(1, 2, 3), 1) → L1 norm",
];
static NORM_RELATED: [&str; 2] = ["normalize", "conditionnumber"];

impl FunctionPlugin for NormFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "norm",
            description: "Compute the norm of a matrix or vector",
            usage: "norm(matrix_or_vector, ord?)",
            args: &NORM_ARGS,
            returns: "Number",
            examples: &NORM_EXAMPLES,
            category: "matrix",
            source: None,
            related: &NORM_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() || args.len() > 2 {
            return Value::Error(FolioError::arg_count("norm", 1, args.len()));
        }

        // Try as vector first, then as matrix
        if let Ok(vector) = extract_vector(&args[0], "norm", "input") {
            return compute_vector_norm(vector, args.get(1));
        }

        if let Ok(matrix) = extract_matrix(&args[0], "norm", "input") {
            return compute_matrix_norm(matrix, args.get(1));
        }

        Value::Error(FolioError::arg_type("norm", "input", "matrix or vector", args[0].type_name()))
    }
}

fn compute_vector_norm(vector: Vector, ord: Option<&Value>) -> Value {
    let float_vec = vector.to_float();

    let norm_type = match ord {
        None => 2, // Default to L2 norm
        Some(Value::Number(n)) => {
            match n.to_i64() {
                Some(1) => 1,
                Some(2) => 2,
                _ => return Value::Error(FolioError::domain_error("Vector norm order must be 1 or 2")),
            }
        }
        Some(Value::Text(s)) if s == "inf" => 0, // Use 0 as sentinel for infinity norm
        Some(_) => return Value::Error(FolioError::domain_error("Invalid norm type")),
    };

    let result = match norm_type {
        1 => float_vec.iter().map(|x| x.abs()).sum::<f64>(),
        2 => float_vec.iter().map(|x| x * x).sum::<f64>().sqrt(),
        0 => float_vec.iter().map(|x| x.abs()).fold(0.0_f64, |a, b| a.max(b)),
        _ => unreachable!(),
    };

    Value::Number(Number::from_f64(result))
}

fn compute_matrix_norm(matrix: Matrix, ord: Option<&Value>) -> Value {
    let float_mat = matrix.to_float();

    let norm_type = match ord {
        None => "fro",
        Some(Value::Text(s)) if s == "fro" => "fro",
        Some(Value::Text(s)) if s == "inf" => "inf",
        Some(Value::Number(n)) => {
            match n.to_i64() {
                Some(1) => "1",
                Some(2) => "2",
                _ => return Value::Error(FolioError::domain_error("Matrix norm order must be 1, 2, 'fro', or 'inf'")),
            }
        }
        Some(_) => return Value::Error(FolioError::domain_error("Invalid norm type")),
    };

    let result = match norm_type {
        "fro" => {
            // Frobenius norm: sqrt(sum of squares)
            float_mat.iter().map(|x| x * x).sum::<f64>().sqrt()
        }
        "1" => {
            // Maximum column sum
            (0..float_mat.ncols())
                .map(|j| (0..float_mat.nrows()).map(|i| float_mat[(i, j)].abs()).sum::<f64>())
                .fold(0.0_f64, |a, b| a.max(b))
        }
        "2" => {
            // Spectral norm: largest singular value
            let svd = float_mat.svd(false, false);
            svd.singular_values.iter().cloned().fold(0.0_f64, f64::max)
        }
        "inf" => {
            // Maximum row sum
            (0..float_mat.nrows())
                .map(|i| (0..float_mat.ncols()).map(|j| float_mat[(i, j)].abs()).sum::<f64>())
                .fold(0.0_f64, |a, b| a.max(b))
        }
        _ => unreachable!(),
    };

    Value::Number(Number::from_f64(result))
}

// ============================================================================
// NORMALIZE - Normalize a vector to unit length
// ============================================================================

pub struct NormalizeFn;

static NORMALIZE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "vector",
    typ: "Vector",
    description: "Vector to normalize",
    optional: false,
    default: None,
}];
static NORMALIZE_EXAMPLES: [&str; 2] = [
    "normalize(Vector(3, 4)) → [0.6, 0.8]",
    "normalize(Vector(1, 1, 1)) → unit vector",
];
static NORMALIZE_RELATED: [&str; 2] = ["norm", "dot"];

impl FunctionPlugin for NormalizeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "normalize",
            description: "Normalize a vector to unit length",
            usage: "normalize(vector)",
            args: &NORMALIZE_ARGS,
            returns: "List",
            examples: &NORMALIZE_EXAMPLES,
            category: "matrix",
            source: None,
            related: &NORMALIZE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("normalize", 1, args.len()));
        }

        let vector = match extract_vector(&args[0], "normalize", "vector") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let float_vec = vector.to_float();
        let norm: f64 = float_vec.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm < 1e-15 {
            return Value::Error(FolioError::domain_error("Cannot normalize zero vector"));
        }

        let normalized: Vec<f64> = float_vec.iter().map(|x| x / norm).collect();

        // Return as list of numbers
        Value::List(
            normalized.into_iter()
                .map(|x| Value::Number(Number::from_f64(x)))
                .collect()
        )
    }
}

// ============================================================================
// CONDITION_NUMBER - Compute matrix condition number
// ============================================================================

pub struct ConditionNumberFn;

static COND_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to analyze",
    optional: false,
    default: None,
}];
static COND_EXAMPLES: [&str; 2] = [
    "conditionnumber(Identity(3)) → 1",
    "conditionnumber(Matrix(2, 2, 1, 2, 3, 4))",
];
static COND_RELATED: [&str; 2] = ["norm", "inverse"];

impl FunctionPlugin for ConditionNumberFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "conditionnumber",
            description: "Compute the condition number of a matrix",
            usage: "conditionnumber(matrix)",
            args: &COND_ARGS,
            returns: "Number",
            examples: &COND_EXAMPLES,
            category: "matrix",
            source: None,
            related: &COND_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("conditionnumber", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "conditionnumber", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if matrix.rows() != matrix.cols() {
            return Value::Error(FolioError::domain_error("conditionnumber requires a square matrix"));
        }

        // Use 2-norm condition number (ratio of largest to smallest singular value)
        let float_mat = matrix.to_float();
        let svd = float_mat.svd(false, false);

        let singular_values = &svd.singular_values;
        let max_sv = singular_values.iter().cloned().fold(0.0_f64, f64::max);
        let min_sv = singular_values.iter().cloned().fold(f64::INFINITY, f64::min);

        if min_sv < 1e-15 {
            return Value::Number(Number::from_f64(f64::INFINITY));
        }

        Value::Number(Number::from_f64(max_sv / min_sv))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construct::{VectorFn, IdentityFn};
    use std::sync::Arc;

    fn ctx() -> EvalContext {
        EvalContext::new(Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_vector_norm() {
        // 3-4-5 right triangle
        let v = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
        ])], &ctx());

        if let Value::Number(n) = NormFn.call(&[v], &ctx()) {
            let f = n.to_f64().unwrap();
            assert!((f - 5.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_normalize() {
        let v = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
        ])], &ctx());

        if let Value::List(normalized) = NormalizeFn.call(&[v], &ctx()) {
            assert_eq!(normalized.len(), 2);
            if let (Value::Number(x), Value::Number(y)) = (&normalized[0], &normalized[1]) {
                let xf = x.to_f64().unwrap();
                let yf = y.to_f64().unwrap();
                assert!((xf - 0.6).abs() < 1e-10);
                assert!((yf - 0.8).abs() < 1e-10);
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_condition_number() {
        // Identity has condition number 1
        let identity = IdentityFn.call(&[Value::Number(Number::from_i64(3))], &ctx());

        if let Value::Number(n) = ConditionNumberFn.call(&[identity], &ctx()) {
            let f = n.to_f64().unwrap();
            assert!((f - 1.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }
    }
}
