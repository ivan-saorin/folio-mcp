//! Mathematical constants with sources

use folio_plugin::ConstantDef;

pub fn phi() -> ConstantDef {
    ConstantDef {
        name: "φ".to_string(),
        formula: "(1 + sqrt(5)) / 2".to_string(),
        source: "https://oeis.org/A001622".to_string(),
        category: "algebraic".to_string(),
    }
}

pub fn pi() -> ConstantDef {
    ConstantDef {
        name: "π".to_string(),
        formula: "pi".to_string(),
        source: "https://oeis.org/A000796".to_string(),
        category: "transcendental".to_string(),
    }
}

pub fn e() -> ConstantDef {
    ConstantDef {
        name: "e".to_string(),
        formula: "exp(1)".to_string(),
        source: "https://oeis.org/A001113".to_string(),
        category: "transcendental".to_string(),
    }
}
