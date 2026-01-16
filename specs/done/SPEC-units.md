# Folio Unit Conversion Specification

## Overview

Physical quantities with dimensional analysis. Explicit syntax `value @ unit`, automatic conversion between compatible units, hard errors on dimensional mismatches. Preserves user's original unit form in display.

---

## Module Structure

```
folio-units/
├── Cargo.toml
└── src/
    ├── lib.rs           # Registration
    ├── quantity.rs      # Quantity type (value + unit)
    ├── unit.rs          # Unit representation
    ├── dimension.rs     # Dimensional analysis
    ├── convert.rs       # Conversion logic
    ├── parse.rs         # Unit string parsing
    ├── systems/
    │   ├── mod.rs
    │   ├── si.rs        # SI units
    │   ├── imperial.rs  # Imperial units
    │   └── derived.rs   # Derived units (N, J, W, etc.)
    └── temperature.rs   # Special handling for temperature
```

---

## Syntax

### Quantity Creation

```markdown
| distance | 100 @ km        |           |
| time     | 2 @ h           |           |
| speed    | distance / time | 50 @ km/h |
```

The `@` operator creates a `Quantity` from a number and unit string.

### Grammar Addition

```pest
quantity = { expression ~ "@" ~ unit_expr }
unit_expr = { unit_term ~ (("*" | "/") ~ unit_term)* }
unit_term = { unit_base ~ ("^" ~ integer)? }
unit_base = @{ ASCII_ALPHA+ }
```

Examples:
- `100 @ km`
- `9.8 @ m/s^2`
- `5 @ kg*m/s^2`

---

## Types

### Quantity

```rust
pub struct Quantity {
    value: Number,
    unit: Unit,
    display_unit: String,  // Preserve original form
}

impl Quantity {
    pub fn new(value: Number, unit_str: &str) -> Result<Self, FolioError>;
    pub fn convert_to(&self, target: &str) -> Result<Self, FolioError>;
    pub fn is_compatible(&self, other: &Quantity) -> bool;
}
```

### Unit

```rust
pub struct Unit {
    /// Dimension vector: [length, mass, time, current, temperature, amount, luminosity]
    dimensions: [i32; 7],
    
    /// Conversion factor to SI base
    to_si_factor: Number,
    
    /// Offset (for temperature)
    to_si_offset: Number,
}
```

### Dimension Indices

| Index | Dimension | SI Base |
|-------|-----------|---------|
| 0 | Length | m |
| 1 | Mass | kg |
| 2 | Time | s |
| 3 | Electric Current | A |
| 4 | Temperature | K |
| 5 | Amount | mol |
| 6 | Luminous Intensity | cd |

---

## Base Unit System

### Configuration

```markdown
## Calculation @base_unit:SI
```

or

```markdown
## Calculation @base_unit:imperial
```

Default: SI

### Behavior

When `@base_unit:SI`:
- Auto-conversion target: SI units
- `5 @ km + 3 @ m` → result in `m`
- `force / area` → result in `Pa`

When `@base_unit:imperial`:
- Auto-conversion target: imperial units
- `5 @ mi + 3 @ ft` → result in `ft`
- Display prefers imperial derived units

---

## Supported Units

### Length

| Unit | Symbol | SI Factor |
|------|--------|-----------|
| Meter | m | 1 |
| Kilometer | km | 1000 |
| Centimeter | cm | 0.01 |
| Millimeter | mm | 0.001 |
| Micrometer | um, μm | 1e-6 |
| Nanometer | nm | 1e-9 |
| Mile | mi | 1609.344 |
| Yard | yd | 0.9144 |
| Foot | ft | 0.3048 |
| Inch | in | 0.0254 |
| Nautical Mile | nmi | 1852 |
| Light Year | ly | 9.461e15 |
| Astronomical Unit | au | 1.496e11 |
| Angstrom | A, Å | 1e-10 |

### Mass

| Unit | Symbol | SI Factor (kg) |
|------|--------|----------------|
| Kilogram | kg | 1 |
| Gram | g | 0.001 |
| Milligram | mg | 1e-6 |
| Microgram | ug, μg | 1e-9 |
| Tonne | t | 1000 |
| Pound | lb | 0.453592 |
| Ounce | oz | 0.0283495 |
| Stone | st | 6.35029 |
| Slug | slug | 14.5939 |

### Time

| Unit | Symbol | SI Factor (s) |
|------|--------|---------------|
| Second | s | 1 |
| Millisecond | ms | 0.001 |
| Microsecond | us, μs | 1e-6 |
| Nanosecond | ns | 1e-9 |
| Minute | min | 60 |
| Hour | h | 3600 |
| Day | d | 86400 |
| Week | wk | 604800 |
| Year | yr | 31557600 (Julian) |

