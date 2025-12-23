# Folio Matrix/Linear Algebra Specification

## Overview

Matrix operations and linear algebra for Folio. Dual-precision architecture: exact `Number` (BigRational) for small matrices, high-precision float for large matrices. Dockerized deployment using pure Rust (`nalgebra` + `faer`), no external BLAS dependencies.

---

## Module Structure

```
folio-matrix/
├── Cargo.toml
└── src/
    ├── lib.rs           # Registration
    ├── types.rs         # Matrix, Vector types
    ├── construct.rs     # matrix(), vector(), identity(), etc.
    ├── ops.rs           # matmul, transpose, inverse, etc.
    ├── decompose.rs     # LU, QR, SVD, Cholesky, Eigen
    ├── solve.rs         # solve(), lstsq()
    ├── norms.rs         # norm(), condition_number()
    └── precision.rs     # Precision strategy and conversion
```

---

## Precision Strategy

### Automatic Selection

| Matrix Size | Type | Rationale |
|-------------|------|-----------|
| ≤ 10×10 | `Number` (BigRational) | Exact arithmetic, no precision loss |
| > 10×10 | `Float` (f64/f128) | Performance, memory |

Threshold configurable via context: `@matrix_exact_limit:20`

### Explicit Override

```markdown
| m  | matrix([[1,2],[3,4]], "exact")  |  |  // Force BigRational
| m  | matrix([[1,2],[3,4]], "fast")   |  |  // Force float
```

### Precision for Float Mode

Default: f64 (IEEE 754)
Optional: 128-bit float via `rug` for critical calculations

---

## Types

### Matrix

```rust
pub enum Matrix {
    Exact(ExactMatrix),   // Vec<Vec<Number>>
    Float(FloatMatrix),   // nalgebra::DMatrix<f64>
}

pub struct ExactMatrix {
    data: Vec<Vec<Number>>,
    rows: usize,
    cols: usize,
}

pub struct FloatMatrix {
    data: nalgebra::DMatrix<f64>,
}
```

### Vector

```rust
pub enum Vector {
    Exact(Vec<Number>),
    Float(nalgebra::DVector<f64>),
}
```

### Value Integration

```rust
impl From<Matrix> for Value {
    fn from(m: Matrix) -> Value {
        Value::Object(hashmap! {
            "type".into() => Value::Text("Matrix".into()),
            "rows".into() => Value::Number(Number::from_i64(m.rows() as i64)),
            "cols".into() => Value::Number(Number::from_i64(m.cols() as i64)),
            "_matrix".into() => Value::Native(Box::new(m)),  // Internal storage
        })
    }
}
```

---

## Size Limits

| Limit | Default | Max Override |
|-------|---------|--------------|
| Matrix dimension | 1000×1000 | 10000×10000 |
| Exact arithmetic | 10×10 | 100×100 |
| SVD/Eigen | 500×500 | 2000×2000 |

Override via section attribute: `@matrix_max_size:5000`

---

## Functions

### Construction

#### `matrix(data, [mode])`

Create matrix from nested list.

```markdown
| m | matrix([[1,2,3],[4,5,6],[7,8,9]]) | |
```

**Returns Object:**
```json
{
  "type": "Matrix",
  "rows": 3,
  "cols": 3
}
```

Access elements: `m[0][1]` or `m.get(0, 1)`

#### `vector(data, [mode])`

Create column vector.

```markdown
| v | vector([1, 2, 3]) | |
```

#### `row_vector(data, [mode])`

Create row vector (1×n matrix).

```markdown
| v | row_vector([1, 2, 3]) | |
```

#### `identity(n, [mode])`

n×n identity matrix.

```markdown
| I | identity(3) | |
```

#### `zeros(rows, cols, [mode])`

Matrix of zeros.

```markdown
| z | zeros(3, 4) | |
```

#### `ones(rows, cols, [mode])`

Matrix of ones.

```markdown
| o | ones(2, 3) | |
```

