//! Folio Sequence Plugin
//!
//! Sequence generation, named sequences, pattern detection, and series operations.
//! All functions follow the never-panic philosophy and return `Value::Error` on failure.

mod helpers;
mod expr;
mod generators;
mod named;
mod recurrence;
mod pattern;
mod series;

use folio_plugin::PluginRegistry;

/// Load sequence functions into registry
pub fn load_sequence_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Basic generators
        .with_function(generators::Range)
        .with_function(generators::Linspace)
        .with_function(generators::Logspace)
        .with_function(generators::Arithmetic)
        .with_function(generators::Geometric)
        .with_function(generators::Harmonic)
        .with_function(generators::RepeatSeq)
        .with_function(generators::Cycle)

        // Named sequences
        .with_function(named::Fibonacci)
        .with_function(named::Lucas)
        .with_function(named::Tribonacci)
        .with_function(named::Primes)
        .with_function(named::PrimesUpTo)
        .with_function(named::FactorialSeq)
        .with_function(named::Triangular)
        .with_function(named::SquareNumbers)
        .with_function(named::CubeNumbers)
        .with_function(named::Powers)
        .with_function(named::Catalan)
        .with_function(named::Bell)
        .with_function(named::Pentagonal)
        .with_function(named::Hexagonal)

        // Recurrence relations
        .with_function(recurrence::Recurrence)
        .with_function(recurrence::RecurrenceNamed)

        // Pattern detection
        .with_function(pattern::DetectPattern)
        .with_function(pattern::ExtendPattern)
        .with_function(pattern::IsArithmetic)
        .with_function(pattern::IsGeometric)
        .with_function(pattern::CommonDiff)
        .with_function(pattern::CommonRatio)
        .with_function(pattern::NthTermFormula)

        // Series operations
        .with_function(series::SumSeq)
        .with_function(series::ProductSeq)
        .with_function(series::PartialSums)
        .with_function(series::PartialProducts)
        .with_function(series::AlternatingSum)
        .with_function(series::SumFormula)

        // Utility functions
        .with_function(helpers::Nth)
        .with_function(helpers::IndexOfSeq)
        .with_function(helpers::IsInSequence)
        .with_function(helpers::ReverseSeq)
        .with_function(helpers::Interleave)
        .with_function(helpers::ZipSeq)
        .with_function(helpers::TakeSeq)
        .with_function(helpers::DropSeq)
        .with_function(helpers::SliceSeq)
}