### Temperature

| Unit | Symbol | Conversion |
|------|--------|------------|
| Kelvin | K | base |
| Celsius | C, °C | K = C + 273.15 |
| Fahrenheit | F, °F | K = (F + 459.67) × 5/9 |
| Rankine | R, °R | K = R × 5/9 |

**Temperature is special** - see Temperature Handling section.

### Electric Current

| Unit | Symbol | SI Factor (A) |
|------|--------|---------------|
| Ampere | A | 1 |
| Milliampere | mA | 0.001 |
| Microampere | uA | 1e-6 |

### Amount of Substance

| Unit | Symbol | SI Factor (mol) |
|------|--------|-----------------|
| Mole | mol | 1 |
| Millimole | mmol | 0.001 |

### Derived Units - Mechanics

| Unit | Symbol | Dimensions | SI Equivalent |
|------|--------|------------|---------------|
| Newton | N | kg·m/s² | 1 |
| Dyne | dyn | g·cm/s² | 1e-5 N |
| Pound-force | lbf | - | 4.44822 N |
| Joule | J | kg·m²/s² | 1 |
| Calorie | cal | - | 4.184 J |
| Kilocalorie | kcal | - | 4184 J |
| BTU | BTU | - | 1055.06 J |
| Electron-volt | eV | - | 1.602e-19 J |
| Kilowatt-hour | kWh | - | 3.6e6 J |
| Watt | W | kg·m²/s³ | 1 |
| Horsepower | hp | - | 745.7 W |
| Pascal | Pa | kg/(m·s²) | 1 |
| Bar | bar | - | 1e5 Pa |
| Atmosphere | atm | - | 101325 Pa |
| PSI | psi | - | 6894.76 Pa |
| Torr | torr | - | 133.322 Pa |

### Derived Units - Other

| Unit | Symbol | Dimensions | SI Equivalent |
|------|--------|------------|---------------|
| Hertz | Hz | 1/s | 1 |
| Volt | V | kg·m²/(A·s³) | 1 |
| Ohm | Ω, ohm | kg·m²/(A²·s³) | 1 |
| Farad | F | A²·s⁴/(kg·m²) | 1 |
| Henry | H | kg·m²/(A²·s²) | 1 |
| Coulomb | C | A·s | 1 |
| Tesla | T | kg/(A·s²) | 1 |
| Weber | Wb | kg·m²/(A·s²) | 1 |
| Lumen | lm | cd·sr | 1 |
| Lux | lx | cd·sr/m² | 1 |

### Area

| Unit | Symbol | SI Factor (m²) |
|------|--------|----------------|
| Square meter | m^2 | 1 |
| Square kilometer | km^2 | 1e6 |
| Hectare | ha | 1e4 |
| Acre | acre | 4046.86 |
| Square foot | ft^2 | 0.0929 |
| Square mile | mi^2 | 2.59e6 |

### Volume

| Unit | Symbol | SI Factor (m³) |
|------|--------|----------------|
| Cubic meter | m^3 | 1 |
| Liter | L | 0.001 |
| Milliliter | mL | 1e-6 |
| Gallon (US) | gal | 0.00378541 |
| Gallon (UK) | gal_uk | 0.00454609 |
| Quart | qt | 0.000946353 |
| Pint | pt | 0.000473176 |
| Fluid ounce | fl_oz | 2.9574e-5 |
| Cubic foot | ft^3 | 0.0283168 |
| Cubic inch | in^3 | 1.6387e-5 |

### Speed

| Unit | Symbol | SI Factor (m/s) |
|------|--------|-----------------|
| Meters per second | m/s | 1 |
| Kilometers per hour | km/h | 0.277778 |
| Miles per hour | mph | 0.44704 |
| Knots | kn, knot | 0.514444 |
| Feet per second | ft/s | 0.3048 |
| Mach | mach | 340.29 (at sea level) |
| Speed of light | c | 299792458 |

---

## Functions

### Core

#### `convert(quantity, target_unit)`

Explicit unit conversion.

```markdown
| speed_kmh | 100 @ km/h             |              |
| speed_mph | convert(speed_kmh, "mph") | 62.137 @ mph |
```

Error if units incompatible.

#### `to_base(quantity)`

Convert to SI base units.

```markdown
| force   | 10 @ lbf         |                   |
| force_si | to_base(force)   | 44.482 @ kg*m/s^2 |
```

#### `simplify(quantity)`

Simplify to named derived unit if possible.

```markdown
| q     | 100 @ kg*m/s^2    |         |
| named | simplify(q)       | 100 @ N |
```

Only simplifies if exact match. Preserves original if no named unit matches.

