//! Unit conversion functions for Folio

use folio_plugin::prelude::*;
use folio_core::Number;
use crate::Quantity;
use crate::unit::ConversionError;
use crate::units::UNITS;
use crate::parse::{parse_unit, parse_conversion, parse_quantity_string};

fn get_precision(ctx: &EvalContext) -> u32 {
    ctx.precision
}

fn conversion_error_to_folio(e: ConversionError) -> FolioError {
    FolioError::domain_error(&format!("{}", e))
}

// ============ convert ============

pub struct Convert;

static CONVERT_ARGS: [ArgMeta; 3] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to convert",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "from_unit",
        typ: "Text",
        description: "Source unit (e.g., \"km\")",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "to_unit",
        typ: "Text",
        description: "Target unit (e.g., \"mi\")",
        optional: false,
        default: None,
    },
];

static CONVERT_EXAMPLES: [&str; 3] = [
    "convert(100, \"km\", \"mi\") → 62.137",
    "convert(32, \"F\", \"C\") → 0",
    "convert(1, \"kg\", \"lb\") → 2.205",
];

static CONVERT_RELATED: [&str; 3] = ["to_base", "in_units", "quantity"];

impl FunctionPlugin for Convert {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "convert",
            description: "Convert a value from one unit to another",
            usage: "convert(value, from_unit, to_unit)",
            args: &CONVERT_ARGS,
            returns: "Number",
            examples: &CONVERT_EXAMPLES,
            category: "units",
            source: None,
            related: &CONVERT_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 3 {
            return Value::Error(FolioError::arg_count("convert", 3, args.len()));
        }

        let value = match args[0].as_number() {
            Some(n) => n.clone(),
            None => return Value::Error(FolioError::arg_type("convert", "value", "Number", args[0].type_name())),
        };

        let from_str = match args[1].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("convert", "from_unit", "Text", args[1].type_name())),
        };

        let to_str = match args[2].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("convert", "to_unit", "Text", args[2].type_name())),
        };

        let from_unit = match parse_unit(from_str) {
            Ok(u) => u,
            Err(e) => return Value::Error(conversion_error_to_folio(e)),
        };

        let to_unit = match parse_unit(to_str) {
            Ok(u) => u,
            Err(e) => return Value::Error(conversion_error_to_folio(e)),
        };

        let precision = get_precision(ctx);
        match from_unit.convert_to(&value, &to_unit, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(conversion_error_to_folio(e)),
        }
    }
}

// ============ to_base ============

pub struct ToBase;

static TO_BASE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to convert",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "unit",
        typ: "Text",
        description: "Source unit",
        optional: false,
        default: None,
    },
];

static TO_BASE_EXAMPLES: [&str; 2] = [
    "to_base(5, \"km\") → 5000",
    "to_base(100, \"C\") → 373.15",
];

static TO_BASE_RELATED: [&str; 2] = ["convert", "in_units"];

impl FunctionPlugin for ToBase {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "to_base",
            description: "Convert a value to SI base units",
            usage: "to_base(value, unit)",
            args: &TO_BASE_ARGS,
            returns: "Number",
            examples: &TO_BASE_EXAMPLES,
            category: "units",
            source: None,
            related: &TO_BASE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("to_base", 2, args.len()));
        }

        let value = match args[0].as_number() {
            Some(n) => n.clone(),
            None => return Value::Error(FolioError::arg_type("to_base", "value", "Number", args[0].type_name())),
        };

        let unit_str = match args[1].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("to_base", "unit", "Text", args[1].type_name())),
        };

        let unit = match parse_unit(unit_str) {
            Ok(u) => u,
            Err(e) => return Value::Error(conversion_error_to_folio(e)),
        };

        let si_value = unit.to_si(&value);
        Value::Number(si_value)
    }
}

// ============ simplify ============

pub struct Simplify;

static SIMPLIFY_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to simplify",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "unit",
        typ: "Text",
        description: "Unit of the value",
        optional: false,
        default: None,
    },
];

