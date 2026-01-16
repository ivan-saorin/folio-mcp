//! Core matrix and vector types

use std::fmt;
use folio_core::{Number, Value, FolioError};
use nalgebra::{DMatrix, DVector};
use serde::{Serialize, Deserialize};

/// Default threshold for exact vs float arithmetic
pub const DEFAULT_EXACT_LIMIT: usize = 10;

/// Matrix computation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatrixMode {
    /// Use exact Number (BigRational) arithmetic
    Exact,
    /// Use f64 floating point arithmetic
    Fast,
    /// Automatically choose based on size
    Auto,
}

impl Default for MatrixMode {
    fn default() -> Self {
        MatrixMode::Auto
    }
}

impl MatrixMode {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "exact" => Some(MatrixMode::Exact),
            "fast" | "float" => Some(MatrixMode::Fast),
            "auto" => Some(MatrixMode::Auto),
            _ => None,
        }
    }
}

/// A matrix that can use either exact or floating-point arithmetic
#[derive(Debug, Clone)]
pub enum Matrix {
    /// Exact arithmetic using Number (BigRational)
    Exact(ExactMatrix),
    /// Fast floating-point arithmetic
    Float(FloatMatrix),
}

/// Matrix using exact Number arithmetic
#[derive(Debug, Clone)]
pub struct ExactMatrix {
    pub data: Vec<Vec<Number>>,
    pub rows: usize,
    pub cols: usize,
}

/// Matrix using f64 floating-point arithmetic
#[derive(Debug, Clone)]
pub struct FloatMatrix {
    pub data: DMatrix<f64>,
}

/// A vector that can use either exact or floating-point arithmetic
#[derive(Debug, Clone)]
pub enum Vector {
    /// Exact arithmetic using Number (BigRational)
    Exact(Vec<Number>),
    /// Fast floating-point arithmetic
    Float(DVector<f64>),
}

impl Matrix {
    /// Create a new matrix from nested lists, choosing mode based on size
    pub fn from_nested_list(data: Vec<Vec<Number>>, mode: MatrixMode) -> Result<Self, FolioError> {
        if data.is_empty() {
            return Err(FolioError::domain_error("matrix: empty data"));
        }

        let rows = data.len();
        let cols = data[0].len();

        // Validate all rows have same length
        for (i, row) in data.iter().enumerate() {
            if row.len() != cols {
                return Err(FolioError::domain_error(&format!(
                    "matrix: row {} has {} columns, expected {}",
                    i, row.len(), cols
                )));
            }
        }

        // Determine which mode to use
        let use_exact = match mode {
            MatrixMode::Exact => true,
            MatrixMode::Fast => false,
            MatrixMode::Auto => rows <= DEFAULT_EXACT_LIMIT && cols <= DEFAULT_EXACT_LIMIT,
        };

        if use_exact {
            Ok(Matrix::Exact(ExactMatrix { data, rows, cols }))
        } else {
            // Convert to f64
            let mut float_data = DMatrix::zeros(rows, cols);
            for (i, row) in data.iter().enumerate() {
                for (j, val) in row.iter().enumerate() {
                    float_data[(i, j)] = val.to_f64().unwrap_or(f64::NAN);
                }
            }
            Ok(Matrix::Float(FloatMatrix { data: float_data }))
        }
    }

    /// Create from nalgebra DMatrix
    pub fn from_dmatrix(data: DMatrix<f64>) -> Self {
        Matrix::Float(FloatMatrix { data })
    }

    /// Get number of rows
    pub fn rows(&self) -> usize {
        match self {
            Matrix::Exact(m) => m.rows,
            Matrix::Float(m) => m.data.nrows(),
        }
    }

    /// Get number of columns
    pub fn cols(&self) -> usize {
        match self {
            Matrix::Exact(m) => m.cols,
            Matrix::Float(m) => m.data.ncols(),
        }
    }

    /// Get element at (row, col)
    pub fn get(&self, row: usize, col: usize) -> Option<Number> {
        match self {
            Matrix::Exact(m) => {
                if row < m.rows && col < m.cols {
                    Some(m.data[row][col].clone())
                } else {
                    None
                }
            }
            Matrix::Float(m) => {
                if row < m.data.nrows() && col < m.data.ncols() {
                    Some(Number::from_f64(m.data[(row, col)]))
                } else {
                    None
                }
            }
        }
    }

    /// Check if matrix is square
    pub fn is_square(&self) -> bool {
        self.rows() == self.cols()
    }

