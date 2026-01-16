//! Matrix property functions

use folio_core::{Number, Value};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use crate::types::Matrix;
use crate::helpers::extract_matrix;

// ============================================================================
// ROWS - Get number of rows
// ============================================================================

pub struct RowsFn;

static ROWS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to query",
    optional: false,
    default: None,
}];
static ROWS_EXAMPLES: [&str; 2] = [
    "rows(Matrix(3, 2, 1, 2, 3, 4, 5, 6)) → 3",
    "rows(Identity(4)) → 4",
];
static ROWS_RELATED: [&str; 2] = ["cols", "shape"];

impl FunctionPlugin for RowsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "rows",
            description: "Get the number of rows in a matrix",
            usage: "rows(matrix)",
            args: &ROWS_ARGS,
            returns: "Number",
            examples: &ROWS_EXAMPLES,
            category: "matrix",
            source: None,
            related: &ROWS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(folio_core::FolioError::arg_count("rows", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "rows", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        Value::Number(Number::from_i64(matrix.rows() as i64))
    }
}

// ============================================================================
// COLS - Get number of columns
// ============================================================================

pub struct ColsFn;

static COLS_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to query",
    optional: false,
    default: None,
}];
static COLS_EXAMPLES: [&str; 2] = [
    "cols(Matrix(3, 2, 1, 2, 3, 4, 5, 6)) → 2",
    "cols(Identity(4)) → 4",
];
static COLS_RELATED: [&str; 2] = ["rows", "shape"];

impl FunctionPlugin for ColsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cols",
            description: "Get the number of columns in a matrix",
            usage: "cols(matrix)",
            args: &COLS_ARGS,
            returns: "Number",
            examples: &COLS_EXAMPLES,
            category: "matrix",
            source: None,
            related: &COLS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(folio_core::FolioError::arg_count("cols", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "cols", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        Value::Number(Number::from_i64(matrix.cols() as i64))
    }
}

// ============================================================================
// SHAPE - Get matrix dimensions as list [rows, cols]
// ============================================================================

pub struct ShapeFn;

static SHAPE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to query",
    optional: false,
    default: None,
}];
static SHAPE_EXAMPLES: [&str; 2] = [
    "shape(Matrix(3, 2, 1, 2, 3, 4, 5, 6)) → [3, 2]",
    "shape(Identity(4)) → [4, 4]",
];
static SHAPE_RELATED: [&str; 2] = ["rows", "cols"];

impl FunctionPlugin for ShapeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "shape",
            description: "Get matrix dimensions as [rows, cols]",
            usage: "shape(matrix)",
            args: &SHAPE_ARGS,
            returns: "List",
            examples: &SHAPE_EXAMPLES,
            category: "matrix",
            source: None,
            related: &SHAPE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(folio_core::FolioError::arg_count("shape", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "shape", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        Value::List(vec![
            Value::Number(Number::from_i64(matrix.rows() as i64)),
            Value::Number(Number::from_i64(matrix.cols() as i64)),
        ])
    }
}

// ============================================================================
// IS_SQUARE - Check if matrix is square
// ============================================================================

pub struct IsSquareFn;

static IS_SQUARE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to check",
    optional: false,
    default: None,
}];
static IS_SQUARE_EXAMPLES: [&str; 2] = [
    "issquare(Identity(3)) → true",
    "issquare(Matrix(2, 3, 1, 2, 3, 4, 5, 6)) → false",
];
static IS_SQUARE_RELATED: [&str; 2] = ["issymmetric", "ispositivedefinite"];

impl FunctionPlugin for IsSquareFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "issquare",
            description: "Check if matrix is square (rows == cols)",
            usage: "issquare(matrix)",
            args: &IS_SQUARE_ARGS,
            returns: "Boolean",
            examples: &IS_SQUARE_EXAMPLES,
            category: "matrix",
            source: None,
            related: &IS_SQUARE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(folio_core::FolioError::arg_count("issquare", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "issquare", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        Value::Bool(matrix.rows() == matrix.cols())
    }
}

// ============================================================================
// IS_SYMMETRIC - Check if matrix is symmetric (A == A^T)
// ============================================================================

pub struct IsSymmetricFn;

static IS_SYMMETRIC_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to check",
    optional: false,
    default: None,
}];
static IS_SYMMETRIC_EXAMPLES: [&str; 2] = [
    "issymmetric(Identity(3)) → true",
    "issymmetric(Matrix(2, 2, 1, 2, 2, 1)) → true",
];
static IS_SYMMETRIC_RELATED: [&str; 2] = ["issquare", "ispositivedefinite"];

