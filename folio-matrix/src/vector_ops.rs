//! Vector-specific operations

use folio_core::{Number, Value, FolioError};
use folio_plugin::{FunctionPlugin, FunctionMeta, ArgMeta, EvalContext};
use crate::types::Vector;
use crate::helpers::extract_vector;

// ============================================================================
// DOT - Dot product of two vectors
// ============================================================================

pub struct DotFn;

static DOT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "a",
        typ: "Vector",
        description: "First vector",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "b",
        typ: "Vector",
        description: "Second vector",
        optional: false,
        default: None,
    },
];
static DOT_EXAMPLES: [&str; 2] = [
    "dot(Vector(1, 2, 3), Vector(4, 5, 6)) → 32",
    "dot(Vector(1, 0), Vector(0, 1)) → 0",
];
static DOT_RELATED: [&str; 2] = ["cross", "norm"];

impl FunctionPlugin for DotFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "dot",
            description: "Compute the dot product of two vectors",
            usage: "dot(a, b)",
            args: &DOT_ARGS,
            returns: "Number",
            examples: &DOT_EXAMPLES,
            category: "matrix",
            source: None,
            related: &DOT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("dot", 2, args.len()));
        }

        let a = match extract_vector(&args[0], "dot", "a") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let b = match extract_vector(&args[1], "dot", "b") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        // Try exact computation first
        match (&a, &b) {
            (Vector::Exact(va), Vector::Exact(vb)) => {
                if va.len() != vb.len() {
                    return Value::Error(FolioError::domain_error("Vectors must have same length"));
                }
                let mut sum = Number::from_i64(0);
                for i in 0..va.len() {
                    sum = sum.add(&va[i].mul(&vb[i]));
                }
                Value::Number(sum)
            }
            _ => {
                let va = a.to_float();
                let vb = b.to_float();
                if va.len() != vb.len() {
                    return Value::Error(FolioError::domain_error("Vectors must have same length"));
                }
                let dot: f64 = va.iter().zip(vb.iter()).map(|(x, y)| x * y).sum();
                Value::Number(Number::from_f64(dot))
            }
        }
    }
}

// ============================================================================
// CROSS - Cross product of two 3D vectors
// ============================================================================

pub struct CrossFn;

static CROSS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "a",
        typ: "Vector",
        description: "First 3D vector",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "b",
        typ: "Vector",
        description: "Second 3D vector",
        optional: false,
        default: None,
    },
];
static CROSS_EXAMPLES: [&str; 2] = [
    "cross(Vector(1, 0, 0), Vector(0, 1, 0)) → [0, 0, 1]",
    "cross(Vector(1, 2, 3), Vector(4, 5, 6))",
];
static CROSS_RELATED: [&str; 2] = ["dot", "outer"];

impl FunctionPlugin for CrossFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cross",
            description: "Compute the cross product of two 3D vectors",
            usage: "cross(a, b)",
            args: &CROSS_ARGS,
            returns: "List",
            examples: &CROSS_EXAMPLES,
            category: "matrix",
            source: None,
            related: &CROSS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("cross", 2, args.len()));
        }

        let a = match extract_vector(&args[0], "cross", "a") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let b = match extract_vector(&args[1], "cross", "b") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        // Cross product only defined for 3D vectors
        match (&a, &b) {
            (Vector::Exact(va), Vector::Exact(vb)) => {
                if va.len() != 3 || vb.len() != 3 {
                    return Value::Error(FolioError::domain_error("Cross product requires 3D vectors"));
                }
                let result = vec![
                    va[1].mul(&vb[2]).sub(&va[2].mul(&vb[1])),
                    va[2].mul(&vb[0]).sub(&va[0].mul(&vb[2])),
                    va[0].mul(&vb[1]).sub(&va[1].mul(&vb[0])),
                ];
                Value::List(result.into_iter().map(Value::Number).collect())
            }
            _ => {
                let va = a.to_float();
                let vb = b.to_float();
                if va.len() != 3 || vb.len() != 3 {
                    return Value::Error(FolioError::domain_error("Cross product requires 3D vectors"));
                }
                let result = vec![
                    va[1] * vb[2] - va[2] * vb[1],
                    va[2] * vb[0] - va[0] * vb[2],
                    va[0] * vb[1] - va[1] * vb[0],
                ];
                Value::List(result.into_iter().map(|x| Value::Number(Number::from_f64(x))).collect())
            }
        }
    }
}