static SIMPLIFY_EXAMPLES: [&str; 2] = [
    "simplify(5000, \"m\") → \"5 km\"",
    "simplify(0.001, \"kg\") → \"1 g\"",
];

static SIMPLIFY_RELATED: [&str; 2] = ["convert", "to_base"];

impl FunctionPlugin for Simplify {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "simplify",
            description: "Simplify a value by choosing an appropriate unit prefix",
            usage: "simplify(value, unit)",
            args: &SIMPLIFY_ARGS,
            returns: "Text",
            examples: &SIMPLIFY_EXAMPLES,
            category: "units",
            source: None,
            related: &SIMPLIFY_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("simplify", 2, args.len()));
        }

        let value = match args[0].as_number() {
            Some(n) => n.clone(),
            None => return Value::Error(FolioError::arg_type("simplify", "value", "Number", args[0].type_name())),
        };

        let unit_str = match args[1].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("simplify", "unit", "Text", args[1].type_name())),
        };

        let unit = match parse_unit(unit_str) {
            Ok(u) => u,
            Err(e) => return Value::Error(conversion_error_to_folio(e)),
        };

        let quantity = Quantity::new(value, unit);
        let simplified = simplify_quantity(&quantity, get_precision(ctx));

        Value::Text(format!("{}", simplified))
    }
}

/// Find a better unit for displaying a quantity
fn simplify_quantity(q: &Quantity, precision: u32) -> Quantity {
    // Get all units in the same category
    let category = &q.unit.category;
    let compatible_units: Vec<_> = UNITS.by_category(category)
        .into_iter()
        .filter(|u| u.is_compatible(&q.unit))
        .collect();

    if compatible_units.is_empty() {
        return q.clone();
    }

    // Convert to each compatible unit and find the one with the "nicest" value
    // (closest to 1-1000 range)
    let mut best = q.clone();
    let mut best_score = score_value(&q.value);

    for unit in compatible_units {
        if let Ok(converted) = q.convert_to(unit, precision) {
            let score = score_value(&converted.value);
            if score > best_score {
                best_score = score;
                best = converted;
            }
        }
    }

    best
}

/// Score a value based on how "nice" it is for display (1-1000 is ideal)
fn score_value(v: &Number) -> f64 {
    let abs_val = v.abs();
    if abs_val.is_zero() {
        return 1.0;
    }

    // Try to get approximate f64 for scoring
    let approx = match abs_val.to_f64() {
        Some(f) => f,
        None => return 10.0, // Can't convert, give default score
    };

    // Prefer values in 1-1000 range
    if approx >= 1.0 && approx <= 1000.0 {
        100.0 - (approx.log10() - 1.5).abs() * 10.0
    } else if approx >= 0.001 && approx < 1.0 {
        50.0 - (approx.log10().abs()) * 5.0
    } else if approx > 1000.0 && approx <= 1_000_000.0 {
        50.0 - (approx.log10() - 3.0) * 5.0
    } else {
        10.0
    }
}

// ============ in_units ============

pub struct InUnits;

static IN_UNITS_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Value to convert",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "conversion",
        typ: "Text",
        description: "Conversion spec like \"km->mi\"",
        optional: false,
        default: None,
    },
];

static IN_UNITS_EXAMPLES: [&str; 2] = [
    "in_units(100, \"km->mi\") → 62.137",
    "in_units(0, \"C->F\") → 32",
];

static IN_UNITS_RELATED: [&str; 2] = ["convert", "to_base"];

