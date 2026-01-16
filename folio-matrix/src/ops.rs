//! Matrix operations: access, arithmetic, transformations

use folio_plugin::prelude::*;
use folio_core::Number;
use nalgebra::DMatrix;
use crate::types::{Matrix, Vector, MatrixMode, ExactMatrix, FloatMatrix};
use crate::helpers::*;

// ============ get ============

pub struct GetFn;

static GET_ARGS: [ArgMeta; 3] = [
    ArgMeta { name: "matrix_or_vector", typ: "Matrix|Vector", description: "Matrix or Vector", optional: false, default: None },
    ArgMeta { name: "row_or_index", typ: "Number", description: "Row index for matrix, or element index for vector (0-based)", optional: false, default: None },
    ArgMeta { name: "col", typ: "Number", description: "Column index for matrix (0-based), omit for vectors", optional: true, default: None },
];

static GET_EXAMPLES: [&str; 2] = ["get(m, 0, 1) → element at row 0, col 1", "get(v, 2) → element at index 2"];
static GET_RELATED: [&str; 2] = ["set", "row"];

impl FunctionPlugin for GetFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "get", description: "Get matrix or vector element", usage: "get(matrix, row, col) or get(vector, index)",
            args: &GET_ARGS, returns: "Number", examples: &GET_EXAMPLES,
            category: "matrix/access", source: None, related: &GET_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("get", 2, args.len())); }

        // Try vector first (2 args case)
        if args.len() == 2 {
            if let Ok(v) = extract_vector(&args[0], "get", "vector") {
                let idx = match extract_usize(&args[1], "get", "index") { Ok(i) => i, Err(e) => return Value::Error(e) };
                return match v.get(idx) {
                    Some(n) => Value::Number(n),
                    None => Value::Error(FolioError::domain_error(&format!("get: index {} out of bounds for vector of length {}", idx, v.len()))),
                };
            }
            // If not a vector, it's an error - matrix requires 3 args
            return Value::Error(FolioError::arg_count("get", 3, 2));
        }

        // Matrix case (3 args)
        let m = match extract_matrix(&args[0], "get", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let row = match extract_usize(&args[1], "get", "row") { Ok(r) => r, Err(e) => return Value::Error(e) };
        let col = match extract_usize(&args[2], "get", "col") { Ok(c) => c, Err(e) => return Value::Error(e) };

        match m.get(row, col) {
            Some(n) => Value::Number(n),
            None => Value::Error(FolioError::domain_error(&format!("get: index ({}, {}) out of bounds for {}×{} matrix", row, col, m.rows(), m.cols()))),
        }
    }
}

// ============ set ============

pub struct SetFn;

static SET_ARGS: [ArgMeta; 4] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
    ArgMeta { name: "row", typ: "Number", description: "Row index", optional: false, default: None },
    ArgMeta { name: "col", typ: "Number", description: "Column index", optional: false, default: None },
    ArgMeta { name: "value", typ: "Number", description: "New value", optional: false, default: None },
];

static SET_EXAMPLES: [&str; 1] = ["set(m, 0, 1, 99) → new matrix with m[0][1] = 99"];
static SET_RELATED: [&str; 2] = ["get", "matrix"];

impl FunctionPlugin for SetFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "set", description: "Set matrix element (returns new matrix)", usage: "set(matrix, row, col, value)",
            args: &SET_ARGS, returns: "Matrix", examples: &SET_EXAMPLES,
            category: "matrix/access", source: None, related: &SET_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 4 { return Value::Error(FolioError::arg_count("set", 4, args.len())); }

        let m = match extract_matrix(&args[0], "set", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let row = match extract_usize(&args[1], "set", "row") { Ok(r) => r, Err(e) => return Value::Error(e) };
        let col = match extract_usize(&args[2], "set", "col") { Ok(c) => c, Err(e) => return Value::Error(e) };
        let value = match extract_number(&args[3], "set", "value") { Ok(n) => n, Err(e) => return Value::Error(e) };

        if row >= m.rows() || col >= m.cols() {
            return Value::Error(FolioError::domain_error(&format!("set: index ({}, {}) out of bounds", row, col)));
        }

        let mut data = m.to_nested_list();
        data[row][col] = value;

        match Matrix::from_nested_list(data, MatrixMode::Auto) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ row ============

pub struct RowFn;

static ROW_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
    ArgMeta { name: "index", typ: "Number", description: "Row index", optional: false, default: None },
];

