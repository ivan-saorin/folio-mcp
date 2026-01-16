//! Unit definitions - ~100 common units organized by category

use std::collections::HashMap;
use std::sync::LazyLock;
use folio_core::Number;
use crate::{Unit, Dimension};

/// Global unit registry
pub static UNITS: LazyLock<UnitRegistry> = LazyLock::new(|| UnitRegistry::new());

/// Registry of all known units
pub struct UnitRegistry {
    units: HashMap<String, Unit>,
    aliases: HashMap<String, String>,
}

impl UnitRegistry {
    pub fn new() -> Self {
        let mut registry = UnitRegistry {
            units: HashMap::new(),
            aliases: HashMap::new(),
        };
        registry.register_all_units();
        registry
    }

    /// Get a unit by symbol or alias
    pub fn get(&self, symbol: &str) -> Option<&Unit> {
        // Try direct lookup first
        if let Some(unit) = self.units.get(symbol) {
            return Some(unit);
        }
        // Try alias lookup
        if let Some(canonical) = self.aliases.get(symbol) {
            return self.units.get(canonical);
        }
        None
    }

    /// Get all units in a category
    pub fn by_category(&self, category: &str) -> Vec<&Unit> {
        self.units.values()
            .filter(|u| u.category == category)
            .collect()
    }

    /// Get all unit symbols
    pub fn symbols(&self) -> Vec<&str> {
        self.units.keys().map(|s| s.as_str()).collect()
    }

    fn register(&mut self, unit: Unit) {
        self.units.insert(unit.symbol.clone(), unit);
    }

    fn alias(&mut self, alias: &str, symbol: &str) {
        self.aliases.insert(alias.to_string(), symbol.to_string());
    }

    fn register_all_units(&mut self) {
        self.register_length_units();
        self.register_mass_units();
        self.register_time_units();
        self.register_temperature_units();
        self.register_current_units();
        self.register_amount_units();
        self.register_luminosity_units();
        self.register_area_units();
        self.register_volume_units();
        self.register_velocity_units();
        self.register_acceleration_units();
        self.register_force_units();
        self.register_energy_units();
        self.register_power_units();
        self.register_pressure_units();
        self.register_frequency_units();
        self.register_electrical_units();
        self.register_data_units();
        self.register_angle_units();
    }

    fn register_length_units(&mut self) {
        // SI length units
        self.register(Unit::new("m", "meter", Dimension::LENGTH, Number::from_i64(1), "length"));
        self.register(Unit::new("km", "kilometer", Dimension::LENGTH, Number::from_i64(1000), "length"));
        self.register(Unit::new("cm", "centimeter", Dimension::LENGTH, Number::from_str("0.01").unwrap(), "length"));
        self.register(Unit::new("mm", "millimeter", Dimension::LENGTH, Number::from_str("0.001").unwrap(), "length"));
        self.register(Unit::new("um", "micrometer", Dimension::LENGTH, Number::from_str("0.000001").unwrap(), "length"));
        self.register(Unit::new("nm", "nanometer", Dimension::LENGTH, Number::from_str("0.000000001").unwrap(), "length"));
        self.register(Unit::new("pm", "picometer", Dimension::LENGTH, Number::from_str("0.000000000001").unwrap(), "length"));

        // Imperial/US length units
        self.register(Unit::new("in", "inch", Dimension::LENGTH, Number::from_str("0.0254").unwrap(), "length"));
        self.register(Unit::new("ft", "foot", Dimension::LENGTH, Number::from_str("0.3048").unwrap(), "length"));
        self.register(Unit::new("yd", "yard", Dimension::LENGTH, Number::from_str("0.9144").unwrap(), "length"));
        self.register(Unit::new("mi", "mile", Dimension::LENGTH, Number::from_str("1609.344").unwrap(), "length"));
        self.register(Unit::new("nmi", "nautical mile", Dimension::LENGTH, Number::from_i64(1852), "length"));

        // Other length units
        self.register(Unit::new("au", "astronomical unit", Dimension::LENGTH, Number::from_str("149597870700").unwrap(), "length"));
        self.register(Unit::new("ly", "light year", Dimension::LENGTH, Number::from_str("9460730472580800").unwrap(), "length"));
        self.register(Unit::new("pc", "parsec", Dimension::LENGTH, Number::from_str("30856775814913673").unwrap(), "length"));

        // Aliases
        self.alias("meter", "m");
        self.alias("meters", "m");
        self.alias("metre", "m");
        self.alias("metres", "m");
        self.alias("kilometer", "km");
        self.alias("kilometers", "km");
        self.alias("kilometre", "km");
        self.alias("kilometres", "km");
        self.alias("centimeter", "cm");
        self.alias("centimeters", "cm");
        self.alias("millimeter", "mm");
        self.alias("millimeters", "mm");
        self.alias("inch", "in");
        self.alias("inches", "in");
        self.alias("foot", "ft");
        self.alias("feet", "ft");
        self.alias("yard", "yd");
        self.alias("yards", "yd");
        self.alias("mile", "mi");
        self.alias("miles", "mi");
        self.alias("μm", "um");
        self.alias("micron", "um");
        self.alias("microns", "um");
    }

