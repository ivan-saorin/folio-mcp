//! Folio Kitchen Functions Plugin
//!
//! Kitchen measurement conversions between US and Italian/Metric systems.
//! Features:
//! - Ingredient density database for volume-to-weight conversions (cups ↔ grams)
//! - Named oven temperatures and gas mark conversions
//! - Recipe scaling by servings and pan size
//! - Cooking time adjustments for altitude and convection ovens
//!
//! For direct unit conversions (cups ↔ mL, F ↔ C), use folio-units.

mod helpers;
mod density;
mod temperature;
mod scaling;
mod cooking;

use folio_plugin::PluginRegistry;

/// Load kitchen functions into registry
pub fn load_kitchen_library(registry: PluginRegistry) -> PluginRegistry {
    registry
        // Density conversions (4 functions) - the core US→Italian feature
        .with_function(density::CupsToGrams)
        .with_function(density::GramsToCups)
        .with_function(density::IngredientDensity)
        .with_function(density::ListIngredients)

        // Temperature conversions (3 functions) - kitchen-specific
        .with_function(temperature::OvenTemp)
        .with_function(temperature::GasMark)
        .with_function(temperature::GasMarkFromTemp)

        // Recipe scaling (3 functions)
        .with_function(scaling::ScaleRecipe)
        .with_function(scaling::PanScale)
        .with_function(scaling::BatchMultiply)

        // Cooking adjustments (3 functions)
        .with_function(cooking::AltitudeTime)
        .with_function(cooking::ConvectionTemp)
        .with_function(cooking::ConvectionTime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_kitchen_library() {
        let registry = PluginRegistry::new();
        let registry = load_kitchen_library(registry);

        // Verify all functions are registered
        assert!(registry.get_function("cups_to_grams").is_some());
        assert!(registry.get_function("grams_to_cups").is_some());
        assert!(registry.get_function("ingredient_density").is_some());
        assert!(registry.get_function("list_ingredients").is_some());
        assert!(registry.get_function("oven_temp").is_some());
        assert!(registry.get_function("gas_mark").is_some());
        assert!(registry.get_function("gas_mark_from_temp").is_some());
        assert!(registry.get_function("scale_recipe").is_some());
        assert!(registry.get_function("pan_scale").is_some());
        assert!(registry.get_function("batch_multiply").is_some());
        assert!(registry.get_function("altitude_time").is_some());
        assert!(registry.get_function("convection_temp").is_some());
        assert!(registry.get_function("convection_time").is_some());
    }
}