static ROW_EXAMPLES: [&str; 1] = ["row(m, 0) → first row as vector"];
static ROW_RELATED: [&str; 2] = ["col", "diag"];

impl FunctionPlugin for RowFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "row", description: "Extract row as vector", usage: "row(matrix, index)",
            args: &ROW_ARGS, returns: "Vector", examples: &ROW_EXAMPLES,
            category: "matrix/access", source: None, related: &ROW_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("row", 2, args.len())); }

        let m = match extract_matrix(&args[0], "row", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let idx = match extract_usize(&args[1], "row", "index") { Ok(i) => i, Err(e) => return Value::Error(e) };

        match m.get_row(idx) {
            Some(v) => v.into(),
            None => Value::Error(FolioError::domain_error(&format!("row: index {} out of bounds", idx))),
        }
    }
}

// ============ col ============

pub struct ColFn;

static COL_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
    ArgMeta { name: "index", typ: "Number", description: "Column index", optional: false, default: None },
];

static COL_EXAMPLES: [&str; 1] = ["col(m, 0) → first column as vector"];
static COL_RELATED: [&str; 2] = ["row", "diag"];

impl FunctionPlugin for ColFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "col", description: "Extract column as vector", usage: "col(matrix, index)",
            args: &COL_ARGS, returns: "Vector", examples: &COL_EXAMPLES,
            category: "matrix/access", source: None, related: &COL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("col", 2, args.len())); }

        let m = match extract_matrix(&args[0], "col", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let idx = match extract_usize(&args[1], "col", "index") { Ok(i) => i, Err(e) => return Value::Error(e) };

        match m.get_col(idx) {
            Some(v) => v.into(),
            None => Value::Error(FolioError::domain_error(&format!("col: index {} out of bounds", idx))),
        }
    }
}

// ============ diag ============

pub struct DiagFn;

static DIAG_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
];

static DIAG_EXAMPLES: [&str; 1] = ["diag(m) → diagonal elements"];
static DIAG_RELATED: [&str; 2] = ["trace", "diagonal"];

impl FunctionPlugin for DiagFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "diag", description: "Extract diagonal as vector", usage: "diag(matrix)",
            args: &DIAG_ARGS, returns: "Vector", examples: &DIAG_EXAMPLES,
            category: "matrix/access", source: None, related: &DIAG_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() { return Value::Error(FolioError::arg_count("diag", 1, 0)); }

        let m = match extract_matrix(&args[0], "diag", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        m.get_diag().into()
    }
}

// ============ submatrix ============

pub struct SubmatrixFn;

static SUBMATRIX_ARGS: [ArgMeta; 5] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
    ArgMeta { name: "row_start", typ: "Number", description: "Starting row", optional: false, default: None },
    ArgMeta { name: "row_end", typ: "Number", description: "Ending row (exclusive)", optional: false, default: None },
    ArgMeta { name: "col_start", typ: "Number", description: "Starting column", optional: false, default: None },
    ArgMeta { name: "col_end", typ: "Number", description: "Ending column (exclusive)", optional: false, default: None },
];

static SUBMATRIX_EXAMPLES: [&str; 1] = ["submatrix(m, 0, 2, 1, 3) → rows 0-1, cols 1-2"];
static SUBMATRIX_RELATED: [&str; 2] = ["row", "col"];

