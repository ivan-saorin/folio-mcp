//! Hypothesis testing functions: t_test, chi_test, f_test, anova

use folio_plugin::prelude::*;
use crate::helpers::{extract_numbers, extract_two_lists, mean, variance_impl};
use crate::distributions::t::t_cdf_f64;
use crate::distributions::chi::chi_cdf_f64;
use crate::distributions::f::f_cdf_f64;
use std::collections::HashMap;

// ============ One-Sample T-Test ============

pub struct TTest1;

static T_TEST_1_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list",
        typ: "List<Number>",
        description: "Sample data",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "μ0",
        typ: "Number",
        description: "Hypothesized population mean",
        optional: false,
        default: None,
    },
];

static T_TEST_1_EXAMPLES: [&str; 1] = ["t_test_1([1,2,3,4,5], 3)"];

static T_TEST_1_RELATED: [&str; 2] = ["t_test_2", "ci"];

impl FunctionPlugin for TTest1 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_test_1",
            description: "One-sample t-test",
            usage: "t_test_1(list, μ0)",
            args: &T_TEST_1_ARGS,
            returns: "Object",
            examples: &T_TEST_1_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &T_TEST_1_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        if args.len() != 2 {
            return Value::Error(FolioError::arg_count("t_test_1", 2, args.len()));
        }

        let numbers = match extract_numbers(&args[0..1]) {
            Ok(n) => n,
            Err(e) => return Value::Error(e),
        };

        let mu0 = match &args[1] {
            Value::Number(n) => n,
            Value::Error(e) => return Value::Error(e.clone()),
            other => return Value::Error(FolioError::arg_type("t_test_1", "μ0", "Number", other.type_name())),
        };

        if numbers.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "t_test_1() requires at least 2 observations",
            ));
        }

        let n = numbers.len();
        let df = (n - 1) as f64;

        let m = match mean(&numbers) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&numbers, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let se = match var.sqrt(ctx.precision) {
            Ok(v) => v.checked_div(&Number::from_i64(n as i64).sqrt(ctx.precision).unwrap_or(Number::from_i64(1))),
            Err(e) => return Value::Error(e.into()),
        };

        let se = match se {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if se.is_zero() {
            return Value::Error(FolioError::domain_error(
                "t_test_1() requires non-zero standard error",
            ));
        }

        let mean_diff = m.sub(mu0);
        let t_stat = match mean_diff.checked_div(&se) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let t_f64 = t_stat.to_f64().unwrap_or(0.0);

        // Two-tailed p-value
        let p_value = 2.0 * (1.0 - t_cdf_f64(t_f64.abs(), df));

        // 95% confidence interval
        let t_crit = 1.96; // Approximate for large samples
        let margin = se.mul(&Number::from_str(&format!("{:.15}", t_crit)).unwrap_or(Number::from_i64(2)));
        let ci_low = m.sub(&margin);
        let ci_high = m.add(&margin);

        let mut result = HashMap::new();
        result.insert("t".to_string(), Value::Number(t_stat));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df".to_string(), Value::Number(Number::from_i64(df as i64)));
        result.insert("ci_low".to_string(), Value::Number(ci_low));
        result.insert("ci_high".to_string(), Value::Number(ci_high));
        result.insert("mean_diff".to_string(), Value::Number(mean_diff));

        Value::Object(result)
    }
}

// ============ Two-Sample T-Test (Welch's) ============

pub struct TTest2;

static T_TEST_2_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List<Number>",
        description: "First sample",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List<Number>",
        description: "Second sample",
        optional: false,
        default: None,
    },
];

static T_TEST_2_EXAMPLES: [&str; 1] = ["t_test_2([1,2,3], [4,5,6])"];

static T_TEST_2_RELATED: [&str; 2] = ["t_test_1", "t_test_paired"];

