//! Recipe scaling functions
//!
//! Scale recipes by servings, pan size, or batch count.

use folio_core::{FolioError, Number, Value};
use folio_plugin::{ArgMeta, EvalContext, FunctionMeta, FunctionPlugin};

use crate::helpers::{extract_number, extract_text, extract_optional_number, validate_positive};

// ============ scale_recipe ============

pub struct ScaleRecipe;

static SCALE_RECIPE_ARGS: [ArgMeta; 3] = [
    ArgMeta::required("amount", "Number", "Original ingredient amount"),
    ArgMeta::required("from_servings", "Number", "Original recipe servings"),
    ArgMeta::required("to_servings", "Number", "Desired servings"),
];

static SCALE_RECIPE_EXAMPLES: [&str; 3] = [
    "scale_recipe(2, 4, 8) -> 4",
    "scale_recipe(1.5, 6, 4) -> 1",
    "scale_recipe(100, 12, 6) -> 50",
];

static SCALE_RECIPE_RELATED: [&str; 2] = ["pan_scale", "batch_multiply"];

impl FunctionPlugin for ScaleRecipe {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "scale_recipe",
            description: "Scale an ingredient amount for different serving sizes",
            usage: "scale_recipe(amount, from_servings, to_servings)",
            args: &SCALE_RECIPE_ARGS,
            returns: "Number",
            examples: &SCALE_RECIPE_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &SCALE_RECIPE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("scale_recipe", 3, args.len()));
        }

        let amount = match extract_number(&args[0], "scale_recipe", "amount") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let from = match extract_number(&args[1], "scale_recipe", "from_servings") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let to = match extract_number(&args[2], "scale_recipe", "to_servings") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        if let Err(e) = validate_positive(&from, "scale_recipe", "from_servings") {
            return Value::Error(e);
        }
        if let Err(e) = validate_positive(&to, "scale_recipe", "to_servings") {
            return Value::Error(e);
        }

        // scaled = amount * (to / from)
        match to.checked_div(&from) {
            Ok(scale) => Value::Number(amount.mul(&scale)),
            Err(e) => Value::Error(e.into()),
        }
    }
}

// ============ pan_scale ============

pub struct PanScale;

static PAN_SCALE_ARGS: [ArgMeta; 4] = [
    ArgMeta::required("amount", "Number", "Original ingredient amount"),
    ArgMeta::required("original_pan", "Text", "Original pan size (e.g., \"9x13\", \"8inch_round\", \"9inch_round\")"),
    ArgMeta::required("new_pan", "Text", "New pan size"),
    ArgMeta::optional("depth_ratio", "Number", "Depth ratio if pans have different depths", "1"),
];

static PAN_SCALE_EXAMPLES: [&str; 3] = [
    "pan_scale(2, \"8inch_round\", \"9inch_round\") -> 2.53",
    "pan_scale(1, \"9x13\", \"8x8\") -> 0.55",
    "pan_scale(100, \"9inch_round\", \"6inch_round\") -> 44.4",
];

static PAN_SCALE_RELATED: [&str; 2] = ["scale_recipe", "batch_multiply"];

impl FunctionPlugin for PanScale {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "pan_scale",
            description: "Scale ingredient for different baking pan sizes",
            usage: "pan_scale(amount, original_pan, new_pan, [depth_ratio])",
            args: &PAN_SCALE_ARGS,
            returns: "Number",
            examples: &PAN_SCALE_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &PAN_SCALE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("pan_scale", 3, args.len()));
        }

        let amount = match extract_number(&args[0], "pan_scale", "amount") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let original = match extract_text(&args[1], "pan_scale", "original_pan") {
            Ok(s) => s.to_lowercase().replace(' ', ""),
            Err(e) => return Value::Error(e),
        };
        let new = match extract_text(&args[2], "pan_scale", "new_pan") {
            Ok(s) => s.to_lowercase().replace(' ', ""),
            Err(e) => return Value::Error(e),
        };
        let depth_ratio = extract_optional_number(args, 3)
            .unwrap_or_else(|| Number::from_i64(1));

        let original_area = match pan_area(&original) {
            Some(a) => a,
            None => return Value::Error(FolioError::domain_error(format!(
                "pan_scale: Unknown pan size '{}'. Valid formats: 6inch_round, 8inch_round, 9inch_round, 10inch_round, 12inch_round, 8x8, 9x9, 9x13, 10x15, 11x17, loaf, 9x5_loaf, 8x4_loaf, springform_9, bundt",
                original
            ))),
        };

        let new_area = match pan_area(&new) {
            Some(a) => a,
            None => return Value::Error(FolioError::domain_error(format!(
                "pan_scale: Unknown pan size '{}'. Valid formats: 6inch_round, 8inch_round, 9inch_round, 10inch_round, 12inch_round, 8x8, 9x9, 9x13, 10x15, 11x17, loaf, 9x5_loaf, 8x4_loaf, springform_9, bundt",
                new
            ))),
        };

        // scale = (new_area / original_area) * depth_ratio
        let original_num = Number::from_f64(original_area);
        let new_num = Number::from_f64(new_area);

        match new_num.checked_div(&original_num) {
            Ok(area_ratio) => {
                let scale = area_ratio.mul(&depth_ratio);
                Value::Number(amount.mul(&scale))
            }
            Err(e) => Value::Error(e.into()),
        }
    }
}