    fn register_mass_units(&mut self) {
        // SI mass units
        self.register(Unit::new("kg", "kilogram", Dimension::MASS, Number::from_i64(1), "mass"));
        self.register(Unit::new("g", "gram", Dimension::MASS, Number::from_str("0.001").unwrap(), "mass"));
        self.register(Unit::new("mg", "milligram", Dimension::MASS, Number::from_str("0.000001").unwrap(), "mass"));
        self.register(Unit::new("ug", "microgram", Dimension::MASS, Number::from_str("0.000000001").unwrap(), "mass"));
        self.register(Unit::new("t", "tonne", Dimension::MASS, Number::from_i64(1000), "mass"));

        // Imperial/US mass units
        self.register(Unit::new("lb", "pound", Dimension::MASS, Number::from_str("0.45359237").unwrap(), "mass"));
        self.register(Unit::new("oz", "ounce", Dimension::MASS, Number::from_str("0.028349523125").unwrap(), "mass"));
        self.register(Unit::new("st", "stone", Dimension::MASS, Number::from_str("6.35029318").unwrap(), "mass"));
        self.register(Unit::new("ton", "short ton", Dimension::MASS, Number::from_str("907.18474").unwrap(), "mass"));
        self.register(Unit::new("lton", "long ton", Dimension::MASS, Number::from_str("1016.0469088").unwrap(), "mass"));

        // Other mass units
        self.register(Unit::new("ct", "carat", Dimension::MASS, Number::from_str("0.0002").unwrap(), "mass"));
        self.register(Unit::new("gr", "grain", Dimension::MASS, Number::from_str("0.00006479891").unwrap(), "mass"));

        // Aliases
        self.alias("kilogram", "kg");
        self.alias("kilograms", "kg");
        self.alias("gram", "g");
        self.alias("grams", "g");
        self.alias("milligram", "mg");
        self.alias("milligrams", "mg");
        self.alias("pound", "lb");
        self.alias("pounds", "lb");
        self.alias("lbs", "lb");
        self.alias("ounce", "oz");
        self.alias("ounces", "oz");
        self.alias("tonne", "t");
        self.alias("tonnes", "t");
        self.alias("metric ton", "t");
        self.alias("μg", "ug");
        self.alias("mcg", "ug");
    }

    fn register_time_units(&mut self) {
        self.register(Unit::new("s", "second", Dimension::TIME, Number::from_i64(1), "time"));
        self.register(Unit::new("ms", "millisecond", Dimension::TIME, Number::from_str("0.001").unwrap(), "time"));
        self.register(Unit::new("us", "microsecond", Dimension::TIME, Number::from_str("0.000001").unwrap(), "time"));
        self.register(Unit::new("ns", "nanosecond", Dimension::TIME, Number::from_str("0.000000001").unwrap(), "time"));
        self.register(Unit::new("min", "minute", Dimension::TIME, Number::from_i64(60), "time"));
        self.register(Unit::new("h", "hour", Dimension::TIME, Number::from_i64(3600), "time"));
        self.register(Unit::new("d", "day", Dimension::TIME, Number::from_i64(86400), "time"));
        self.register(Unit::new("wk", "week", Dimension::TIME, Number::from_i64(604800), "time"));
        self.register(Unit::new("mo", "month", Dimension::TIME, Number::from_str("2629746").unwrap(), "time")); // average
        self.register(Unit::new("yr", "year", Dimension::TIME, Number::from_str("31556952").unwrap(), "time")); // average

        // Aliases
        self.alias("second", "s");
        self.alias("seconds", "s");
        self.alias("sec", "s");
        self.alias("millisecond", "ms");
        self.alias("milliseconds", "ms");
        self.alias("microsecond", "us");
        self.alias("microseconds", "us");
        self.alias("μs", "us");
        self.alias("minute", "min");
        self.alias("minutes", "min");
        self.alias("hour", "h");
        self.alias("hours", "h");
        self.alias("hr", "h");
        self.alias("day", "d");
        self.alias("days", "d");
        self.alias("week", "wk");
        self.alias("weeks", "wk");
        self.alias("month", "mo");
        self.alias("months", "mo");
        self.alias("year", "yr");
        self.alias("years", "yr");
        self.alias("y", "yr");
    }