#### `diagonal(values, [mode])`

Diagonal matrix from list.

```markdown
| d | diagonal([1, 2, 3]) | |
```

Result:
```
[1, 0, 0]
[0, 2, 0]
[0, 0, 3]
```

#### `from_columns(col1, col2, ...)`

Build matrix from column vectors.

```markdown
| m | from_columns([1,2,3], [4,5,6]) | |
```

#### `from_rows(row1, row2, ...)`

Build matrix from row vectors.

```markdown
| m | from_rows([1,2,3], [4,5,6]) | |
```

#### `random_matrix(rows, cols, [min], [max], [seed])`

Random matrix (float mode only).

```markdown
| r | random_matrix(3, 3, 0, 1) | |
```

---

### Element Access

#### `get(matrix, row, col)`

Get single element.

```markdown
| val | get(m, 0, 1) | 2 |
```

Also via indexing: `m[0][1]`

#### `set(matrix, row, col, value)`

Create new matrix with updated element (immutable).

```markdown
| m2 | set(m, 0, 1, 99) | |
```

#### `row(matrix, index)`

Extract row as vector.

```markdown
| r | row(m, 0) | [1, 2, 3] |
```

#### `col(matrix, index)`

Extract column as vector.

```markdown
| c | col(m, 1) | [2, 5, 8] |
```

#### `diag(matrix)`

Extract diagonal as vector.

```markdown
| d | diag(m) | [1, 5, 9] |
```

#### `submatrix(matrix, row_start, row_end, col_start, col_end)`

Extract submatrix.

```markdown
| sub | submatrix(m, 0, 2, 1, 3) | |
```

---

### Basic Operations

#### `transpose(matrix)` / `T(matrix)`

Matrix transpose.

```markdown
| mt | transpose(m) | |
| mt | T(m)         | |
```

#### `matmul(a, b)` / `mm(a, b)`

Matrix multiplication.

```markdown
| c | matmul(a, b) | |
| c | mm(a, b)     | |
```

Returns error if dimensions incompatible.

#### `add(a, b)`

Element-wise addition.

```markdown
| c | add(a, b) | |
```

Also via `a + b` if operator overloading supported.

#### `sub(a, b)`

Element-wise subtraction.

```markdown
| c | sub(a, b) | |
```

#### `scale(matrix, scalar)`

Scalar multiplication.

```markdown
| m2 | scale(m, 2) | |
```

#### `hadamard(a, b)`

Element-wise multiplication (Hadamard product).

```markdown
| c | hadamard(a, b) | |
```

#### `element_div(a, b)`

Element-wise division.

```markdown
| c | element_div(a, b) | |
```

#### `power(matrix, n)`

Matrix power (repeated multiplication).

```markdown
| m3 | power(m, 3) | |
```

n must be non-negative integer.

---

### Matrix Properties

#### `rows(matrix)`

Number of rows.

```markdown
| r | rows(m) | 3 |
```

#### `cols(matrix)`

Number of columns.

```markdown
| c | cols(m) | 3 |
```

#### `shape(matrix)`

Dimensions as [rows, cols].

```markdown
| s | shape(m) | [3, 3] |
```

#### `is_square(matrix)`

Check if square.

```markdown
| sq | is_square(m) | true |
```

#### `is_symmetric(matrix, [tolerance])`

Check if symmetric.

```markdown
| sym | is_symmetric(m) | true |
```

#### `is_positive_definite(matrix)`

Check positive definiteness.

```markdown
| pd | is_positive_definite(m) | true |
```

#### `rank(matrix, [tolerance])`

Matrix rank.

```markdown
| r | rank(m) | 3 |
```

#### `trace(matrix)`

Sum of diagonal elements.

```markdown
| t | trace(m) | 15 |
```

#### `determinant(matrix)` / `det(matrix)`

Matrix determinant.

```markdown
| d | det(m) | -306 |
```

Error if not square.

---

### Inverse and Pseudoinverse

