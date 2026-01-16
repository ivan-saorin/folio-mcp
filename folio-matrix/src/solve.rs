//! Linear system solvers

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use crate::types::Matrix;
use crate::helpers::{extract_matrix, extract_vector};
use nalgebra::DMatrix;

// ============================================================================
// SOLVE - Solve linear system Ax = b
// ============================================================================

pub struct SolveFn;

static SOLVE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "A",
        typ: "Matrix",
        description: "Coefficient matrix (square)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "b",
        typ: "Vector | Matrix",
        description: "Right-hand side vector or matrix",
        optional: false,
        default: None,
    },
];
static SOLVE_EXAMPLES: [&str; 2] = [
    "solve(Matrix(2, 2, 2, 1, 1, 3), Vector(5, 5)) → [2, 1]",
    "solve(Identity(3), Vector(1, 2, 3)) → [1, 2, 3]",
];
static SOLVE_RELATED: [&str; 2] = ["lstsq", "inverse"];

impl FunctionPlugin for SolveFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "solve",
            description: "Solve linear system Ax = b for x",
            usage: "solve(A, b)",
            args: &SOLVE_ARGS,
            returns: "List | Matrix",
            examples: &SOLVE_EXAMPLES,
            category: "matrix",
            source: None,
            related: &SOLVE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("solve", 2, args.len()));
        }

        let a_matrix = match extract_matrix(&args[0], "solve", "A") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if a_matrix.rows() != a_matrix.cols() {
            return Value::Error(FolioError::domain_error("solve requires a square coefficient matrix"));
        }

        let a_float = a_matrix.to_float();

        // Try to extract b as vector or matrix
        if let Ok(b_vec) = extract_vector(&args[1], "solve", "b") {
            let b_float = b_vec.to_float();

            if b_float.len() != a_float.nrows() {
                return Value::Error(FolioError::domain_error("Dimension mismatch: b must have same length as A rows"));
            }

            // Convert vector to column matrix for solving
            let b_mat = DMatrix::from_column_slice(b_float.len(), 1, &b_float);

            match a_float.clone().lu().solve(&b_mat) {
                Some(x) => {
                    // Return as list
                    let result: Vec<Value> = x.iter()
                        .map(|&v| Value::Number(Number::from_f64(v)))
                        .collect();
                    Value::List(result)
                }
                None => {
                    Value::Error(FolioError::domain_error("System is singular or nearly singular"))
                }
            }
        } else if let Ok(b_matrix) = extract_matrix(&args[1], "solve", "b") {
            let b_float = b_matrix.to_float();

            if b_float.nrows() != a_float.nrows() {
                return Value::Error(FolioError::domain_error("Dimension mismatch: b rows must equal A rows"));
            }

            match a_float.clone().lu().solve(&b_float) {
                Some(x) => Matrix::Float(crate::types::FloatMatrix { data: x }).to_value(),
                None => Value::Error(FolioError::domain_error("System is singular or nearly singular")),
            }
        } else {
            Value::Error(FolioError::arg_type("solve", "b", "vector or matrix", args[1].type_name()))
        }
    }
}

// ============================================================================
// LSTSQ - Least squares solution
// ============================================================================

pub struct LstsqFn;

static LSTSQ_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "A",
        typ: "Matrix",
        description: "Coefficient matrix",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "b",
        typ: "Vector",
        description: "Right-hand side vector",
        optional: false,
        default: None,
    },
];
static LSTSQ_EXAMPLES: [&str; 1] = [
    "lstsq(Matrix(3, 2, 1, 1, 1, 2, 1, 3), Vector(1, 2, 2))",
];
static LSTSQ_RELATED: [&str; 2] = ["solve", "pinv"];

impl FunctionPlugin for LstsqFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "lstsq",
            description: "Least squares solution to overdetermined system Ax ≈ b",
            usage: "lstsq(A, b)",
            args: &LSTSQ_ARGS,
            returns: "List",
            examples: &LSTSQ_EXAMPLES,
            category: "matrix",
            source: None,
            related: &LSTSQ_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("lstsq", 2, args.len()));
        }

        let a_matrix = match extract_matrix(&args[0], "lstsq", "A") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        let b_vec = match extract_vector(&args[1], "lstsq", "b") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let a_float = a_matrix.to_float();
        let b_float = b_vec.to_float();

        if b_float.len() != a_float.nrows() {
            return Value::Error(FolioError::domain_error("Dimension mismatch: b must have same length as A rows"));
        }

        // Use SVD for least squares
        let b_mat = DMatrix::from_column_slice(b_float.len(), 1, &b_float);
        let svd = a_float.svd(true, true);

        match svd.solve(&b_mat, 1e-10) {
            Ok(x) => {
                let result: Vec<Value> = x.iter()
                    .map(|&v| Value::Number(Number::from_f64(v)))
                    .collect();
                Value::List(result)
            }
            Err(_) => {
                Value::Error(FolioError::domain_error("Least squares solution failed"))
            }
        }
    }
}