    fn register_temperature_units(&mut self) {
        // Kelvin is the SI base unit
        self.register(Unit::new("K", "kelvin", Dimension::TEMPERATURE, Number::from_i64(1), "temperature"));

        // Celsius: K = C + 273.15
        self.register(Unit::with_offset(
            "degC", "celsius", Dimension::TEMPERATURE,
            Number::from_i64(1),
            Number::from_str("273.15").unwrap(),
            "temperature"
        ));

        // Fahrenheit: K = (F - 32) * 5/9 + 273.15 = F * 5/9 + 255.372...
        // Actually: K = (F + 459.67) * 5/9
        self.register(Unit::with_offset(
            "degF", "fahrenheit", Dimension::TEMPERATURE,
            Number::from_str("0.5555555555555556").unwrap(), // 5/9
            Number::from_str("255.3722222222222").unwrap(),  // (459.67 * 5/9)
            "temperature"
        ));

        // Rankine: K = R * 5/9
        self.register(Unit::new("R", "rankine", Dimension::TEMPERATURE,
            Number::from_str("0.5555555555555556").unwrap(), "temperature"));

        // Aliases
        self.alias("kelvin", "K");
        self.alias("C", "degC");
        self.alias("celsius", "degC");
        self.alias("°C", "degC");
        self.alias("F", "degF");
        self.alias("fahrenheit", "degF");
        self.alias("°F", "degF");
        self.alias("rankine", "R");
        self.alias("°R", "R");
    }

    fn register_current_units(&mut self) {
        self.register(Unit::new("A", "ampere", Dimension::CURRENT, Number::from_i64(1), "current"));
        self.register(Unit::new("mA", "milliampere", Dimension::CURRENT, Number::from_str("0.001").unwrap(), "current"));
        self.register(Unit::new("uA", "microampere", Dimension::CURRENT, Number::from_str("0.000001").unwrap(), "current"));
        self.register(Unit::new("kA", "kiloampere", Dimension::CURRENT, Number::from_i64(1000), "current"));

        self.alias("ampere", "A");
        self.alias("amperes", "A");
        self.alias("amp", "A");
        self.alias("amps", "A");
        self.alias("μA", "uA");
    }

    fn register_amount_units(&mut self) {
        self.register(Unit::new("mol", "mole", Dimension::AMOUNT, Number::from_i64(1), "amount"));
        self.register(Unit::new("mmol", "millimole", Dimension::AMOUNT, Number::from_str("0.001").unwrap(), "amount"));
        self.register(Unit::new("umol", "micromole", Dimension::AMOUNT, Number::from_str("0.000001").unwrap(), "amount"));
        self.register(Unit::new("kmol", "kilomole", Dimension::AMOUNT, Number::from_i64(1000), "amount"));

        self.alias("mole", "mol");
        self.alias("moles", "mol");
        self.alias("μmol", "umol");
    }

    fn register_luminosity_units(&mut self) {
        self.register(Unit::new("cd", "candela", Dimension::LUMINOSITY, Number::from_i64(1), "luminosity"));
        self.register(Unit::new("lm", "lumen", Dimension::LUMINOSITY, Number::from_i64(1), "luminosity")); // cd·sr
        self.register(Unit::new("lx", "lux", Dimension::LUMINOSITY, Number::from_i64(1), "luminosity")); // lm/m²

        self.alias("candela", "cd");
        self.alias("lumen", "lm");
        self.alias("lumens", "lm");
        self.alias("lux", "lx");
    }

