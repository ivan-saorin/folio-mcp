//! Ingredient density database for volume-to-weight conversions
//!
//! This module provides the core US→Italian/Metric conversion feature:
//! converting cups to grams based on ingredient density.

use folio_core::{FolioError, Number, Value};
use folio_plugin::{ArgMeta, EvalContext, FunctionMeta, FunctionPlugin};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::helpers::{extract_number, extract_text, extract_optional_text, normalize_ingredient};

/// Ingredient density entry (grams per US cup)
#[derive(Debug, Clone)]
pub struct IngredientData {
    pub name: &'static str,
    pub grams_per_cup: f64,
    pub category: &'static str,
    pub aliases: &'static [&'static str],
}

/// The ingredient density database
static INGREDIENTS: LazyLock<HashMap<String, IngredientData>> = LazyLock::new(|| {
    let entries = vec![
        // === FLOURS (category: "flour") ===
        IngredientData { name: "all purpose flour", grams_per_cup: 125.0, category: "flour",
            aliases: &["ap flour", "plain flour", "white flour", "flour"] },
        IngredientData { name: "bread flour", grams_per_cup: 127.0, category: "flour",
            aliases: &["strong flour", "high gluten flour"] },
        IngredientData { name: "cake flour", grams_per_cup: 114.0, category: "flour",
            aliases: &["pastry flour"] },
        IngredientData { name: "whole wheat flour", grams_per_cup: 120.0, category: "flour",
            aliases: &["wholemeal flour", "graham flour"] },
        IngredientData { name: "almond flour", grams_per_cup: 96.0, category: "flour",
            aliases: &["almond meal", "ground almonds"] },
        IngredientData { name: "coconut flour", grams_per_cup: 112.0, category: "flour",
            aliases: &[] },
        IngredientData { name: "rye flour", grams_per_cup: 102.0, category: "flour",
            aliases: &[] },
        IngredientData { name: "semolina", grams_per_cup: 167.0, category: "flour",
            aliases: &["semolina flour", "semola"] },
        IngredientData { name: "cornmeal", grams_per_cup: 138.0, category: "flour",
            aliases: &["polenta"] },
        IngredientData { name: "cornstarch", grams_per_cup: 128.0, category: "flour",
            aliases: &["corn starch", "maize starch"] },
        IngredientData { name: "tapioca flour", grams_per_cup: 120.0, category: "flour",
            aliases: &["tapioca starch"] },
        IngredientData { name: "rice flour", grams_per_cup: 158.0, category: "flour",
            aliases: &[] },
        IngredientData { name: "oat flour", grams_per_cup: 92.0, category: "flour",
            aliases: &[] },
        IngredientData { name: "buckwheat flour", grams_per_cup: 120.0, category: "flour",
            aliases: &[] },
        IngredientData { name: "spelt flour", grams_per_cup: 99.0, category: "flour",
            aliases: &["farro flour"] },
        IngredientData { name: "tipo 00 flour", grams_per_cup: 125.0, category: "flour",
            aliases: &["00 flour", "doppio zero"] },

        // === SUGARS (category: "sugar") ===
        IngredientData { name: "granulated sugar", grams_per_cup: 200.0, category: "sugar",
            aliases: &["white sugar", "sugar", "caster sugar", "zucchero"] },
        IngredientData { name: "brown sugar packed", grams_per_cup: 220.0, category: "sugar",
            aliases: &["packed brown sugar", "dark brown sugar packed", "light brown sugar packed"] },
        IngredientData { name: "brown sugar loose", grams_per_cup: 145.0, category: "sugar",
            aliases: &["unpacked brown sugar"] },
        IngredientData { name: "powdered sugar", grams_per_cup: 120.0, category: "sugar",
            aliases: &["confectioners sugar", "icing sugar", "10x sugar", "zucchero a velo"] },
        IngredientData { name: "coconut sugar", grams_per_cup: 168.0, category: "sugar",
            aliases: &[] },
        IngredientData { name: "turbinado sugar", grams_per_cup: 180.0, category: "sugar",
            aliases: &["demerara sugar", "raw sugar"] },
        IngredientData { name: "maple sugar", grams_per_cup: 165.0, category: "sugar",
            aliases: &[] },

        // === LIQUIDS (category: "liquid") ===
        IngredientData { name: "water", grams_per_cup: 237.0, category: "liquid",
            aliases: &["acqua"] },
        IngredientData { name: "milk", grams_per_cup: 245.0, category: "liquid",
            aliases: &["whole milk", "latte"] },
        IngredientData { name: "heavy cream", grams_per_cup: 232.0, category: "liquid",
            aliases: &["heavy whipping cream", "double cream", "whipping cream", "panna"] },
        IngredientData { name: "half and half", grams_per_cup: 242.0, category: "liquid",
            aliases: &["half & half"] },
        IngredientData { name: "buttermilk", grams_per_cup: 245.0, category: "liquid",
            aliases: &["latticello"] },
        IngredientData { name: "sour cream", grams_per_cup: 242.0, category: "liquid",
            aliases: &["panna acida"] },
        IngredientData { name: "yogurt", grams_per_cup: 245.0, category: "liquid",
            aliases: &["plain yogurt", "greek yogurt", "yogurt greco"] },
        IngredientData { name: "vegetable oil", grams_per_cup: 218.0, category: "liquid",
            aliases: &["oil", "canola oil", "sunflower oil", "olio di semi"] },
        IngredientData { name: "olive oil", grams_per_cup: 216.0, category: "liquid",
            aliases: &["extra virgin olive oil", "olio d'oliva", "olio evo"] },
        IngredientData { name: "coconut oil", grams_per_cup: 218.0, category: "liquid",
            aliases: &["olio di cocco"] },
        IngredientData { name: "honey", grams_per_cup: 340.0, category: "liquid",
            aliases: &["miele"] },
        IngredientData { name: "maple syrup", grams_per_cup: 322.0, category: "liquid",
            aliases: &["sciroppo d'acero"] },
        IngredientData { name: "corn syrup", grams_per_cup: 328.0, category: "liquid",
            aliases: &["light corn syrup"] },
        IngredientData { name: "molasses", grams_per_cup: 340.0, category: "liquid",
            aliases: &["blackstrap molasses", "melassa"] },

        // === FATS (category: "fat") ===
        IngredientData { name: "butter", grams_per_cup: 227.0, category: "fat",
            aliases: &["unsalted butter", "salted butter", "burro"] },
        IngredientData { name: "butter melted", grams_per_cup: 227.0, category: "fat",
            aliases: &["melted butter", "burro fuso"] },
        IngredientData { name: "shortening", grams_per_cup: 191.0, category: "fat",
            aliases: &["vegetable shortening", "crisco"] },
        IngredientData { name: "lard", grams_per_cup: 205.0, category: "fat",
            aliases: &["strutto"] },
        IngredientData { name: "cream cheese", grams_per_cup: 232.0, category: "fat",
            aliases: &["philadelphia"] },
        IngredientData { name: "mascarpone", grams_per_cup: 227.0, category: "fat",
            aliases: &[] },

        // === GRAINS & CEREALS (category: "grain") ===
        IngredientData { name: "white rice", grams_per_cup: 185.0, category: "grain",
            aliases: &["rice", "long grain rice", "riso"] },
        IngredientData { name: "brown rice", grams_per_cup: 190.0, category: "grain",
            aliases: &["riso integrale"] },
        IngredientData { name: "arborio rice", grams_per_cup: 200.0, category: "grain",
            aliases: &["risotto rice", "carnaroli rice"] },
        IngredientData { name: "rolled oats", grams_per_cup: 80.0, category: "grain",
            aliases: &["oats", "old fashioned oats", "fiocchi d'avena"] },
        IngredientData { name: "quick oats", grams_per_cup: 80.0, category: "grain",
            aliases: &["instant oats"] },
        IngredientData { name: "steel cut oats", grams_per_cup: 160.0, category: "grain",
            aliases: &[] },
        IngredientData { name: "quinoa", grams_per_cup: 170.0, category: "grain",
            aliases: &[] },
        IngredientData { name: "couscous", grams_per_cup: 173.0, category: "grain",
            aliases: &["cuscus"] },
        IngredientData { name: "bulgur", grams_per_cup: 140.0, category: "grain",
            aliases: &["bulgur wheat"] },
        IngredientData { name: "breadcrumbs dry", grams_per_cup: 108.0, category: "grain",
            aliases: &["dry breadcrumbs", "panko", "pangrattato"] },
        IngredientData { name: "breadcrumbs fresh", grams_per_cup: 60.0, category: "grain",
            aliases: &["fresh breadcrumbs"] },
        IngredientData { name: "pasta dry", grams_per_cup: 100.0, category: "grain",
            aliases: &["dry pasta", "pasta secca"] },

        // === NUTS & SEEDS (category: "nuts") ===
        IngredientData { name: "almonds whole", grams_per_cup: 143.0, category: "nuts",
            aliases: &["whole almonds", "mandorle"] },
        IngredientData { name: "almonds sliced", grams_per_cup: 92.0, category: "nuts",
            aliases: &["sliced almonds", "mandorle a fette"] },
        IngredientData { name: "almonds slivered", grams_per_cup: 108.0, category: "nuts",
            aliases: &["slivered almonds"] },
        IngredientData { name: "walnuts chopped", grams_per_cup: 117.0, category: "nuts",
            aliases: &["chopped walnuts", "walnuts", "noci"] },
        IngredientData { name: "pecans chopped", grams_per_cup: 109.0, category: "nuts",
            aliases: &["chopped pecans", "pecans", "noci pecan"] },
        IngredientData { name: "peanuts", grams_per_cup: 146.0, category: "nuts",
            aliases: &["arachidi"] },
        IngredientData { name: "hazelnuts", grams_per_cup: 135.0, category: "nuts",
            aliases: &["filberts", "nocciole"] },
        IngredientData { name: "cashews", grams_per_cup: 137.0, category: "nuts",
            aliases: &["anacardi"] },
        IngredientData { name: "pine nuts", grams_per_cup: 135.0, category: "nuts",
            aliases: &["pignoli", "pinoli"] },
        IngredientData { name: "pistachios", grams_per_cup: 123.0, category: "nuts",
            aliases: &["pistacchi"] },
        IngredientData { name: "sunflower seeds", grams_per_cup: 140.0, category: "nuts",
            aliases: &["semi di girasole"] },
        IngredientData { name: "pumpkin seeds", grams_per_cup: 129.0, category: "nuts",
            aliases: &["pepitas", "semi di zucca"] },
        IngredientData { name: "sesame seeds", grams_per_cup: 144.0, category: "nuts",
            aliases: &["semi di sesamo"] },
        IngredientData { name: "chia seeds", grams_per_cup: 163.0, category: "nuts",
            aliases: &["semi di chia"] },
        IngredientData { name: "flax seeds", grams_per_cup: 168.0, category: "nuts",
            aliases: &["linseed", "semi di lino"] },
        IngredientData { name: "ground flax", grams_per_cup: 112.0, category: "nuts",
            aliases: &["flax meal"] },
        IngredientData { name: "peanut butter", grams_per_cup: 258.0, category: "nuts",
            aliases: &["burro di arachidi"] },
        IngredientData { name: "almond butter", grams_per_cup: 256.0, category: "nuts",
            aliases: &["burro di mandorle"] },
        IngredientData { name: "tahini", grams_per_cup: 240.0, category: "nuts",
            aliases: &["sesame paste"] },

        // === CHOCOLATE & COCOA (category: "chocolate") ===
        IngredientData { name: "cocoa powder", grams_per_cup: 85.0, category: "chocolate",
            aliases: &["cocoa", "unsweetened cocoa", "cacao in polvere"] },
        IngredientData { name: "chocolate chips", grams_per_cup: 170.0, category: "chocolate",
            aliases: &["semisweet chocolate chips", "gocce di cioccolato"] },
        IngredientData { name: "chocolate chopped", grams_per_cup: 170.0, category: "chocolate",
            aliases: &["chopped chocolate", "cioccolato tritato"] },

        // === LEAVENERS & BAKING (category: "leavener") ===
        IngredientData { name: "baking powder", grams_per_cup: 230.0, category: "leavener",
            aliases: &["lievito in polvere"] },
        IngredientData { name: "baking soda", grams_per_cup: 288.0, category: "leavener",
            aliases: &["bicarbonate of soda", "bicarbonato"] },
        IngredientData { name: "active dry yeast", grams_per_cup: 150.0, category: "leavener",
            aliases: &["dry yeast", "yeast", "lievito secco"] },
        IngredientData { name: "instant yeast", grams_per_cup: 150.0, category: "leavener",
            aliases: &["rapid rise yeast", "bread machine yeast", "lievito istantaneo"] },
        IngredientData { name: "cream of tartar", grams_per_cup: 150.0, category: "leavener",
            aliases: &["cremor tartaro"] },

        // === SALT & SPICES (category: "spice") ===
        IngredientData { name: "table salt", grams_per_cup: 292.0, category: "spice",
            aliases: &["salt", "fine salt", "sale fino"] },
        IngredientData { name: "kosher salt", grams_per_cup: 240.0, category: "spice",
            aliases: &["coarse salt", "sale grosso"] },
        IngredientData { name: "sea salt", grams_per_cup: 227.0, category: "spice",
            aliases: &["flaky salt", "sale marino"] },
        IngredientData { name: "cinnamon", grams_per_cup: 132.0, category: "spice",
            aliases: &["ground cinnamon", "cannella"] },
        IngredientData { name: "nutmeg", grams_per_cup: 113.0, category: "spice",
            aliases: &["ground nutmeg", "noce moscata"] },
        IngredientData { name: "ginger", grams_per_cup: 96.0, category: "spice",
            aliases: &["ground ginger", "zenzero in polvere"] },
        IngredientData { name: "black pepper", grams_per_cup: 116.0, category: "spice",
            aliases: &["ground pepper", "pepper", "pepe nero"] },

        // === CHEESE (category: "cheese") ===
        IngredientData { name: "parmesan grated", grams_per_cup: 90.0, category: "cheese",
            aliases: &["grated parmesan", "parmigiano reggiano grated", "parmigiano grattugiato"] },
        IngredientData { name: "parmesan shredded", grams_per_cup: 110.0, category: "cheese",
            aliases: &["shredded parmesan"] },
        IngredientData { name: "pecorino grated", grams_per_cup: 90.0, category: "cheese",
            aliases: &["grated pecorino", "pecorino romano"] },
        IngredientData { name: "cheddar shredded", grams_per_cup: 113.0, category: "cheese",
            aliases: &["shredded cheddar"] },
        IngredientData { name: "mozzarella shredded", grams_per_cup: 112.0, category: "cheese",
            aliases: &["shredded mozzarella"] },
        IngredientData { name: "ricotta", grams_per_cup: 246.0, category: "cheese",
            aliases: &[] },
        IngredientData { name: "cottage cheese", grams_per_cup: 226.0, category: "cheese",
            aliases: &["fiocchi di latte"] },
        IngredientData { name: "feta crumbled", grams_per_cup: 150.0, category: "cheese",
            aliases: &["crumbled feta"] },
        IngredientData { name: "gorgonzola", grams_per_cup: 150.0, category: "cheese",
            aliases: &[] },

        // === DRIED FRUIT (category: "dried fruit") ===
        IngredientData { name: "raisins", grams_per_cup: 145.0, category: "dried fruit",
            aliases: &["sultanas", "uvetta"] },
        IngredientData { name: "dried cranberries", grams_per_cup: 120.0, category: "dried fruit",
            aliases: &["craisins", "mirtilli rossi secchi"] },
        IngredientData { name: "dried apricots chopped", grams_per_cup: 130.0, category: "dried fruit",
            aliases: &["chopped apricots", "albicocche secche"] },
        IngredientData { name: "dates chopped", grams_per_cup: 147.0, category: "dried fruit",
            aliases: &["chopped dates", "datteri"] },
        IngredientData { name: "shredded coconut", grams_per_cup: 85.0, category: "dried fruit",
            aliases: &["desiccated coconut", "coconut flakes", "cocco grattugiato"] },

        // === EGGS (category: "egg") ===
        IngredientData { name: "whole eggs", grams_per_cup: 243.0, category: "egg",
            aliases: &["eggs beaten", "scrambled eggs liquid", "uova"] },
        IngredientData { name: "egg whites", grams_per_cup: 243.0, category: "egg",
            aliases: &["albumi"] },
        IngredientData { name: "egg yolks", grams_per_cup: 243.0, category: "egg",
            aliases: &["tuorli"] },

        // === LEGUMES (category: "legume") ===
        IngredientData { name: "dried beans", grams_per_cup: 180.0, category: "legume",
            aliases: &["kidney beans dry", "black beans dry", "pinto beans dry", "fagioli secchi"] },
        IngredientData { name: "lentils", grams_per_cup: 192.0, category: "legume",
            aliases: &["dried lentils", "lenticchie"] },
        IngredientData { name: "chickpeas dry", grams_per_cup: 200.0, category: "legume",
            aliases: &["garbanzo beans dry", "ceci secchi"] },
        IngredientData { name: "split peas", grams_per_cup: 195.0, category: "legume",
            aliases: &["piselli spezzati"] },
        IngredientData { name: "cannellini beans dry", grams_per_cup: 180.0, category: "legume",
            aliases: &["white beans dry", "fagioli cannellini"] },
    ];

    let mut map = HashMap::new();
    for entry in entries {
        // Add primary name
        let normalized = normalize_ingredient(entry.name);
        map.insert(normalized, entry.clone());
        // Add all aliases
        for alias in entry.aliases {
            let normalized_alias = normalize_ingredient(alias);
            if !map.contains_key(&normalized_alias) {
                map.insert(normalized_alias, entry.clone());
            }
        }
    }
    map
});

