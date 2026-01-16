//! Matrix decomposition functions

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use crate::types::Matrix;
use crate::helpers::extract_matrix;

// ============================================================================
// LU - LU decomposition
// ============================================================================

pub struct LuFn;

static LU_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Square matrix to decompose",
    optional: false,
    default: None,
}];
static LU_EXAMPLES: [&str; 1] = [
    "lu(Matrix(3, 3, 1, 2, 3, 4, 5, 6, 7, 8, 10)) → {L, U, P}",
];
static LU_RELATED: [&str; 2] = ["qr", "cholesky"];

impl FunctionPlugin for LuFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "lu",
            description: "LU decomposition with partial pivoting (returns {L, U, P})",
            usage: "lu(matrix)",
            args: &LU_ARGS,
            returns: "Object",
            examples: &LU_EXAMPLES,
            category: "matrix",
            source: None,
            related: &LU_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("lu", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "lu", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if matrix.rows() != matrix.cols() {
            return Value::Error(FolioError::domain_error("LU decomposition requires a square matrix"));
        }

        let float_mat = matrix.to_float();
        let lu = float_mat.clone().lu();
        let (p, l, u) = lu.unpack();

        // Convert to Matrix objects
        let l_mat = Matrix::Float(crate::types::FloatMatrix { data: l });
        let u_mat = Matrix::Float(crate::types::FloatMatrix { data: u });

        // Convert permutation to matrix by applying to identity
        let n = float_mat.nrows();
        let mut p_data = nalgebra::DMatrix::<f64>::identity(n, n);
        p.permute_rows(&mut p_data);
        let p_mat = Matrix::Float(crate::types::FloatMatrix { data: p_data });

        // Return as object with L, U, P fields
        Value::Object(vec![
            ("L".to_string(), l_mat.to_value()),
            ("U".to_string(), u_mat.to_value()),
            ("P".to_string(), p_mat.to_value()),
        ].into_iter().collect())
    }
}

// ============================================================================
// QR - QR decomposition
// ============================================================================

pub struct QrFn;

static QR_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to decompose (m >= n)",
    optional: false,
    default: None,
}];
static QR_EXAMPLES: [&str; 1] = [
    "qr(Matrix(3, 2, 1, 2, 3, 4, 5, 6)) → {Q, R}",
];
static QR_RELATED: [&str; 2] = ["lu", "svd"];

impl FunctionPlugin for QrFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "qr",
            description: "QR decomposition (returns {Q, R})",
            usage: "qr(matrix)",
            args: &QR_ARGS,
            returns: "Object",
            examples: &QR_EXAMPLES,
            category: "matrix",
            source: None,
            related: &QR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("qr", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "qr", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        let float_mat = matrix.to_float();
        let qr = float_mat.qr();

        let q = qr.q();
        let r = qr.r();

        let q_mat = Matrix::Float(crate::types::FloatMatrix { data: q });
        let r_mat = Matrix::Float(crate::types::FloatMatrix { data: r });

        Value::Object(vec![
            ("Q".to_string(), q_mat.to_value()),
            ("R".to_string(), r_mat.to_value()),
        ].into_iter().collect())
    }
}

// ============================================================================
// SVD - Singular Value Decomposition
// ============================================================================

pub struct SvdFn;

static SVD_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Matrix to decompose",
    optional: false,
    default: None,
}];
static SVD_EXAMPLES: [&str; 1] = [
    "svd(Matrix(3, 2, 1, 2, 3, 4, 5, 6)) → {U, S, V}",
];
static SVD_RELATED: [&str; 2] = ["qr", "eigen"];

impl FunctionPlugin for SvdFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "svd",
            description: "Singular value decomposition (returns {U, S, V})",
            usage: "svd(matrix)",
            args: &SVD_ARGS,
            returns: "Object",
            examples: &SVD_EXAMPLES,
            category: "matrix",
            source: None,
            related: &SVD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("svd", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "svd", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        let float_mat = matrix.to_float();
        let svd = float_mat.svd(true, true);

        let u = svd.u.unwrap();
        let v_t = svd.v_t.unwrap();
        let s = svd.singular_values;

        let u_mat = Matrix::Float(crate::types::FloatMatrix { data: u });
        let v_mat = Matrix::Float(crate::types::FloatMatrix { data: v_t.transpose() });

        // Convert singular values to list
        let s_list: Vec<Value> = s.iter()
            .map(|&x| Value::Number(Number::from_f64(x)))
            .collect();

        Value::Object(vec![
            ("U".to_string(), u_mat.to_value()),
            ("S".to_string(), Value::List(s_list)),
            ("V".to_string(), v_mat.to_value()),
        ].into_iter().collect())
    }
}

