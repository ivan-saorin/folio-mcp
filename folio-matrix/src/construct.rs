//! Matrix and vector construction functions

use folio_plugin::prelude::*;
use folio_core::Number;
use crate::types::{Matrix, Vector, MatrixMode, ExactMatrix};
use crate::helpers::*;

// ============ matrix ============

pub struct MatrixFn;

static MATRIX_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "data",
        typ: "List",
        description: "Nested list of numbers [[row1], [row2], ...]",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "mode",
        typ: "Text",
        description: "Computation mode: 'exact', 'fast', or 'auto'",
        optional: true,
        default: Some("auto"),
    },
];

static MATRIX_EXAMPLES: [&str; 2] = [
    "matrix([[1,2,3],[4,5,6]]) → 2×3 matrix",
    "matrix([[1,2],[3,4]], \"exact\") → exact 2×2 matrix",
];

static MATRIX_RELATED: [&str; 3] = ["vector", "identity", "zeros"];

impl FunctionPlugin for MatrixFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "matrix",
            description: "Create a matrix from nested list",
            usage: "matrix(data, [mode])",
            args: &MATRIX_ARGS,
            returns: "Matrix",
            examples: &MATRIX_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &MATRIX_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("matrix", 1, 0));
        }

        let mode = parse_mode(args.get(1));
        match extract_matrix(&args[0], "matrix", "data") {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ vector ============

pub struct VectorFn;

static VECTOR_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "data",
        typ: "List",
        description: "List of numbers",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "mode",
        typ: "Text",
        description: "Computation mode",
        optional: true,
        default: Some("auto"),
    },
];

static VECTOR_EXAMPLES: [&str; 1] = ["vector([1, 2, 3]) → column vector"];

static VECTOR_RELATED: [&str; 3] = ["matrix", "row_vector", "dot"];

impl FunctionPlugin for VectorFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "vector",
            description: "Create a column vector from list",
            usage: "vector(data, [mode])",
            args: &VECTOR_ARGS,
            returns: "Vector",
            examples: &VECTOR_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &VECTOR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("vector", 1, 0));
        }

        let mode = parse_mode(args.get(1));
        match extract_number_list(&args[0], "vector", "data") {
            Ok(data) => Vector::from_list(data, mode).into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ row_vector ============

pub struct RowVectorFn;

static ROW_VECTOR_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "data",
        typ: "List",
        description: "List of numbers",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "mode",
        typ: "Text",
        description: "Computation mode",
        optional: true,
        default: Some("auto"),
    },
];

static ROW_VECTOR_EXAMPLES: [&str; 1] = ["row_vector([1, 2, 3]) → 1×3 matrix"];

static ROW_VECTOR_RELATED: [&str; 2] = ["vector", "matrix"];

impl FunctionPlugin for RowVectorFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "row_vector",
            description: "Create a row vector (1×n matrix) from list",
            usage: "row_vector(data, [mode])",
            args: &ROW_VECTOR_ARGS,
            returns: "Matrix",
            examples: &ROW_VECTOR_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &ROW_VECTOR_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("row_vector", 1, 0));
        }

        let mode = parse_mode(args.get(1));
        match extract_number_list(&args[0], "row_vector", "data") {
            Ok(data) => {
                let v = Vector::from_list(data, mode);
                v.to_row_matrix().into()
            }
            Err(e) => Value::Error(e),
        }
    }
}

// ============ identity ============

pub struct IdentityFn;

static IDENTITY_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "n",
        typ: "Number",
        description: "Size of identity matrix",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "mode",
        typ: "Text",
        description: "Computation mode",
        optional: true,
        default: Some("auto"),
    },
];

static IDENTITY_EXAMPLES: [&str; 1] = ["identity(3) → 3×3 identity matrix"];

static IDENTITY_RELATED: [&str; 3] = ["zeros", "ones", "diagonal"];