fn pan_area(pan: &str) -> Option<f64> {
    let pi = std::f64::consts::PI;

    match pan {
        // Round pans (area = pi * r^2, where r = diameter/2)
        "6inch_round" | "6\"_round" | "6round" | "6in_round" => Some(pi * 9.0),       // 6" diameter, r=3
        "7inch_round" | "7\"_round" | "7round" | "7in_round" => Some(pi * 12.25),     // 7" diameter, r=3.5
        "8inch_round" | "8\"_round" | "8round" | "8in_round" => Some(pi * 16.0),      // 8" diameter, r=4
        "9inch_round" | "9\"_round" | "9round" | "9in_round" => Some(pi * 20.25),     // 9" diameter, r=4.5
        "10inch_round" | "10\"_round" | "10round" | "10in_round" => Some(pi * 25.0),  // 10" diameter, r=5
        "12inch_round" | "12\"_round" | "12round" | "12in_round" => Some(pi * 36.0),  // 12" diameter, r=6

        // Square/rectangular pans (area = length * width)
        "8x8" => Some(64.0),
        "9x9" => Some(81.0),
        "9x13" => Some(117.0),
        "10x15" | "jelly_roll" => Some(150.0),
        "11x17" => Some(187.0),
        "sheet" | "half_sheet" | "13x18" => Some(234.0),    // 13x18 half sheet
        "quarter_sheet" => Some(117.0),                      // 9x13

        // Loaf pans
        "loaf" | "9x5_loaf" | "9x5" => Some(45.0),           // Standard loaf pan
        "8x4_loaf" | "8x4" => Some(32.0),
        "8.5x4.5_loaf" => Some(38.25),

        // Specialty pans
        "bundt" | "10inch_bundt" => Some(pi * 25.0 * 0.7),   // Ring shape, ~70% of full circle
        "springform_9" | "9inch_springform" => Some(pi * 20.25),
        "springform_10" | "10inch_springform" => Some(pi * 25.0),
        "pie_9" | "9inch_pie" => Some(pi * 20.25),
        "pie_10" | "10inch_pie" => Some(pi * 25.0),
        "tart_9" | "9inch_tart" => Some(pi * 20.25),

        // Muffin/cupcake (per cup)
        "muffin" | "standard_muffin" => Some(pi * 1.5625),   // ~2.5" diameter
        "mini_muffin" => Some(pi * 0.5625),                  // ~1.5" diameter
        "jumbo_muffin" => Some(pi * 3.0625),                 // ~3.5" diameter

        _ => None,
    }
}

// ============ batch_multiply ============

pub struct BatchMultiply;

static BATCH_MULTIPLY_ARGS: [ArgMeta; 2] = [
    ArgMeta::required("amount", "Number", "Single batch amount"),
    ArgMeta::required("batches", "Number", "Number of batches"),
];

static BATCH_MULTIPLY_EXAMPLES: [&str; 3] = [
    "batch_multiply(2, 3) -> 6",
    "batch_multiply(0.5, 4) -> 2",
    "batch_multiply(125, 2.5) -> 312.5",
];

static BATCH_MULTIPLY_RELATED: [&str; 2] = ["scale_recipe", "pan_scale"];

impl FunctionPlugin for BatchMultiply {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "batch_multiply",
            description: "Multiply ingredient amount by number of batches",
            usage: "batch_multiply(amount, batches)",
            args: &BATCH_MULTIPLY_ARGS,
            returns: "Number",
            examples: &BATCH_MULTIPLY_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &BATCH_MULTIPLY_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("batch_multiply", 2, args.len()));
        }

        let amount = match extract_number(&args[0], "batch_multiply", "amount") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let batches = match extract_number(&args[1], "batch_multiply", "batches") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        Value::Number(amount.mul(&batches))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use folio_plugin::EvalContext;
    use std::sync::Arc;

    fn ctx() -> EvalContext {
        EvalContext::new(Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_scale_recipe_double() {
        let scale = ScaleRecipe;
        let result = scale.call(&[
            Value::Number(Number::from_i64(2)),
            Value::Number(Number::from_i64(4)),
            Value::Number(Number::from_i64(8)),
        ], &ctx());

        if let Value::Number(n) = result {
            assert_eq!(n.to_i64(), Some(4));
        } else {
            panic!("Expected Number");
        }
    }

    #[test]
    fn test_pan_area_round() {
        let area_8 = pan_area("8inch_round").unwrap();
        let area_9 = pan_area("9inch_round").unwrap();

        // 9" round should be larger than 8" round
        assert!(area_9 > area_8);

        // Ratio should be approximately (9/8)^2 = 1.265625
        let ratio = area_9 / area_8;
        assert!((ratio - 1.265625).abs() < 0.01);
    }

    #[test]
    fn test_batch_multiply() {
        let batch = BatchMultiply;
        let result = batch.call(&[
            Value::Number(Number::from_i64(100)),
            Value::Number(Number::from_i64(3)),
        ], &ctx());

        if let Value::Number(n) = result {
            assert_eq!(n.to_i64(), Some(300));
        } else {
            panic!("Expected Number");
        }
    }
}