impl FunctionPlugin for SubmatrixFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "submatrix", description: "Extract submatrix", usage: "submatrix(m, row_start, row_end, col_start, col_end)",
            args: &SUBMATRIX_ARGS, returns: "Matrix", examples: &SUBMATRIX_EXAMPLES,
            category: "matrix/access", source: None, related: &SUBMATRIX_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 5 { return Value::Error(FolioError::arg_count("submatrix", 5, args.len())); }

        let m = match extract_matrix(&args[0], "submatrix", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let r0 = match extract_usize(&args[1], "submatrix", "row_start") { Ok(n) => n, Err(e) => return Value::Error(e) };
        let r1 = match extract_usize(&args[2], "submatrix", "row_end") { Ok(n) => n, Err(e) => return Value::Error(e) };
        let c0 = match extract_usize(&args[3], "submatrix", "col_start") { Ok(n) => n, Err(e) => return Value::Error(e) };
        let c1 = match extract_usize(&args[4], "submatrix", "col_end") { Ok(n) => n, Err(e) => return Value::Error(e) };

        if r1 <= r0 || c1 <= c0 {
            return Value::Error(FolioError::domain_error("submatrix: end indices must be greater than start"));
        }
        if r1 > m.rows() || c1 > m.cols() {
            return Value::Error(FolioError::domain_error("submatrix: indices out of bounds"));
        }

        let data = m.to_nested_list();
        let sub_data: Vec<Vec<Number>> = (r0..r1)
            .map(|i| (c0..c1).map(|j| data[i][j].clone()).collect())
            .collect();

        match Matrix::from_nested_list(sub_data, MatrixMode::Auto) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ transpose ============

pub struct TransposeFn;

static TRANSPOSE_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix to transpose", optional: false, default: None },
];

static TRANSPOSE_EXAMPLES: [&str; 1] = ["transpose(m) → m transposed"];
static TRANSPOSE_RELATED: [&str; 2] = ["matmul", "inverse"];

impl FunctionPlugin for TransposeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "transpose", description: "Matrix transpose", usage: "transpose(matrix)",
            args: &TRANSPOSE_ARGS, returns: "Matrix", examples: &TRANSPOSE_EXAMPLES,
            category: "matrix/ops", source: None, related: &TRANSPOSE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() { return Value::Error(FolioError::arg_count("transpose", 1, 0)); }

        let m = match extract_matrix(&args[0], "transpose", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };

        match m {
            Matrix::Exact(em) => {
                let data: Vec<Vec<Number>> = (0..em.cols)
                    .map(|j| (0..em.rows).map(|i| em.data[i][j].clone()).collect())
                    .collect();
                Matrix::Exact(ExactMatrix { data, rows: em.cols, cols: em.rows }).into()
            }
            Matrix::Float(fm) => Matrix::from_dmatrix(fm.data.transpose()).into(),
        }
    }
}

// ============ matmul ============

pub struct MatmulFn;

static MATMUL_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "a", typ: "Matrix", description: "First matrix", optional: false, default: None },
    ArgMeta { name: "b", typ: "Matrix", description: "Second matrix", optional: false, default: None },
];

static MATMUL_EXAMPLES: [&str; 1] = ["matmul(a, b) → a × b"];
static MATMUL_RELATED: [&str; 2] = ["hadamard", "transpose"];

impl FunctionPlugin for MatmulFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "matmul", description: "Matrix multiplication", usage: "matmul(a, b)",
            args: &MATMUL_ARGS, returns: "Matrix", examples: &MATMUL_EXAMPLES,
            category: "matrix/ops", source: None, related: &MATMUL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("matmul", 2, args.len())); }

        let a = match extract_matrix(&args[0], "matmul", "a") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let b = match extract_matrix(&args[1], "matmul", "b") { Ok(m) => m, Err(e) => return Value::Error(e) };

        if let Err(e) = check_matmul_dims(&a, &b, "matmul") { return Value::Error(e); }

        // Use float for multiplication (faster)
        let da = a.to_dmatrix();
        let db = b.to_dmatrix();
        Matrix::from_dmatrix(&da * &db).into()
    }
}

