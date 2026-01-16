//! Helper functions for matrix operations

use folio_core::{Number, Value, FolioError};
use folio_plugin::EvalContext;
use crate::types::{Matrix, Vector, MatrixMode};

/// Extract a Number from a Value
pub fn extract_number(value: &Value, func: &str, arg: &str) -> Result<Number, FolioError> {
    match value {
        Value::Number(n) => Ok(n.clone()),
        _ => Err(FolioError::arg_type(func, arg, "Number", value.type_name())),
    }
}

/// Extract an integer from a Value
pub fn extract_int(value: &Value, func: &str, arg: &str) -> Result<i64, FolioError> {
    match value {
        Value::Number(n) => n.to_i64()
            .ok_or_else(|| FolioError::domain_error(&format!("{}: {} must be an integer", func, arg))),
        _ => Err(FolioError::arg_type(func, arg, "Number", value.type_name())),
    }
}

/// Extract a usize from a Value
pub fn extract_usize(value: &Value, func: &str, arg: &str) -> Result<usize, FolioError> {
    let i = extract_int(value, func, arg)?;
    if i < 0 {
        return Err(FolioError::domain_error(&format!("{}: {} must be non-negative", func, arg)));
    }
    Ok(i as usize)
}

/// Extract a list of Numbers from a Value
pub fn extract_number_list(value: &Value, func: &str, arg: &str) -> Result<Vec<Number>, FolioError> {
    match value {
        Value::List(items) => {
            let mut numbers = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                match item {
                    Value::Number(n) => numbers.push(n.clone()),
                    _ => return Err(FolioError::domain_error(&format!(
                        "{}: {} element {} must be a Number", func, arg, i
                    ))),
                }
            }
            Ok(numbers)
        }
        _ => Err(FolioError::arg_type(func, arg, "List", value.type_name())),
    }
}

/// Extract a matrix from a Value (either a Matrix object or nested list)
pub fn extract_matrix(value: &Value, func: &str, arg: &str) -> Result<Matrix, FolioError> {
    match value {
        Value::Object(obj) => {
            // Check if it's a Matrix object
            if let Some(Value::Text(t)) = obj.get("type") {
                if t == "Matrix" {
                    if let Some(Value::List(data)) = obj.get("data") {
                        return extract_matrix_from_nested_list(data, func, arg);
                    }
                }
            }
            Err(FolioError::arg_type(func, arg, "Matrix", "Object"))
        }
        Value::List(rows) => {
            extract_matrix_from_nested_list(rows, func, arg)
        }
        _ => Err(FolioError::arg_type(func, arg, "Matrix", value.type_name())),
    }
}

/// Extract a matrix from a nested list of Values
fn extract_matrix_from_nested_list(rows: &[Value], func: &str, arg: &str) -> Result<Matrix, FolioError> {
    if rows.is_empty() {
        return Err(FolioError::domain_error(&format!("{}: {} cannot be empty", func, arg)));
    }

    let mut data = Vec::with_capacity(rows.len());

    for (i, row_val) in rows.iter().enumerate() {
        match row_val {
            Value::List(cols) => {
                let mut row = Vec::with_capacity(cols.len());
                for (j, col_val) in cols.iter().enumerate() {
                    match col_val {
                        Value::Number(n) => row.push(n.clone()),
                        _ => return Err(FolioError::domain_error(&format!(
                            "{}: {}[{}][{}] must be a Number", func, arg, i, j
                        ))),
                    }
                }
                data.push(row);
            }
            _ => return Err(FolioError::domain_error(&format!(
                "{}: {} row {} must be a list", func, arg, i
            ))),
        }
    }

    Matrix::from_nested_list(data, MatrixMode::Auto)
}

/// Extract a vector from a Value
pub fn extract_vector(value: &Value, func: &str, arg: &str) -> Result<Vector, FolioError> {
    match value {
        Value::List(items) => {
            let numbers = extract_number_list(value, func, arg)?;
            Ok(Vector::from_list(numbers, MatrixMode::Auto))
        }
        Value::Object(obj) => {
            // Check if it's a Matrix object with one row or column
            if let Some(Value::Text(t)) = obj.get("type") {
                if t == "Matrix" {
                    let m = extract_matrix(value, func, arg)?;
                    if m.rows() == 1 {
                        return Ok(m.get_row(0).unwrap());
                    } else if m.cols() == 1 {
                        return Ok(m.get_col(0).unwrap());
                    }
                    return Err(FolioError::domain_error(&format!(
                        "{}: {} must be a vector (1×n or n×1 matrix)", func, arg
                    )));
                }
            }
            Err(FolioError::arg_type(func, arg, "Vector", "Object"))
        }
        _ => Err(FolioError::arg_type(func, arg, "Vector or List", value.type_name())),
    }
}

/// Parse matrix mode from optional argument
pub fn parse_mode(value: Option<&Value>) -> MatrixMode {
    match value {
        Some(Value::Text(s)) => MatrixMode::from_str(s).unwrap_or(MatrixMode::Auto),
        _ => MatrixMode::Auto,
    }
}

/// Get precision from context
pub fn get_precision(ctx: &EvalContext) -> u32 {
    ctx.precision
}

/// Check that two matrices have compatible dimensions for multiplication
pub fn check_matmul_dims(a: &Matrix, b: &Matrix, func: &str) -> Result<(), FolioError> {
    if a.cols() != b.rows() {
        return Err(FolioError::domain_error(&format!(
            "{}: incompatible dimensions {}×{} and {}×{}",
            func, a.rows(), a.cols(), b.rows(), b.cols()
        )));
    }
    Ok(())
}

/// Check that two matrices have the same dimensions
pub fn check_same_dims(a: &Matrix, b: &Matrix, func: &str) -> Result<(), FolioError> {
    if a.rows() != b.rows() || a.cols() != b.cols() {
        return Err(FolioError::domain_error(&format!(
            "{}: matrices must have same dimensions: {}×{} vs {}×{}",
            func, a.rows(), a.cols(), b.rows(), b.cols()
        )));
    }
    Ok(())
}

/// Check that a matrix is square
pub fn check_square(m: &Matrix, func: &str) -> Result<(), FolioError> {
    if !m.is_square() {
        return Err(FolioError::domain_error(&format!(
            "{}: requires square matrix, got {}×{}", func, m.rows(), m.cols()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_number() {
        let val = Value::Number(Number::from_i64(42));
        let result = extract_number(&val, "test", "x").unwrap();
        assert_eq!(result.to_i64(), Some(42));
    }

    #[test]
    fn test_extract_number_list() {
        let val = Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ]);
        let result = extract_number_list(&val, "test", "x").unwrap();
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_extract_matrix() {
        let val = Value::List(vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
            ]),
            Value::List(vec![
                Value::Number(Number::from_i64(3)),
                Value::Number(Number::from_i64(4)),
            ]),
        ]);
        let result = extract_matrix(&val, "test", "m").unwrap();
        assert_eq!(result.rows(), 2);
        assert_eq!(result.cols(), 2);
    }
}
