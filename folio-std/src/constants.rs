//! Mathematical and physical constants with sources

use folio_plugin::ConstantDef;

// ============================================================================
// Mathematical Constants
// ============================================================================

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

pub fn sqrt3() -> ConstantDef {
    ConstantDef {
        name: "sqrt3".to_string(),
        formula: "sqrt(3)".to_string(),
        source: "https://oeis.org/A002194".to_string(),
        category: "algebraic".to_string(),
    }
}

pub fn sqrt2() -> ConstantDef {
    ConstantDef {
        name: "sqrt2".to_string(),
        formula: "sqrt(2)".to_string(),
        source: "https://oeis.org/A002193".to_string(),
        category: "algebraic".to_string(),
    }
}

// ============================================================================
// Particle Masses (PDG 2024 / CODATA 2022)
// Values in MeV unless otherwise noted
// ============================================================================

pub fn m_e() -> ConstantDef {
    ConstantDef {
        name: "m_e".to_string(),
        formula: "0.51099895000".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - electron mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

pub fn m_mu() -> ConstantDef {
    ConstantDef {
        name: "m_μ".to_string(),
        formula: "105.6583755".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - muon mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

pub fn m_tau() -> ConstantDef {
    ConstantDef {
        name: "m_τ".to_string(),
        formula: "1776.86".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - tau mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

pub fn m_higgs() -> ConstantDef {
    ConstantDef {
        name: "m_H".to_string(),
        formula: "125350".to_string(),  // MeV (125.35 GeV)
        source: "CMS/ATLAS 2024 - Higgs boson mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

// ============================================================================
// CKM Matrix Elements (PDG 2024-2025)
// ============================================================================

pub fn v_us() -> ConstantDef {
    ConstantDef {
        name: "V_us".to_string(),
        formula: "0.2243".to_string(),
        source: "PDG 2024 - CKM element |V_us| from kaon decays".to_string(),
        category: "ckm".to_string(),
    }
}

pub fn v_cb() -> ConstantDef {
    ConstantDef {
        name: "V_cb".to_string(),
        formula: "0.04121".to_string(),
        source: "PDG 2025 / HFLAV - CKM element |V_cb|".to_string(),
        category: "ckm".to_string(),
    }
}

pub fn v_ub() -> ConstantDef {
    ConstantDef {
        name: "V_ub".to_string(),
        formula: "0.00382".to_string(),
        source: "PDG 2024 - CKM element |V_ub| inclusive+exclusive avg".to_string(),
        category: "ckm".to_string(),
    }
}

pub fn v_ts() -> ConstantDef {
    ConstantDef {
        name: "V_ts".to_string(),
        formula: "0.0411".to_string(),
        source: "PDG 2024 - CKM element |V_ts|".to_string(),
        category: "ckm".to_string(),
    }
}

// ============================================================================
// Fundamental Physical Constants
// ============================================================================

pub fn c() -> ConstantDef {
    ConstantDef {
        name: "c".to_string(),
        formula: "299792458".to_string(),  // m/s (exact)
        source: "CODATA 2022 - speed of light in vacuum (exact)".to_string(),
        category: "physical".to_string(),
    }
}

pub fn alpha() -> ConstantDef {
    ConstantDef {
        name: "α".to_string(),
        formula: "0.0072973525693".to_string(),  // fine-structure constant
        source: "CODATA 2022 - fine-structure constant".to_string(),
        category: "physical".to_string(),
    }
}

// ============================================================================
// ASCII Aliases for Unicode Constants
// These allow users to type "phi" instead of "φ", etc.
// ============================================================================

pub fn phi_ascii() -> ConstantDef {
    ConstantDef {
        name: "phi".to_string(),
        formula: "(1 + sqrt(5)) / 2".to_string(),
        source: "https://oeis.org/A001622".to_string(),
        category: "algebraic".to_string(),
    }
}

pub fn pi_ascii() -> ConstantDef {
    ConstantDef {
        name: "pi".to_string(),
        formula: "pi".to_string(),
        source: "https://oeis.org/A000796".to_string(),
        category: "transcendental".to_string(),
    }
}

pub fn alpha_ascii() -> ConstantDef {
    ConstantDef {
        name: "alpha".to_string(),
        formula: "0.0072973525693".to_string(),
        source: "CODATA 2022 - fine-structure constant".to_string(),
        category: "physical".to_string(),
    }
}

pub fn m_mu_ascii() -> ConstantDef {
    ConstantDef {
        name: "m_mu".to_string(),
        formula: "105.6583755".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - muon mass".to_string(),
        category: "particle_mass".to_string(),
    }
}

pub fn m_tau_ascii() -> ConstantDef {
    ConstantDef {
        name: "m_tau".to_string(),
        formula: "1776.86".to_string(),  // MeV
        source: "PDG 2024 / CODATA 2022 - tau mass".to_string(),
        category: "particle_mass".to_string(),
    }
}
