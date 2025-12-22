//! ISIS Transform Functions
//!
//! X(n) = -ln(n) × φ / (2π × ln(φ))
//!
//! This maps positive numbers n to a "golden ratio normalized" coordinate system.
//! Key properties:
//! - X(φ) = -1 (φ maps to -1)
//! - X(1) = 0 (1 maps to origin)
//! - X(1/φ) = 1 (1/φ maps to 1)

use folio_plugin::prelude::*;

pub struct IsisTransform;
pub struct IsisInverse;

static ISIS_ARGS: [ArgMeta; 1] = [ArgMeta { name: "n", typ: "Number", description: "Value to transform (positive)", optional: false, default: None }];
static ISIS_EXAMPLES: [&str; 3] = ["ISIS(2)", "ISIS(phi)", "ISIS(1)"];
static ISIS_RELATED: [&str; 1] = ["ISIS_INV"];

static ISIS_INV_ARGS: [ArgMeta; 1] = [ArgMeta { name: "x", typ: "Number", description: "X-space value", optional: false, default: None }];
static ISIS_INV_EXAMPLES: [&str; 3] = ["ISIS_INV(0)", "ISIS_INV(-1)", "ISIS_INV(1)"];
static ISIS_INV_RELATED: [&str; 1] = ["ISIS"];

impl FunctionPlugin for IsisTransform {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ISIS",
            description: "ISIS transform: X(n) = -ln(n) × φ / (2π × ln(φ))",
            usage: "ISIS(n)",
            args: &ISIS_ARGS,
            returns: "Number",
            examples: &ISIS_EXAMPLES,
            category: "isis",
            source: Some("https://example.com/isis-docs"),
            related: &ISIS_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        // Check argument count
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("ISIS", 1, args.len()));
        }

        // Get the input number
        let n = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("ISIS", "n", "Number", other.type_name())),
        };

        // Domain check: n must be positive
        if n.is_zero() {
            return Value::Error(FolioError::domain_error("ISIS transform undefined for n=0 (ln(0) is undefined)"));
        }
        if n.is_negative() {
            return Value::Error(FolioError::domain_error("ISIS transform requires positive n (ln of negative undefined)"));
        }

        // Compute: X(n) = -ln(n) × φ / (2π × ln(φ))
        let phi = Number::phi(ctx.precision);
        let pi = Number::pi(ctx.precision);
        let two = Number::from_i64(2);

        // Compute ln(n)
        let ln_n = match n.ln(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Compute ln(φ)
        let ln_phi = match phi.ln(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Compute 2π
        let two_pi = two.mul(&pi);

        // Compute denominator: 2π × ln(φ)
        let denominator = two_pi.mul(&ln_phi);

        // Compute numerator: -ln(n) × φ
        let zero = Number::from_i64(0);
        let neg_ln_n = zero.sub(&ln_n);
        let numerator = neg_ln_n.mul(&phi);

        // Compute X(n) = numerator / denominator
        match numerator.checked_div(&denominator) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(e.into()),
        }
    }
}

impl FunctionPlugin for IsisInverse {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ISIS_INV",
            description: "Inverse ISIS transform: find n where X(n) = x. n = exp(-x × 2π × ln(φ) / φ)",
            usage: "ISIS_INV(x)",
            args: &ISIS_INV_ARGS,
            returns: "Number",
            examples: &ISIS_INV_EXAMPLES,
            category: "isis",
            source: Some("https://example.com/isis-docs"),
            related: &ISIS_INV_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        // Check argument count
        if args.len() != 1 {
            return Value::Error(FolioError::arg_count("ISIS_INV", 1, args.len()));
        }

        // Get the input number
        let x = match &args[0] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("ISIS_INV", "x", "Number", other.type_name())),
        };

        // Inverse: n = exp(-x × 2π × ln(φ) / φ)
        let phi = Number::phi(ctx.precision);
        let pi = Number::pi(ctx.precision);
        let two = Number::from_i64(2);
        let zero = Number::from_i64(0);

        // Compute ln(φ)
        let ln_phi = match phi.ln(ctx.precision) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Compute 2π
        let two_pi = two.mul(&pi);

        // Compute -x × 2π × ln(φ)
        let neg_x = zero.sub(x);
        let temp = neg_x.mul(&two_pi).mul(&ln_phi);

        // Divide by φ
        let exponent = match temp.checked_div(&phi) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        // Compute exp(exponent)
        Value::Number(exponent.exp(ctx.precision))
    }
}