impl FunctionPlugin for IsSymmetricFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "issymmetric",
            description: "Check if matrix is symmetric (A = A^T)",
            usage: "issymmetric(matrix)",
            args: &IS_SYMMETRIC_ARGS,
            returns: "Boolean",
            examples: &IS_SYMMETRIC_EXAMPLES,
            category: "matrix",
            source: None,
            related: &IS_SYMMETRIC_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(folio_core::FolioError::arg_count("issymmetric", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "issymmetric", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if matrix.rows() != matrix.cols() {
            return Value::Bool(false);
        }

        // Check if A[i,j] == A[j,i] for all i,j
        let float_mat = matrix.to_float();
        let eps = 1e-10;
        for i in 0..float_mat.nrows() {
            for j in (i + 1)..float_mat.ncols() {
                if (float_mat[(i, j)] - float_mat[(j, i)]).abs() > eps {
                    return Value::Bool(false);
                }
            }
        }

        Value::Bool(true)
    }
}

// ============================================================================
// IS_POSITIVE_DEFINITE - Check if matrix is positive definite
// ============================================================================

pub struct IsPositiveDefiniteFn;

static IS_PD_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Symmetric matrix to check",
    optional: false,
    default: None,
}];
static IS_PD_EXAMPLES: [&str; 2] = [
    "ispositivedefinite(Identity(3)) → true",
    "ispositivedefinite(Matrix(2, 2, 2, 1, 1, 2)) → true",
];
static IS_PD_RELATED: [&str; 2] = ["issymmetric", "cholesky"];

impl FunctionPlugin for IsPositiveDefiniteFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ispositivedefinite",
            description: "Check if symmetric matrix is positive definite",
            usage: "ispositivedefinite(matrix)",
            args: &IS_PD_ARGS,
            returns: "Boolean",
            examples: &IS_PD_EXAMPLES,
            category: "matrix",
            source: None,
            related: &IS_PD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(folio_core::FolioError::arg_count("ispositivedefinite", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "ispositivedefinite", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if matrix.rows() != matrix.cols() {
            return Value::Bool(false);
        }

        // Use Cholesky decomposition attempt to check positive definiteness
        let float_matrix = matrix.to_float();
        match float_matrix.cholesky() {
            Some(_) => Value::Bool(true),
            None => Value::Bool(false),
        }
    }
}

// ============================================================================
// RANK - Compute matrix rank
// ============================================================================

pub struct RankFn;

static RANK_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to analyze",
    optional: false,
    default: None,
}];
static RANK_EXAMPLES: [&str; 2] = [
    "rank(Identity(3)) → 3",
    "rank(Matrix(2, 3, 1, 2, 3, 2, 4, 6)) → 1",
];
static RANK_RELATED: [&str; 2] = ["nullspace", "columnspace"];

impl FunctionPlugin for RankFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "rank",
            description: "Compute the rank of a matrix",
            usage: "rank(matrix)",
            args: &RANK_ARGS,
            returns: "Number",
            examples: &RANK_EXAMPLES,
            category: "matrix",
            source: None,
            related: &RANK_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(folio_core::FolioError::arg_count("rank", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "rank", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        // Use SVD to compute rank (count singular values above threshold)
        let float_matrix = matrix.to_float();
        let svd = float_matrix.svd(false, false);

        let eps = 1e-10;
        let rank = svd.singular_values.iter()
            .filter(|&&s| s.abs() > eps)
            .count();

        Value::Number(Number::from_i64(rank as i64))
    }
}

// ============================================================================
// TRACE - Sum of diagonal elements
// ============================================================================

pub struct TraceFn;

static TRACE_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Square matrix",
    optional: false,
    default: None,
}];
static TRACE_EXAMPLES: [&str; 2] = [
    "trace(Identity(3)) → 3",
    "trace(Matrix(2, 2, 1, 2, 3, 4)) → 5",
];
static TRACE_RELATED: [&str; 2] = ["diag", "determinant"];

impl FunctionPlugin for TraceFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "trace",
            description: "Compute the trace (sum of diagonal elements)",
            usage: "trace(matrix)",
            args: &TRACE_ARGS,
            returns: "Number",
            examples: &TRACE_EXAMPLES,
            category: "matrix",
            source: None,
            related: &TRACE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(folio_core::FolioError::arg_count("trace", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "trace", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if matrix.rows() != matrix.cols() {
            return Value::Error(folio_core::FolioError::domain_error("trace requires a square matrix"));
        }

        match matrix {
            Matrix::Exact(ref m) => {
                let mut sum = Number::from_i64(0);
                for i in 0..m.rows {
                    sum = sum.add(&m.data[i][i]);
                }
                Value::Number(sum)
            }
            Matrix::Float(ref m) => {
                let sum: f64 = (0..m.data.nrows()).map(|i| m.data[(i, i)]).sum();
                Value::Number(Number::from_f64(sum))
            }
        }
    }
}

// ============================================================================
// DETERMINANT - Compute matrix determinant
// ============================================================================

pub struct DeterminantFn;