// ============================================================================
// SOLVE_TRIANGULAR - Solve triangular system
// ============================================================================

pub struct SolveTriangularFn;

static SOLVE_TRI_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "A",
        typ: "Matrix",
        description: "Triangular matrix",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "b",
        typ: "Vector",
        description: "Right-hand side vector",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "lower",
        typ: "Boolean",
        description: "true for lower triangular, false for upper",
        optional: true,
        default: Some("false"),
    },
];
static SOLVE_TRI_EXAMPLES: [&str; 2] = [
    "solvetriangular(Matrix(2, 2, 1, 2, 0, 3), Vector(5, 6), false)",
    "solvetriangular(Matrix(2, 2, 2, 0, 1, 3), Vector(4, 5), true)",
];
static SOLVE_TRI_RELATED: [&str; 2] = ["solve", "lu"];

impl FunctionPlugin for SolveTriangularFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "solvetriangular",
            description: "Solve triangular system Ax = b",
            usage: "solvetriangular(A, b, lower?)",
            args: &SOLVE_TRI_ARGS,
            returns: "List",
            examples: &SOLVE_TRI_EXAMPLES,
            category: "matrix",
            source: None,
            related: &SOLVE_TRI_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 || args.len() > 3 {
            return Value::Error(FolioError::arg_count("solvetriangular", 2, args.len()));
        }

        let a_matrix = match extract_matrix(&args[0], "solvetriangular", "A") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        let b_vec = match extract_vector(&args[1], "solvetriangular", "b") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let lower = match args.get(2) {
            Some(Value::Bool(b)) => *b,
            Some(_) => return Value::Error(FolioError::arg_type("solvetriangular", "lower", "Boolean", args[2].type_name())),
            None => false,
        };

        if a_matrix.rows() != a_matrix.cols() {
            return Value::Error(FolioError::domain_error("solvetriangular requires a square matrix"));
        }

        let a_float = a_matrix.to_float();
        let b_float = b_vec.to_float();

        if b_float.len() != a_float.nrows() {
            return Value::Error(FolioError::domain_error("Dimension mismatch"));
        }

        let n = a_float.nrows();
        let mut x = vec![0.0; n];

        if lower {
            // Forward substitution for lower triangular
            for i in 0..n {
                let mut sum = b_float[i];
                for j in 0..i {
                    sum -= a_float[(i, j)] * x[j];
                }
                if a_float[(i, i)].abs() < 1e-15 {
                    return Value::Error(FolioError::domain_error("Matrix is singular"));
                }
                x[i] = sum / a_float[(i, i)];
            }
        } else {
            // Back substitution for upper triangular
            for i in (0..n).rev() {
                let mut sum = b_float[i];
                for j in (i + 1)..n {
                    sum -= a_float[(i, j)] * x[j];
                }
                if a_float[(i, i)].abs() < 1e-15 {
                    return Value::Error(FolioError::domain_error("Matrix is singular"));
                }
                x[i] = sum / a_float[(i, i)];
            }
        }

        Value::List(x.into_iter().map(|v| Value::Number(Number::from_f64(v))).collect())
    }
}

// ============================================================================
// NULL_SPACE - Compute null space basis
// ============================================================================

pub struct NullSpaceFn;

static NULL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to analyze",
    optional: false,
    default: None,
}];
static NULL_EXAMPLES: [&str; 1] = [
    "nullspace(Matrix(2, 3, 1, 2, 3, 2, 4, 6))",
];
static NULL_RELATED: [&str; 2] = ["columnspace", "rank"];