// ============================================================================
// OUTER - Outer product of two vectors
// ============================================================================

pub struct OuterFn;

static OUTER_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "a",
        typ: "Vector",
        description: "First vector (becomes column)",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "b",
        typ: "Vector",
        description: "Second vector (becomes row)",
        optional: false,
        default: None,
    },
];
static OUTER_EXAMPLES: [&str; 2] = [
    "outer(Vector(1, 2), Vector(3, 4)) → 2×2 matrix",
    "outer(Vector(1, 2, 3), Vector(4, 5)) → 3×2 matrix",
];
static OUTER_RELATED: [&str; 2] = ["cross", "matmul"];

impl FunctionPlugin for OuterFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "outer",
            description: "Compute the outer product of two vectors (returns matrix)",
            usage: "outer(a, b)",
            args: &OUTER_ARGS,
            returns: "Matrix",
            examples: &OUTER_EXAMPLES,
            category: "matrix",
            source: None,
            related: &OUTER_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("outer", 2, args.len()));
        }

        let a = match extract_vector(&args[0], "outer", "a") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let b = match extract_vector(&args[1], "outer", "b") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        // Compute outer product: result[i,j] = a[i] * b[j]
        match (&a, &b) {
            (Vector::Exact(va), Vector::Exact(vb)) => {
                let rows = va.len();
                let cols = vb.len();
                let mut data = Vec::with_capacity(rows);
                for ai in va.iter() {
                    let row: Vec<Number> = vb.iter().map(|bj| ai.mul(bj)).collect();
                    data.push(row);
                }
                use crate::types::{ExactMatrix, Matrix};
                let matrix = Matrix::Exact(ExactMatrix { data, rows, cols });
                matrix.to_value()
            }
            _ => {
                let va = a.to_float();
                let vb = b.to_float();
                let rows = va.len();
                let cols = vb.len();
                let mut data = nalgebra::DMatrix::zeros(rows, cols);
                for i in 0..rows {
                    for j in 0..cols {
                        data[(i, j)] = va[i] * vb[j];
                    }
                }
                use crate::types::{FloatMatrix, Matrix};
                let matrix = Matrix::Float(FloatMatrix { data });
                matrix.to_value()
            }
        }
    }
}

// ============================================================================
// ANGLE - Angle between two vectors
// ============================================================================

pub struct AngleFn;

static ANGLE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "a",
        typ: "Vector",
        description: "First vector",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "b",
        typ: "Vector",
        description: "Second vector",
        optional: false,
        default: None,
    },
];
static ANGLE_EXAMPLES: [&str; 2] = [
    "angle(Vector(1, 0), Vector(0, 1)) → π/2",
    "angle(Vector(1, 1), Vector(1, 0)) → π/4",
];
static ANGLE_RELATED: [&str; 2] = ["dot", "normalize"];

impl FunctionPlugin for AngleFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "angle",
            description: "Compute the angle between two vectors (in radians)",
            usage: "angle(a, b)",
            args: &ANGLE_ARGS,
            returns: "Number",
            examples: &ANGLE_EXAMPLES,
            category: "matrix",
            source: None,
            related: &ANGLE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("angle", 2, args.len()));
        }

        let a = match extract_vector(&args[0], "angle", "a") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let b = match extract_vector(&args[1], "angle", "b") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let va = a.to_float();
        let vb = b.to_float();

        if va.len() != vb.len() {
            return Value::Error(FolioError::domain_error("Vectors must have same length"));
        }

        let dot: f64 = va.iter().zip(vb.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = va.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = vb.iter().map(|x| x * x).sum::<f64>().sqrt();

        if norm_a < 1e-15 || norm_b < 1e-15 {
            return Value::Error(FolioError::domain_error("Cannot compute angle with zero vector"));
        }

        // cos(theta) = (a · b) / (|a| * |b|)
        let cos_theta = (dot / (norm_a * norm_b)).clamp(-1.0, 1.0);
        let angle = cos_theta.acos();

        Value::Number(Number::from_f64(angle))
    }
}

