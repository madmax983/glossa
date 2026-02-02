use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, StatementKind,
};
use smol_str::SmolStr;
use std::collections::HashSet;

pub struct Oracle<'a> {
    program: &'a AnalyzedProgram,
}

impl<'a> Oracle<'a> {
    pub fn new(program: &'a AnalyzedProgram) -> Self {
        Self { program }
    }

    pub fn consult(&self) -> Vec<Prophecy> {
        let mut prophecies = Vec::new();
        prophecies.extend(self.check_unused_bindings());
        prophecies.extend(self.check_complexity());
        prophecies.extend(self.check_empty_blocks());
        prophecies
    }

    // Rule 1: The Silent Echo (Unused Bindings)
    fn check_unused_bindings(&self) -> Vec<Prophecy> {
        let mut defined = HashSet::new();
        let mut used = HashSet::new();

        for stmt in &self.program.statements {
            self.collect_definitions(stmt, &mut defined);
            self.collect_usages(stmt, &mut used);
        }

        let mut prophecies = Vec::new();
        for var in defined {
            if !used.contains(&var) {
                prophecies.push(Prophecy {
                    message: format!("The Silent Echo: Variable '{}' is summoned but never speaks.", var),
                    severity: Severity::Warning,
                });
            }
        }
        prophecies
    }