impl FunctionPlugin for InUnits {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "in_units",
            description: "Convert a value using a conversion spec",
            usage: "in_units(value, \"from->to\")",
            args: &IN_UNITS_ARGS,
            returns: "Number",
            examples: &IN_UNITS_EXAMPLES,
            category: "units",
            source: None,
            related: &IN_UNITS_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("in_units", 2, args.len()));
        }

        let value = match args[0].as_number() {
            Some(n) => n.clone(),
            None => return Value::Error(FolioError::arg_type("in_units", "value", "Number", args[0].type_name())),
        };

        let conv_str = match args[1].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("in_units", "conversion", "Text", args[1].type_name())),
        };

        let (from_unit, to_unit) = match parse_conversion(conv_str) {
            Ok((f, t)) => (f, t),
            Err(e) => return Value::Error(conversion_error_to_folio(e)),
        };

        let precision = get_precision(ctx);
        match from_unit.convert_to(&value, &to_unit, precision) {
            Ok(result) => Value::Number(result),
            Err(e) => Value::Error(conversion_error_to_folio(e)),
        }
    }
}

// ============ extract_value ============

pub struct ExtractValue;

static EXTRACT_VALUE_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "quantity",
        typ: "Text",
        description: "Quantity string like \"5 km\"",
        optional: false,
        default: None,
    },
];

static EXTRACT_VALUE_EXAMPLES: [&str; 2] = [
    "extract_value(\"5 km\") → 5",
    "extract_value(\"3.14 rad\") → 3.14",
];

static EXTRACT_VALUE_RELATED: [&str; 2] = ["extract_unit", "quantity"];

impl FunctionPlugin for ExtractValue {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "extract_value",
            description: "Extract the numeric value from a quantity string",
            usage: "extract_value(quantity)",
            args: &EXTRACT_VALUE_ARGS,
            returns: "Number",
            examples: &EXTRACT_VALUE_EXAMPLES,
            category: "units",
            source: None,
            related: &EXTRACT_VALUE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("extract_value", 1, 0));
        }

        let qty_str = match args[0].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("extract_value", "quantity", "Text", args[0].type_name())),
        };

        match parse_quantity_string(qty_str) {
            Ok((value, _)) => Value::Number(value),
            Err(e) => Value::Error(conversion_error_to_folio(e)),
        }
    }
}

// ============ extract_unit ============

pub struct ExtractUnit;

static EXTRACT_UNIT_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "quantity",
        typ: "Text",
        description: "Quantity string like \"5 km\"",
        optional: false,
        default: None,
    },
];

static EXTRACT_UNIT_EXAMPLES: [&str; 2] = [
    "extract_unit(\"5 km\") → \"km\"",
    "extract_unit(\"3.14 rad\") → \"rad\"",
];

static EXTRACT_UNIT_RELATED: [&str; 2] = ["extract_value", "quantity"];

impl FunctionPlugin for ExtractUnit {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "extract_unit",
            description: "Extract the unit symbol from a quantity string",
            usage: "extract_unit(quantity)",
            args: &EXTRACT_UNIT_ARGS,
            returns: "Text",
            examples: &EXTRACT_UNIT_EXAMPLES,
            category: "units",
            source: None,
            related: &EXTRACT_UNIT_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("extract_unit", 1, 0));
        }

        let qty_str = match args[0].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("extract_unit", "quantity", "Text", args[0].type_name())),
        };

        match parse_quantity_string(qty_str) {
            Ok((_, unit)) => Value::Text(unit.symbol),
            Err(e) => Value::Error(conversion_error_to_folio(e)),
        }
    }
}

// ============ dimensions ============

pub struct Dimensions;

static DIMENSIONS_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "unit",
        typ: "Text",
        description: "Unit string like \"m/s\"",
        optional: false,
        default: None,
    },
];

static DIMENSIONS_EXAMPLES: [&str; 3] = [
    "dimensions(\"m\") → \"L\"",
    "dimensions(\"m/s\") → \"L T^-1\"",
    "dimensions(\"N\") → \"L M T^-2\"",
];

static DIMENSIONS_RELATED: [&str; 2] = ["is_dimensionless", "compatible"];