    fn register_area_units(&mut self) {
        self.register(Unit::new("m2", "square meter", Dimension::AREA, Number::from_i64(1), "area"));
        self.register(Unit::new("km2", "square kilometer", Dimension::AREA, Number::from_i64(1_000_000), "area"));
        self.register(Unit::new("cm2", "square centimeter", Dimension::AREA, Number::from_str("0.0001").unwrap(), "area"));
        self.register(Unit::new("mm2", "square millimeter", Dimension::AREA, Number::from_str("0.000001").unwrap(), "area"));
        self.register(Unit::new("ha", "hectare", Dimension::AREA, Number::from_i64(10000), "area"));
        self.register(Unit::new("ac", "acre", Dimension::AREA, Number::from_str("4046.8564224").unwrap(), "area"));
        self.register(Unit::new("ft2", "square foot", Dimension::AREA, Number::from_str("0.09290304").unwrap(), "area"));
        self.register(Unit::new("in2", "square inch", Dimension::AREA, Number::from_str("0.00064516").unwrap(), "area"));
        self.register(Unit::new("mi2", "square mile", Dimension::AREA, Number::from_str("2589988.110336").unwrap(), "area"));
        self.register(Unit::new("yd2", "square yard", Dimension::AREA, Number::from_str("0.83612736").unwrap(), "area"));

        self.alias("m²", "m2");
        self.alias("sq m", "m2");
        self.alias("sqm", "m2");
        self.alias("km²", "km2");
        self.alias("cm²", "cm2");
        self.alias("mm²", "mm2");
        self.alias("ft²", "ft2");
        self.alias("sq ft", "ft2");
        self.alias("sqft", "ft2");
        self.alias("in²", "in2");
        self.alias("mi²", "mi2");
        self.alias("yd²", "yd2");
        self.alias("hectare", "ha");
        self.alias("hectares", "ha");
        self.alias("acre", "ac");
        self.alias("acres", "ac");
    }

    fn register_volume_units(&mut self) {
        self.register(Unit::new("m3", "cubic meter", Dimension::VOLUME, Number::from_i64(1), "volume"));
        self.register(Unit::new("L", "liter", Dimension::VOLUME, Number::from_str("0.001").unwrap(), "volume"));
        self.register(Unit::new("dL", "deciliter", Dimension::VOLUME, Number::from_str("0.0001").unwrap(), "volume"));
        self.register(Unit::new("cL", "centiliter", Dimension::VOLUME, Number::from_str("0.00001").unwrap(), "volume"));
        self.register(Unit::new("mL", "milliliter", Dimension::VOLUME, Number::from_str("0.000001").unwrap(), "volume"));
        self.register(Unit::new("cm3", "cubic centimeter", Dimension::VOLUME, Number::from_str("0.000001").unwrap(), "volume"));
        self.register(Unit::new("mm3", "cubic millimeter", Dimension::VOLUME, Number::from_str("0.000000001").unwrap(), "volume"));

        // US fluid volumes
        self.register(Unit::new("gal", "gallon", Dimension::VOLUME, Number::from_str("0.003785411784").unwrap(), "volume"));
        self.register(Unit::new("qt", "quart", Dimension::VOLUME, Number::from_str("0.000946352946").unwrap(), "volume"));
        self.register(Unit::new("pt", "pint", Dimension::VOLUME, Number::from_str("0.000473176473").unwrap(), "volume"));
        self.register(Unit::new("cup", "cup", Dimension::VOLUME, Number::from_str("0.0002365882365").unwrap(), "volume"));
        self.register(Unit::new("floz", "fluid ounce", Dimension::VOLUME, Number::from_str("0.0000295735295625").unwrap(), "volume"));
        self.register(Unit::new("tbsp", "tablespoon", Dimension::VOLUME, Number::from_str("0.00001478676478125").unwrap(), "volume"));
        self.register(Unit::new("tsp", "teaspoon", Dimension::VOLUME, Number::from_str("0.00000492892159375").unwrap(), "volume"));

        // Imperial volumes
        self.register(Unit::new("impgal", "imperial gallon", Dimension::VOLUME, Number::from_str("0.00454609").unwrap(), "volume"));
        self.register(Unit::new("imppt", "imperial pint", Dimension::VOLUME, Number::from_str("0.00056826125").unwrap(), "volume"));

        self.register(Unit::new("ft3", "cubic foot", Dimension::VOLUME, Number::from_str("0.028316846592").unwrap(), "volume"));
        self.register(Unit::new("in3", "cubic inch", Dimension::VOLUME, Number::from_str("0.000016387064").unwrap(), "volume"));

        self.alias("m³", "m3");
        self.alias("liter", "L");
        self.alias("liters", "L");
        self.alias("litre", "L");
        self.alias("litres", "L");
        self.alias("l", "L");
        self.alias("milliliter", "mL");
        self.alias("milliliters", "mL");
        self.alias("ml", "mL");
        self.alias("deciliter", "dL");
        self.alias("deciliters", "dL");
        self.alias("dl", "dL");
        self.alias("centiliter", "cL");
        self.alias("centiliters", "cL");
        self.alias("cl", "cL");
        self.alias("cc", "cm3");
        self.alias("cm³", "cm3");
        self.alias("mm³", "mm3");
        self.alias("gallon", "gal");
        self.alias("gallons", "gal");
        self.alias("quart", "qt");
        self.alias("quarts", "qt");
        self.alias("pint", "pt");
        self.alias("pints", "pt");
        self.alias("ft³", "ft3");
        self.alias("in³", "in3");
    }