impl FunctionPlugin for IdentityFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "identity",
            description: "Create n×n identity matrix",
            usage: "identity(n, [mode])",
            args: &IDENTITY_ARGS,
            returns: "Matrix",
            examples: &IDENTITY_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &IDENTITY_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("identity", 1, 0));
        }

        let n = match extract_usize(&args[0], "identity", "n") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if n == 0 {
            return Value::Error(FolioError::domain_error("identity: n must be positive"));
        }

        let mode = parse_mode(args.get(1));
        let one = Number::from_i64(1);
        let zero = Number::from_i64(0);

        let data: Vec<Vec<Number>> = (0..n)
            .map(|i| {
                (0..n).map(|j| {
                    if i == j { one.clone() } else { zero.clone() }
                }).collect()
            })
            .collect();

        match Matrix::from_nested_list(data, mode) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ zeros ============

pub struct ZerosFn;

static ZEROS_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "rows",
        typ: "Number",
        description: "Number of rows",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "cols",
        typ: "Number",
        description: "Number of columns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "mode",
        typ: "Text",
        description: "Computation mode",
        optional: true,
        default: Some("auto"),
    },
];

static ZEROS_EXAMPLES: [&str; 1] = ["zeros(3, 4) → 3×4 matrix of zeros"];

static ZEROS_RELATED: [&str; 2] = ["ones", "identity"];

impl FunctionPlugin for ZerosFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "zeros",
            description: "Create matrix of zeros",
            usage: "zeros(rows, cols, [mode])",
            args: &ZEROS_ARGS,
            returns: "Matrix",
            examples: &ZEROS_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &ZEROS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("zeros", 2, args.len()));
        }

        let rows = match extract_usize(&args[0], "zeros", "rows") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let cols = match extract_usize(&args[1], "zeros", "cols") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if rows == 0 || cols == 0 {
            return Value::Error(FolioError::domain_error("zeros: dimensions must be positive"));
        }

        let mode = parse_mode(args.get(2));
        let zero = Number::from_i64(0);
        let data: Vec<Vec<Number>> = vec![vec![zero; cols]; rows];

        match Matrix::from_nested_list(data, mode) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ ones ============

pub struct OnesFn;

static ONES_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "rows",
        typ: "Number",
        description: "Number of rows",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "cols",
        typ: "Number",
        description: "Number of columns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "mode",
        typ: "Text",
        description: "Computation mode",
        optional: true,
        default: Some("auto"),
    },
];

static ONES_EXAMPLES: [&str; 1] = ["ones(2, 3) → 2×3 matrix of ones"];

static ONES_RELATED: [&str; 2] = ["zeros", "identity"];

impl FunctionPlugin for OnesFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ones",
            description: "Create matrix of ones",
            usage: "ones(rows, cols, [mode])",
            args: &ONES_ARGS,
            returns: "Matrix",
            examples: &ONES_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &ONES_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("ones", 2, args.len()));
        }

        let rows = match extract_usize(&args[0], "ones", "rows") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let cols = match extract_usize(&args[1], "ones", "cols") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if rows == 0 || cols == 0 {
            return Value::Error(FolioError::domain_error("ones: dimensions must be positive"));
        }

        let mode = parse_mode(args.get(2));
        let one = Number::from_i64(1);
        let data: Vec<Vec<Number>> = vec![vec![one; cols]; rows];

        match Matrix::from_nested_list(data, mode) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ diagonal ============

pub struct DiagonalFn;

static DIAGONAL_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "values",
        typ: "List",
        description: "Diagonal values",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "mode",
        typ: "Text",
        description: "Computation mode",
        optional: true,
        default: Some("auto"),
    },
];

static DIAGONAL_EXAMPLES: [&str; 1] = ["diagonal([1, 2, 3]) → diag matrix with 1,2,3"];

static DIAGONAL_RELATED: [&str; 2] = ["identity", "diag"];

