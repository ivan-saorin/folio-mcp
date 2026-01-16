//! Folio Finance Functions Plugin
//!
//! Financial calculations for time value of money, loans, depreciation,
//! bonds, investment returns, and interest rate conversions.
//! All calculations use Number (BigRational) for precision.

mod helpers;
mod tvm;
mod loans;
mod depreciation;
mod bonds;
mod returns;
mod rates;

use folio_plugin::PluginRegistry;

/// Load finance functions into registry
pub fn load_finance_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // TVM (7 functions)
        .with_function(tvm::Pv)
        .with_function(tvm::Fv)
        .with_function(tvm::Npv)
        .with_function(tvm::Xnpv)
        .with_function(tvm::Irr)
        .with_function(tvm::Xirr)
        .with_function(tvm::Mirr)

        // Loans (8 functions)
        .with_function(loans::Pmt)
        .with_function(loans::Ppmt)
        .with_function(loans::Ipmt)
        .with_function(loans::Nper)
        .with_function(loans::Rate)
        .with_function(loans::Amortization)
        .with_function(loans::Cumipmt)
        .with_function(loans::Cumprinc)

        // Depreciation (5 functions)
        .with_function(depreciation::Sln)
        .with_function(depreciation::Ddb)
        .with_function(depreciation::Syd)
        .with_function(depreciation::Vdb)
        .with_function(depreciation::DepreciationSchedule)

        // Bonds (6 functions)
        .with_function(bonds::BondPrice)
        .with_function(bonds::BondYield)
        .with_function(bonds::Duration)
        .with_function(bonds::Mduration)
        .with_function(bonds::Convexity)
        .with_function(bonds::Accrint)

        // Returns (12 functions)
        .with_function(returns::Cagr)
        .with_function(returns::Roi)
        .with_function(returns::HoldingPeriodReturn)
        .with_function(returns::AnnualizedReturn)
        .with_function(returns::Sharpe)
        .with_function(returns::Sortino)
        .with_function(returns::MaxDrawdown)
        .with_function(returns::Calmar)
        .with_function(returns::Volatility)
        .with_function(returns::Beta)
        .with_function(returns::Alpha)
        .with_function(returns::Treynor)

        // Rates (5 functions)
        .with_function(rates::EffectiveRate)
        .with_function(rates::NominalRate)
        .with_function(rates::ContinuousRate)
        .with_function(rates::DiscountRate)
        .with_function(rates::RealRate)
}