// ============================================================================
// PROJECT - Project vector a onto vector b
// ============================================================================

pub struct ProjectFn;

static PROJECT_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "a",
        typ: "Vector",
        description: "Vector to project",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "b",
        typ: "Vector",
        description: "Vector to project onto",
        optional: false,
        default: None,
    },
];
static PROJECT_EXAMPLES: [&str; 2] = [
    "project(Vector(3, 4), Vector(1, 0)) → [3, 0]",
    "project(Vector(1, 1), Vector(1, 2))",
];
static PROJECT_RELATED: [&str; 2] = ["dot", "normalize"];

impl FunctionPlugin for ProjectFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "project",
            description: "Project vector a onto vector b",
            usage: "project(a, b)",
            args: &PROJECT_ARGS,
            returns: "List",
            examples: &PROJECT_EXAMPLES,
            category: "matrix",
            source: None,
            related: &PROJECT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("project", 2, args.len()));
        }

        let a = match extract_vector(&args[0], "project", "a") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let b = match extract_vector(&args[1], "project", "b") {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let va = a.to_float();
        let vb = b.to_float();

        if va.len() != vb.len() {
            return Value::Error(FolioError::domain_error("Vectors must have same length"));
        }

        let dot_ab: f64 = va.iter().zip(vb.iter()).map(|(x, y)| x * y).sum();
        let dot_bb: f64 = vb.iter().map(|x| x * x).sum();

        if dot_bb < 1e-15 {
            return Value::Error(FolioError::domain_error("Cannot project onto zero vector"));
        }

        // proj_b(a) = (a · b) / (b · b) * b
        let scale = dot_ab / dot_bb;
        let result: Vec<Value> = vb.iter()
            .map(|&x| Value::Number(Number::from_f64(x * scale)))
            .collect();

        Value::List(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construct::VectorFn;
    use std::f64::consts::PI;
    use std::sync::Arc;

    fn ctx() -> EvalContext {
        EvalContext::new(Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_dot_product() {
        let a = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(3)),
        ])], &ctx());

        let b = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(5)),
            Value::Number(Number::from_i64(6)),
        ])], &ctx());

        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        if let Value::Number(n) = DotFn.call(&[a, b], &ctx()) {
            let val = n.to_f64().unwrap();
            assert!((val - 32.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_cross_product() {
        // i × j = k
        let i = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(0)),
        ])], &ctx());

        let j = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(0)),
        ])], &ctx());

        if let Value::List(result) = CrossFn.call(&[i, j], &ctx()) {
            assert_eq!(result.len(), 3);
            if let (Value::Number(x), Value::Number(y), Value::Number(z)) =
                (&result[0], &result[1], &result[2]) {
                assert!((x.to_f64().unwrap() - 0.0).abs() < 1e-10);
                assert!((y.to_f64().unwrap() - 0.0).abs() < 1e-10);
                assert!((z.to_f64().unwrap() - 1.0).abs() < 1e-10);
            }
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_angle() {
        // 90 degrees between (1, 0) and (0, 1)
        let a = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(0)),
        ])], &ctx());

        let b = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(0)),
            Value::Number(Number::from_i64(1)),
        ])], &ctx());

        if let Value::Number(n) = AngleFn.call(&[a, b], &ctx()) {
            let angle = n.to_f64().unwrap();
            assert!((angle - PI / 2.0).abs() < 1e-10);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_project() {
        // Project (3, 4) onto (1, 0) should give (3, 0)
        let a = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(3)),
            Value::Number(Number::from_i64(4)),
        ])], &ctx());

        let b = VectorFn.call(&[Value::List(vec![
            Value::Number(Number::from_i64(1)),
            Value::Number(Number::from_i64(0)),
        ])], &ctx());

        if let Value::List(result) = ProjectFn.call(&[a, b], &ctx()) {
            assert_eq!(result.len(), 2);
            if let (Value::Number(x), Value::Number(y)) = (&result[0], &result[1]) {
                let xf = x.to_f64().unwrap();
                let yf = y.to_f64().unwrap();
                assert!((xf - 3.0).abs() < 1e-10);
                assert!(yf.abs() < 1e-10);
            }
        } else {
            panic!("Expected list");
        }
    }
}