// ============================================================================
// CHOLESKY - Cholesky decomposition
// ============================================================================

pub struct CholeskyFn;

static CHOL_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Symmetric positive definite matrix",
    optional: false,
    default: None,
}];
static CHOL_EXAMPLES: [&str; 1] = [
    "cholesky(Matrix(2, 2, 4, 2, 2, 5)) → lower triangular L",
];
static CHOL_RELATED: [&str; 2] = ["lu", "ispositivedefinite"];

impl FunctionPlugin for CholeskyFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cholesky",
            description: "Cholesky decomposition (returns lower triangular L where A = L*L')",
            usage: "cholesky(matrix)",
            args: &CHOL_ARGS,
            returns: "Matrix",
            examples: &CHOL_EXAMPLES,
            category: "matrix",
            source: None,
            related: &CHOL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("cholesky", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "cholesky", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if matrix.rows() != matrix.cols() {
            return Value::Error(FolioError::domain_error("Cholesky decomposition requires a square matrix"));
        }

        let float_mat = matrix.to_float();

        match float_mat.cholesky() {
            Some(chol) => {
                let l = chol.l();
                Matrix::Float(crate::types::FloatMatrix { data: l }).to_value()
            }
            None => {
                Value::Error(FolioError::domain_error("Matrix is not positive definite"))
            }
        }
    }
}

// ============================================================================
// EIGEN - Eigenvalue decomposition
// ============================================================================

pub struct EigenFn;

static EIGEN_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Square matrix",
    optional: false,
    default: None,
}];
static EIGEN_EXAMPLES: [&str; 1] = [
    "eigen(Matrix(2, 2, 1, 2, 2, 1)) → {values, vectors}",
];
static EIGEN_RELATED: [&str; 2] = ["svd", "schur"];

impl FunctionPlugin for EigenFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "eigen",
            description: "Eigenvalue decomposition (returns {values, vectors})",
            usage: "eigen(matrix)",
            args: &EIGEN_ARGS,
            returns: "Object",
            examples: &EIGEN_EXAMPLES,
            category: "matrix",
            source: None,
            related: &EIGEN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("eigen", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "eigen", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if matrix.rows() != matrix.cols() {
            return Value::Error(FolioError::domain_error("Eigen decomposition requires a square matrix"));
        }

        let float_mat = matrix.to_float();

        // For symmetric matrices, use symmetric eigendecomposition
        let is_symmetric = {
            let eps = 1e-10;
            let mut sym = true;
            for i in 0..float_mat.nrows() {
                for j in (i + 1)..float_mat.ncols() {
                    if (float_mat[(i, j)] - float_mat[(j, i)]).abs() > eps {
                        sym = false;
                        break;
                    }
                }
                if !sym { break; }
            }
            sym
        };

        if is_symmetric {
            let eigen = float_mat.symmetric_eigen();

            let values: Vec<Value> = eigen.eigenvalues.iter()
                .map(|&x| Value::Number(Number::from_f64(x)))
                .collect();

            let vectors_mat = Matrix::Float(crate::types::FloatMatrix { data: eigen.eigenvectors });

            Value::Object(vec![
                ("values".to_string(), Value::List(values)),
                ("vectors".to_string(), vectors_mat.to_value()),
            ].into_iter().collect())
        } else {
            // For non-symmetric, use Schur decomposition to get eigenvalues
            let schur = float_mat.schur();
            let t = schur.unpack().1;

            // Extract diagonal (eigenvalues for upper triangular)
            let values: Vec<Value> = (0..t.nrows())
                .map(|i| Value::Number(Number::from_f64(t[(i, i)])))
                .collect();

            Value::Object(vec![
                ("values".to_string(), Value::List(values)),
                ("note".to_string(), Value::Text("For non-symmetric matrices, only real eigenvalues are returned".to_string())),
            ].into_iter().collect())
        }
    }
}

