//! Document evaluator
//!
//! Evaluates document expressions in dependency order.

use crate::ast::{Document, Expr, BinOp, UnaryOp};
use folio_plugin::EvalContext;
use folio_core::{Value, FolioError};
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
            
            Expr::Variable(parts) => {
                let name = parts.join(".");
                ctx.get_var(&name)
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
                
                ctx.registry.call_function(name, &evaluated_args, ctx)
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
        
        // Get numbers
        let l = match left.as_number() {
            Some(n) => n,
            None => return Value::Error(FolioError::type_error("Number", left.type_name())),
        };
        let r = match right.as_number() {
            Some(n) => n,
            None => return Value::Error(FolioError::type_error("Number", right.type_name())),
        };
        
        // Perform operation
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
                        return Value::Number(folio_core::Number::from_i64(0));
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
                match value.as_number() {
                    Some(n) => {
                        let zero = folio_core::Number::from_i64(0);
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
