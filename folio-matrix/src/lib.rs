//! Folio Matrix - Linear Algebra and Matrix Operations
//!
//! Provides matrix operations and linear algebra for Folio:
//! - Matrix construction (matrix, vector, identity, zeros, ones, diagonal)
//! - Basic operations (transpose, matmul, add, sub, scale, hadamard)
//! - Matrix properties (rows, cols, shape, rank, trace, det)
//! - Inverse operations (inverse, pinv)
//! - Norms (norm, normalize, condition_number)
//! - Decompositions (lu, qr, svd, cholesky, eigen)
//! - Linear solvers (solve, lstsq)
//! - Vector operations (dot, cross, outer, angle, project)
//!
//! Uses dual precision: exact Number arithmetic for small matrices (≤10×10),
//! f64 floating point for larger matrices.

mod types;
mod helpers;
mod construct;
mod ops;
mod props;
mod decompose;
mod solve;
mod norms;
mod vector_ops;

pub use types::{Matrix, Vector, MatrixMode};
pub use helpers::extract_matrix;

use folio_plugin::PluginRegistry;

/// Load matrix functions into registry
pub fn load_matrix_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Construction (10 functions)
        .with_function(construct::MatrixFn)
        .with_function(construct::VectorFn)
        .with_function(construct::RowVectorFn)
        .with_function(construct::IdentityFn)
        .with_function(construct::ZerosFn)
        .with_function(construct::OnesFn)
        .with_function(construct::DiagonalFn)
        .with_function(construct::FromColumnsFn)
        .with_function(construct::FromRowsFn)
        .with_function(construct::RandomMatrixFn)

        // Access (6 functions)
        .with_function(ops::GetFn)
        .with_function(ops::SetFn)
        .with_function(ops::RowFn)
        .with_function(ops::ColFn)
        .with_function(ops::DiagFn)
        .with_function(ops::SubmatrixFn)

        // Basic operations (8 functions)
        .with_function(ops::TransposeFn)
        .with_function(ops::MatmulFn)
        .with_function(ops::MatAddFn)
        .with_function(ops::MatSubFn)
        .with_function(ops::ScaleFn)
        .with_function(ops::HadamardFn)
        .with_function(ops::ElementDivFn)
        .with_function(ops::MatPowerFn)

        // Properties (10 functions)
        .with_function(props::RowsFn)
        .with_function(props::ColsFn)
        .with_function(props::ShapeFn)
        .with_function(props::IsSquareFn)
        .with_function(props::IsSymmetricFn)
        .with_function(props::IsPositiveDefiniteFn)
        .with_function(props::RankFn)
        .with_function(props::TraceFn)
        .with_function(props::DeterminantFn)

        // Inverse (2 functions)
        .with_function(ops::InverseFn)
        .with_function(ops::PinvFn)

        // Norms (3 functions)
        .with_function(norms::NormFn)
        .with_function(norms::NormalizeFn)
        .with_function(norms::ConditionNumberFn)

        // Decompositions (6 functions)
        .with_function(decompose::LuFn)
        .with_function(decompose::QrFn)
        .with_function(decompose::SvdFn)
        .with_function(decompose::CholeskyFn)
        .with_function(decompose::EigenFn)
        .with_function(decompose::SchurFn)

        // Solving (5 functions)
        .with_function(solve::SolveFn)
        .with_function(solve::LstsqFn)
        .with_function(solve::SolveTriangularFn)
        .with_function(solve::NullSpaceFn)
        .with_function(solve::ColumnSpaceFn)

        // Vector operations (5 functions)
        .with_function(vector_ops::DotFn)
        .with_function(vector_ops::CrossFn)
        .with_function(vector_ops::OuterFn)
        .with_function(vector_ops::AngleFn)
        .with_function(vector_ops::ProjectFn)

        // Utility (7 functions)
        .with_function(ops::ReshapeFn)
        .with_function(ops::FlattenFn)
        .with_function(ops::StackHFn)
        .with_function(ops::StackVFn)
        .with_function(ops::ToListFn)
        .with_function(ops::FormatMatrixFn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_matrix_library() {
        let registry = PluginRegistry::new();
        let registry = load_matrix_library(registry);

        // Verify some functions are registered
        assert!(registry.get_function("matrix").is_some());
        assert!(registry.get_function("vector").is_some());
        assert!(registry.get_function("identity").is_some());
        assert!(registry.get_function("transpose").is_some());
        assert!(registry.get_function("matmul").is_some());
        assert!(registry.get_function("solve").is_some());
    }
}
