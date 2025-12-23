//! Document evaluator
//!
//! Evaluates document expressions in dependency order.

use crate::ast::{Document, Expr, BinOp, UnaryOp};
use folio_plugin::EvalContext;
use folio_core::{Value, FolioError, Number};
use std::collections::{HashMap, HashSet, VecDeque};

/// Result of document evaluation
#[derive(Debug)]
pub struct EvalResult {
    /// Rendered markdown with results
    pub markdown: String,
    /// All computed values by cell name
    pub values: HashMap<String, Value>,
    /// Errors encountered
    pub errors: Vec<FolioError>,
    /// Warnings (non-fatal)
    pub warnings: Vec<FolioError>,
}

impl EvalResult {
    /// Create error result for parse failure
    pub fn parse_error(error: FolioError) -> Self {
        Self {
            markdown: format!("# Parse Error\n\n{}", error),
            values: HashMap::new(),
            errors: vec![error],
            warnings: vec![],
        }
    }
}

/// Document evaluator
pub struct Evaluator;

impl Evaluator {
    pub fn new() -> Self {
        Self
    }
    
    /// Evaluate document, return values by cell name
    pub fn eval(&self, doc: &Document, ctx: &mut EvalContext) -> HashMap<String, Value> {
        let mut values = HashMap::new();

        // Collect all cells and their formulas
        let mut cells: HashMap<String, (Option<&Expr>, &str, u32)> = HashMap::new();
        let mut section_precisions: HashMap<String, u32> = HashMap::new();

        for section in &doc.sections {
            let section_precision = section.attributes
                .get("precision")
                .and_then(|p| p.parse().ok())
                .unwrap_or(ctx.precision);

            for row in &section.table.rows {
                for cell in &row.cells {
                    cells.insert(
                        cell.name.clone(),
                        (cell.formula.as_ref(), &cell.raw_text, section_precision)
                    );
                    section_precisions.insert(cell.name.clone(), section_precision);
                }
            }
        }

        // Build dependency graph
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();
        for (name, (formula, _, _)) in &cells {
            let deps = if let Some(expr) = formula {
                self.extract_dependencies(expr)
            } else {
                HashSet::new()
            };
            // Filter to only existing cells
            let filtered_deps: Vec<String> = deps.into_iter()
                .filter(|d| cells.contains_key(d))
                .collect();
            dependencies.insert(name.clone(), filtered_deps);
        }

        // Detect cycles and compute topological order
        match self.topological_sort(&dependencies) {
            Ok(order) => {
                // Evaluate in topological order
                for cell_name in order {
                    if let Some((formula, raw_text, precision)) = cells.get(&cell_name) {
                        ctx.precision = *precision;

                        // Check if this variable was already set externally
                        // External variables take precedence over hardcoded values
                        let existing = ctx.get_var(&cell_name);
                        if !existing.is_error() && formula.is_none() {
                            // External variable exists and cell is a literal - use external value
                            values.insert(cell_name.clone(), existing);
                            continue;
                        }

                        let value = match formula {
                            Some(expr) => {
                                let deps = dependencies.get(&cell_name).cloned().unwrap_or_default();
                                let result = self.eval_expr(expr, ctx);
                                if ctx.tracing {
                                    ctx.record_trace(
                                        cell_name.clone(),
                                        raw_text.to_string(),
                                        result.clone(),
                                        deps,
                                    );
                                }
                                result
                            }
                            None => self.parse_literal(raw_text),
                        };
                        ctx.set_var(cell_name.clone(), value.clone());
                        values.insert(cell_name.clone(), value);
                    }
                }
            }
            Err(cycle) => {
                // Return circular reference error for all cells in the cycle
                let error = FolioError::circular_ref(&cycle);
                for cell_name in cycle {
                    values.insert(cell_name.clone(), Value::Error(error.clone()));
                }
                // Evaluate remaining cells in document order
                for section in &doc.sections {
                    let section_precision = section.attributes
                        .get("precision")
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(ctx.precision);
                    ctx.precision = section_precision;

                    for row in &section.table.rows {
                        for cell in &row.cells {
                            if !values.contains_key(&cell.name) {
                                // Check if this variable was already set externally
                                let existing = ctx.get_var(&cell.name);
                                if !existing.is_error() && cell.formula.is_none() {
                                    // External variable exists and cell is a literal - use external value
                                    values.insert(cell.name.clone(), existing);
                                    continue;
                                }

                                let value = match &cell.formula {
                                    Some(expr) => self.eval_expr(expr, ctx),
                                    None => self.parse_literal(&cell.raw_text),
                                };
                                ctx.set_var(cell.name.clone(), value.clone());
                                values.insert(cell.name.clone(), value);
                            }
                        }
                    }
                }
            }
        }

        values
    }

