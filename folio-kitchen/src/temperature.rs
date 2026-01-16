//! Kitchen temperature functions
//!
//! Provides oven temperature conversions including gas marks and named temperatures.
//! For basic F↔C conversions, use folio-units convert() function.

use folio_core::{FolioError, Number, Value};
use folio_plugin::{ArgMeta, EvalContext, FunctionMeta, FunctionPlugin};
use std::collections::HashMap;

use crate::helpers::{extract_number, extract_text};

// ============ oven_temp ============

pub struct OvenTemp;

static OVEN_TEMP_ARGS: [ArgMeta; 1] = [
    ArgMeta::required("description", "Text",
        "Named temperature: \"cool\", \"very slow\", \"slow\", \"warm\", \"moderate\", \"medium\", \"moderately hot\", \"hot\", \"very hot\", \"extremely hot\""),
];

static OVEN_TEMP_EXAMPLES: [&str; 3] = [
    "oven_temp(\"moderate\") -> {f: 350, c: 177, gas: 4}",
    "oven_temp(\"hot\") -> {f: 425, c: 218, gas: 7}",
    "oven_temp(\"slow\") -> {f: 300, c: 149, gas: 2}",
];

static OVEN_TEMP_RELATED: [&str; 2] = ["gas_mark", "gas_mark_from_temp"];

impl FunctionPlugin for OvenTemp {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "oven_temp",
            description: "Get oven temperature from descriptive name",
            usage: "oven_temp(description)",
            args: &OVEN_TEMP_ARGS,
            returns: "Object {f: Number, c: Number, gas: Number}",
            examples: &OVEN_TEMP_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &OVEN_TEMP_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("oven_temp", 1, 0));
        }

        let desc = match extract_text(&args[0], "oven_temp", "description") {
            Ok(s) => s.to_lowercase(),
            Err(e) => return Value::Error(e),
        };

        // Return (F, C, Gas Mark)
        let (f, c, gas): (i64, i64, f64) = match desc.as_str() {
            "cool" | "very cool" => (200, 93, 0.25),
            "very slow" | "very low" => (250, 121, 0.5),
            "slow" | "low" => (300, 149, 2.0),
            "warm" => (325, 163, 3.0),
            "moderate" | "medium" => (350, 177, 4.0),
            "moderately hot" => (375, 191, 5.0),
            "fairly hot" => (400, 204, 6.0),
            "hot" => (425, 218, 7.0),
            "very hot" => (450, 232, 8.0),
            "extremely hot" | "broil" => (475, 246, 9.0),
            _ => return Value::Error(FolioError::domain_error(format!(
                "oven_temp: Unknown description '{}'. Valid: cool, very slow, slow, warm, moderate, moderately hot, fairly hot, hot, very hot, extremely hot",
                desc
            ))),
        };

        let mut obj = HashMap::new();
        obj.insert("f".to_string(), Value::Number(Number::from_i64(f)));
        obj.insert("c".to_string(), Value::Number(Number::from_i64(c)));
        obj.insert("gas".to_string(), Value::Number(Number::from_f64(gas)));
        Value::Object(obj)
    }
}

// ============ gas_mark ============

pub struct GasMark;

static GAS_MARK_ARGS: [ArgMeta; 1] = [
    ArgMeta::required("mark", "Number", "UK gas mark (0.25 to 10)"),
];

static GAS_MARK_EXAMPLES: [&str; 3] = [
    "gas_mark(4) -> {f: 350, c: 177}",
    "gas_mark(6) -> {f: 400, c: 204}",
    "gas_mark(8) -> {f: 450, c: 232}",
];

static GAS_MARK_RELATED: [&str; 2] = ["oven_temp", "gas_mark_from_temp"];