    fn register_velocity_units(&mut self) {
        self.register(Unit::new("m/s", "meter per second", Dimension::VELOCITY, Number::from_i64(1), "velocity"));
        self.register(Unit::new("km/h", "kilometer per hour", Dimension::VELOCITY, Number::from_str("0.2777777777777778").unwrap(), "velocity"));
        self.register(Unit::new("mph", "mile per hour", Dimension::VELOCITY, Number::from_str("0.44704").unwrap(), "velocity"));
        self.register(Unit::new("ft/s", "foot per second", Dimension::VELOCITY, Number::from_str("0.3048").unwrap(), "velocity"));
        self.register(Unit::new("kn", "knot", Dimension::VELOCITY, Number::from_str("0.5144444444444444").unwrap(), "velocity"));
        self.register(Unit::new("c", "speed of light", Dimension::VELOCITY, Number::from_i64(299792458), "velocity"));
        self.register(Unit::new("mach", "mach", Dimension::VELOCITY, Number::from_str("340.29").unwrap(), "velocity")); // at sea level

        self.alias("kph", "km/h");
        self.alias("kmh", "km/h");
        self.alias("kmph", "km/h");
        self.alias("knot", "kn");
        self.alias("knots", "kn");
        self.alias("fps", "ft/s");
    }

    fn register_acceleration_units(&mut self) {
        self.register(Unit::new("m/s2", "meter per second squared", Dimension::ACCELERATION, Number::from_i64(1), "acceleration"));
        self.register(Unit::new("g0", "standard gravity", Dimension::ACCELERATION, Number::from_str("9.80665").unwrap(), "acceleration"));
        self.register(Unit::new("ft/s2", "foot per second squared", Dimension::ACCELERATION, Number::from_str("0.3048").unwrap(), "acceleration"));
        self.register(Unit::new("Gal", "galileo", Dimension::ACCELERATION, Number::from_str("0.01").unwrap(), "acceleration"));

        self.alias("m/s²", "m/s2");
        self.alias("ft/s²", "ft/s2");
        self.alias("gee", "g0");
    }

    fn register_force_units(&mut self) {
        self.register(Unit::new("N", "newton", Dimension::FORCE, Number::from_i64(1), "force"));
        self.register(Unit::new("kN", "kilonewton", Dimension::FORCE, Number::from_i64(1000), "force"));
        self.register(Unit::new("mN", "millinewton", Dimension::FORCE, Number::from_str("0.001").unwrap(), "force"));
        self.register(Unit::new("dyn", "dyne", Dimension::FORCE, Number::from_str("0.00001").unwrap(), "force"));
        self.register(Unit::new("lbf", "pound-force", Dimension::FORCE, Number::from_str("4.4482216152605").unwrap(), "force"));
        self.register(Unit::new("kgf", "kilogram-force", Dimension::FORCE, Number::from_str("9.80665").unwrap(), "force"));

        self.alias("newton", "N");
        self.alias("newtons", "N");
    }