    /// Extract variable dependencies from an expression
    fn extract_dependencies(&self, expr: &Expr) -> HashSet<String> {
        let mut deps = HashSet::new();
        self.collect_deps(expr, &mut deps);
        deps
    }

    fn collect_deps(&self, expr: &Expr, deps: &mut HashSet<String>) {
        match expr {
            Expr::Number(_) => {}
            Expr::StringLiteral(_) => {}
            Expr::Variable(parts) => {
                // The root variable is the dependency
                if !parts.is_empty() {
                    deps.insert(parts[0].clone());
                }
            }
            Expr::BinaryOp(left, _, right) => {
                self.collect_deps(left, deps);
                self.collect_deps(right, deps);
            }
            Expr::UnaryOp(_, inner) => {
                self.collect_deps(inner, deps);
            }
            Expr::FunctionCall(_, args) => {
                for arg in args {
                    self.collect_deps(arg, deps);
                }
            }
            Expr::List(elements) => {
                for elem in elements {
                    self.collect_deps(elem, deps);
                }
            }
            Expr::FieldAccess(base_expr, _) => {
                // Collect dependencies from the base expression
                self.collect_deps(base_expr, deps);
            }
        }
    }

    /// Topological sort using Kahn's algorithm
    /// Returns Ok(ordered_cells) or Err(cycle_cells)
    fn topological_sort(&self, dependencies: &HashMap<String, Vec<String>>) -> Result<Vec<String>, Vec<String>> {
        // Build in-degree map and reverse dependency map
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut reverse_deps: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize in-degree for all nodes
        for name in dependencies.keys() {
            in_degree.entry(name.clone()).or_insert(0);
            reverse_deps.entry(name.clone()).or_default();
        }

        // Build in-degree counts and reverse mapping
        for (name, deps) in dependencies {
            for dep in deps {
                *in_degree.entry(name.clone()).or_insert(0) += 1;
                reverse_deps.entry(dep.clone()).or_default().push(name.clone());
            }
        }

        // Find all nodes with in-degree 0
        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            if let Some(dependents) = reverse_deps.get(&node) {
                for dependent in dependents {
                    if let Some(deg) = in_degree.get_mut(dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dependent.clone());
                        }
                    }
                }
            }
        }

        // Check for cycles
        if result.len() < dependencies.len() {
            // Find nodes in cycles (those with remaining in-degree > 0)
            let cycle: Vec<String> = in_degree.iter()
                .filter(|(_, &deg)| deg > 0)
                .map(|(name, _)| name.clone())
                .collect();
            Err(cycle)
        } else {
            Ok(result)
        }
    }
    
    /// Evaluate single expression
    fn eval_expr(&self, expr: &Expr, ctx: &EvalContext) -> Value {
        match expr {
            Expr::Number(s) => self.parse_literal(s),

            Expr::StringLiteral(s) => Value::Text(s.clone()),

            Expr::Variable(parts) => {
                let name = parts.join(".");
                let result = ctx.get_var(&name);
                // Add variable name context to errors for better debugging
                if let Value::Error(e) = result {
                    Value::Error(e.with_note(&format!("when resolving '{}'", name)))
                } else {
                    result
                }
            }

            Expr::BinaryOp(left, op, right) => {
                let l = self.eval_expr(left, ctx);
                let r = self.eval_expr(right, ctx);
                self.eval_binary_op(l, *op, r, ctx.precision)
            }

            Expr::UnaryOp(op, inner) => {
                let v = self.eval_expr(inner, ctx);
                self.eval_unary_op(*op, v)
            }

            Expr::FunctionCall(name, args) => {
                let evaluated_args: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a, ctx))
                    .collect();

                // Check for errors in arguments and add function context
                for (i, arg) in evaluated_args.iter().enumerate() {
                    if let Value::Error(e) = arg {
                        return Value::Error(e.clone().with_note(&format!("in argument {} of {}()", i + 1, name)));
                    }
                }

                ctx.registry.call_function(name, &evaluated_args, ctx)
            }

            Expr::List(elements) => {
                let evaluated: Vec<Value> = elements
                    .iter()
                    .map(|e| self.eval_expr(e, ctx))
                    .collect();

                // Check for errors in list elements
                for (i, elem) in evaluated.iter().enumerate() {
                    if let Value::Error(e) = elem {
                        return Value::Error(e.clone().with_note(&format!("in list element {}", i + 1)));
                    }
                }

                Value::List(evaluated)
            }

            Expr::FieldAccess(base_expr, fields) => {
                let base_value = self.eval_expr(base_expr, ctx);

                // If base evaluation resulted in error, propagate it
                if let Value::Error(e) = base_value {
                    return Value::Error(e.with_note(&format!("in field access .{}", fields.join("."))));
                }

                // Navigate through the fields
                let mut current = base_value;
                for field in fields {
                    match &current {
                        Value::Object(map) => {
                            if let Some(value) = map.get(field) {
                                current = value.clone();
                            } else {
                                return Value::Error(
                                    FolioError::new("FIELD_NOT_FOUND", format!("Field '{}' not found in object", field))
                                        .with_suggestion(&format!("Available fields: {}", map.keys().cloned().collect::<Vec<_>>().join(", ")))
                                );
                            }
                        }
                        _ => {
                            return Value::Error(
                                FolioError::new("NOT_OBJECT", format!("Cannot access field '{}' on non-object value", field))
                                    .with_note(&format!("Value is: {:?}", current))
                            );
                        }
                    }
                }
                current
            }
        }
    }
    
    fn parse_literal(&self, s: &str) -> Value {
        match folio_core::Number::from_str(s.trim()) {
            Ok(n) => Value::Number(n),
            Err(_) => Value::Text(s.to_string()),
        }
    }
    
    fn eval_binary_op(&self, left: Value, op: BinOp, right: Value, precision: u32) -> Value {
        // Propagate errors
        if let Value::Error(e) = &left {
            return Value::Error(e.clone().with_note("from left operand"));
        }
        if let Value::Error(e) = &right {
            return Value::Error(e.clone().with_note("from right operand"));
        }

        // Handle DateTime/Duration arithmetic
        match (&left, &right, op) {
            // DateTime + Duration -> DateTime
            (Value::DateTime(dt), Value::Duration(dur), BinOp::Add) => {
                return Value::DateTime(dt.add_duration(dur));
            }
            // Duration + DateTime -> DateTime
            (Value::Duration(dur), Value::DateTime(dt), BinOp::Add) => {
                return Value::DateTime(dt.add_duration(dur));
            }
            // DateTime - Duration -> DateTime
            (Value::DateTime(dt), Value::Duration(dur), BinOp::Sub) => {
                return Value::DateTime(dt.sub_duration(dur));
            }
            // DateTime - DateTime -> Duration
            (Value::DateTime(dt1), Value::DateTime(dt2), BinOp::Sub) => {
                return Value::Duration(dt1.duration_since(dt2));
            }
            // Duration + Duration -> Duration
            (Value::Duration(d1), Value::Duration(d2), BinOp::Add) => {
                return Value::Duration(d1.add(d2));
            }
            // Duration - Duration -> Duration
            (Value::Duration(d1), Value::Duration(d2), BinOp::Sub) => {
                return Value::Duration(d1.sub(d2));
            }
            // Duration * Number -> Duration
            (Value::Duration(dur), Value::Number(n), BinOp::Mul) => {
                if let Some(scalar) = n.to_i64() {
                    return Value::Duration(dur.mul(scalar));
                } else {
                    // Use floating point for non-integer multipliers
                    let f = n.to_f64().unwrap_or(1.0);
                    return Value::Duration(dur.mul_f64(f));
                }
            }
            // Number * Duration -> Duration
            (Value::Number(n), Value::Duration(dur), BinOp::Mul) => {
                if let Some(scalar) = n.to_i64() {
                    return Value::Duration(dur.mul(scalar));
                } else {
                    let f = n.to_f64().unwrap_or(1.0);
                    return Value::Duration(dur.mul_f64(f));
                }
            }
            // Duration / Number -> Duration
            (Value::Duration(dur), Value::Number(n), BinOp::Div) => {
                if let Some(scalar) = n.to_i64() {
                    if scalar == 0 {
                        return Value::Error(FolioError::div_zero());
                    }
                    return Value::Duration(dur.div(scalar).unwrap());
                } else {
                    let f = n.to_f64().unwrap_or(1.0);
                    if f == 0.0 {
                        return Value::Error(FolioError::div_zero());
                    }
                    return Value::Duration(dur.mul_f64(1.0 / f));
                }
            }
            // Duration / Duration -> Number (ratio)
            (Value::Duration(dur1), Value::Duration(dur2), BinOp::Div) => {
                let nanos2 = dur2.as_nanos();
                if nanos2 == 0 {
                    return Value::Error(FolioError::div_zero());
                }
                let nanos1 = dur1.as_nanos();
                // Return the ratio as a Number (truncate to i64)
                let ratio = (nanos1 / nanos2) as i64;
                return Value::Number(Number::from_i64(ratio));
            }
            // Invalid DateTime/Duration operations
            (Value::DateTime(_), Value::DateTime(_), BinOp::Add) => {
                return Value::Error(FolioError::type_error(
                    "Duration (to add to DateTime)", "DateTime"
                ).with_note("cannot add two DateTimes; use dt - dt to get Duration"));
            }
            (Value::DateTime(_), _, BinOp::Mul) | (_, Value::DateTime(_), BinOp::Mul) => {
                return Value::Error(FolioError::type_error(
                    "Number or Duration", "DateTime"
                ).with_note("DateTime cannot be multiplied"));
            }
            (Value::DateTime(_), _, BinOp::Div) => {
                return Value::Error(FolioError::type_error(
                    "Duration", "DateTime"
                ).with_note("DateTime cannot be divided"));
            }
            (Value::DateTime(_), _, BinOp::Pow) | (_, Value::DateTime(_), BinOp::Pow) => {
                return Value::Error(FolioError::type_error(
                    "Number", "DateTime"
                ).with_note("DateTime cannot be used with power operator"));
            }
            (Value::Duration(_), _, BinOp::Pow) | (_, Value::Duration(_), BinOp::Pow) => {
                return Value::Error(FolioError::type_error(
                    "Number", "Duration"
                ).with_note("Duration cannot be used with power operator"));
            }
            // Type mismatch for DateTime/Duration with other types
            (Value::DateTime(_), other, _) if !matches!(other, Value::Duration(_) | Value::DateTime(_)) => {
                return Value::Error(FolioError::type_error("DateTime or Duration", other.type_name()));
            }
            (other, Value::DateTime(_), _) if !matches!(other, Value::Duration(_) | Value::DateTime(_) | Value::Number(_)) => {
                return Value::Error(FolioError::type_error("DateTime, Duration, or Number", other.type_name()));
            }
            (Value::Duration(_), other, _) if !matches!(other, Value::Duration(_) | Value::DateTime(_) | Value::Number(_)) => {
                return Value::Error(FolioError::type_error("DateTime, Duration, or Number", other.type_name()));
            }
            (other, Value::Duration(_), _) if !matches!(other, Value::Duration(_) | Value::DateTime(_) | Value::Number(_)) => {
                return Value::Error(FolioError::type_error("DateTime, Duration, or Number", other.type_name()));
            }
            _ => {}
        }

        // Get numbers (standard numeric operations)
        let l = match left.as_number() {
            Some(n) => n,
            None => return Value::Error(FolioError::type_error("Number", left.type_name())),
        };
        let r = match right.as_number() {
            Some(n) => n,
            None => return Value::Error(FolioError::type_error("Number", right.type_name())),
        };

        // Perform numeric operation
        match op {
            BinOp::Add => Value::Number(l.add(r)),
            BinOp::Sub => Value::Number(l.sub(r)),
            BinOp::Mul => Value::Number(l.mul(r)),
            BinOp::Div => {
                match l.checked_div(r) {
                    Ok(n) => Value::Number(n),
                    Err(e) => Value::Error(e.into()),
                }
            }
            // Comparison operators
            BinOp::Lt => Value::Bool(l.cmp(r) == std::cmp::Ordering::Less),
            BinOp::Gt => Value::Bool(l.cmp(r) == std::cmp::Ordering::Greater),
            BinOp::Le => Value::Bool(l.cmp(r) != std::cmp::Ordering::Greater),
            BinOp::Ge => Value::Bool(l.cmp(r) != std::cmp::Ordering::Less),
            BinOp::Eq => Value::Bool(l.cmp(r) == std::cmp::Ordering::Equal),
            BinOp::Ne => Value::Bool(l.cmp(r) != std::cmp::Ordering::Equal),
            BinOp::Pow => {
                // Power with integer exponent
                if let Some(exp_i64) = r.to_i64() {
                    if exp_i64 >= i32::MIN as i64 && exp_i64 <= i32::MAX as i64 {
                        Value::Number(l.pow(exp_i64 as i32))
                    } else {
                        Value::Error(FolioError::domain_error("exponent too large for integer power"))
                    }
                } else {
                    // Non-integer exponent: x^y = e^(y * ln(x))
                    if l.is_negative() {
                        return Value::Error(FolioError::domain_error(
                            "negative base with non-integer exponent"
                        ));
                    }
                    if l.is_zero() {
                        if r.is_negative() {
                            return Value::Error(FolioError::div_zero()
                                .with_note("0 raised to negative power"));
                        }
                        return Value::Number(Number::from_i64(0));
                    }
                    match l.ln(precision) {
                        Ok(ln_l) => {
                            let y_ln_x = r.mul(&ln_l);
                            Value::Number(y_ln_x.exp(precision))
                        }
                        Err(e) => Value::Error(e.into()),
                    }
                }
            }
        }
    }
    
    fn eval_unary_op(&self, op: UnaryOp, value: Value) -> Value {
        if let Value::Error(e) = &value {
            return Value::Error(e.clone());
        }

        match op {
            UnaryOp::Neg => {
                // Handle Duration negation
                if let Some(d) = value.as_duration() {
                    return Value::Duration(d.neg());
                }
                // Handle DateTime (not allowed)
                if value.is_datetime() {
                    return Value::Error(FolioError::type_error("Number or Duration", "DateTime")
                        .with_note("DateTime cannot be negated"));
                }
                // Handle Number
                match value.as_number() {
                    Some(n) => {
                        let zero = Number::from_i64(0);
                        Value::Number(zero.sub(n))
                    }
                    None => Value::Error(FolioError::type_error("Number", value.type_name())),
                }
            }
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}