impl FunctionPlugin for GasMark {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "gas_mark",
            description: "Convert UK gas mark to temperature",
            usage: "gas_mark(mark)",
            args: &GAS_MARK_ARGS,
            returns: "Object {f: Number, c: Number}",
            examples: &GAS_MARK_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &GAS_MARK_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("gas_mark", 1, 0));
        }

        let mark = match extract_number(&args[0], "gas_mark", "mark") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        // Gas Mark formula: F = (mark * 25) + 250
        // This gives: GM 1 = 275F, GM 4 = 350F, GM 6 = 400F, etc.
        let twenty_five = Number::from_i64(25);
        let two_fifty = Number::from_i64(250);
        let f = mark.mul(&twenty_five).add(&two_fifty);

        // Convert to C: C = (F - 32) * 5/9
        let five = Number::from_i64(5);
        let nine = Number::from_i64(9);
        let thirty_two = Number::from_i64(32);

        let c = match f.sub(&thirty_two).mul(&five).checked_div(&nine) {
            Ok(c) => c,
            Err(e) => return Value::Error(e.into()),
        };

        let mut obj = HashMap::new();
        obj.insert("f".to_string(), Value::Number(f));
        obj.insert("c".to_string(), Value::Number(c));
        Value::Object(obj)
    }
}

// ============ gas_mark_from_temp ============

pub struct GasMarkFromTemp;

static GAS_MARK_FROM_TEMP_ARGS: [ArgMeta; 2] = [
    ArgMeta::required("temp", "Number", "Temperature value"),
    ArgMeta::optional("unit", "Text", "Temperature unit: \"F\" (default) or \"C\"", "F"),
];

static GAS_MARK_FROM_TEMP_EXAMPLES: [&str; 3] = [
    "gas_mark_from_temp(350) -> 4",
    "gas_mark_from_temp(180, \"C\") -> 4",
    "gas_mark_from_temp(425, \"F\") -> 7",
];

static GAS_MARK_FROM_TEMP_RELATED: [&str; 2] = ["gas_mark", "oven_temp"];

impl FunctionPlugin for GasMarkFromTemp {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "gas_mark_from_temp",
            description: "Convert temperature to nearest UK gas mark",
            usage: "gas_mark_from_temp(temp, [unit])",
            args: &GAS_MARK_FROM_TEMP_ARGS,
            returns: "Number",
            examples: &GAS_MARK_FROM_TEMP_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &GAS_MARK_FROM_TEMP_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("gas_mark_from_temp", 1, 0));
        }

        let temp = match extract_number(&args[0], "gas_mark_from_temp", "temp") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let unit = args.get(1)
            .and_then(|v| match v {
                Value::Text(s) => Some(s.to_uppercase()),
                _ => None,
            })
            .unwrap_or_else(|| "F".to_string());

        // Convert to Fahrenheit if needed
        let f_temp = match unit.as_str() {
            "F" | "FAHRENHEIT" => temp,
            "C" | "CELSIUS" => {
                // F = C * 9/5 + 32
                let nine = Number::from_i64(9);
                let five = Number::from_i64(5);
                let thirty_two = Number::from_i64(32);
                match temp.mul(&nine).checked_div(&five) {
                    Ok(r) => r.add(&thirty_two),
                    Err(e) => return Value::Error(e.into()),
                }
            }
            _ => return Value::Error(FolioError::domain_error(format!(
                "gas_mark_from_temp: Unknown unit '{}'. Use \"F\" or \"C\".",
                unit
            ))),
        };

        // Gas Mark = (F - 250) / 25
        let two_fifty = Number::from_i64(250);
        let twenty_five = Number::from_i64(25);

        match f_temp.sub(&two_fifty).checked_div(&twenty_five) {
            Ok(mark) => Value::Number(mark),
            Err(e) => Value::Error(e.into()),
        }
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
    fn test_oven_temp_moderate() {
        let oven = OvenTemp;
        let result = oven.call(&[Value::Text("moderate".to_string())], &ctx());

        if let Value::Object(obj) = result {
            assert_eq!(obj.get("f").unwrap().as_number().unwrap().to_i64(), Some(350));
            assert_eq!(obj.get("c").unwrap().as_number().unwrap().to_i64(), Some(177));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_gas_mark_4() {
        let gas = GasMark;
        let result = gas.call(&[Value::Number(Number::from_i64(4))], &ctx());

        if let Value::Object(obj) = result {
            assert_eq!(obj.get("f").unwrap().as_number().unwrap().to_i64(), Some(350));
        } else {
            panic!("Expected Object");
        }
    }

    #[test]
    fn test_gas_mark_from_temp_350f() {
        let func = GasMarkFromTemp;
        let result = func.call(&[Value::Number(Number::from_i64(350))], &ctx());

        if let Value::Number(n) = result {
            assert_eq!(n.to_i64(), Some(4));
        } else {
            panic!("Expected Number");
        }
    }
}