impl FunctionPlugin for Dimensions {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "dimensions",
            description: "Get the dimensional signature of a unit",
            usage: "dimensions(unit)",
            args: &DIMENSIONS_ARGS,
            returns: "Text",
            examples: &DIMENSIONS_EXAMPLES,
            category: "units",
            source: None,
            related: &DIMENSIONS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("dimensions", 1, 0));
        }

        let unit_str = match args[0].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("dimensions", "unit", "Text", args[0].type_name())),
        };

        match parse_unit(unit_str) {
            Ok(unit) => Value::Text(format!("{}", unit.dimension)),
            Err(e) => Value::Error(conversion_error_to_folio(e)),
        }
    }
}

// ============ is_dimensionless ============

pub struct IsDimensionless;

static IS_DIMLESS_ARGS: [ArgMeta; 1] = [
    ArgMeta {
        name: "unit",
        typ: "Text",
        description: "Unit string",
        optional: false,
        default: None,
    },
];

static IS_DIMLESS_EXAMPLES: [&str; 3] = [
    "is_dimensionless(\"rad\") → true",
    "is_dimensionless(\"m\") → false",
    "is_dimensionless(\"m/m\") → true",
];

static IS_DIMLESS_RELATED: [&str; 2] = ["dimensions", "compatible"];

impl FunctionPlugin for IsDimensionless {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "is_dimensionless",
            description: "Check if a unit is dimensionless",
            usage: "is_dimensionless(unit)",
            args: &IS_DIMLESS_ARGS,
            returns: "Bool",
            examples: &IS_DIMLESS_EXAMPLES,
            category: "units",
            source: None,
            related: &IS_DIMLESS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("is_dimensionless", 1, 0));
        }

        let unit_str = match args[0].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("is_dimensionless", "unit", "Text", args[0].type_name())),
        };

        match parse_unit(unit_str) {
            Ok(unit) => Value::Bool(unit.dimension.is_dimensionless()),
            Err(e) => Value::Error(conversion_error_to_folio(e)),
        }
    }
}

// ============ compatible ============

pub struct Compatible;

static COMPATIBLE_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "unit1",
        typ: "Text",
        description: "First unit",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "unit2",
        typ: "Text",
        description: "Second unit",
        optional: false,
        default: None,
    },
];

static COMPATIBLE_EXAMPLES: [&str; 3] = [
    "compatible(\"km\", \"mi\") → true",
    "compatible(\"m\", \"s\") → false",
    "compatible(\"N\", \"kg*m/s^2\") → true",
];

static COMPATIBLE_RELATED: [&str; 2] = ["dimensions", "convert"];

impl FunctionPlugin for Compatible {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "compatible",
            description: "Check if two units are dimensionally compatible",
            usage: "compatible(unit1, unit2)",
            args: &COMPATIBLE_ARGS,
            returns: "Bool",
            examples: &COMPATIBLE_EXAMPLES,
            category: "units",
            source: None,
            related: &COMPATIBLE_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("compatible", 2, args.len()));
        }

        let unit1_str = match args[0].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("compatible", "unit1", "Text", args[0].type_name())),
        };

        let unit2_str = match args[1].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("compatible", "unit2", "Text", args[1].type_name())),
        };

        let unit1 = match parse_unit(unit1_str) {
            Ok(u) => u,
            Err(e) => return Value::Error(conversion_error_to_folio(e)),
        };

        let unit2 = match parse_unit(unit2_str) {
            Ok(u) => u,
            Err(e) => return Value::Error(conversion_error_to_folio(e)),
        };

        Value::Bool(unit1.is_compatible(&unit2))
    }
}

// ============ quantity ============

pub struct QuantityFn;

static QUANTITY_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "value",
        typ: "Number",
        description: "Numeric value",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "unit",
        typ: "Text",
        description: "Unit string",
        optional: false,
        default: None,
    },
];

static QUANTITY_EXAMPLES: [&str; 2] = [
    "quantity(5, \"km\") → \"5 km\"",
    "quantity(3.14, \"rad\") → \"3.14 rad\"",
];

static QUANTITY_RELATED: [&str; 2] = ["extract_value", "extract_unit"];