// ============================================================================
// SCHUR - Schur decomposition
// ============================================================================

pub struct SchurFn;

static SCHUR_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "matrix",
    typ: "Matrix",
    description: "Square matrix",
    optional: false,
    default: None,
}];
static SCHUR_EXAMPLES: [&str; 1] = [
    "schur(Matrix(3, 3, 1, 2, 3, 0, 4, 5, 0, 0, 6)) → {Q, T}",
];
static SCHUR_RELATED: [&str; 2] = ["eigen", "qr"];

impl FunctionPlugin for SchurFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "schur",
            description: "Schur decomposition (returns {Q, T} where A = Q*T*Q')",
            usage: "schur(matrix)",
            args: &SCHUR_ARGS,
            returns: "Object",
            examples: &SCHUR_EXAMPLES,
            category: "matrix",
            source: None,
            related: &SCHUR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("schur", 1, args.len()));
        }

        let matrix = match extract_matrix(&args[0], "schur", "matrix") {
            Ok(m) => m,
            Err(e) => return Value::Error(e),
        };

        if matrix.rows() != matrix.cols() {
            return Value::Error(FolioError::domain_error("Schur decomposition requires a square matrix"));
        }

        let float_mat = matrix.to_float();
        let schur = float_mat.schur();
        let (q, t) = schur.unpack();

        let q_mat = Matrix::Float(crate::types::FloatMatrix { data: q });
        let t_mat = Matrix::Float(crate::types::FloatMatrix { data: t });

        Value::Object(vec![
            ("Q".to_string(), q_mat.to_value()),
            ("T".to_string(), t_mat.to_value()),
        ].into_iter().collect())
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
    fn test_lu_decomposition() {
        // 2x2 matrix [[4, 3], [6, 3]]
        let matrix = MatrixFn.call(&[Value::List(vec![
            Value::List(vec![Value::Number(Number::from_i64(4)), Value::Number(Number::from_i64(3))]),
            Value::List(vec![Value::Number(Number::from_i64(6)), Value::Number(Number::from_i64(3))]),
        ])], &ctx());

        if let Value::Object(result) = LuFn.call(&[matrix], &ctx()) {
            assert!(result.contains_key("L"));
            assert!(result.contains_key("U"));
            assert!(result.contains_key("P"));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_qr_decomposition() {
        // 3x2 matrix [[1, 2], [3, 4], [5, 6]]
        let matrix = MatrixFn.call(&[Value::List(vec![
            Value::List(vec![Value::Number(Number::from_i64(1)), Value::Number(Number::from_i64(2))]),
            Value::List(vec![Value::Number(Number::from_i64(3)), Value::Number(Number::from_i64(4))]),
            Value::List(vec![Value::Number(Number::from_i64(5)), Value::Number(Number::from_i64(6))]),
        ])], &ctx());

        if let Value::Object(result) = QrFn.call(&[matrix], &ctx()) {
            assert!(result.contains_key("Q"));
            assert!(result.contains_key("R"));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_svd() {
        // 2x2 matrix [[1, 2], [3, 4]]
        let matrix = MatrixFn.call(&[Value::List(vec![
            Value::List(vec![Value::Number(Number::from_i64(1)), Value::Number(Number::from_i64(2))]),
            Value::List(vec![Value::Number(Number::from_i64(3)), Value::Number(Number::from_i64(4))]),
        ])], &ctx());

        if let Value::Object(result) = SvdFn.call(&[matrix], &ctx()) {
            assert!(result.contains_key("U"));
            assert!(result.contains_key("S"));
            assert!(result.contains_key("V"));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_schur() {
        let matrix = IdentityFn.call(&[Value::Number(Number::from_i64(3))], &ctx());

        if let Value::Object(result) = SchurFn.call(&[matrix], &ctx()) {
            assert!(result.contains_key("Q"));
            assert!(result.contains_key("T"));
        } else {
            panic!("Expected object");
        }
    }
}
