//! Folio Standard Library

pub mod functions;
pub mod analyzers;
pub mod commands;
pub mod constants;

use folio_plugin::PluginRegistry;

/// Load standard library into registry
pub fn load_standard_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Math functions
        .with_function(functions::Sqrt)
        .with_function(functions::Ln)
        .with_function(functions::Exp)
        .with_function(functions::Pow)
        .with_function(functions::Abs)
        .with_function(functions::Sin)
        .with_function(functions::Cos)
        .with_function(functions::Tan)
        .with_function(functions::Sum)
        .with_function(functions::Round)
        .with_function(functions::Floor)
        .with_function(functions::Ceil)
        // DateTime functions - Construction
        .with_function(functions::DateFn)
        .with_function(functions::TimeFn)
        .with_function(functions::DateTimeFn)
        .with_function(functions::NowFn)
        // DateTime functions - Parsing
        .with_function(functions::ParseDateFn)
        .with_function(functions::ParseTimeFn)
        // DateTime functions - Extraction
        .with_function(functions::YearFn)
        .with_function(functions::MonthFn)
        .with_function(functions::DayFn)
        .with_function(functions::HourFn)
        .with_function(functions::MinuteFn)
        .with_function(functions::SecondFn)
        .with_function(functions::WeekdayFn)
        .with_function(functions::DayOfYearFn)
        .with_function(functions::WeekFn)
        // DateTime functions - Formatting
        .with_function(functions::FormatDateFn)
        .with_function(functions::FormatTimeFn)
        .with_function(functions::FormatDateTimeFn)
        // DateTime functions - Duration construction
        .with_function(functions::WeeksDur)
        .with_function(functions::DaysDur)
        .with_function(functions::HoursDur)
        .with_function(functions::MinutesDur)
        .with_function(functions::SecondsDur)
        .with_function(functions::MillisecondsDur)
        // DateTime functions - Arithmetic
        .with_function(functions::AddDaysFn)
        .with_function(functions::AddMonthsFn)
        .with_function(functions::AddYearsFn)
        .with_function(functions::DiffFn)
        // DateTime functions - Comparison
        .with_function(functions::IsBeforeFn)
        .with_function(functions::IsAfterFn)
        .with_function(functions::IsSameDayFn)
        // DateTime functions - Utilities
        .with_function(functions::StartOfDayFn)
        .with_function(functions::EndOfDayFn)
        .with_function(functions::StartOfMonthFn)
        .with_function(functions::StartOfYearFn)
        // DateTime shortcuts - End of period
        .with_function(functions::EodFn)
        .with_function(functions::EowFn)
        .with_function(functions::EomFn)
        .with_function(functions::EoqFn)
        .with_function(functions::EoyFn)
        // DateTime shortcuts - Start of period
        .with_function(functions::SodFn)
        .with_function(functions::SowFn)
        .with_function(functions::SomFn)
        .with_function(functions::SoqFn)
        .with_function(functions::SoyFn)
        // DateTime shortcuts - Navigation
        .with_function(functions::TomorrowFn)
        .with_function(functions::NextWeekFn)
        .with_function(functions::NextMonthFn)
        .with_function(functions::NextMonthWdFn)
        // Workday functions
        .with_function(functions::IsWorkdayFn)
        .with_function(functions::NextWorkdayFn)
        .with_function(functions::PrevWorkdayFn)
        .with_function(functions::AddWorkdaysFn)
        // Utility functions (LLM experience)
        .with_function(functions::FieldsFn)
        .with_function(functions::HeadFn)
        .with_function(functions::TailFn)
        .with_function(functions::TakeFn)
        .with_function(functions::TypeofFn)
        .with_function(functions::DescribeFn)
        .with_function(functions::LenFn)
        .with_function(functions::NthFn)
        // Analyzers
        .with_analyzer(analyzers::PhiAnalyzer)
        .with_analyzer(analyzers::PiAnalyzer)
        .with_analyzer(analyzers::EAnalyzer)
        .with_command(commands::Trace)
        .with_command(commands::Explain)
        // Mathematical constants
        .with_constant(constants::phi())
        .with_constant(constants::pi())
        .with_constant(constants::e())
        .with_constant(constants::sqrt2())
        .with_constant(constants::sqrt3())
        // Particle masses (MeV)
        .with_constant(constants::m_e())
        .with_constant(constants::m_mu())
        .with_constant(constants::m_tau())
        .with_constant(constants::m_higgs())
        // CKM matrix elements
        .with_constant(constants::v_us())
        .with_constant(constants::v_cb())
        .with_constant(constants::v_ub())
        .with_constant(constants::v_ts())
        // Physical constants
        .with_constant(constants::c())
        .with_constant(constants::alpha())
        // ASCII aliases for Unicode constants
        .with_constant(constants::phi_ascii())    // "phi" alias for "φ"
        .with_constant(constants::pi_ascii())     // "pi" alias for "π"
        .with_constant(constants::alpha_ascii())  // "alpha" alias for "α"
        .with_constant(constants::m_mu_ascii())   // "m_mu" alias for "m_μ"
        .with_constant(constants::m_tau_ascii())  // "m_tau" alias for "m_τ"
}

/// Create registry with standard library
pub fn standard_registry() -> PluginRegistry {
    load_standard_library(PluginRegistry::new())
}