#### `inverse(matrix)` / `inv(matrix)`

Matrix inverse.

```markdown
| m_inv | inv(m) | |
```

Returns error if singular.

#### `pinv(matrix, [tolerance])`

Moore-Penrose pseudoinverse.

```markdown
| m_pinv | pinv(m) | |
```

Works for any matrix (including non-square, singular).

---

### Norms

#### `norm(matrix, [type])`

Matrix or vector norm.

```markdown
| n  | norm(v)         | |  // Default: L2 (Euclidean)
| n1 | norm(v, 1)      | |  // L1 (Manhattan)
| ni | norm(v, "inf")  | |  // L-infinity (max)
| nf | norm(m, "fro")  | |  // Frobenius
```

Types:
- `1`: L1 norm
- `2` (default): L2 norm (spectral for matrices)
- `"inf"`: Infinity norm
- `"fro"`: Frobenius norm

#### `normalize(vector)`

Unit vector (L2 normalized).

```markdown
| u | normalize(v) | |
```

#### `condition_number(matrix, [norm])`

Condition number (measures numerical stability).

```markdown
| cn | condition_number(m) | 14.93 |
```

High condition number = ill-conditioned matrix.

---

### Decompositions

#### `lu(matrix)`

LU decomposition with partial pivoting.

```markdown
| decomp | lu(m)    | |
| L      | decomp.L | |
| U      | decomp.U | |
| P      | decomp.P | |
```

**Returns Object:**
```json
{
  "L": Matrix,  // Lower triangular
  "U": Matrix,  // Upper triangular
  "P": Matrix   // Permutation matrix
}
```

PA = LU

#### `qr(matrix)`

QR decomposition.

```markdown
| decomp | qr(m)    | |
| Q      | decomp.Q | |
| R      | decomp.R | |
```

**Returns Object:**
```json
{
  "Q": Matrix,  // Orthogonal
  "R": Matrix   // Upper triangular
}
```

A = QR

#### `svd(matrix)`

Singular Value Decomposition.

```markdown
| decomp | svd(m)         | |
| U      | decomp.U       | |
| S      | decomp.S       | |  // Singular values as vector
| Vt     | decomp.Vt      | |
| rank   | decomp.rank    | |
```

**Returns Object:**
```json
{
  "U": Matrix,     // Left singular vectors
  "S": Vector,     // Singular values (descending)
  "Vt": Matrix,    // Right singular vectors (transposed)
  "rank": Number   // Numerical rank
}
```

A = U × diag(S) × Vt

#### `cholesky(matrix)`

Cholesky decomposition (for positive definite matrices).

```markdown
| L | cholesky(m) | |
```

Returns lower triangular L where A = L × L^T.

Error if not positive definite.

#### `eigen(matrix)`

Eigenvalue decomposition.

```markdown
| decomp   | eigen(m)            | |
| values   | decomp.values       | |  // Eigenvalues
| vectors  | decomp.vectors      | |  // Eigenvectors (columns)
```

**Returns Object:**
```json
{
  "values": Vector,    // Eigenvalues (may be complex as pairs)
  "vectors": Matrix,   // Eigenvectors as columns
  "real_values": Vector,     // Real parts
  "imag_values": Vector      // Imaginary parts (0 if real)
}
```

For symmetric matrices, all values are real.

#### `schur(matrix)`

Schur decomposition.

```markdown
| decomp | schur(m)  | |
| Q      | decomp.Q  | |
| T      | decomp.T  | |
```

A = Q × T × Q^T

---

### Linear Systems

#### `solve(A, b)`

Solve Ax = b for x.

```markdown
## System: 2x + 3y = 8, 4x + y = 6

| A | matrix([[2,3],[4,1]]) | |
| b | vector([8, 6])        | |
| x | solve(A, b)           | |
```

Uses LU decomposition. Returns error if singular.

#### `lstsq(A, b)`

Least squares solution (minimizes ||Ax - b||).