    fn register_energy_units(&mut self) {
        self.register(Unit::new("J", "joule", Dimension::ENERGY, Number::from_i64(1), "energy"));
        self.register(Unit::new("kJ", "kilojoule", Dimension::ENERGY, Number::from_i64(1000), "energy"));
        self.register(Unit::new("MJ", "megajoule", Dimension::ENERGY, Number::from_i64(1_000_000), "energy"));
        self.register(Unit::new("GJ", "gigajoule", Dimension::ENERGY, Number::from_i64(1_000_000_000), "energy"));
        self.register(Unit::new("mJ", "millijoule", Dimension::ENERGY, Number::from_str("0.001").unwrap(), "energy"));
        self.register(Unit::new("cal", "calorie", Dimension::ENERGY, Number::from_str("4.184").unwrap(), "energy"));
        self.register(Unit::new("kcal", "kilocalorie", Dimension::ENERGY, Number::from_str("4184").unwrap(), "energy"));
        self.register(Unit::new("Wh", "watt-hour", Dimension::ENERGY, Number::from_i64(3600), "energy"));
        self.register(Unit::new("kWh", "kilowatt-hour", Dimension::ENERGY, Number::from_i64(3_600_000), "energy"));
        self.register(Unit::new("eV", "electronvolt", Dimension::ENERGY, Number::from_str("1.602176634e-19").unwrap(), "energy"));
        self.register(Unit::new("BTU", "British thermal unit", Dimension::ENERGY, Number::from_str("1055.05585262").unwrap(), "energy"));
        self.register(Unit::new("erg", "erg", Dimension::ENERGY, Number::from_str("0.0000001").unwrap(), "energy"));
        self.register(Unit::new("ftlb", "foot-pound", Dimension::ENERGY, Number::from_str("1.3558179483314").unwrap(), "energy"));

        self.alias("joule", "J");
        self.alias("joules", "J");
        self.alias("calorie", "cal");
        self.alias("calories", "cal");
        self.alias("kilocalorie", "kcal");
        self.alias("kilocalories", "kcal");
        self.alias("Cal", "kcal");
        self.alias("Calorie", "kcal");
        self.alias("Calories", "kcal");
    }

    fn register_power_units(&mut self) {
        self.register(Unit::new("W", "watt", Dimension::POWER, Number::from_i64(1), "power"));
        self.register(Unit::new("kW", "kilowatt", Dimension::POWER, Number::from_i64(1000), "power"));
        self.register(Unit::new("MW", "megawatt", Dimension::POWER, Number::from_i64(1_000_000), "power"));
        self.register(Unit::new("GW", "gigawatt", Dimension::POWER, Number::from_i64(1_000_000_000), "power"));
        self.register(Unit::new("mW", "milliwatt", Dimension::POWER, Number::from_str("0.001").unwrap(), "power"));
        self.register(Unit::new("uW", "microwatt", Dimension::POWER, Number::from_str("0.000001").unwrap(), "power"));
        self.register(Unit::new("hp", "horsepower", Dimension::POWER, Number::from_str("745.699872").unwrap(), "power"));
        self.register(Unit::new("PS", "metric horsepower", Dimension::POWER, Number::from_str("735.49875").unwrap(), "power"));

        self.alias("watt", "W");
        self.alias("watts", "W");
        self.alias("horsepower", "hp");
        self.alias("μW", "uW");
    }

    fn register_pressure_units(&mut self) {
        self.register(Unit::new("Pa", "pascal", Dimension::PRESSURE, Number::from_i64(1), "pressure"));
        self.register(Unit::new("kPa", "kilopascal", Dimension::PRESSURE, Number::from_i64(1000), "pressure"));
        self.register(Unit::new("MPa", "megapascal", Dimension::PRESSURE, Number::from_i64(1_000_000), "pressure"));
        self.register(Unit::new("hPa", "hectopascal", Dimension::PRESSURE, Number::from_i64(100), "pressure"));
        self.register(Unit::new("bar", "bar", Dimension::PRESSURE, Number::from_i64(100000), "pressure"));
        self.register(Unit::new("mbar", "millibar", Dimension::PRESSURE, Number::from_i64(100), "pressure"));
        self.register(Unit::new("atm", "atmosphere", Dimension::PRESSURE, Number::from_i64(101325), "pressure"));
        self.register(Unit::new("psi", "pounds per square inch", Dimension::PRESSURE, Number::from_str("6894.757293168").unwrap(), "pressure"));
        self.register(Unit::new("mmHg", "millimeter of mercury", Dimension::PRESSURE, Number::from_str("133.322387415").unwrap(), "pressure"));
        self.register(Unit::new("torr", "torr", Dimension::PRESSURE, Number::from_str("133.322368421").unwrap(), "pressure"));
        self.register(Unit::new("inHg", "inch of mercury", Dimension::PRESSURE, Number::from_str("3386.389").unwrap(), "pressure"));

        self.alias("pascal", "Pa");
        self.alias("pascals", "Pa");
        self.alias("atmosphere", "atm");
        self.alias("atmospheres", "atm");
    }