impl FunctionPlugin for QuantityFn {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "quantity",
            description: "Create a quantity string from value and unit",
            usage: "quantity(value, unit)",
            args: &QUANTITY_ARGS,
            returns: "Text",
            examples: &QUANTITY_EXAMPLES,
            category: "units",
            source: None,
            related: &QUANTITY_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("quantity", 2, args.len()));
        }

        let value = match args[0].as_number() {
            Some(n) => n.clone(),
            None => return Value::Error(FolioError::arg_type("quantity", "value", "Number", args[0].type_name())),
        };

        let unit_str = match args[1].as_text() {
            Some(s) => s,
            None => return Value::Error(FolioError::arg_type("quantity", "unit", "Text", args[1].type_name())),
        };

        let unit = match parse_unit(unit_str) {
            Ok(u) => u,
            Err(e) => return Value::Error(conversion_error_to_folio(e)),
        };

        let quantity = Quantity::new(value, unit);
        Value::Text(format!("{}", quantity))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_convert_length() {
        let f = Convert;
        let args = vec![
            Value::Number(Number::from_i64(1)),
            Value::Text("km".to_string()),
            Value::Text("m".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        let value = result.as_number().unwrap();
        assert_eq!(*value, Number::from_i64(1000));
    }

    #[test]
    fn test_convert_temperature() {
        let f = Convert;
        let args = vec![
            Value::Number(Number::from_i64(0)),
            Value::Text("C".to_string()),
            Value::Text("K".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        let value = result.as_number().unwrap();
        let expected = Number::from_str("273.15").unwrap();
        assert_eq!(*value, expected);
    }

    #[test]
    fn test_to_base() {
        let f = ToBase;
        let args = vec![
            Value::Number(Number::from_i64(5)),
            Value::Text("km".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        let value = result.as_number().unwrap();
        assert_eq!(*value, Number::from_i64(5000));
    }

    #[test]
    fn test_in_units() {
        let f = InUnits;
        let args = vec![
            Value::Number(Number::from_i64(1000)),
            Value::Text("m->km".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        let value = result.as_number().unwrap();
        assert_eq!(*value, Number::from_i64(1));
    }

    #[test]
    fn test_extract_value() {
        let f = ExtractValue;
        let args = vec![Value::Text("5 km".to_string())];
        let result = f.call(&args, &eval_ctx());
        let value = result.as_number().unwrap();
        assert_eq!(*value, Number::from_i64(5));
    }

    #[test]
    fn test_extract_unit() {
        let f = ExtractUnit;
        let args = vec![Value::Text("5 km".to_string())];
        let result = f.call(&args, &eval_ctx());
        let text = result.as_text().unwrap();
        assert_eq!(text, "km");
    }

    #[test]
    fn test_dimensions() {
        let f = Dimensions;
        let args = vec![Value::Text("m/s".to_string())];
        let result = f.call(&args, &eval_ctx());
        let text = result.as_text().unwrap();
        assert_eq!(text, "L T^-1");
    }

    #[test]
    fn test_is_dimensionless() {
        let f = IsDimensionless;

        // rad is dimensionless
        let args = vec![Value::Text("rad".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_bool(), Some(true));

        // m is not dimensionless
        let args = vec![Value::Text("m".to_string())];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_compatible() {
        let f = Compatible;

        // km and mi are compatible (both length)
        let args = vec![
            Value::Text("km".to_string()),
            Value::Text("mi".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_bool(), Some(true));

        // m and s are not compatible
        let args = vec![
            Value::Text("m".to_string()),
            Value::Text("s".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        assert_eq!(result.as_bool(), Some(false));
    }

    #[test]
    fn test_quantity() {
        let f = QuantityFn;
        let args = vec![
            Value::Number(Number::from_i64(5)),
            Value::Text("km".to_string()),
        ];
        let result = f.call(&args, &eval_ctx());
        let text = result.as_text().unwrap();
        // Number displays with decimals, so check it contains the right parts
        assert!(text.contains("5"));
        assert!(text.contains("km"));
    }
}