```markdown
| x | lstsq(A, b) | |
```

**Returns Object:**
```json
{
  "solution": Vector,
  "residuals": Number,     // ||Ax - b||²
  "rank": Number
}
```

Works for overdetermined systems (more equations than unknowns).

#### `solve_triangular(A, b, lower)`

Solve triangular system (fast).

```markdown
| x | solve_triangular(L, b, true)  | |  // Lower triangular
| x | solve_triangular(U, b, false) | |  // Upper triangular
```

#### `null_space(matrix, [tolerance])`

Basis for null space.

```markdown
| ns | null_space(m) | |
```

Returns matrix whose columns span null(A).

#### `column_space(matrix, [tolerance])`

Basis for column space.

```markdown
| cs | column_space(m) | |
```

---

### Vector Operations

#### `dot(a, b)`

Dot product.

```markdown
| d | dot(v1, v2) | 32 |
```

#### `cross(a, b)`

Cross product (3D only).

```markdown
| c | cross(v1, v2) | [x, y, z] |
```

Error if vectors not 3D.

#### `outer(a, b)`

Outer product (a × b^T).

```markdown
| m | outer(v1, v2) | |
```

#### `angle(a, b)`

Angle between vectors (radians).

```markdown
| theta | angle(v1, v2) | 0.785 |
```

#### `project(a, b)`

Project a onto b.

```markdown
| p | project(v1, v2) | |
```

Formula: (a·b / b·b) × b

---

### Utility

#### `reshape(matrix, rows, cols)`

Reshape matrix (same total elements).

```markdown
| m2 | reshape(m, 2, 6) | |
```

#### `flatten(matrix)`

Flatten to 1D vector (row-major).

```markdown
| v | flatten(m) | [1,2,3,4,5,6,7,8,9] |
```

#### `stack_h(m1, m2, ...)`

Horizontal stack (concatenate columns).

```markdown
| m | stack_h(a, b) | |
```

#### `stack_v(m1, m2, ...)`

Vertical stack (concatenate rows).

```markdown
| m | stack_v(a, b) | |
```

#### `map(matrix, fn)`

Apply function to each element.

```markdown
| m2 | map(m, x -> x * 2) | |
```

Requires lambda support.

#### `to_list(matrix)`

Convert to nested list.

```markdown
| l | to_list(m) | [[1,2,3],[4,5,6],[7,8,9]] |
```

#### `format_matrix(matrix, [precision])`

Pretty-print matrix as text.

```markdown
| txt | format_matrix(m, 2) | "[1.00, 2.00]\n[3.00, 4.00]" |
```

---

## Implementation Notes

### Docker Compatibility

Pure Rust stack, no external dependencies:

```toml
[dependencies]
folio-core = { path = "../folio-core" }
folio-plugin = { path = "../folio-plugin" }
nalgebra = "0.32"    # Core linear algebra
faer = "0.18"        # High-performance decompositions
```

No BLAS/LAPACK bindings (complex Docker setup, OpenBLAS issues).

### Performance Optimizations

```rust
impl FloatMatrix {
    /// Use faer for large decompositions (faster than nalgebra)
    fn svd_large(&self) -> SvdResult {
        let mat = faer::Mat::from_fn(self.rows(), self.cols(), |i, j| {
            self.data[(i, j)]
        });
        let svd = mat.svd();
        // Convert back...
    }
}
```

### Exact Arithmetic

```rust
impl ExactMatrix {
    /// Gaussian elimination with exact fractions
    fn solve_exact(&self, b: &ExactVector) -> Result<ExactVector, FolioError> {
        // No numerical instability - fractions stay exact
        let mut aug = self.augment(b);
        aug.gaussian_elimination()?;
        aug.back_substitute()
    }
}
```

### Error Handling