// ============ mat_add ============

pub struct MatAddFn;

static MAT_ADD_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "a", typ: "Matrix", description: "First matrix", optional: false, default: None },
    ArgMeta { name: "b", typ: "Matrix", description: "Second matrix", optional: false, default: None },
];

static MAT_ADD_EXAMPLES: [&str; 1] = ["mat_add(a, b) → a + b"];
static MAT_ADD_RELATED: [&str; 2] = ["mat_sub", "scale"];

impl FunctionPlugin for MatAddFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mat_add", description: "Element-wise matrix addition", usage: "mat_add(a, b)",
            args: &MAT_ADD_ARGS, returns: "Matrix", examples: &MAT_ADD_EXAMPLES,
            category: "matrix/ops", source: None, related: &MAT_ADD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("mat_add", 2, args.len())); }

        let a = match extract_matrix(&args[0], "mat_add", "a") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let b = match extract_matrix(&args[1], "mat_add", "b") { Ok(m) => m, Err(e) => return Value::Error(e) };

        if let Err(e) = check_same_dims(&a, &b, "mat_add") { return Value::Error(e); }

        let da = a.to_dmatrix();
        let db = b.to_dmatrix();
        Matrix::from_dmatrix(&da + &db).into()
    }
}

// ============ mat_sub ============

pub struct MatSubFn;

static MAT_SUB_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "a", typ: "Matrix", description: "First matrix", optional: false, default: None },
    ArgMeta { name: "b", typ: "Matrix", description: "Second matrix", optional: false, default: None },
];

static MAT_SUB_EXAMPLES: [&str; 1] = ["mat_sub(a, b) → a - b"];
static MAT_SUB_RELATED: [&str; 2] = ["mat_add", "scale"];

impl FunctionPlugin for MatSubFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mat_sub", description: "Element-wise matrix subtraction", usage: "mat_sub(a, b)",
            args: &MAT_SUB_ARGS, returns: "Matrix", examples: &MAT_SUB_EXAMPLES,
            category: "matrix/ops", source: None, related: &MAT_SUB_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("mat_sub", 2, args.len())); }

        let a = match extract_matrix(&args[0], "mat_sub", "a") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let b = match extract_matrix(&args[1], "mat_sub", "b") { Ok(m) => m, Err(e) => return Value::Error(e) };

        if let Err(e) = check_same_dims(&a, &b, "mat_sub") { return Value::Error(e); }

        let da = a.to_dmatrix();
        let db = b.to_dmatrix();
        Matrix::from_dmatrix(&da - &db).into()
    }
}

// ============ scale ============

pub struct ScaleFn;

static SCALE_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
    ArgMeta { name: "scalar", typ: "Number", description: "Scalar", optional: false, default: None },
];

static SCALE_EXAMPLES: [&str; 1] = ["scale(m, 2) → m × 2"];
static SCALE_RELATED: [&str; 2] = ["matmul", "hadamard"];

impl FunctionPlugin for ScaleFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "scale", description: "Scalar multiplication", usage: "scale(matrix, scalar)",
            args: &SCALE_ARGS, returns: "Matrix", examples: &SCALE_EXAMPLES,
            category: "matrix/ops", source: None, related: &SCALE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("scale", 2, args.len())); }

        let m = match extract_matrix(&args[0], "scale", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let s = match extract_number(&args[1], "scale", "scalar") { Ok(n) => n.to_f64().unwrap_or(f64::NAN), Err(e) => return Value::Error(e) };

        let dm = m.to_dmatrix();
        Matrix::from_dmatrix(dm * s).into()
    }
}

// ============ hadamard ============

pub struct HadamardFn;