#### `in_units(quantity, target)`

Alias for `convert()`.

```markdown
| dist_ft | in_units(100 @ m, "ft") | 328.084 @ ft |
```

### Inspection

#### `value(quantity)`

Extract numeric value.

```markdown
| v | value(100 @ km) | 100 |
```

#### `unit(quantity)`

Extract unit as text.

```markdown
| u | unit(100 @ km/h) | "km/h" |
```

#### `dimensions(quantity)`

Get dimension vector.

```markdown
| d | dimensions(100 @ N) | {length: 1, mass: 1, time: -2} |
```

#### `is_dimensionless(quantity)`

Check if quantity has no dimensions.

```markdown
| dl | is_dimensionless(5 @ rad) | true |
```

#### `compatible(q1, q2)`

Check if units are compatible (same dimensions).

```markdown
| ok | compatible(5 @ km, 3 @ mi) | true  |
| no | compatible(5 @ km, 3 @ kg) | false |
```

### Construction

#### `quantity(value, unit_string)`

Programmatic quantity creation.

```markdown
| q | quantity(100, "km/h") | 100 @ km/h |
```

Equivalent to `100 @ km/h` but useful when unit is dynamic.

---

## Arithmetic Rules

### Addition / Subtraction

Same dimensions required. Result in first operand's unit (or SI if `@base_unit:SI`).

```markdown
| a | 5 @ km          |          |
| b | 3000 @ m        |          |
| c | a + b           | 8 @ km   |  // Converts b to km first
```

```markdown
| a | 5 @ km + 3 @ m  | 5.003 @ km |
| b | 5 @ km + 3 @ kg | ERROR      |  // Dimensional mismatch
```

### Multiplication

Dimensions add.

```markdown
| force | 10 @ kg * 9.8 @ m/s^2 | 98 @ kg*m/s^2 |
```

### Division

Dimensions subtract.

```markdown
| speed | 100 @ km / 2 @ h | 50 @ km/h |
```

### Scalar Operations

Scalar preserves unit.

```markdown
| double | 2 * (50 @ km) | 100 @ km |
```

### Power

Dimensions multiply by exponent.

```markdown
| area | (5 @ m) ^ 2 | 25 @ m^2 |
```

Only integer exponents allowed for units.

---

## Temperature Handling

Temperature is special because Celsius and Fahrenheit have offsets.

### Absolute vs Difference

| Type | Example | Behavior |
|------|---------|----------|
| Absolute | `20 @ C` | Point on temperature scale |
| Difference | `5 @ C` in subtraction | Temperature change |

### Rules

#### Scalar Multiplication

```markdown
| t | 2 * (20 @ C) | 40 @ C |
```

Interpretation: 2 × 20 = 40, treated as absolute temperature.

**Rationale:** When an LLM writes `2 * 20 @ C`, they mean "40 degrees Celsius", not a thermodynamic calculation.

#### Addition of Temperatures

```markdown
| t1 | 20 @ C           |        |
| t2 | 5 @ C            |        |
| t3 | t1 + t2          | 25 @ C |
```

Both treated as absolute, arithmetic on values.

#### Temperature Conversion

```markdown
| c | 20 @ C               |             |
| f | convert(c, "F")      | 68 @ F      |
| k | convert(c, "K")      | 293.15 @ K  |
```

Uses proper offset conversion:
- C → K: K = C + 273.15
- C → F: F = C × 9/5 + 32
- F → C: C = (F - 32) × 5/9

#### Mixed Temperature Addition

```markdown
| t | 20 @ C + 5 @ K | ERROR |
```

Adding temperatures in different scales is an error (ambiguous intent).

Convert explicitly:
```markdown
| t | 20 @ C + convert(5 @ K, "C") | ... |
```

### Temperature Differences

For explicit temperature differences, use calculation:

```markdown
| t1   | 30 @ C           |          |
| t2   | 20 @ C           |          |
| diff | value(t1) - value(t2) | 10 |  // Pure number
| delta| diff @ C         | 10 @ C   |  // As temperature
```

---

## Display Rules

### Preserve Original Form

User/LLM's unit choice is preserved in display:

```markdown
| speed | 100 @ km/h           | 100 @ km/h     |  // Not converted to m/s
| force | 10 @ lbf             | 10 @ lbf       |  // Not converted to N
```

### Computed Results

Arithmetic results use:
1. First operand's unit (for +, -)
2. Composed unit (for ×, ÷)

```markdown
| a | 5 @ km + 3 @ m       | 5.003 @ km     |  // First operand's unit
| b | 10 @ kg * 2 @ m/s^2  | 20 @ kg*m/s^2  |  // Composed
```

### No Auto-Simplification