```rust
fn inverse(m: &Matrix) -> Value {
    // Check square
    if m.rows() != m.cols() {
        return Value::Error(FolioError::domain_error(
            "inverse() requires square matrix"
        ));
    }
    
    // Check size limit
    if m.rows() > ctx.max_matrix_size {
        return Value::Error(FolioError::domain_error(format!(
            "Matrix too large: {}×{} exceeds limit {}",
            m.rows(), m.cols(), ctx.max_matrix_size
        )));
    }
    
    // Check singular
    let det = m.determinant();
    if det.is_zero() || det.abs() < TOLERANCE {
        return Value::Error(FolioError::domain_error(
            "Matrix is singular (non-invertible)"
        ));
    }
    
    // Compute inverse...
}
```

---

## Examples

### Solving Linear Systems

```markdown
## Linear System @matrix_exact_limit:20

## Coefficients
| _ | x | y | z |
|---|---|---|---|
| eq1 | 2 | 1 | -1 |
| eq2 | -3 | -1 | 2 |
| eq3 | -2 | 1 | 2 |

## Constants
| eq | value |
|----|-------|
| eq1 | 8 |
| eq2 | -11 |
| eq3 | -3 |

## Solution

| Metric   | Formula                                        | Result |
|----------|------------------------------------------------|--------|
| A        | matrix([[2,1,-1],[-3,-1,2],[-2,1,2]])          |        |
| b        | vector([8, -11, -3])                           |        |
| solution | solve(A, b)                                    |        |
| x        | solution[0]                                    | 2      |
| y        | solution[1]                                    | 3      |
| z        | solution[2]                                    | -1     |
| verify   | matmul(A, solution)                            | [8, -11, -3] |
```

### Regression with Matrix Operations

```markdown
## Linear Regression via Normal Equations

| Metric     | Formula                              | Result |
|------------|--------------------------------------|--------|
| X          | from_columns(ones(5,1), Data.x)      |        |
| y          | vector(Data.y)                       |        |
| XtX        | matmul(T(X), X)                      |        |
| Xty        | matmul(T(X), y)                      |        |
| beta       | solve(XtX, Xty)                      |        |
| intercept  | beta[0]                              |        |
| slope      | beta[1]                              |        |
| y_pred     | matmul(X, beta)                      |        |
| residuals  | sub(y, y_pred)                       |        |
| ss_res     | dot(residuals, residuals)            |        |
```

### Eigenvalue Analysis

```markdown
## Covariance Matrix Eigensystem

| Metric      | Formula                    | Result |
|-------------|----------------------------|--------|
| cov         | covariance_matrix(Data)    |        |
| eig         | eigen(cov)                 |        |
| values      | eig.values                 |        |
| vectors     | eig.vectors                |        |
| pc1         | col(vectors, 0)            |        |
| var_expl    | values[0] / sum(values)    |        |
```

---

## Function Summary

| Category | Functions |
|----------|-----------|
| **Construction** | `matrix`, `vector`, `row_vector`, `identity`, `zeros`, `ones`, `diagonal`, `from_columns`, `from_rows`, `random_matrix` |
| **Access** | `get`, `set`, `row`, `col`, `diag`, `submatrix` |
| **Operations** | `transpose`/`T`, `matmul`/`mm`, `add`, `sub`, `scale`, `hadamard`, `element_div`, `power` |
| **Properties** | `rows`, `cols`, `shape`, `is_square`, `is_symmetric`, `is_positive_definite`, `rank`, `trace`, `determinant`/`det` |
| **Inverse** | `inverse`/`inv`, `pinv` |
| **Norms** | `norm`, `normalize`, `condition_number` |
| **Decomposition** | `lu`, `qr`, `svd`, `cholesky`, `eigen`, `schur` |
| **Solving** | `solve`, `lstsq`, `solve_triangular`, `null_space`, `column_space` |
| **Vector** | `dot`, `cross`, `outer`, `angle`, `project` |
| **Utility** | `reshape`, `flatten`, `stack_h`, `stack_v`, `map`, `to_list`, `format_matrix` |

Total: 52 functions