static HADAMARD_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "a", typ: "Matrix", description: "First matrix", optional: false, default: None },
    ArgMeta { name: "b", typ: "Matrix", description: "Second matrix", optional: false, default: None },
];

static HADAMARD_EXAMPLES: [&str; 1] = ["hadamard(a, b) → element-wise a * b"];
static HADAMARD_RELATED: [&str; 2] = ["matmul", "element_div"];

impl FunctionPlugin for HadamardFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "hadamard", description: "Element-wise multiplication (Hadamard product)", usage: "hadamard(a, b)",
            args: &HADAMARD_ARGS, returns: "Matrix", examples: &HADAMARD_EXAMPLES,
            category: "matrix/ops", source: None, related: &HADAMARD_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("hadamard", 2, args.len())); }

        let a = match extract_matrix(&args[0], "hadamard", "a") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let b = match extract_matrix(&args[1], "hadamard", "b") { Ok(m) => m, Err(e) => return Value::Error(e) };

        if let Err(e) = check_same_dims(&a, &b, "hadamard") { return Value::Error(e); }

        let da = a.to_dmatrix();
        let db = b.to_dmatrix();
        Matrix::from_dmatrix(da.component_mul(&db)).into()
    }
}

// ============ element_div ============

pub struct ElementDivFn;

static ELEMENT_DIV_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "a", typ: "Matrix", description: "First matrix", optional: false, default: None },
    ArgMeta { name: "b", typ: "Matrix", description: "Second matrix", optional: false, default: None },
];

static ELEMENT_DIV_EXAMPLES: [&str; 1] = ["element_div(a, b) → element-wise a / b"];
static ELEMENT_DIV_RELATED: [&str; 2] = ["hadamard", "scale"];

impl FunctionPlugin for ElementDivFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "element_div", description: "Element-wise division", usage: "element_div(a, b)",
            args: &ELEMENT_DIV_ARGS, returns: "Matrix", examples: &ELEMENT_DIV_EXAMPLES,
            category: "matrix/ops", source: None, related: &ELEMENT_DIV_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("element_div", 2, args.len())); }

        let a = match extract_matrix(&args[0], "element_div", "a") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let b = match extract_matrix(&args[1], "element_div", "b") { Ok(m) => m, Err(e) => return Value::Error(e) };

        if let Err(e) = check_same_dims(&a, &b, "element_div") { return Value::Error(e); }

        let da = a.to_dmatrix();
        let db = b.to_dmatrix();
        Matrix::from_dmatrix(da.component_div(&db)).into()
    }
}

// ============ mat_power ============

pub struct MatPowerFn;

static MAT_POWER_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Square matrix", optional: false, default: None },
    ArgMeta { name: "n", typ: "Number", description: "Power (non-negative integer)", optional: false, default: None },
];

static MAT_POWER_EXAMPLES: [&str; 1] = ["mat_power(m, 3) → m³"];
static MAT_POWER_RELATED: [&str; 2] = ["matmul", "inverse"];

impl FunctionPlugin for MatPowerFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "mat_power", description: "Matrix power (repeated multiplication)", usage: "mat_power(matrix, n)",
            args: &MAT_POWER_ARGS, returns: "Matrix", examples: &MAT_POWER_EXAMPLES,
            category: "matrix/ops", source: None, related: &MAT_POWER_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 { return Value::Error(FolioError::arg_count("mat_power", 2, args.len())); }

        let m = match extract_matrix(&args[0], "mat_power", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let n = match extract_int(&args[1], "mat_power", "n") { Ok(n) => n, Err(e) => return Value::Error(e) };

        if let Err(e) = check_square(&m, "mat_power") { return Value::Error(e); }

        if n < 0 {
            return Value::Error(FolioError::domain_error("mat_power: n must be non-negative"));
        }

        let n = n as usize;
        let dm = m.to_dmatrix();
        let size = dm.nrows();

        if n == 0 {
            return Matrix::from_dmatrix(DMatrix::identity(size, size)).into();
        }

        let mut result = dm.clone();
        for _ in 1..n {
            result = &result * &dm;
        }

        Matrix::from_dmatrix(result).into()
    }
}

