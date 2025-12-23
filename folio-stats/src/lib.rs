//! Folio Statistics Plugin
//!
//! Statistical functions with arbitrary precision using BigRational.
//! All functions follow the never-panic philosophy and return `Value::Error` on failure.

mod helpers;
mod central;
mod dispersion;
mod position;
mod shape;
mod bivariate;
mod regression;
mod distributions;
mod hypothesis;
mod confidence;
mod transform;

use folio_plugin::PluginRegistry;

/// Load statistics functions into registry
pub fn load_stats_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Central tendency
        .with_function(central::Mean)
        .with_function(central::Median)
        .with_function(central::Mode)
        .with_function(central::GeometricMean)
        .with_function(central::HarmonicMean)
        .with_function(central::TrimmedMean)
        .with_function(central::WeightedMean)

        // Dispersion
        .with_function(dispersion::Variance)
        .with_function(dispersion::VarianceP)
        .with_function(dispersion::Stddev)
        .with_function(dispersion::StddevP)
        .with_function(dispersion::Range)
        .with_function(dispersion::Iqr)
        .with_function(dispersion::Mad)
        .with_function(dispersion::Cv)
        .with_function(dispersion::Se)

        // Position
        .with_function(position::Min)
        .with_function(position::Max)
        .with_function(position::Percentile)
        .with_function(position::Quantile)
        .with_function(position::Q1)
        .with_function(position::Q3)
        .with_function(position::Rank)
        .with_function(position::Zscore)

        // Shape
        .with_function(shape::Skewness)
        .with_function(shape::Kurtosis)
        .with_function(shape::Count)
        .with_function(shape::Product)

        // Bivariate
        .with_function(bivariate::Covariance)
        .with_function(bivariate::CovarianceP)
        .with_function(bivariate::Correlation)
        .with_function(bivariate::Spearman)

        // Regression
        .with_function(regression::LinearReg)
        .with_function(regression::Slope)
        .with_function(regression::Intercept)
        .with_function(regression::RSquared)
        .with_function(regression::Predict)
        .with_function(regression::Residuals)

        // Distributions - Normal
        .with_function(distributions::NormPdf)
        .with_function(distributions::NormCdf)
        .with_function(distributions::NormInv)
        .with_function(distributions::SnormPdf)
        .with_function(distributions::SnormCdf)
        .with_function(distributions::SnormInv)
        // Distributions - Student's t
        .with_function(distributions::TPdf)
        .with_function(distributions::TCdf)
        .with_function(distributions::TInv)
        // Distributions - Chi-squared
        .with_function(distributions::ChiPdf)
        .with_function(distributions::ChiCdf)
        .with_function(distributions::ChiInv)
        // Distributions - F
        .with_function(distributions::FPdf)
        .with_function(distributions::FCdf)
        .with_function(distributions::FInv)
        // Distributions - Discrete
        .with_function(distributions::BinomPmf)
        .with_function(distributions::BinomCdf)
        .with_function(distributions::PoissonPmf)
        .with_function(distributions::PoissonCdf)

        // Hypothesis tests
        .with_function(hypothesis::TTest1)
        .with_function(hypothesis::TTest2)
        .with_function(hypothesis::TTestPaired)
        .with_function(hypothesis::ChiTest)
        .with_function(hypothesis::FTest)
        .with_function(hypothesis::Anova)

        // Confidence intervals
        .with_function(confidence::Ci)
        .with_function(confidence::Moe)

        // Transforms
        .with_function(transform::Normalize)
        .with_function(transform::Standardize)
        .with_function(transform::Cumsum)
        .with_function(transform::Differences)
        .with_function(transform::Lag)
        .with_function(transform::MovingAvg)
        .with_function(transform::Ewma)
}