    /// Convert to f64 DMatrix (for nalgebra operations)
    pub fn to_dmatrix(&self) -> DMatrix<f64> {
        match self {
            Matrix::Float(m) => m.data.clone(),
            Matrix::Exact(m) => {
                let mut result = DMatrix::zeros(m.rows, m.cols);
                for i in 0..m.rows {
                    for j in 0..m.cols {
                        result[(i, j)] = m.data[i][j].to_f64().unwrap_or(f64::NAN);
                    }
                }
                result
            }
        }
    }

    /// Convert to nested list of Numbers
    pub fn to_nested_list(&self) -> Vec<Vec<Number>> {
        match self {
            Matrix::Exact(m) => m.data.clone(),
            Matrix::Float(m) => {
                let mut result = Vec::with_capacity(m.data.nrows());
                for i in 0..m.data.nrows() {
                    let mut row = Vec::with_capacity(m.data.ncols());
                    for j in 0..m.data.ncols() {
                        row.push(Number::from_f64(m.data[(i, j)]));
                    }
                    result.push(row);
                }
                result
            }
        }
    }

    /// Check if using exact arithmetic
    pub fn is_exact(&self) -> bool {
        matches!(self, Matrix::Exact(_))
    }

    /// Get a row as a vector
    pub fn get_row(&self, row: usize) -> Option<Vector> {
        if row >= self.rows() {
            return None;
        }

        match self {
            Matrix::Exact(m) => Some(Vector::Exact(m.data[row].clone())),
            Matrix::Float(m) => Some(Vector::Float(m.data.row(row).transpose())),
        }
    }

    /// Get a column as a vector
    pub fn get_col(&self, col: usize) -> Option<Vector> {
        if col >= self.cols() {
            return None;
        }

        match self {
            Matrix::Exact(m) => {
                let data: Vec<Number> = m.data.iter().map(|row| row[col].clone()).collect();
                Some(Vector::Exact(data))
            }
            Matrix::Float(m) => Some(Vector::Float(m.data.column(col).clone_owned())),
        }
    }

    /// Get diagonal as a vector
    pub fn get_diag(&self) -> Vector {
        let n = self.rows().min(self.cols());
        match self {
            Matrix::Exact(m) => {
                let data: Vec<Number> = (0..n).map(|i| m.data[i][i].clone()).collect();
                Vector::Exact(data)
            }
            Matrix::Float(m) => Vector::Float(m.data.diagonal().clone_owned()),
        }
    }

    /// Convert to f64 DMatrix (alias for to_dmatrix for consistency)
    pub fn to_float(&self) -> DMatrix<f64> {
        self.to_dmatrix()
    }

    /// Convert to Value
    pub fn to_value(&self) -> Value {
        Value::from(self.clone())
    }
}

impl Vector {
    /// Create a new vector from list of Numbers
    pub fn from_list(data: Vec<Number>, mode: MatrixMode) -> Self {
        let use_exact = match mode {
            MatrixMode::Exact => true,
            MatrixMode::Fast => false,
            MatrixMode::Auto => data.len() <= DEFAULT_EXACT_LIMIT,
        };

        if use_exact {
            Vector::Exact(data)
        } else {
            let float_data: Vec<f64> = data.iter()
                .map(|n| n.to_f64().unwrap_or(f64::NAN))
                .collect();
            Vector::Float(DVector::from_vec(float_data))
        }
    }