// ============ inverse ============

pub struct InverseFn;

static INVERSE_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Square matrix", optional: false, default: None },
];

static INVERSE_EXAMPLES: [&str; 1] = ["inverse(m) → m⁻¹"];
static INVERSE_RELATED: [&str; 2] = ["pinv", "det"];

impl FunctionPlugin for InverseFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "inverse", description: "Matrix inverse", usage: "inverse(matrix)",
            args: &INVERSE_ARGS, returns: "Matrix", examples: &INVERSE_EXAMPLES,
            category: "matrix/inverse", source: None, related: &INVERSE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() { return Value::Error(FolioError::arg_count("inverse", 1, 0)); }

        let m = match extract_matrix(&args[0], "inverse", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };

        if let Err(e) = check_square(&m, "inverse") { return Value::Error(e); }

        let dm = m.to_dmatrix();
        match dm.clone().try_inverse() {
            Some(inv) => Matrix::from_dmatrix(inv).into(),
            None => Value::Error(FolioError::domain_error("inverse: matrix is singular")),
        }
    }
}

// ============ pinv ============

pub struct PinvFn;

static PINV_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
    ArgMeta { name: "tolerance", typ: "Number", description: "Tolerance for zero singular values", optional: true, default: Some("1e-10") },
];

static PINV_EXAMPLES: [&str; 1] = ["pinv(m) → Moore-Penrose pseudoinverse"];
static PINV_RELATED: [&str; 2] = ["inverse", "svd"];

impl FunctionPlugin for PinvFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pinv", description: "Moore-Penrose pseudoinverse", usage: "pinv(matrix, [tolerance])",
            args: &PINV_ARGS, returns: "Matrix", examples: &PINV_EXAMPLES,
            category: "matrix/inverse", source: None, related: &PINV_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() { return Value::Error(FolioError::arg_count("pinv", 1, 0)); }

        let m = match extract_matrix(&args[0], "pinv", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let tol = args.get(1)
            .and_then(|v| v.as_number())
            .and_then(|n| n.to_f64())
            .unwrap_or(1e-10);

        let dm = m.to_dmatrix();
        let svd = dm.svd(true, true);

        let u = svd.u.unwrap();
        let vt = svd.v_t.unwrap();
        let s = svd.singular_values;

        // Compute S^+ (pseudo-inverse of singular values)
        let mut s_inv = DMatrix::zeros(vt.nrows(), u.ncols());
        for i in 0..s.len() {
            if s[i].abs() > tol {
                s_inv[(i, i)] = 1.0 / s[i];
            }
        }

        // pinv(A) = V × S^+ × U^T
        let pinv = vt.transpose() * s_inv * u.transpose();
        Matrix::from_dmatrix(pinv).into()
    }
}

// ============ reshape ============

pub struct ReshapeFn;

static RESHAPE_ARGS: [ArgMeta; 3] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
    ArgMeta { name: "rows", typ: "Number", description: "New rows", optional: false, default: None },
    ArgMeta { name: "cols", typ: "Number", description: "New columns", optional: false, default: None },
];

static RESHAPE_EXAMPLES: [&str; 1] = ["reshape(m, 2, 6) → reshaped matrix"];
static RESHAPE_RELATED: [&str; 2] = ["flatten", "matrix"];