    fn register_frequency_units(&mut self) {
        self.register(Unit::new("Hz", "hertz", Dimension::FREQUENCY, Number::from_i64(1), "frequency"));
        self.register(Unit::new("kHz", "kilohertz", Dimension::FREQUENCY, Number::from_i64(1000), "frequency"));
        self.register(Unit::new("MHz", "megahertz", Dimension::FREQUENCY, Number::from_i64(1_000_000), "frequency"));
        self.register(Unit::new("GHz", "gigahertz", Dimension::FREQUENCY, Number::from_i64(1_000_000_000), "frequency"));
        self.register(Unit::new("THz", "terahertz", Dimension::FREQUENCY, Number::from_i64(1_000_000_000_000), "frequency"));
        self.register(Unit::new("rpm", "revolutions per minute", Dimension::FREQUENCY, Number::from_str("0.0166666666666667").unwrap(), "frequency"));

        self.alias("hertz", "Hz");
    }

    fn register_electrical_units(&mut self) {
        // Voltage
        self.register(Unit::new("V", "volt", Dimension::VOLTAGE, Number::from_i64(1), "electrical"));
        self.register(Unit::new("mV", "millivolt", Dimension::VOLTAGE, Number::from_str("0.001").unwrap(), "electrical"));
        self.register(Unit::new("kV", "kilovolt", Dimension::VOLTAGE, Number::from_i64(1000), "electrical"));
        self.register(Unit::new("MV", "megavolt", Dimension::VOLTAGE, Number::from_i64(1_000_000), "electrical"));

        // Resistance
        self.register(Unit::new("ohm", "ohm", Dimension::RESISTANCE, Number::from_i64(1), "electrical"));
        self.register(Unit::new("kohm", "kiloohm", Dimension::RESISTANCE, Number::from_i64(1000), "electrical"));
        self.register(Unit::new("Mohm", "megaohm", Dimension::RESISTANCE, Number::from_i64(1_000_000), "electrical"));
        self.register(Unit::new("mohm", "milliohm", Dimension::RESISTANCE, Number::from_str("0.001").unwrap(), "electrical"));

        // Charge - Use "Coul" as primary symbol to avoid conflict with Celsius "C"
        self.register(Unit::new("Coul", "coulomb", Dimension::CHARGE, Number::from_i64(1), "electrical"));
        self.register(Unit::new("mCoul", "millicoulomb", Dimension::CHARGE, Number::from_str("0.001").unwrap(), "electrical"));
        self.register(Unit::new("uCoul", "microcoulomb", Dimension::CHARGE, Number::from_str("0.000001").unwrap(), "electrical"));
        self.register(Unit::new("Ah", "ampere-hour", Dimension::CHARGE, Number::from_i64(3600), "electrical"));
        self.register(Unit::new("mAh", "milliampere-hour", Dimension::CHARGE, Number::from_str("3.6").unwrap(), "electrical"));

        self.alias("volt", "V");
        self.alias("volts", "V");
        self.alias("Ω", "ohm");
        self.alias("ohms", "ohm");
        self.alias("kΩ", "kohm");
        self.alias("MΩ", "Mohm");
        self.alias("mΩ", "mohm");
        self.alias("coulomb", "Coul");
        self.alias("coulombs", "Coul");
        self.alias("mC", "mCoul");
        self.alias("uC", "uCoul");
        self.alias("μC", "uCoul");
    }