```markdown
| force | 10 @ kg * 9.8 @ m/s^2 | 98 @ kg*m/s^2 |  // NOT 98 @ N
```

Use `simplify()` explicitly:

```markdown
| force | simplify(10 @ kg * 9.8 @ m/s^2) | 98 @ N |
```

---

## Error Handling

### Dimensional Mismatch

```rust
fn add_quantities(a: &Quantity, b: &Quantity) -> Value {
    if a.unit.dimensions != b.unit.dimensions {
        return Value::Error(FolioError::domain_error(format!(
            "Cannot add {} and {}: incompatible dimensions",
            a.display_unit, b.display_unit
        )));
    }
    // ...
}
```

### Unknown Unit

```rust
fn parse_unit(s: &str) -> Result<Unit, FolioError> {
    match lookup_unit(s) {
        Some(u) => Ok(u),
        None => Err(FolioError::parse_error(format!(
            "Unknown unit: '{}'. Did you mean: {}?",
            s, suggest_similar(s)
        ))),
    }
}
```

### Invalid Power

```markdown
| bad | (5 @ m) ^ 0.5 | ERROR: Non-integer unit exponent |
```

---

## Implementation Notes

### Unit Parsing

```rust
fn parse_unit_expr(s: &str) -> Result<Unit, FolioError> {
    // "kg*m/s^2" → Unit { dimensions: [1, 1, -2, 0, 0, 0, 0], ... }
    
    let mut dims = [0i32; 7];
    let mut factor = Number::one();
    
    for term in parse_terms(s) {
        let base_unit = lookup_base_unit(&term.name)?;
        let exp = term.exponent;
        
        for i in 0..7 {
            dims[i] += base_unit.dimensions[i] * exp;
        }
        factor = factor.mul(&base_unit.to_si_factor.pow(exp));
    }
    
    Ok(Unit { dimensions: dims, to_si_factor: factor, to_si_offset: Number::zero() })
}
```

### Conversion

```rust
fn convert(q: &Quantity, target: &str) -> Result<Quantity, FolioError> {
    let target_unit = parse_unit_expr(target)?;
    
    if q.unit.dimensions != target_unit.dimensions {
        return Err(FolioError::domain_error("Incompatible dimensions"));
    }
    
    // Handle temperature offset
    let value_in_si = if q.unit.has_offset() {
        q.value.add(&q.unit.to_si_offset).mul(&q.unit.to_si_factor)
    } else {
        q.value.mul(&q.unit.to_si_factor)
    };
    
    let value_in_target = if target_unit.has_offset() {
        value_in_si.div(&target_unit.to_si_factor)?.sub(&target_unit.to_si_offset)
    } else {
        value_in_si.div(&target_unit.to_si_factor)?
    };
    
    Ok(Quantity {
        value: value_in_target,
        unit: target_unit,
        display_unit: target.to_string(),
    })
}
```

---

## Examples

### Physics Calculation

```markdown
## Mechanics @base_unit:SI

| Quantity     | Formula                        | Result           |
|--------------|--------------------------------|------------------|
| mass         | 75 @ kg                        |                  |
| gravity      | 9.81 @ m/s^2                   |                  |
| weight       | mass * gravity                 | 735.75 @ kg*m/s^2|
| weight_N     | simplify(weight)               | 735.75 @ N       |
| weight_lbf   | convert(weight_N, "lbf")       | 165.4 @ lbf      |
```

### Unit Conversion Table

```markdown
## Conversions

| From         | To                        | Result        |
|--------------|---------------------------|---------------|
| 100 @ km/h   | convert($1, "mph")        | 62.14 @ mph   |
| 1 @ atm      | convert($1, "psi")        | 14.70 @ psi   |
| 1 @ hp       | convert($1, "W")          | 745.7 @ W     |
| 100 @ C      | convert($1, "F")          | 212 @ F       |
```

### Energy Comparison

```markdown
## Energy

| Source       | Value                      | In Joules              |
|--------------|----------------------------|------------------------|
| food         | 2000 @ kcal                | convert(food, "J")     |
| battery      | 50 @ Wh                    | convert(battery, "J")  |
| gasoline_L   | 34.2 @ MJ                  | convert(gasoline_L, "J")|
| ratio        | value(food) / value(battery) | ...                  |
```

---

## Function Summary

| Category | Functions |
|----------|-----------|
| **Conversion** | `convert`, `to_base`, `simplify`, `in_units` |
| **Inspection** | `value`, `unit`, `dimensions`, `is_dimensionless`, `compatible` |
| **Construction** | `quantity` (and `@` operator) |

Total: 9 functions + `@` operator

Supported units: ~100 base and derived units across 7 SI dimensions.