impl FunctionPlugin for ReshapeFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "reshape", description: "Reshape matrix (same total elements)", usage: "reshape(matrix, rows, cols)",
            args: &RESHAPE_ARGS, returns: "Matrix", examples: &RESHAPE_EXAMPLES,
            category: "matrix/utility", source: None, related: &RESHAPE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 { return Value::Error(FolioError::arg_count("reshape", 3, args.len())); }

        let m = match extract_matrix(&args[0], "reshape", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let new_rows = match extract_usize(&args[1], "reshape", "rows") { Ok(n) => n, Err(e) => return Value::Error(e) };
        let new_cols = match extract_usize(&args[2], "reshape", "cols") { Ok(n) => n, Err(e) => return Value::Error(e) };

        let total = m.rows() * m.cols();
        if new_rows * new_cols != total {
            return Value::Error(FolioError::domain_error(&format!(
                "reshape: {}×{} = {} elements, but {}×{} = {} requested",
                m.rows(), m.cols(), total, new_rows, new_cols, new_rows * new_cols
            )));
        }

        // Flatten and reshape
        let data = m.to_nested_list();
        let flat: Vec<Number> = data.into_iter().flatten().collect();

        let new_data: Vec<Vec<Number>> = (0..new_rows)
            .map(|i| {
                (0..new_cols).map(|j| flat[i * new_cols + j].clone()).collect()
            })
            .collect();

        match Matrix::from_nested_list(new_data, MatrixMode::Auto) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ flatten ============

pub struct FlattenFn;

static FLATTEN_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
];

static FLATTEN_EXAMPLES: [&str; 1] = ["flatten(m) → row-major 1D vector"];
static FLATTEN_RELATED: [&str; 2] = ["reshape", "to_list"];

impl FunctionPlugin for FlattenFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "flatten", description: "Flatten matrix to 1D vector (row-major)", usage: "flatten(matrix)",
            args: &FLATTEN_ARGS, returns: "Vector", examples: &FLATTEN_EXAMPLES,
            category: "matrix/utility", source: None, related: &FLATTEN_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() { return Value::Error(FolioError::arg_count("flatten", 1, 0)); }

        let m = match extract_matrix(&args[0], "flatten", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let data = m.to_nested_list();
        let flat: Vec<Number> = data.into_iter().flatten().collect();

        Vector::from_list(flat, MatrixMode::Auto).into()
    }
}

// ============ stack_h ============

pub struct StackHFn;

static STACK_H_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "matrices", typ: "Matrix", description: "Matrices to stack", optional: false, default: None },
];

static STACK_H_EXAMPLES: [&str; 1] = ["stack_h(a, b) → [a | b]"];
static STACK_H_RELATED: [&str; 2] = ["stack_v", "matrix"];

impl FunctionPlugin for StackHFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "stack_h", description: "Horizontal stack (concatenate columns)", usage: "stack_h(m1, m2, ...)",
            args: &STACK_H_ARGS, returns: "Matrix", examples: &STACK_H_EXAMPLES,
            category: "matrix/utility", source: None, related: &STACK_H_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() { return Value::Error(FolioError::arg_count("stack_h", 1, 0)); }

        let mut matrices = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            match extract_matrix(arg, "stack_h", &format!("m{}", i)) {
                Ok(m) => matrices.push(m),
                Err(e) => return Value::Error(e),
            }
        }

        let rows = matrices[0].rows();
        for (i, m) in matrices.iter().enumerate() {
            if m.rows() != rows {
                return Value::Error(FolioError::domain_error(&format!(
                    "stack_h: matrix {} has {} rows, expected {}", i, m.rows(), rows
                )));
            }
        }

        let total_cols: usize = matrices.iter().map(|m| m.cols()).sum();
        let mut data = vec![vec![Number::from_i64(0); total_cols]; rows];

        let mut col_offset = 0;
        for m in &matrices {
            let m_data = m.to_nested_list();
            for i in 0..rows {
                for j in 0..m.cols() {
                    data[i][col_offset + j] = m_data[i][j].clone();
                }
            }
            col_offset += m.cols();
        }

        match Matrix::from_nested_list(data, MatrixMode::Auto) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ stack_v ============

pub struct StackVFn;

static STACK_V_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "matrices", typ: "Matrix", description: "Matrices to stack", optional: false, default: None },
];