impl FunctionPlugin for TTest2 {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_test_2",
            description: "Two-sample t-test (Welch's)",
            usage: "t_test_2(list1, list2)",
            args: &T_TEST_2_ARGS,
            returns: "Object",
            examples: &T_TEST_2_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &T_TEST_2_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 || y.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "t_test_2() requires at least 2 observations in each group",
            ));
        }

        let n1 = x.len() as f64;
        let n2 = y.len() as f64;

        let m1 = match mean(&x) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let m2 = match mean(&y) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var1 = match variance_impl(&x, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let var2 = match variance_impl(&y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var1_f64 = var1.to_f64().unwrap_or(0.0);
        let var2_f64 = var2.to_f64().unwrap_or(0.0);

        // Welch's t-test
        let se_squared = var1_f64 / n1 + var2_f64 / n2;
        let se = se_squared.sqrt();

        if se == 0.0 {
            return Value::Error(FolioError::domain_error(
                "t_test_2() requires non-zero pooled standard error",
            ));
        }

        let mean_diff = m1.sub(&m2);
        let mean_diff_f64 = mean_diff.to_f64().unwrap_or(0.0);
        let t_stat = mean_diff_f64 / se;

        // Welch-Satterthwaite degrees of freedom
        let num = se_squared * se_squared;
        let den = (var1_f64 / n1).powi(2) / (n1 - 1.0) + (var2_f64 / n2).powi(2) / (n2 - 1.0);
        let df = num / den;

        // Two-tailed p-value
        let p_value = 2.0 * (1.0 - t_cdf_f64(t_stat.abs(), df));

        // Confidence interval
        let t_crit = 1.96;
        let margin = t_crit * se;
        let ci_low = mean_diff_f64 - margin;
        let ci_high = mean_diff_f64 + margin;

        let mut result = HashMap::new();
        result.insert("t".to_string(), Value::Number(Number::from_str(&format!("{:.15}", t_stat)).unwrap_or(Number::from_i64(0))));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df".to_string(), Value::Number(Number::from_str(&format!("{:.15}", df)).unwrap_or(Number::from_i64(0))));
        result.insert("ci_low".to_string(), Value::Number(Number::from_str(&format!("{:.15}", ci_low)).unwrap_or(Number::from_i64(0))));
        result.insert("ci_high".to_string(), Value::Number(Number::from_str(&format!("{:.15}", ci_high)).unwrap_or(Number::from_i64(0))));
        result.insert("mean_diff".to_string(), Value::Number(mean_diff));

        Value::Object(result)
    }
}

// ============ Paired T-Test ============

pub struct TTestPaired;

static T_TEST_PAIRED_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List<Number>",
        description: "First measurements",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List<Number>",
        description: "Second measurements (paired)",
        optional: false,
        default: None,
    },
];

static T_TEST_PAIRED_EXAMPLES: [&str; 1] = ["t_test_paired([1,2,3], [2,3,4])"];

static T_TEST_PAIRED_RELATED: [&str; 2] = ["t_test_1", "t_test_2"];

impl FunctionPlugin for TTestPaired {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "t_test_paired",
            description: "Paired t-test",
            usage: "t_test_paired(list1, list2)",
            args: &T_TEST_PAIRED_ARGS,
            returns: "Object",
            examples: &T_TEST_PAIRED_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &T_TEST_PAIRED_RELATED,
        }
    }

    fn call(&self, args: &[Value], ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "t_test_paired() requires at least 2 pairs",
            ));
        }

        // Calculate differences
        let diffs: Vec<Number> = x.iter().zip(y.iter())
            .map(|(a, b)| a.sub(b))
            .collect();

        // One-sample t-test on differences with μ0 = 0
        let n = diffs.len();
        let df = (n - 1) as f64;

        let m = match mean(&diffs) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var = match variance_impl(&diffs, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let se = match var.sqrt(ctx.precision) {
            Ok(v) => v.checked_div(&Number::from_i64(n as i64).sqrt(ctx.precision).unwrap_or(Number::from_i64(1))),
            Err(e) => return Value::Error(e.into()),
        };

        let se = match se {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        if se.is_zero() {
            return Value::Error(FolioError::domain_error(
                "t_test_paired() requires non-zero standard error of differences",
            ));
        }

        let t_stat = match m.checked_div(&se) {
            Ok(v) => v,
            Err(e) => return Value::Error(e.into()),
        };

        let t_f64 = t_stat.to_f64().unwrap_or(0.0);
        let p_value = 2.0 * (1.0 - t_cdf_f64(t_f64.abs(), df));

        let t_crit = 1.96;
        let margin = se.mul(&Number::from_str(&format!("{:.15}", t_crit)).unwrap_or(Number::from_i64(2)));
        let ci_low = m.sub(&margin);
        let ci_high = m.add(&margin);

        let mut result = HashMap::new();
        result.insert("t".to_string(), Value::Number(t_stat));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df".to_string(), Value::Number(Number::from_i64(df as i64)));
        result.insert("ci_low".to_string(), Value::Number(ci_low));
        result.insert("ci_high".to_string(), Value::Number(ci_high));
        result.insert("mean_diff".to_string(), Value::Number(m));

        Value::Object(result)
    }
}