/// Lookup ingredient density (returns grams per cup)
pub fn lookup_density(name: &str) -> Option<&IngredientData> {
    INGREDIENTS.get(&normalize_ingredient(name))
}

/// Get all unique ingredient names
pub fn all_ingredient_names() -> Vec<&'static str> {
    let mut seen = std::collections::HashSet::new();
    let mut names: Vec<&'static str> = INGREDIENTS.values()
        .filter_map(|i| {
            if seen.insert(i.name) {
                Some(i.name)
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

/// Get ingredients by category
pub fn ingredients_by_category(category: &str) -> Vec<&'static str> {
    let mut seen = std::collections::HashSet::new();
    let cat_lower = category.to_lowercase();
    let mut names: Vec<&'static str> = INGREDIENTS.values()
        .filter(|i| i.category == cat_lower)
        .filter_map(|i| {
            if seen.insert(i.name) {
                Some(i.name)
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

// ============ cups_to_grams ============

pub struct CupsToGrams;

static CUPS_TO_GRAMS_ARGS: [ArgMeta; 2] = [
    ArgMeta::required("cups", "Number", "Number of US cups"),
    ArgMeta::required("ingredient", "Text", "Ingredient name (e.g., \"flour\", \"sugar\")"),
];

static CUPS_TO_GRAMS_EXAMPLES: [&str; 4] = [
    "cups_to_grams(1, \"flour\") -> 125",
    "cups_to_grams(2, \"granulated sugar\") -> 400",
    "cups_to_grams(0.5, \"butter\") -> 113.5",
    "cups_to_grams(1, \"honey\") -> 340",
];

static CUPS_TO_GRAMS_RELATED: [&str; 3] = ["grams_to_cups", "ingredient_density", "list_ingredients"];

impl FunctionPlugin for CupsToGrams {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "cups_to_grams",
            description: "Convert US cups to grams using ingredient density",
            usage: "cups_to_grams(cups, ingredient)",
            args: &CUPS_TO_GRAMS_ARGS,
            returns: "Number",
            examples: &CUPS_TO_GRAMS_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &CUPS_TO_GRAMS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("cups_to_grams", 2, args.len()));
        }

        let cups = match extract_number(&args[0], "cups_to_grams", "cups") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let ingredient = match extract_text(&args[1], "cups_to_grams", "ingredient") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        match lookup_density(&ingredient) {
            Some(data) => {
                let grams_per_cup = Number::from_f64(data.grams_per_cup);
                Value::Number(cups.mul(&grams_per_cup))
            }
            None => Value::Error(FolioError::domain_error(format!(
                "cups_to_grams: Unknown ingredient '{}'. Use list_ingredients() to see available ingredients.",
                ingredient
            ))),
        }
    }
}

// ============ grams_to_cups ============

pub struct GramsToCups;

static GRAMS_TO_CUPS_ARGS: [ArgMeta; 2] = [
    ArgMeta::required("grams", "Number", "Weight in grams"),
    ArgMeta::required("ingredient", "Text", "Ingredient name"),
];

static GRAMS_TO_CUPS_EXAMPLES: [&str; 3] = [
    "grams_to_cups(250, \"flour\") -> 2",
    "grams_to_cups(100, \"sugar\") -> 0.5",
    "grams_to_cups(227, \"butter\") -> 1",
];

static GRAMS_TO_CUPS_RELATED: [&str; 3] = ["cups_to_grams", "ingredient_density", "list_ingredients"];

impl FunctionPlugin for GramsToCups {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "grams_to_cups",
            description: "Convert grams to US cups using ingredient density",
            usage: "grams_to_cups(grams, ingredient)",
            args: &GRAMS_TO_CUPS_ARGS,
            returns: "Number",
            examples: &GRAMS_TO_CUPS_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &GRAMS_TO_CUPS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.len() < 2 {
            return Value::Error(FolioError::arg_count("grams_to_cups", 2, args.len()));
        }

        let grams = match extract_number(&args[0], "grams_to_cups", "grams") {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let ingredient = match extract_text(&args[1], "grams_to_cups", "ingredient") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        match lookup_density(&ingredient) {
            Some(data) => {
                let grams_per_cup = Number::from_f64(data.grams_per_cup);
                match grams.checked_div(&grams_per_cup) {
                    Ok(result) => Value::Number(result),
                    Err(e) => Value::Error(e.into()),
                }
            }
            None => Value::Error(FolioError::domain_error(format!(
                "grams_to_cups: Unknown ingredient '{}'. Use list_ingredients() to see available ingredients.",
                ingredient
            ))),
        }
    }
}

// ============ ingredient_density ============

pub struct IngredientDensity;

static INGREDIENT_DENSITY_ARGS: [ArgMeta; 1] = [
    ArgMeta::required("ingredient", "Text", "Ingredient name"),
];

static INGREDIENT_DENSITY_EXAMPLES: [&str; 2] = [
    "ingredient_density(\"flour\") -> {name: \"all purpose flour\", grams_per_cup: 125, category: \"flour\"}",
    "ingredient_density(\"honey\") -> {name: \"honey\", grams_per_cup: 340, category: \"liquid\"}",
];

static INGREDIENT_DENSITY_RELATED: [&str; 2] = ["cups_to_grams", "list_ingredients"];

impl FunctionPlugin for IngredientDensity {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "ingredient_density",
            description: "Get density information for an ingredient",
            usage: "ingredient_density(ingredient)",
            args: &INGREDIENT_DENSITY_ARGS,
            returns: "Object",
            examples: &INGREDIENT_DENSITY_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &INGREDIENT_DENSITY_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        if args.is_empty() {
            return Value::Error(FolioError::arg_count("ingredient_density", 1, 0));
        }

        let ingredient = match extract_text(&args[0], "ingredient_density", "ingredient") {
            Ok(s) => s,
            Err(e) => return Value::Error(e),
        };

        match lookup_density(&ingredient) {
            Some(data) => {
                let mut obj = std::collections::HashMap::new();
                obj.insert("name".to_string(), Value::Text(data.name.to_string()));
                obj.insert("grams_per_cup".to_string(),
                    Value::Number(Number::from_f64(data.grams_per_cup)));
                obj.insert("category".to_string(), Value::Text(data.category.to_string()));
                Value::Object(obj)
            }
            None => Value::Error(FolioError::domain_error(format!(
                "ingredient_density: Unknown ingredient '{}'. Use list_ingredients() to see available ingredients.",
                ingredient
            ))),
        }
    }
}

// ============ list_ingredients ============

pub struct ListIngredients;

static LIST_INGREDIENTS_ARGS: [ArgMeta; 1] = [
    ArgMeta::optional("category", "Text",
        "Optional category filter (flour, sugar, liquid, fat, grain, nuts, chocolate, leavener, spice, cheese, dried fruit, egg, legume)",
        ""),
];

static LIST_INGREDIENTS_EXAMPLES: [&str; 2] = [
    "list_ingredients() -> [\"active dry yeast\", \"all purpose flour\", ...]",
    "list_ingredients(\"flour\") -> [\"all purpose flour\", \"almond flour\", \"bread flour\", ...]",
];

static LIST_INGREDIENTS_RELATED: [&str; 2] = ["ingredient_density", "cups_to_grams"];

impl FunctionPlugin for ListIngredients {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "list_ingredients",
            description: "List available ingredients in the density database",
            usage: "list_ingredients([category])",
            args: &LIST_INGREDIENTS_ARGS,
            returns: "List<Text>",
            examples: &LIST_INGREDIENTS_EXAMPLES,
            category: "kitchen",
            source: None,
            related: &LIST_INGREDIENTS_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let category = extract_optional_text(args, 0);

        let names: Vec<Value> = match category {
            Some(cat) if !cat.is_empty() => {
                ingredients_by_category(&cat)
                    .iter()
                    .map(|n| Value::Text(n.to_string()))
                    .collect()
            }
            _ => all_ingredient_names()
                .iter()
                .map(|n| Value::Text(n.to_string()))
                .collect(),
        };

        Value::List(names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_density() {
        assert!(lookup_density("flour").is_some());
        assert!(lookup_density("all purpose flour").is_some());
        assert!(lookup_density("FLOUR").is_some());
        assert!(lookup_density("unknown_ingredient_xyz").is_none());
    }

    #[test]
    fn test_flour_density() {
        let data = lookup_density("flour").unwrap();
        assert_eq!(data.grams_per_cup, 125.0);
        assert_eq!(data.category, "flour");
    }

    #[test]
    fn test_italian_aliases() {
        // Test some Italian ingredient names
        assert!(lookup_density("burro").is_some()); // butter
        assert!(lookup_density("zucchero").is_some()); // sugar
        assert!(lookup_density("farina").is_none()); // flour in Italian - not yet added as alias
    }

    #[test]
    fn test_all_ingredient_names() {
        let names = all_ingredient_names();
        assert!(names.len() > 50);
        assert!(names.contains(&"all purpose flour"));
        assert!(names.contains(&"butter"));
    }

    #[test]
    fn test_ingredients_by_category() {
        let flours = ingredients_by_category("flour");
        assert!(flours.len() > 5);
        assert!(flours.contains(&"all purpose flour"));
        assert!(flours.contains(&"bread flour"));
    }
}