    /// Get length of vector
    pub fn len(&self) -> usize {
        match self {
            Vector::Exact(v) => v.len(),
            Vector::Float(v) => v.len(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get element at index
    pub fn get(&self, index: usize) -> Option<Number> {
        match self {
            Vector::Exact(v) => v.get(index).cloned(),
            Vector::Float(v) => {
                if index < v.len() {
                    Some(Number::from_f64(v[index]))
                } else {
                    None
                }
            }
        }
    }

    /// Convert to list of Numbers
    pub fn to_list(&self) -> Vec<Number> {
        match self {
            Vector::Exact(v) => v.clone(),
            Vector::Float(v) => v.iter().map(|&x| Number::from_f64(x)).collect(),
        }
    }

    /// Convert to DVector<f64>
    pub fn to_dvector(&self) -> DVector<f64> {
        match self {
            Vector::Float(v) => v.clone(),
            Vector::Exact(v) => {
                let data: Vec<f64> = v.iter()
                    .map(|n| n.to_f64().unwrap_or(f64::NAN))
                    .collect();
                DVector::from_vec(data)
            }
        }
    }

    /// Convert vector to column matrix
    pub fn to_column_matrix(&self) -> Matrix {
        let n = self.len();
        match self {
            Vector::Exact(v) => {
                let data: Vec<Vec<Number>> = v.iter()
                    .map(|x| vec![x.clone()])
                    .collect();
                Matrix::Exact(ExactMatrix { data, rows: n, cols: 1 })
            }
            Vector::Float(v) => {
                let mut data = DMatrix::zeros(n, 1);
                for (i, &val) in v.iter().enumerate() {
                    data[(i, 0)] = val;
                }
                Matrix::Float(FloatMatrix { data })
            }
        }
    }

    /// Convert vector to row matrix
    pub fn to_row_matrix(&self) -> Matrix {
        let n = self.len();
        match self {
            Vector::Exact(v) => {
                let data = vec![v.clone()];
                Matrix::Exact(ExactMatrix { data, rows: 1, cols: n })
            }
            Vector::Float(v) => {
                let mut data = DMatrix::zeros(1, n);
                for (i, &val) in v.iter().enumerate() {
                    data[(0, i)] = val;
                }
                Matrix::Float(FloatMatrix { data })
            }
        }
    }

    /// Convert to Vec<f64>
    pub fn to_float(&self) -> Vec<f64> {
        match self {
            Vector::Float(v) => v.as_slice().to_vec(),
            Vector::Exact(v) => v.iter()
                .map(|n| n.to_f64().unwrap_or(f64::NAN))
                .collect(),
        }
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rows = self.rows();
        let cols = self.cols();

        write!(f, "[")?;
        for i in 0..rows {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "[")?;
            for j in 0..cols {
                if j > 0 {
                    write!(f, ", ")?;
                }
                if let Some(val) = self.get(i, j) {
                    write!(f, "{}", val.as_decimal(4))?;
                }
            }
            write!(f, "]")?;
        }
        write!(f, "]")
    }
}

impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, val) in self.to_list().iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", val.as_decimal(4))?;
        }
        write!(f, "]")
    }
}

/// Convert Matrix to Value
impl From<Matrix> for Value {
    fn from(m: Matrix) -> Value {
        let mut obj = std::collections::HashMap::new();
        obj.insert("type".to_string(), Value::Text("Matrix".to_string()));
        obj.insert("rows".to_string(), Value::Number(Number::from_i64(m.rows() as i64)));
        obj.insert("cols".to_string(), Value::Number(Number::from_i64(m.cols() as i64)));

        // Store data as nested list
        let data: Vec<Value> = m.to_nested_list().iter()
            .map(|row| {
                Value::List(row.iter().map(|n| Value::Number(n.clone())).collect())
            })
            .collect();
        obj.insert("data".to_string(), Value::List(data));

        Value::Object(obj)
    }
}

/// Convert Vector to Value
impl From<Vector> for Value {
    fn from(v: Vector) -> Value {
        Value::List(v.to_list().into_iter().map(Value::Number).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_creation() {
        let data = vec![
            vec![Number::from_i64(1), Number::from_i64(2)],
            vec![Number::from_i64(3), Number::from_i64(4)],
        ];
        let m = Matrix::from_nested_list(data, MatrixMode::Auto).unwrap();

        assert_eq!(m.rows(), 2);
        assert_eq!(m.cols(), 2);
        assert!(m.is_square());
        assert!(m.is_exact()); // Should be exact for 2x2
    }

    #[test]
    fn test_matrix_get() {
        let data = vec![
            vec![Number::from_i64(1), Number::from_i64(2)],
            vec![Number::from_i64(3), Number::from_i64(4)],
        ];
        let m = Matrix::from_nested_list(data, MatrixMode::Auto).unwrap();

        assert_eq!(m.get(0, 0), Some(Number::from_i64(1)));
        assert_eq!(m.get(1, 1), Some(Number::from_i64(4)));
        assert_eq!(m.get(2, 2), None);
    }

    #[test]
    fn test_vector_creation() {
        let data = vec![Number::from_i64(1), Number::from_i64(2), Number::from_i64(3)];
        let v = Vector::from_list(data, MatrixMode::Auto);

        assert_eq!(v.len(), 3);
        assert_eq!(v.get(0), Some(Number::from_i64(1)));
    }

    #[test]
    fn test_matrix_mode() {
        // Large matrix should use float mode in auto
        let n = 15;
        let data: Vec<Vec<Number>> = (0..n)
            .map(|i| (0..n).map(|j| Number::from_i64((i * n + j) as i64)).collect())
            .collect();
        let m = Matrix::from_nested_list(data, MatrixMode::Auto).unwrap();

        assert!(!m.is_exact()); // Should use float for 15x15
    }
}