// ============ Chi-Squared Test ============

pub struct ChiTest;

static CHI_TEST_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "observed",
        typ: "List<Number>",
        description: "Observed frequencies",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "expected",
        typ: "List<Number>",
        description: "Expected frequencies",
        optional: false,
        default: None,
    },
];

static CHI_TEST_EXAMPLES: [&str; 1] = ["chi_test([10,20,30], [15,20,25])"];

static CHI_TEST_RELATED: [&str; 2] = ["chi_cdf", "anova"];

impl FunctionPlugin for ChiTest {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "chi_test",
            description: "Chi-squared goodness of fit test",
            usage: "chi_test(observed, expected)",
            args: &CHI_TEST_ARGS,
            returns: "Object",
            examples: &CHI_TEST_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &CHI_TEST_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (observed, expected) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if observed.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "chi_test() requires at least 2 categories",
            ));
        }

        // Calculate chi-squared statistic
        let mut chi_sq = 0.0;
        for (o, e) in observed.iter().zip(expected.iter()) {
            let o_f64 = o.to_f64().unwrap_or(0.0);
            let e_f64 = e.to_f64().unwrap_or(0.0);
            if e_f64 <= 0.0 {
                return Value::Error(FolioError::domain_error(
                    "chi_test() requires all expected values > 0",
                ));
            }
            chi_sq += (o_f64 - e_f64).powi(2) / e_f64;
        }

        let df = (observed.len() - 1) as f64;
        let p_value = 1.0 - chi_cdf_f64(chi_sq, df);

        let mut result = HashMap::new();
        result.insert("chi_sq".to_string(), Value::Number(Number::from_str(&format!("{:.15}", chi_sq)).unwrap_or(Number::from_i64(0))));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df".to_string(), Value::Number(Number::from_i64(df as i64)));

        Value::Object(result)
    }
}

// ============ F-Test ============

pub struct FTest;

static F_TEST_ARGS: [ArgMeta; 2] = [
    ArgMeta {
        name: "list1",
        typ: "List<Number>",
        description: "First sample",
        optional: false,
        default: None,
    },
    ArgMeta {
        name: "list2",
        typ: "List<Number>",
        description: "Second sample",
        optional: false,
        default: None,
    },
];

static F_TEST_EXAMPLES: [&str; 1] = ["f_test([1,2,3], [4,5,6])"];

static F_TEST_RELATED: [&str; 2] = ["f_cdf", "anova"];

impl FunctionPlugin for FTest {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "f_test",
            description: "F-test for variance equality",
            usage: "f_test(list1, list2)",
            args: &F_TEST_ARGS,
            returns: "Object",
            examples: &F_TEST_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &F_TEST_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        let (x, y) = match extract_two_lists(args) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        if x.len() < 2 || y.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "f_test() requires at least 2 observations in each group",
            ));
        }

        let var1 = match variance_impl(&x, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };
        let var2 = match variance_impl(&y, true) {
            Ok(v) => v,
            Err(e) => return Value::Error(e),
        };

        let var1_f64 = var1.to_f64().unwrap_or(0.0);
        let var2_f64 = var2.to_f64().unwrap_or(0.0);

        if var2_f64 == 0.0 {
            return Value::Error(FolioError::domain_error(
                "f_test() requires non-zero variance in second sample",
            ));
        }

        let f_stat = var1_f64 / var2_f64;
        let df1 = (x.len() - 1) as f64;
        let df2 = (y.len() - 1) as f64;

        // Two-tailed p-value
        let p = f_cdf_f64(f_stat, df1, df2);
        let p_value = 2.0 * (if p > 0.5 { 1.0 - p } else { p });

        let mut result = HashMap::new();
        result.insert("f".to_string(), Value::Number(Number::from_str(&format!("{:.15}", f_stat)).unwrap_or(Number::from_i64(0))));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df1".to_string(), Value::Number(Number::from_i64(df1 as i64)));
        result.insert("df2".to_string(), Value::Number(Number::from_i64(df2 as i64)));

        Value::Object(result)
    }
}

// ============ ANOVA ============

pub struct Anova;

static ANOVA_ARGS: [ArgMeta; 1] = [ArgMeta {
    name: "groups",
    typ: "List<List<Number>> | List<Number>...",
    description: "Two or more groups to compare",
    optional: false,
    default: None,
}];