    fn collect_definitions(&self, stmt: &AnalyzedStatement, defined: &mut HashSet<SmolStr>) {
        match &stmt.kind {
            StatementKind::Binding { name, .. } => {
                defined.insert(name.clone());
            }
            StatementKind::FunctionDef {
                name,
                params,
                body,
                ..
            } => {
                // Function name is defined
                defined.insert(name.clone());
                // Params are defined within the function scope (handled locally ideally, but for now global set)
                // Note: This simple approach merges all scopes. For a proper check we need scoped analysis.
                // However, Nova is "Unslop" but "Prototype". Merging scopes might produce false positives if same name used in different scopes.
                // But unused binding check usually implies unique names or careful scoping.
                // Let's refine: We should only check bindings that are unused.
                // If I define 'x' in main and 'x' in func, they are different 'x'.
                // A flat set approach is flawed.
                // Let's try to do it per-scope or just ignore shadowing for the MVP?
                // "Nova's Philosophy: Innovation is connecting two existing modules... even wild ideas must compile and have basic tests."
                // I'll stick to a simpler version: Just traverse and check. If I can't easily handle scopes, I'll just check "globally unique names" assumption or accept false positives.
                // actually, let's just collect all definitions.
                for (param_name, _) in params {
                    defined.insert(param_name.clone());
                }
                for s in body {
                    self.collect_definitions(s, defined);
                }
            }
            StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                for s in then_body {
                    self.collect_definitions(s, defined);
                }
                if let Some(else_body) = else_body {
                    for s in else_body {
                        self.collect_definitions(s, defined);
                    }
                }
            }
            StatementKind::While { body, .. } => {
                for s in body {
                    self.collect_definitions(s, defined);
                }
            }
            StatementKind::For { variable, body, .. } => {
                defined.insert(variable.clone());
                for s in body {
                    self.collect_definitions(s, defined);
                }
            }
            _ => {}
        }
    }

    fn collect_usages(&self, stmt: &AnalyzedStatement, used: &mut HashSet<SmolStr>) {
        // Collect from expressions in this statement, but be careful with Binding/Assignment
        match &stmt.kind {
            StatementKind::Binding { .. } => {
                // Binding: expressions[0] is the target (not a usage), expressions[1] is the value (usage)
                if stmt.expressions.len() >= 2 {
                    self.collect_usages_in_expr(&stmt.expressions[1], used);
                }
            }
            StatementKind::Assignment { .. } => {
                // Assignment: expressions[0] is the target (write, not read), expressions[1] is the value (read)
                if stmt.expressions.len() >= 2 {
                    self.collect_usages_in_expr(&stmt.expressions[1], used);
                }
            }
            _ => {
                // For other statements, all expressions are usages
                for expr in &stmt.expressions {
                    self.collect_usages_in_expr(expr, used);
                }
            }
        }

        // Recurse into nested statements
        match &stmt.kind {
            StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                self.collect_usages_in_expr(condition, used);
                for s in then_body {
                    self.collect_usages(s, used);
                }
                if let Some(else_body) = else_body {
                    for s in else_body {
                        self.collect_usages(s, used);
                    }
                }
            }
            StatementKind::While { condition, body } => {
                self.collect_usages_in_expr(condition, used);
                for s in body {
                    self.collect_usages(s, used);
                }
            }
            StatementKind::For { iterator, body, .. } => {
                self.collect_usages_in_expr(iterator, used);
                for s in body {
                    self.collect_usages(s, used);
                }
            }
            StatementKind::FunctionDef { body, .. } => {
                for s in body {
                    self.collect_usages(s, used);
                }
            }
            StatementKind::Return { value } => {
                if let Some(v) = value {
                    self.collect_usages_in_expr(v, used);
                }
            }
            StatementKind::Match { scrutinee, arms } => {
                self.collect_usages_in_expr(scrutinee, used);
                for (pat, body) in arms {
                    self.collect_usages_in_expr(pat, used);
                    for s in body {
                        self.collect_usages(s, used);
                    }
                }
            }
            _ => {}
        }
    }

    fn collect_usages_in_expr(&self, expr: &AnalyzedExpr, used: &mut HashSet<SmolStr>) {
        match &expr.expr {
            AnalyzedExprKind::Variable(name) => {
                used.insert(name.clone());
            }
            AnalyzedExprKind::PropertyAccess { owner, .. } => {
                self.collect_usages_in_expr(owner, used);
            }
            AnalyzedExprKind::VerbCall { args, .. } => {
                for arg in args {
                    self.collect_usages_in_expr(arg, used);
                }
            }
            AnalyzedExprKind::BinOp { left, right, .. } => {
                self.collect_usages_in_expr(left, used);
                self.collect_usages_in_expr(right, used);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => {
                self.collect_usages_in_expr(operand, used);
            }
            AnalyzedExprKind::Range { start, end, .. } => {
                self.collect_usages_in_expr(start, used);
                self.collect_usages_in_expr(end, used);
            }
            AnalyzedExprKind::ArrayLiteral(elements) => {
                for el in elements {
                    self.collect_usages_in_expr(el, used);
                }
            }
            AnalyzedExprKind::Some(val)
            | AnalyzedExprKind::Ok(val)
            | AnalyzedExprKind::Err(val)
            | AnalyzedExprKind::Unwrap(val)
            | AnalyzedExprKind::Try(val) => {
                self.collect_usages_in_expr(val, used);
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.collect_usages_in_expr(array, used);
                self.collect_usages_in_expr(index, used);
            }
            AnalyzedExprKind::FunctionCall { args, .. } => {
                for arg in args {
                    self.collect_usages_in_expr(arg, used);
                }
            }
            AnalyzedExprKind::MethodCall { receiver, args, .. } => {
                self.collect_usages_in_expr(receiver, used);
                for arg in args {
                    self.collect_usages_in_expr(arg, used);
                }
            }
            AnalyzedExprKind::TraitMethodCall { receiver, args, .. } => {
                self.collect_usages_in_expr(receiver, used);
                for arg in args {
                    self.collect_usages_in_expr(arg, used);
                }
            }
            AnalyzedExprKind::StructInstantiation { args, .. } => {
                for arg in args {
                    self.collect_usages_in_expr(arg, used);
                }
            }
            AnalyzedExprKind::Lambda { body, .. } => {
                self.collect_usages_in_expr(body, used);
            }
            AnalyzedExprKind::IteratorChain { collection, ops } => {
                self.collect_usages_in_expr(collection, used);
                for op in ops {
                    use crate::semantic::AnalyzedIteratorOp::*;
                    match op {
                        Map(e) | Filter(e) | Find(e) | Any(e) | All(e) => {
                            self.collect_usages_in_expr(e, used);
                        }
                        Fold { init, closure } => {
                            self.collect_usages_in_expr(init, used);
                            self.collect_usages_in_expr(closure, used);
                        }
                        Iter | Collect => {}
                    }
                }
            }
            AnalyzedExprKind::CollectionContains {
                collection,
                element,
                ..
            } => {
                self.collect_usages_in_expr(collection, used);
                self.collect_usages_in_expr(element, used);
            }
            _ => {}
        }
    }

    // Rule 2: The Labyrinth (Complexity)
    fn check_complexity(&self) -> Vec<Prophecy> {
        let mut prophecies = Vec::new();
        for stmt in &self.program.statements {
            let depth = self.measure_depth(stmt);
            if depth > 3 {
                // Find a name for context if possible
                let context = match &stmt.kind {
                    StatementKind::FunctionDef { name, .. } => format!("Function '{}'", name),
                    _ => "A block".to_string(),
                };
                prophecies.push(Prophecy {
                    message: format!(
                        "The Labyrinth: {} is too deep (depth {}). The Minotaur awaits.",
                        context, depth
                    ),
                    severity: Severity::Warning,
                });
            }
        }
        prophecies
    }

    fn measure_depth(&self, stmt: &AnalyzedStatement) -> usize {
        match &stmt.kind {
            StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                let then_depth = 1 + then_body
                    .iter()
                    .map(|s| self.measure_depth(s))
                    .max()
                    .unwrap_or(0);
                let else_depth = if let Some(else_body) = else_body {
                    1 + else_body
                        .iter()
                        .map(|s| self.measure_depth(s))
                        .max()
                        .unwrap_or(0)
                } else {
                    0
                };
                std::cmp::max(then_depth, else_depth)
            }
            StatementKind::While { body, .. } => {
                1 + body
                    .iter()
                    .map(|s| self.measure_depth(s))
                    .max()
                    .unwrap_or(0)
            }
            StatementKind::For { body, .. } => {
                1 + body
                    .iter()
                    .map(|s| self.measure_depth(s))
                    .max()
                    .unwrap_or(0)
            }
            StatementKind::FunctionDef { body, .. } => {
                // Function def resets depth? Or adds? Usually top level functions are depth 1 (or 0).
                // But if nested, it adds.
                // Since we iterate top level statements, this will measure depth of function body.
                body.iter().map(|s| self.measure_depth(s)).max().unwrap_or(0)
            }
            _ => 0,
        }
    }

    // Rule 3: The Void (Empty Blocks)
    fn check_empty_blocks(&self) -> Vec<Prophecy> {
        let mut prophecies = Vec::new();
        for stmt in &self.program.statements {
            self.check_empty_blocks_recursive(stmt, &mut prophecies);
        }
        prophecies
    }

    fn check_empty_blocks_recursive(&self, stmt: &AnalyzedStatement, prophecies: &mut Vec<Prophecy>) {
        match &stmt.kind {
            StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                if then_body.is_empty() {
                    prophecies.push(Prophecy {
                        message: "The Void: An 'if' block is empty. It speaks of nothing.".to_string(),
                        severity: Severity::Warning,
                    });
                } else {
                    for s in then_body {
                        self.check_empty_blocks_recursive(s, prophecies);
                    }
                }
                if let Some(else_body) = else_body {
                    if else_body.is_empty() {
                        prophecies.push(Prophecy {
                            message: "The Void: An 'else' block is empty.".to_string(),
                            severity: Severity::Warning,
                        });
                    } else {
                        for s in else_body {
                            self.check_empty_blocks_recursive(s, prophecies);
                        }
                    }
                }
            }
            StatementKind::While { body, .. } => {
                if body.is_empty() {
                    prophecies.push(Prophecy {
                        message: "The Void: A 'while' loop spins on emptiness.".to_string(),
                        severity: Severity::Warning,
                    });
                } else {
                    for s in body {
                        self.check_empty_blocks_recursive(s, prophecies);
                    }
                }
            }
            StatementKind::For { body, variable, .. } => {
                if body.is_empty() {
                    prophecies.push(Prophecy {
                        message: format!("The Void: The loop for '{}' is empty.", variable),
                        severity: Severity::Warning,
                    });
                } else {
                    for s in body {
                        self.check_empty_blocks_recursive(s, prophecies);
                    }
                }
            }
            StatementKind::FunctionDef { name, body, .. } => {
                if body.is_empty() {
                    prophecies.push(Prophecy {
                        message: format!("The Void: Function '{}' is an empty shell.", name),
                        severity: Severity::Warning,
                    });
                } else {
                    for s in body {
                        self.check_empty_blocks_recursive(s, prophecies);
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Prophecy {
    pub message: String,
    pub severity: Severity,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExprKind, GlossaType, StatementKind};

    fn mock_binding(name: &str) -> AnalyzedStatement {
        AnalyzedStatement {
            kind: StatementKind::Binding {
                name: name.into(),
                value_type: GlossaType::Number,
                mutable: false,
            },
            expressions: vec![],
        }
    }

    fn mock_print_var(name: &str) -> AnalyzedStatement {
        AnalyzedStatement {
            kind: StatementKind::Print,
            expressions: vec![AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(name.into()),
                glossa_type: GlossaType::Number,
            }],
        }
    }

    fn mock_program(statements: Vec<AnalyzedStatement>) -> AnalyzedProgram {
        use crate::semantic::Scope;
        AnalyzedProgram {
            statements,
            scope: Scope::new(),
        }
    }

    #[test]
    fn test_unused_binding() {
        // ξ defined but not used
        let prog = mock_program(vec![mock_binding("ξ")]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        assert!(prophecies.iter().any(|p| p.message.contains("ξ")));
    }

    #[test]
    fn test_used_binding() {
        // ξ defined and used
        let prog = mock_program(vec![mock_binding("ξ"), mock_print_var("ξ")]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        assert!(!prophecies.iter().any(|p| p.message.contains("ξ")));
    }

    #[test]
    fn test_empty_function() {
        let stmt = AnalyzedStatement {
            kind: StatementKind::FunctionDef {
                name: "f".into(),
                params: vec![],
                body: vec![],
                return_type: None,
            },
            expressions: vec![],
        };
        let prog = mock_program(vec![stmt]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        assert!(prophecies.iter().any(|p| p.message.contains("empty shell")));
    }
}