impl FunctionPlugin for DiagonalFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "diagonal",
            description: "Create diagonal matrix from list",
            usage: "diagonal(values, [mode])",
            args: &DIAGONAL_ARGS,
            returns: "Matrix",
            examples: &DIAGONAL_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &DIAGONAL_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("diagonal", 1, 0));
        }

        let values = match extract_number_list(&args[0], "diagonal", "values") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if values.is_empty() {
            return Value::Error(FolioError::domain_error("diagonal: values cannot be empty"));
        }

        let n = values.len();
        let mode = parse_mode(args.get(1));
        let zero = Number::from_i64(0);

        let data: Vec<Vec<Number>> = (0..n)
            .map(|i| {
                (0..n).map(|j| {
                    if i == j { values[i].clone() } else { zero.clone() }
                }).collect()
            })
            .collect();

        match Matrix::from_nested_list(data, mode) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ from_columns ============

pub struct FromColumnsFn;

static FROM_COLUMNS_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "columns",
        typ: "List",
        description: "Column vectors",
        optional: false,
        default: None,
    },
];

static FROM_COLUMNS_EXAMPLES: [&str; 1] = ["from_columns([1,2,3], [4,5,6]) → 3×2 matrix"];

static FROM_COLUMNS_RELATED: [&str; 2] = ["from_rows", "matrix"];

impl FunctionPlugin for FromColumnsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "from_columns",
            description: "Build matrix from column vectors",
            usage: "from_columns(col1, col2, ...)",
            args: &FROM_COLUMNS_ARGS,
            returns: "Matrix",
            examples: &FROM_COLUMNS_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &FROM_COLUMNS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("from_columns", 1, 0));
        }

        // Each arg is a column
        let mut columns: Vec<Vec<Number>> = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            match extract_number_list(arg, "from_columns", &format!("col{}", i)) {
                Ok(col) => columns.push(col),
                Err(e) => return Value::Error(e),
            }
        }

        if columns.is_empty() {
            return Value::Error(FolioError::domain_error("from_columns: no columns provided"));
        }

        let rows = columns[0].len();
        for (i, col) in columns.iter().enumerate() {
            if col.len() != rows {
                return Value::Error(FolioError::domain_error(&format!(
                    "from_columns: column {} has {} elements, expected {}", i, col.len(), rows
                )));
            }
        }

        // Transpose columns to rows
        let cols = columns.len();
        let data: Vec<Vec<Number>> = (0..rows)
            .map(|i| {
                (0..cols).map(|j| columns[j][i].clone()).collect()
            })
            .collect();

        match Matrix::from_nested_list(data, MatrixMode::Auto) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ from_rows ============

pub struct FromRowsFn;

static FROM_ROWS_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "rows",
        typ: "List",
        description: "Row vectors",
        optional: false,
        default: None,
    },
];

static FROM_ROWS_EXAMPLES: [&str; 1] = ["from_rows([1,2,3], [4,5,6]) → 2×3 matrix"];

static FROM_ROWS_RELATED: [&str; 2] = ["from_columns", "matrix"];

impl FunctionPlugin for FromRowsFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "from_rows",
            description: "Build matrix from row vectors",
            usage: "from_rows(row1, row2, ...)",
            args: &FROM_ROWS_ARGS,
            returns: "Matrix",
            examples: &FROM_ROWS_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &FROM_ROWS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("from_rows", 1, 0));
        }

        // Each arg is a row
        let mut data: Vec<Vec<Number>> = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            match extract_number_list(arg, "from_rows", &format!("row{}", i)) {
                Ok(row) => data.push(row),
                Err(e) => return Value::Error(e),
            }
        }

        if data.is_empty() {
            return Value::Error(FolioError::domain_error("from_rows: no rows provided"));
        }

        let cols = data[0].len();
        for (i, row) in data.iter().enumerate() {
            if row.len() != cols {
                return Value::Error(FolioError::domain_error(&format!(
                    "from_rows: row {} has {} elements, expected {}", i, row.len(), cols
                )));
            }
        }

        match Matrix::from_nested_list(data, MatrixMode::Auto) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
        }
    }
}