static ANOVA_EXAMPLES: [&str; 1] = ["anova([1,2,3], [4,5,6], [7,8,9])"];

static ANOVA_RELATED: [&str; 2] = ["f_test", "t_test_2"];

impl FunctionPlugin for Anova {
    fn meta(&self) -> FunctionMeta {
        FunctionMeta {
            name: "anova",
            description: "One-way ANOVA",
            usage: "anova(group1, group2, ...)",
            args: &ANOVA_ARGS,
            returns: "Object",
            examples: &ANOVA_EXAMPLES,
            category: "stats/hypothesis",
            source: None,
            related: &ANOVA_RELATED,
        }
    }

    fn call(&self, args: &[Value], _ctx: &EvalContext) -> Value {
        // Extract groups
        let mut groups: Vec<Vec<f64>> = Vec::new();

        for arg in args {
            match arg {
                Value::List(list) => {
                    let mut group = Vec::new();
                    for item in list {
                        match item {
                            Value::Number(n) => group.push(n.to_f64().unwrap_or(0.0)),
                            Value::Error(e) => return Value::Error(e.clone()),
                            _ => return Value::Error(FolioError::type_error("Number", item.type_name())),
                        }
                    }
                    groups.push(group);
                }
                Value::Error(e) => return Value::Error(e.clone()),
                _ => return Value::Error(FolioError::type_error("List", arg.type_name())),
            }
        }

        if groups.len() < 2 {
            return Value::Error(FolioError::domain_error(
                "anova() requires at least 2 groups",
            ));
        }

        for g in &groups {
            if g.is_empty() {
                return Value::Error(FolioError::domain_error(
                    "anova() requires non-empty groups",
                ));
            }
        }

        // Calculate grand mean
        let total_n: usize = groups.iter().map(|g| g.len()).sum();
        let total_sum: f64 = groups.iter().flat_map(|g| g.iter()).sum();
        let grand_mean = total_sum / total_n as f64;

        // Calculate SS_between and SS_within
        let mut ss_between = 0.0;
        let mut ss_within = 0.0;

        for group in &groups {
            let n = group.len() as f64;
            let group_mean: f64 = group.iter().sum::<f64>() / n;

            ss_between += n * (group_mean - grand_mean).powi(2);

            for x in group {
                ss_within += (x - group_mean).powi(2);
            }
        }

        let k = groups.len() as f64;
        let df_between = k - 1.0;
        let df_within = total_n as f64 - k;

        let ms_between = ss_between / df_between;
        let ms_within = ss_within / df_within;

        let f_stat = if ms_within > 0.0 {
            ms_between / ms_within
        } else {
            return Value::Error(FolioError::domain_error(
                "anova() requires non-zero within-group variance",
            ));
        };

        let p_value = 1.0 - f_cdf_f64(f_stat, df_between, df_within);

        let mut result = HashMap::new();
        result.insert("f".to_string(), Value::Number(Number::from_str(&format!("{:.15}", f_stat)).unwrap_or(Number::from_i64(0))));
        result.insert("p".to_string(), Value::Number(Number::from_str(&format!("{:.15}", p_value)).unwrap_or(Number::from_i64(0))));
        result.insert("df_between".to_string(), Value::Number(Number::from_i64(df_between as i64)));
        result.insert("df_within".to_string(), Value::Number(Number::from_i64(df_within as i64)));
        result.insert("ss_between".to_string(), Value::Number(Number::from_str(&format!("{:.15}", ss_between)).unwrap_or(Number::from_i64(0))));
        result.insert("ss_within".to_string(), Value::Number(Number::from_str(&format!("{:.15}", ss_within)).unwrap_or(Number::from_i64(0))));

        Value::Object(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_ctx() -> EvalContext {
        EvalContext::new(std::sync::Arc::new(folio_plugin::PluginRegistry::new()))
    }

    #[test]
    fn test_t_test_1() {
        let t_test = TTest1;
        let args = vec![
            Value::List(vec![
                Value::Number(Number::from_i64(1)),
                Value::Number(Number::from_i64(2)),
                Value::Number(Number::from_i64(3)),
                Value::Number(Number::from_i64(4)),
                Value::Number(Number::from_i64(5)),
            ]),
            Value::Number(Number::from_i64(3)),
        ];
        let result = t_test.call(&args, &eval_ctx());
        assert!(result.as_object().is_some());
    }
}