impl FunctionPlugin for NullSpaceFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "nullspace",
            description: "Compute orthonormal basis for the null space",
            usage: "nullspace(matrix)",
            args: &NULL_ARGS,
            returns: "List",
            examples: &NULL_EXAMPLES,
            category: "matrix",
            source: None,
            related: &NULL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("nullspace", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "nullspace", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        let float_mat = matrix.to_float();
        let svd = float_mat.svd(false, true);

        let eps = 1e-10;
        let v_t = svd.v_t.unwrap();
        let singular_values = &svd.singular_values;

        // Find null space vectors (corresponding to near-zero singular values)
        let mut null_vectors: Vec<Vec<f64>> = Vec::new();

        for (i, &sv) in singular_values.iter().enumerate() {
            if sv.abs() < eps {
                // This right singular vector is in the null space
                let v: Vec<f64> = (0..v_t.ncols()).map(|j| v_t[(i, j)]).collect();
                null_vectors.push(v);
            }
        }

        // Also check remaining rows of V^T if matrix is wide
        for i in singular_values.len()..v_t.nrows() {
            let v: Vec<f64> = (0..v_t.ncols()).map(|j| v_t[(i, j)]).collect();
            null_vectors.push(v);
        }

        if null_vectors.is_empty() {
            // Empty null space
            return Value::List(vec![]);
        }

        // Return as list of vectors
        let result: Vec<Value> = null_vectors.into_iter()
            .map(|v| {
                Value::List(v.into_iter()
                    .map(|x| Value::Number(Number::from_f64(x)))
                    .collect())
            })
            .collect();

        Value::List(result)
    }
}

// ============================================================================
// COLUMN_SPACE - Compute column space basis
// ============================================================================

pub struct ColumnSpaceFn;

static COL_SPACE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to analyze",
    optional: false,
    default: None,
}];
static COL_SPACE_EXAMPLES: [&str; 1] = [
    "columnspace(Matrix(3, 2, 1, 2, 3, 4, 5, 6))",
];
static COL_SPACE_RELATED: [&str; 2] = ["nullspace", "rank"];

impl FunctionPlugin for ColumnSpaceFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "columnspace",
            description: "Compute orthonormal basis for the column space",
            usage: "columnspace(matrix)",
            args: &COL_SPACE_ARGS,
            returns: "List",
            examples: &COL_SPACE_EXAMPLES,
            category: "matrix",
            source: None,
            related: &COL_SPACE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("columnspace", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "columnspace", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        let float_mat = matrix.to_float();
        let svd = float_mat.svd(true, false);

        let eps = 1e-10;
        let u = svd.u.unwrap();
        let singular_values = &svd.singular_values;

        // Find column space vectors (corresponding to non-zero singular values)
        let mut col_vectors: Vec<Vec<f64>> = Vec::new();

        for (i, &sv) in singular_values.iter().enumerate() {
            if sv.abs() > eps {
                // This left singular vector is in the column space
                let v: Vec<f64> = (0..u.nrows()).map(|j| u[(j, i)]).collect();
                col_vectors.push(v);
            }
        }

        if col_vectors.is_empty() {
            // Zero matrix - empty column space
            return Value::List(vec![]);
        }

        // Return as list of vectors
        let result: Vec<Value> = col_vectors.into_iter()
            .map(|v| {
                Value::List(v.into_iter()
                    .map(|x| Value::Number(Number::from_f64(x)))
                    .collect())
            })
            .collect();

        Value::List(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construct::{MatrixFn, VectorFn, IdentityFn};
    use std::sync::Arc;

    fn ctx() -> EvalContext {
        EvalContext::new(Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_solve() {
        // 2x + y = 5
        // x + 3y = 5
        // Solution: x = 2, y = 1
        let a = MatrixFn.call(&[Value::List(vec![
            Value::List(vec![Value::Number(Number::from_i64(2)), Value::Number(Number::from_i64(1))]),
            Value::List(vec![Value::Number(Number::from_i64(1)), Value::Number(Number::from_i64(3))]),
        ])], &ctx());

        let b = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(5)),
        ])], &ctx());

        if let Value::List(solution) = SolveFn.call(&[a, b], &ctx()) {
            assert_eq!(solution.len(), 2);
            if let (Value::Number(x), Value::Number(y)) = (&solution[0], &solution[1]) {
                let xf = x.to_f64().unwrap();
                let yf = y.to_f64().unwrap();
                assert!((xf - 2.0).abs() < 1e-10);
                assert!((yf - 1.0).abs() < 1e-10);
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_solve_identity() {
        let a = IdentityFn.call(&[Value::Number(Number::from_i64(3))], &ctx());
        let b = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])], &ctx());

        if let Value::List(solution) = SolveFn.call(&[a, b], &ctx()) {
            assert_eq!(solution.len(), 3);
            if let Value::Number(x) = &solution[0] {
                assert!((x.to_f64().unwrap() - 1.0).abs() < 1e-10);
            }
        } else {
            panic!("Expected list");
        }
    }
}