static DET_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Square matrix",
    optional: false,
    default: None,
}];
static DET_EXAMPLES: [&str; 2] = [
    "determinant(Identity(3)) → 1",
    "determinant(Matrix(2, 2, 1, 2, 3, 4)) → -2",
];
static DET_RELATED: [&str; 2] = ["inverse", "rank"];

impl FunctionPlugin for DeterminantFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "determinant",
            description: "Compute the determinant of a square matrix",
            usage: "determinant(matrix)",
            args: &DET_ARGS,
            returns: "Number",
            examples: &DET_EXAMPLES,
            category: "matrix",
            source: None,
            related: &DET_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(folio_core::FolioError::arg_count("determinant", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "determinant", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if matrix.rows() != matrix.cols() {
            return Value::Error(folio_core::FolioError::domain_error("determinant requires a square matrix"));
        }

        match matrix {
            Matrix::Exact(ref m) => {
                // For small exact matrices, compute directly
                let n = m.rows;
                if n == 1 {
                    return Value::Number(m.data[0][0].clone());
                }
                if n == 2 {
                    let det = m.data[0][0].mul(&m.data[1][1]).sub(&m.data[0][1].mul(&m.data[1][0]));
                    return Value::Number(det);
                }
                // For larger matrices, use float
                let float_m = matrix.to_float();
                let det = float_m.determinant();
                Value::Number(Number::from_f64(det))
            }
            Matrix::Float(ref m) => {
                let det = m.data.determinant();
                Value::Number(Number::from_f64(det))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construct::{MatrixFn, IdentityFn};
    use std::sync::Arc;

    fn ctx() -> EvalContext {
        EvalContext::new(Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_rows_cols_shape() {
        // 2x3 matrix [[1,2,3],[4,5,6]]
        let matrix = MatrixFn.call(&[Value::List(vec![
            Value::List(vec![Value::Number(Number::from_i64(1)), Value::Number(Number::from_i64(2)), Value::Number(Number::from_i64(3))]),
            Value::List(vec![Value::Number(Number::from_i64(4)), Value::Number(Number::from_i64(5)), Value::Number(Number::from_i64(6))]),
        ])], &ctx());

        let rows = RowsFn.call(&[matrix.clone()], &ctx()).as_number().unwrap().to_i64().unwrap();
        assert_eq!(rows, 2);

        let cols = ColsFn.call(&[matrix.clone()], &ctx()).as_number().unwrap().to_i64().unwrap();
        assert_eq!(cols, 3);

        if let Value::List(shape) = ShapeFn.call(&[matrix], &ctx()) {
            assert_eq!(shape.len(), 2);
            let r = shape[0].as_number().unwrap().to_i64().unwrap();
            let c = shape[1].as_number().unwrap().to_i64().unwrap();
            assert_eq!(r, 2);
            assert_eq!(c, 3);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_is_square() {
        let identity = IdentityFn.call(&[Value::Number(Number::from_i64(3))], &ctx());
        if let Value::Bool(is_sq) = IsSquareFn.call(&[identity], &ctx()) {
            assert!(is_sq);
        } else {
            panic!("Expected bool");
        }

        // 2x3 matrix (not square)
        let rect = MatrixFn.call(&[Value::List(vec![
            Value::List(vec![Value::Number(Number::from_i64(1)), Value::Number(Number::from_i64(2)), Value::Number(Number::from_i64(3))]),
            Value::List(vec![Value::Number(Number::from_i64(4)), Value::Number(Number::from_i64(5)), Value::Number(Number::from_i64(6))]),
        ])], &ctx());
        if let Value::Bool(is_sq) = IsSquareFn.call(&[rect], &ctx()) {
            assert!(!is_sq);
        } else {
            panic!("Expected bool");
        }
    }

    #[test]
    fn test_trace() {
        let identity = IdentityFn.call(&[Value::Number(Number::from_i64(3))], &ctx());
        let trace = TraceFn.call(&[identity], &ctx()).as_number().unwrap().to_i64().unwrap();
        assert_eq!(trace, 3);
    }

    #[test]
    fn test_determinant() {
        let identity = IdentityFn.call(&[Value::Number(Number::from_i64(3))], &ctx());
        let det = DeterminantFn.call(&[identity], &ctx()).as_number().unwrap().to_f64().unwrap();
        assert!((det - 1.0).abs() < 1e-10);

        // 2x2 matrix: det([[1, 2], [3, 4]]) = 1*4 - 2*3 = -2
        let matrix = MatrixFn.call(&[Value::List(vec![
            Value::List(vec![Value::Number(Number::from_i64(1)), Value::Number(Number::from_i64(2))]),
            Value::List(vec![Value::Number(Number::from_i64(3)), Value::Number(Number::from_i64(4))]),
        ])], &ctx());
        let det = DeterminantFn.call(&[matrix], &ctx()).as_number().unwrap().to_f64().unwrap();
        assert!((det - (-2.0)).abs() < 1e-10);
    }
}
