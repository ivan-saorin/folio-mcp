//! Cooking time and temperature adjustments
//!
//! Adjustments for altitude and convection ovens.

use folio_core::{FolioError, Number, Value};
use folio_plugin::{ArgMeta, EvalContext, FunctionMeta, FunctionPlugin};

use crate::helpers::extract_number;

// ============ altitude_time ============

pub struct AltitudeTime;

static ALTITUDE_TIME_ARGS: [ArgMeta; 2] = [
    ArgMeta::required("time_minutes", "Number", "Original cooking time in minutes (at sea level)"),
    ArgMeta::required("altitude_ft", "Number", "Altitude in feet above sea level"),
];

static ALTITUDE_TIME_EXAMPLES: [&str; 3] = [
    "altitude_time(60, 5000) -> 72",
    "altitude_time(30, 7500) -> 39",
    "altitude_time(45, 3000) -> 45",
];

static ALTITUDE_TIME_RELATED: [&str; 2] = ["convection_temp", "convection_time"];

impl FunctionPlugin for AltitudeTime {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "altitude_time",
            description: "Adjust baking time for high altitude (increases time above 3000ft)",
            usage: "altitude_time(time_minutes, altitude_ft)",
            args: &ALTITUDE_TIME_ARGS,
            returns: "Number",
            examples: &ALTITUDE_TIME_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &ALTITUDE_TIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("altitude_time", 2, args.len()));
        }

        let time = match extract_number(&args[0], "altitude_time", "time_minutes") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };
        let altitude = match extract_number(&args[1], "altitude_time", "altitude_ft") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        // General rule: add 5-8% per 1000 feet above 3000 feet
        // Using 6% per 1000 feet as a middle ground
        let three_thousand = Number::from_i64(3000);

        if altitude <= three_thousand {
            return Value::Number(time);
        }

        let excess_altitude = altitude.sub(&three_thousand);
        let thousand = Number::from_i64(1000);

        // Calculate percentage increase: (altitude - 3000) / 1000 * 0.06
        let percent_increase = match excess_altitude.checked_div(&thousand) {
            Ok(thousands) => {
                let factor = Number::from_f64(0.06);
                thousands.mul(&factor)
            }
            Err(e) => return Value::Error(e.into()),
        };

        // adjusted_time = time * (1 + percent_increase)
        let one = Number::from_i64(1);
        let multiplier = one.add(&percent_increase);
        Value::Number(time.mul(&multiplier))
    }
}

// ============ convection_temp ============

pub struct ConvectionTemp;

static CONVECTION_TEMP_ARGS: [ArgMeta; 1] = [
    ArgMeta::required("regular_temp", "Number", "Conventional oven temperature (Fahrenheit)"),
];

static CONVECTION_TEMP_EXAMPLES: [&str; 3] = [
    "convection_temp(350) -> 325",
    "convection_temp(400) -> 375",
    "convection_temp(425) -> 400",
];

static CONVECTION_TEMP_RELATED: [&str; 2] = ["convection_time", "altitude_time"];

impl FunctionPlugin for ConvectionTemp {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "convection_temp",
            description: "Convert conventional oven temperature to convection (reduces by 25F)",
            usage: "convection_temp(regular_temp)",
            args: &CONVECTION_TEMP_ARGS,
            returns: "Number",
            examples: &CONVECTION_TEMP_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &CONVECTION_TEMP_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("convection_temp", 1, 0));
        }

        let temp = match extract_number(&args[0], "convection_temp", "regular_temp") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        // Standard rule: reduce temperature by 25F for convection
        let twenty_five = Number::from_i64(25);
        Value::Number(temp.sub(&twenty_five))
    }
}

// ============ convection_time ============

pub struct ConvectionTime;

static CONVECTION_TIME_ARGS: [ArgMeta; 1] = [
    ArgMeta::required("regular_time", "Number", "Conventional oven time (minutes)"),
];

static CONVECTION_TIME_EXAMPLES: [&str; 3] = [
    "convection_time(60) -> 51",
    "convection_time(30) -> 25.5",
    "convection_time(45) -> 38.25",
];

static CONVECTION_TIME_RELATED: [&str; 2] = ["convection_temp", "altitude_time"];

impl FunctionPlugin for ConvectionTime {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "convection_time",
            description: "Convert conventional oven time to convection (reduces by 15%)",
            usage: "convection_time(regular_time)",
            args: &CONVECTION_TIME_ARGS,
            returns: "Number",
            examples: &CONVECTION_TIME_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &CONVECTION_TIME_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("convection_time", 1, 0));
        }

        let time = match extract_number(&args[0], "convection_time", "regular_time") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        // Standard rule: reduce time by 15% for convection
        let factor = Number::from_f64(0.85);
        Value::Number(time.mul(&factor))
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
    fn test_altitude_time_sea_level() {
        let alt = AltitudeTime;
        let result = alt.call(&[
            Value::Number(Number::from_i64(60)),
            Value::Number(Number::from_i64(0)),
        ], &ctx());

        if let Value::Number(n) = result {
            assert_eq!(n.to_i64(), Some(60));
        } else {
            panic!("Expected Number");
        }
    }

    #[test]
    fn test_altitude_time_5000ft() {
        let alt = AltitudeTime;
        let result = alt.call(&[
            Value::Number(Number::from_i64(60)),
            Value::Number(Number::from_i64(5000)),
        ], &ctx());

        if let Value::Number(n) = result {
            // At 5000ft, 2000ft above threshold, should be 60 * (1 + 0.12) = 67.2
            let val = n.to_f64().unwrap();
            assert!((val - 67.2).abs() < 0.1);
        } else {
            panic!("Expected Number");
        }
    }

    #[test]
    fn test_convection_temp() {
        let conv = ConvectionTemp;
        let result = conv.call(&[Value::Number(Number::from_i64(350))], &ctx());

        if let Value::Number(n) = result {
            assert_eq!(n.to_i64(), Some(325));
        } else {
            panic!("Expected Number");
        }
    }

    #[test]
    fn test_convection_time() {
        let conv = ConvectionTime;
        let result = conv.call(&[Value::Number(Number::from_i64(60))], &ctx());

        if let Value::Number(n) = result {
            assert_eq!(n.to_i64(), Some(51));
        } else {
            panic!("Expected Number");
        }
    }
}
