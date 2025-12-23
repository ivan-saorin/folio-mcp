//! Statistical distributions: normal, t, chi-squared, F, binomial, Poisson

pub mod normal;
pub mod t;
pub mod chi;
pub mod f;
mod discrete;

pub use normal::{NormPdf, NormCdf, NormInv, SnormPdf, SnormCdf, SnormInv};
pub use t::{TPdf, TCdf, TInv};
pub use chi::{ChiPdf, ChiCdf, ChiInv};
pub use f::{FPdf, FCdf, FInv};
pub use discrete::{BinomPmf, BinomCdf, PoissonPmf, PoissonCdf};