static STACK_V_EXAMPLES: [&str; 1] = ["stack_v(a, b) → [a; b]"];
static STACK_V_RELATED: [&str; 2] = ["stack_h", "matrix"];

impl FunctionPlugin for StackVFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "stack_v", description: "Vertical stack (concatenate rows)", usage: "stack_v(m1, m2, ...)",
            args: &STACK_V_ARGS, returns: "Matrix", examples: &STACK_V_EXAMPLES,
            category: "matrix/utility", source: None, related: &STACK_V_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() { return Value::Error(FolioError::arg_count("stack_v", 1, 0)); }

        let mut matrices = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            match extract_matrix(arg, "stack_v", &format!("m{}", i)) {
                Ok(m) => matrices.push(m),
                Err(e) => return Value::Error(e),
            }
        }

        let cols = matrices[0].cols();
        for (i, m) in matrices.iter().enumerate() {
            if m.cols() != cols {
                return Value::Error(FolioError::domain_error(&format!(
                    "stack_v: matrix {} has {} cols, expected {}", i, m.cols(), cols
                )));
            }
        }

        let mut data = Vec::new();
        for m in &matrices {
            data.extend(m.to_nested_list());
        }

        match Matrix::from_nested_list(data, MatrixMode::Auto) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ to_list ============

pub struct ToListFn;

static TO_LIST_ARGS: [ArgMeta; 1] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
];

static TO_LIST_EXAMPLES: [&str; 1] = ["to_list(m) → nested list"];
static TO_LIST_RELATED: [&str; 2] = ["matrix", "flatten"];

impl FunctionPlugin for ToListFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "to_list", description: "Convert matrix to nested list", usage: "to_list(matrix)",
            args: &TO_LIST_ARGS, returns: "List", examples: &TO_LIST_EXAMPLES,
            category: "matrix/utility", source: None, related: &TO_LIST_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() { return Value::Error(FolioError::arg_count("to_list", 1, 0)); }

        let m = match extract_matrix(&args[0], "to_list", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let data = m.to_nested_list();

        Value::List(
            data.into_iter()
                .map(|row| Value::List(row.into_iter().map(Value::Number).collect()))
                .collect()
        )
    }
}

// ============ format_matrix ============

pub struct FormatMatrixFn;

static FORMAT_MATRIX_ARGS: [ArgMeta; 2] = [
    ArgMeta { name: "matrix", typ: "Matrix", description: "Matrix", optional: false, default: None },
    ArgMeta { name: "precision", typ: "Number", description: "Decimal places", optional: true, default: Some("4") },
];

static FORMAT_MATRIX_EXAMPLES: [&str; 1] = ["format_matrix(m, 2) → pretty string"];
static FORMAT_MATRIX_RELATED: [&str; 2] = ["to_list", "matrix"];

impl FunctionPlugin for FormatMatrixFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "format_matrix", description: "Pretty-print matrix as text", usage: "format_matrix(matrix, [precision])",
            args: &FORMAT_MATRIX_ARGS, returns: "Text", examples: &FORMAT_MATRIX_EXAMPLES,
            category: "matrix/utility", source: None, related: &FORMAT_MATRIX_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() { return Value::Error(FolioError::arg_count("format_matrix", 1, 0)); }

        let m = match extract_matrix(&args[0], "format_matrix", "matrix") { Ok(m) => m, Err(e) => return Value::Error(e) };
        let prec = args.get(1)
            .and_then(|v| v.as_number())
            .and_then(|n| n.to_i64())
            .unwrap_or(4) as usize;

        let data = m.to_nested_list();
        let lines: Vec<String> = data.iter()
            .map(|row| {
                let elements: Vec<String> = row.iter()
                    .map(|n| n.as_decimal(prec as u32))
                    .collect();
                format!("[{}]", elements.join(", "))
            })
            .collect();

        Value::Text(lines.join("\n"))
    }
}