// ============ random_matrix ============

pub struct RandomMatrixFn;

static RANDOM_MATRIX_ARGS: [ArgMeta; 5] = [
    ArgMeta {
        name: "rows",
        typ: "Number",
        description: "Number of rows",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "cols",
        typ: "Number",
        description: "Number of columns",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "min",
        typ: "Number",
        description: "Minimum value",
        optional: true,
        default: Some("0"),
    },
    ArgMeta {
        name: "max",
        typ: "Number",
        description: "Maximum value",
        optional: true,
        default: Some("1"),
    },
    ArgMeta {
        name: "seed",
        typ: "Number",
        description: "Random seed for reproducibility",
        optional: true,
        default: None,
    },
];

static RANDOM_MATRIX_EXAMPLES: [&str; 1] = ["random_matrix(3, 3, 0, 1) → random 3×3"];

static RANDOM_MATRIX_RELATED: [&str; 2] = ["zeros", "ones"];

impl FunctionPlugin for RandomMatrixFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "random_matrix",
            description: "Create random matrix (float mode only)",
            usage: "random_matrix(rows, cols, [min], [max], [seed])",
            args: &RANDOM_MATRIX_ARGS,
            returns: "Matrix",
            examples: &RANDOM_MATRIX_EXAMPLES,
            category: "matrix/construct",
            source: None,
            related: &RANDOM_MATRIX_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("random_matrix", 2, args.len()));
        }

        let rows = match extract_usize(&args[0], "random_matrix", "rows") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let cols = match extract_usize(&args[1], "random_matrix", "cols") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if rows == 0 || cols == 0 {
            return Value::Error(FolioError::domain_error("random_matrix: dimensions must be positive"));
        }

        let min = args.get(2)
            .and_then(|v| v.as_number())
            .and_then(|n| n.to_f64())
            .unwrap_or(0.0);
        let max = args.get(3)
            .and_then(|v| v.as_number())
            .and_then(|n| n.to_f64())
            .unwrap_or(1.0);

        let seed = args.get(4)
            .and_then(|v| v.as_number())
            .and_then(|n| n.to_i64())
            .map(|n| n as u64);

        // Simple LCG random number generator
        let mut state = seed.unwrap_or(12345);
        let range = max - min;

        let data: Vec<Vec<Number>> = (0..rows)
            .map(|_| {
                (0..cols).map(|_| {
                    // Simple LCG: state = (a * state + c) mod m
                    state = state.wrapping_mul(1103515245).wrapping_add(12345);
                    let r = ((state >> 16) & 0x7fff) as f64 / 32768.0;
                    Number::from_f64(min + r * range)
                }).collect()
            })
            .collect();

        // Random matrices always use float mode
        match Matrix::from_nested_list(data, MatrixMode::Fast) {
            Ok(m) => m.into(),
            Err(e) => Value::Error(e),
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
    fn test_matrix_fn() {
        let f = MatrixFn;
        let args = vec![Value::List(vec![
            Value::List(vec![Value::Number(Number::from_i64(1)), Value::Number(Number::from_i64(2))]),
            Value::List(vec![Value::Number(Number::from_i64(3)), Value::Number(Number::from_i64(4))]),
        ])];
        let result = f.call(&args, &eval_ctx());
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_identity_fn() {
        let f = IdentityFn;
        let args = vec![Value::Number(Number::from_i64(3))];
        let result = f.call(&args, &eval_ctx());
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_zeros_fn() {
        let f = ZerosFn;
        let args = vec![
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ];
        let result = f.call(&args, &eval_ctx());
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_diagonal_fn() {
        let f = DiagonalFn;
        let args = vec![Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])];
        let result = f.call(&args, &eval_ctx());
        assert!(matches!(result, Value::Object(_)));
    }
}