    fn register_data_units(&mut self) {
        // Use dimensionless for data since it's not a physical quantity
        self.register(Unit::new("bit", "bit", Dimension::DIMENSIONLESS, Number::from_i64(1), "data"));
        self.register(Unit::new("byte", "byte", Dimension::DIMENSIONLESS, Number::from_i64(8), "data"));
        self.register(Unit::new("kB", "kilobyte", Dimension::DIMENSIONLESS, Number::from_i64(8000), "data"));
        self.register(Unit::new("MB", "megabyte", Dimension::DIMENSIONLESS, Number::from_i64(8_000_000), "data"));
        self.register(Unit::new("GB", "gigabyte", Dimension::DIMENSIONLESS, Number::from_i64(8_000_000_000), "data"));
        self.register(Unit::new("TB", "terabyte", Dimension::DIMENSIONLESS, Number::from_i64(8_000_000_000_000), "data"));

        // Binary units (IEC)
        self.register(Unit::new("KiB", "kibibyte", Dimension::DIMENSIONLESS, Number::from_i64(8 * 1024), "data"));
        self.register(Unit::new("MiB", "mebibyte", Dimension::DIMENSIONLESS, Number::from_i64(8 * 1024 * 1024), "data"));
        self.register(Unit::new("GiB", "gibibyte", Dimension::DIMENSIONLESS, Number::from_i64(8 * 1024 * 1024 * 1024), "data"));
        self.register(Unit::new("TiB", "tebibyte", Dimension::DIMENSIONLESS, Number::from_str(&(8i64 * 1024 * 1024 * 1024 * 1024).to_string()).unwrap(), "data"));

        // Data rate
        self.register(Unit::new("bps", "bits per second", Dimension::DIMENSIONLESS, Number::from_i64(1), "data_rate"));
        self.register(Unit::new("kbps", "kilobits per second", Dimension::DIMENSIONLESS, Number::from_i64(1000), "data_rate"));
        self.register(Unit::new("Mbps", "megabits per second", Dimension::DIMENSIONLESS, Number::from_i64(1_000_000), "data_rate"));
        self.register(Unit::new("Gbps", "gigabits per second", Dimension::DIMENSIONLESS, Number::from_i64(1_000_000_000), "data_rate"));

        self.alias("bits", "bit");
        self.alias("bytes", "byte");
        self.alias("B", "byte");
        self.alias("kilobyte", "kB");
        self.alias("kilobytes", "kB");
        self.alias("megabyte", "MB");
        self.alias("megabytes", "MB");
        self.alias("gigabyte", "GB");
        self.alias("gigabytes", "GB");
        self.alias("terabyte", "TB");
        self.alias("terabytes", "TB");
    }

    fn register_angle_units(&mut self) {
        // Angles are dimensionless
        self.register(Unit::new("rad", "radian", Dimension::DIMENSIONLESS, Number::from_i64(1), "angle"));
        self.register(Unit::new("deg", "degree", Dimension::DIMENSIONLESS, Number::from_str("0.017453292519943295").unwrap(), "angle")); // pi/180
        self.register(Unit::new("grad", "gradian", Dimension::DIMENSIONLESS, Number::from_str("0.015707963267948967").unwrap(), "angle")); // pi/200
        self.register(Unit::new("arcmin", "arcminute", Dimension::DIMENSIONLESS, Number::from_str("0.0002908882086657216").unwrap(), "angle")); // pi/10800
        self.register(Unit::new("arcsec", "arcsecond", Dimension::DIMENSIONLESS, Number::from_str("0.000004848136811095360").unwrap(), "angle")); // pi/648000
        self.register(Unit::new("turn", "turn", Dimension::DIMENSIONLESS, Number::from_str("6.283185307179586").unwrap(), "angle")); // 2*pi

        self.alias("radian", "rad");
        self.alias("radians", "rad");
        self.alias("degree", "deg");
        self.alias("degrees", "deg");
        self.alias("°", "deg");
        self.alias("gradian", "grad");
        self.alias("gradians", "grad");
        self.alias("gon", "grad");
        self.alias("'", "arcmin");
        self.alias("\"", "arcsec");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_registry() {
        let reg = UnitRegistry::new();

        // Test basic lookup
        assert!(reg.get("m").is_some());
        assert!(reg.get("kg").is_some());
        assert!(reg.get("s").is_some());

        // Test alias lookup
        assert!(reg.get("meter").is_some());
        assert!(reg.get("meters").is_some());
        assert!(reg.get("kilogram").is_some());

        // Test unknown unit
        assert!(reg.get("unknown_xyz").is_none());
    }

    #[test]
    fn test_length_conversions() {
        let reg = UnitRegistry::new();

        let m = reg.get("m").unwrap();
        let km = reg.get("km").unwrap();

        // 1 km = 1000 m
        let value = Number::from_i64(1);
        let in_meters = km.to_si(&value);
        assert_eq!(in_meters, Number::from_i64(1000));
    }

    #[test]
    fn test_temperature_conversions() {
        let reg = UnitRegistry::new();

        let c = reg.get("C").unwrap();
        let k = reg.get("K").unwrap();

        // 0 C = 273.15 K
        let zero_c = Number::from_i64(0);
        let in_kelvin = c.to_si(&zero_c);
        let expected = Number::from_str("273.15").unwrap();
        assert_eq!(in_kelvin, expected);
    }

    #[test]
    fn test_by_category() {
        let reg = UnitRegistry::new();

        let length_units = reg.by_category("length");
        assert!(length_units.len() > 5);

        // All should have LENGTH dimension
        for unit in length_units {
            assert_eq!(unit.dimension, Dimension::LENGTH);
        }
    }
}
