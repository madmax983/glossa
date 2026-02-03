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
                    message: format!(
                        "The Silent Echo: Variable '{}' is summoned but never speaks.",
                        var
                    ),
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
                name, params, body, ..
            } => {
                // Function name is defined
                defined.insert(name.clone());
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
            StatementKind::TraitImplementation { methods, .. } => {
                for method in methods {
                    for (param_name, _) in &method.params {
                        defined.insert(param_name.clone());
                    }
                    for s in &method.body {
                        self.collect_definitions(s, defined);
                    }
                }
            }
            StatementKind::TraitDefinition { methods, .. } => {
                for method in methods {
                    for (param_name, _) in &method.params {
                        defined.insert(param_name.clone());
                    }
                    if let Some(body) = &method.body {
                        for s in body {
                            self.collect_definitions(s, defined);
                        }
                    }
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
            StatementKind::Return { value: Some(v) } => {
                self.collect_usages_in_expr(v, used);
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
            StatementKind::TraitImplementation { methods, .. } => {
                for method in methods {
                    for s in &method.body {
                        self.collect_usages(s, used);
                    }
                }
            }
            StatementKind::TraitDefinition { methods, .. } => {
                for method in methods {
                    if let Some(body) = &method.body {
                        for s in body {
                            self.collect_usages(s, used);
                        }
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
            AnalyzedExprKind::CollectionNew { .. } => {}
            AnalyzedExprKind::StringLiteral(_)
            | AnalyzedExprKind::NumberLiteral(_)
            | AnalyzedExprKind::BooleanLiteral(_)
            | AnalyzedExprKind::None
            | AnalyzedExprKind::Literal(_) => {}
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
                body.iter()
                    .map(|s| self.measure_depth(s))
                    .max()
                    .unwrap_or(0)
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

    fn check_empty_blocks_recursive(
        &self,
        stmt: &AnalyzedStatement,
        prophecies: &mut Vec<Prophecy>,
    ) {
        match &stmt.kind {
            StatementKind::If {
                then_body,
                else_body,
                ..
            } => {
                if then_body.is_empty() {
                    prophecies.push(Prophecy {
                        message: "The Void: An 'if' block is empty. It speaks of nothing."
                            .to_string(),
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
            expressions: vec![
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(name.into()),
                    glossa_type: GlossaType::Number,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(5),
                    glossa_type: GlossaType::Number,
                },
            ],
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

    #[test]
    fn test_nested_usage_in_if() {
        let binding = mock_binding("ξ");
        let if_stmt = AnalyzedStatement {
            kind: StatementKind::If {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                then_body: vec![mock_print_var("ξ")],
                else_body: None,
            },
            expressions: vec![],
        };

        let prog = mock_program(vec![binding, if_stmt]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        assert!(
            !prophecies.iter().any(|p| p.message.contains("ξ")),
            "Variable used in if block should be marked used"
        );
    }

    #[test]
    fn test_complexity_check() {
        // Create deep nesting: If -> While -> For -> If (Depth 4)
        let deep_stmt = AnalyzedStatement {
            kind: StatementKind::If {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                then_body: vec![AnalyzedStatement {
                    kind: StatementKind::While {
                        condition: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::BooleanLiteral(true),
                            glossa_type: GlossaType::Boolean,
                        }),
                        body: vec![AnalyzedStatement {
                            kind: StatementKind::For {
                                variable: "i".into(),
                                iterator: Box::new(AnalyzedExpr {
                                    expr: AnalyzedExprKind::Range {
                                        start: Box::new(AnalyzedExpr {
                                            expr: AnalyzedExprKind::NumberLiteral(0),
                                            glossa_type: GlossaType::Number,
                                        }),
                                        end: Box::new(AnalyzedExpr {
                                            expr: AnalyzedExprKind::NumberLiteral(10),
                                            glossa_type: GlossaType::Number,
                                        }),
                                        inclusive: false,
                                    },
                                    glossa_type: GlossaType::Unknown,
                                }),
                                body: vec![AnalyzedStatement {
                                    kind: StatementKind::If {
                                        condition: Box::new(AnalyzedExpr {
                                            expr: AnalyzedExprKind::BooleanLiteral(true),
                                            glossa_type: GlossaType::Boolean,
                                        }),
                                        then_body: vec![], // Empty to also trigger void check
                                        else_body: None,
                                    },
                                    expressions: vec![],
                                }],
                            },
                            expressions: vec![],
                        }],
                    },
                    expressions: vec![],
                }],
                else_body: None,
            },
            expressions: vec![],
        };

        let prog = mock_program(vec![deep_stmt]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        assert!(
            prophecies.iter().any(|p| p.message.contains("Labyrinth")),
            "Should detect high complexity"
        );
    }

    #[test]
    fn test_unused_function_param() {
        let func = AnalyzedStatement {
            kind: StatementKind::FunctionDef {
                name: "f".into(),
                params: vec![("unused_param".into(), Some(GlossaType::Number))],
                body: vec![mock_print_var("other_var")], // Using something else
                return_type: None,
            },
            expressions: vec![],
        };

        let prog = mock_program(vec![func]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        assert!(
            prophecies
                .iter()
                .any(|p| p.message.contains("unused_param")),
            "Should detect unused function parameter"
        );
    }

    #[test]
    fn test_exhaustive_usages() {
        // This test aims to hit every match arm in collect_usages_in_expr
        // We create a statement that uses a variable 'x' in every possible way.
        // If the oracle counts 'x' as used, we know the visitor worked.

        let var_x = AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("x".into()),
            glossa_type: GlossaType::Number,
        };
        let var_y = AnalyzedExpr {
            expr: AnalyzedExprKind::Variable("y".into()),
            glossa_type: GlossaType::Number,
        };

        let expressions = vec![
            // PropertyAccess
            AnalyzedExpr {
                expr: AnalyzedExprKind::PropertyAccess {
                    owner: Box::new(var_x.clone()),
                    property: "prop".into(),
                },
                glossa_type: GlossaType::Unknown,
            },
            // VerbCall
            AnalyzedExpr {
                expr: AnalyzedExprKind::VerbCall {
                    verb: "run".into(),
                    args: vec![var_x.clone()],
                },
                glossa_type: GlossaType::Unit,
            },
            // BinOp
            AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(var_x.clone()),
                    op: crate::morphology::lexicon::BinaryOp::Add,
                    right: Box::new(var_y.clone()),
                },
                glossa_type: GlossaType::Number,
            },
            // UnaryOp
            AnalyzedExpr {
                expr: AnalyzedExprKind::UnaryOp {
                    op: crate::morphology::lexicon::UnaryOp::Neg,
                    operand: Box::new(var_x.clone()),
                },
                glossa_type: GlossaType::Number,
            },
            // Range
            AnalyzedExpr {
                expr: AnalyzedExprKind::Range {
                    start: Box::new(var_x.clone()),
                    end: Box::new(var_y.clone()),
                    inclusive: true,
                },
                glossa_type: GlossaType::Unknown,
            },
            // ArrayLiteral
            AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(vec![var_x.clone()]),
                glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
            },
            // Wrappers
            AnalyzedExpr {
                expr: AnalyzedExprKind::Some(Box::new(var_x.clone())),
                glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
            },
            // IndexAccess
            AnalyzedExpr {
                expr: AnalyzedExprKind::IndexAccess {
                    array: Box::new(var_y.clone()),
                    index: Box::new(var_x.clone()),
                },
                glossa_type: GlossaType::Number,
            },
            // FunctionCall
            AnalyzedExpr {
                expr: AnalyzedExprKind::FunctionCall {
                    func: "f".into(),
                    args: vec![var_x.clone()],
                },
                glossa_type: GlossaType::Unit,
            },
            // MethodCall
            AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(var_y.clone()),
                    method: "m".into(),
                    args: vec![var_x.clone()],
                },
                glossa_type: GlossaType::Unit,
            },
            // TraitMethodCall
            AnalyzedExpr {
                expr: AnalyzedExprKind::TraitMethodCall {
                    receiver: Box::new(var_y.clone()),
                    trait_name: "T".into(),
                    method_name: "m".into(),
                    args: vec![var_x.clone()],
                },
                glossa_type: GlossaType::Unit,
            },
            // StructInstantiation
            AnalyzedExpr {
                expr: AnalyzedExprKind::StructInstantiation {
                    type_name: "S".into(),
                    fields: vec!["f".into()],
                    args: vec![var_x.clone()],
                },
                glossa_type: GlossaType::Unknown,
            },
            // Lambda
            AnalyzedExpr {
                expr: AnalyzedExprKind::Lambda {
                    params: vec![],
                    body: Box::new(var_x.clone()),
                    capture_mode: crate::ast::CaptureMode::Move,
                },
                glossa_type: GlossaType::Unknown,
            },
            // IteratorChain
            AnalyzedExpr {
                expr: AnalyzedExprKind::IteratorChain {
                    collection: Box::new(var_y.clone()),
                    ops: vec![crate::semantic::AnalyzedIteratorOp::Map(Box::new(
                        var_x.clone(),
                    ))],
                },
                glossa_type: GlossaType::Unknown,
            },
            // CollectionContains
            AnalyzedExpr {
                expr: AnalyzedExprKind::CollectionContains {
                    collection: Box::new(var_y.clone()),
                    element: Box::new(var_x.clone()),
                    is_map: false,
                },
                glossa_type: GlossaType::Boolean,
            },
        ];

        // Define 'x' and 'y' and usage
        let binding_x = mock_binding("x");
        let binding_y = mock_binding("y");

        let usage_stmt = AnalyzedStatement {
            kind: StatementKind::Expression,
            expressions,
        };

        let prog = mock_program(vec![binding_x, binding_y, usage_stmt]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        // If 'x' or 'y' were unused, we'd get a warning.
        // We assert NO warnings about x or y.
        assert!(
            !prophecies.iter().any(|p| p.message.contains("'x'")),
            "Variable x should be considered used"
        );
        assert!(
            !prophecies.iter().any(|p| p.message.contains("'y'")),
            "Variable y should be considered used"
        );
    }

    #[test]
    fn test_exhaustive_definitions() {
        // Test Trait definitions and implementations
        use crate::semantic::{AnalyzedImplMethod, AnalyzedTraitMethod};

        let trait_def = AnalyzedStatement {
            kind: StatementKind::TraitDefinition {
                name: "T".into(),
                methods: vec![
                    AnalyzedTraitMethod {
                        name: "m".into(),
                        params: vec![("p1".into(), GlossaType::Number)],
                        is_default: true,
                        body: Some(vec![
                            // Nested usage to test recursion
                            AnalyzedStatement {
                                kind: StatementKind::If {
                                    condition: Box::new(AnalyzedExpr {
                                        expr: AnalyzedExprKind::BooleanLiteral(true),
                                        glossa_type: GlossaType::Boolean,
                                    }),
                                    then_body: vec![mock_print_var("p1")],
                                    else_body: None,
                                },
                                expressions: vec![],
                            },
                        ]),
                        return_type: None,
                    },
                    // Method with no body (abstract)
                    AnalyzedTraitMethod {
                        name: "abs".into(),
                        params: vec![("p3".into(), GlossaType::Number)],
                        is_default: false,
                        body: None,
                        return_type: None,
                    },
                ],
            },
            expressions: vec![],
        };

        let trait_impl = AnalyzedStatement {
            kind: StatementKind::TraitImplementation {
                trait_name: "T".into(),
                type_name: "S".into(),
                methods: vec![AnalyzedImplMethod {
                    name: "m".into(),
                    params: vec![("p2".into(), GlossaType::Number)],
                    body: vec![mock_print_var("p2")],
                    return_type: None,
                }],
            },
            expressions: vec![],
        };

        let prog = mock_program(vec![trait_def, trait_impl]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        // p1 and p2 are defined and used. Should be no warnings.
        assert!(
            !prophecies.iter().any(|p| p.message.contains("'p1'")),
            "Param p1 should be detected as used"
        );
        assert!(
            !prophecies.iter().any(|p| p.message.contains("'p2'")),
            "Param p2 should be detected as used"
        );
        // p3 is defined (abstract method param) but has no body to use it in.
        // It technically counts as "unused" in this simple model.
        assert!(
            prophecies.iter().any(|p| p.message.contains("'p3'")),
            "Abstract param p3 should be detected as unused (since it has no body)"
        );
    }

    #[test]
    fn test_assignment_and_match_usage() {
        let binding_x = mock_binding("x");
        let binding_y = mock_binding("y");

        let assign = AnalyzedStatement {
            kind: StatementKind::Assignment {
                name: "x".into(),
                value_type: GlossaType::Number,
            },
            expressions: vec![
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("x".into()),
                    glossa_type: GlossaType::Number,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("y".into()), // Use y
                    glossa_type: GlossaType::Number,
                },
            ],
        };

        let match_stmt = AnalyzedStatement {
            kind: StatementKind::Match {
                scrutinee: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("x".into()), // Use x
                    glossa_type: GlossaType::Number,
                }),
                arms: vec![(
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    },
                    vec![],
                )],
            },
            expressions: vec![],
        };

        let prog = mock_program(vec![binding_x, binding_y, assign, match_stmt]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        assert!(!prophecies.iter().any(|p| p.message.contains("'x'")));
        assert!(!prophecies.iter().any(|p| p.message.contains("'y'")));
    }

    #[test]
    fn test_exhaustive_control_flow() {
        // Test If, While, For to ensure their definitions and usages are collected
        let var_used = mock_binding("used_var");
        let var_unused = mock_binding("unused_var");

        // IF
        let if_stmt = AnalyzedStatement {
            kind: StatementKind::If {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("used_var".into()),
                    glossa_type: GlossaType::Boolean,
                }),
                then_body: vec![mock_print_var("used_var")],
                else_body: Some(vec![mock_print_var("used_var")]),
            },
            expressions: vec![],
        };

        // WHILE
        let while_stmt = AnalyzedStatement {
            kind: StatementKind::While {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("used_var".into()),
                    glossa_type: GlossaType::Boolean,
                }),
                body: vec![mock_print_var("used_var")],
            },
            expressions: vec![],
        };

        // FOR
        let for_stmt = AnalyzedStatement {
            kind: StatementKind::For {
                variable: "loop_var".into(),
                iterator: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("used_var".into()),
                    glossa_type: GlossaType::Unknown,
                }),
                body: vec![mock_print_var("loop_var")],
            },
            expressions: vec![],
        };

        let prog = mock_program(vec![var_used, var_unused, if_stmt, while_stmt, for_stmt]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        // used_var should be used
        assert!(!prophecies.iter().any(|p| p.message.contains("'used_var'")));

        // unused_var should be unused
        assert!(
            prophecies
                .iter()
                .any(|p| p.message.contains("'unused_var'"))
        );

        // loop_var is defined in For and used in body. Should NOT be unused.
        assert!(!prophecies.iter().any(|p| p.message.contains("'loop_var'")));
    }

    #[test]
    fn test_type_definitions() {
        let type_def = AnalyzedStatement {
            kind: StatementKind::TypeDefinition {
                name: "User".into(),
                fields: vec![("name".into(), GlossaType::String)],
            },
            expressions: vec![],
        };

        // Type definitions don't define variables in the runtime binding sense for this Oracle,
        // but we should ensure it doesn't crash or falsely report.
        // Actually, the current oracle ignores TypeDefinition in collect_definitions.
        // So this test mainly ensures we hit the catch-all `_ => {}` or similar branches safely.

        let prog = mock_program(vec![type_def]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        assert!(prophecies.is_empty());
    }

    #[test]
    fn test_return_none_and_break_continue() {
        let ret = AnalyzedStatement {
            kind: StatementKind::Return { value: None },
            expressions: vec![],
        };
        let brk = AnalyzedStatement {
            kind: StatementKind::Break,
            expressions: vec![],
        };
        let cont = AnalyzedStatement {
            kind: StatementKind::Continue,
            expressions: vec![],
        };

        let prog = mock_program(vec![ret, brk, cont]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        assert!(prophecies.is_empty());
    }

    #[test]
    fn test_print_statement() {
        let print = AnalyzedStatement {
            kind: StatementKind::Print,
            expressions: vec![AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral("hello".into()),
                glossa_type: GlossaType::String,
            }],
        };
        let prog = mock_program(vec![print]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();
        assert!(prophecies.is_empty());
    }

    #[test]
    fn test_expression_statement() {
        let expr_stmt = AnalyzedStatement {
            kind: StatementKind::Expression,
            expressions: vec![AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(42),
                glossa_type: GlossaType::Number,
            }],
        };
        let prog = mock_program(vec![expr_stmt]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();
        assert!(prophecies.is_empty());
    }

    #[test]
    fn test_query_statement() {
        let query = AnalyzedStatement {
            kind: StatementKind::Query,
            expressions: vec![AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("x".into()),
                glossa_type: GlossaType::Number,
            }],
        };
        // x is used here. If x was defined, it would be marked used.
        // Since x isn't defined, we get no "unused variable" warning.
        // And no other warnings.
        let prog = mock_program(vec![query]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();
        assert!(prophecies.is_empty());
    }

    #[test]
    fn test_trait_definition_no_body() {
        use crate::semantic::AnalyzedTraitMethod;
        let trait_def = AnalyzedStatement {
            kind: StatementKind::TraitDefinition {
                name: "T".into(),
                methods: vec![AnalyzedTraitMethod {
                    name: "m".into(),
                    params: vec![("p".into(), GlossaType::Number)],
                    is_default: false,
                    body: None, // No body
                    return_type: None,
                }],
            },
            expressions: vec![],
        };
        let prog = mock_program(vec![trait_def]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        // p is defined but has no usage (no body). Should be unused.
        assert!(prophecies.iter().any(|p| p.message.contains("'p'")));
    }

    #[test]
    fn test_malformed_binding() {
        // Binding with missing value expression (shouldn't happen in valid AST, but defensive check exists)
        let binding = AnalyzedStatement {
            kind: StatementKind::Binding {
                name: "x".into(),
                value_type: GlossaType::Number,
                mutable: false,
            },
            expressions: vec![
                // Only target, no value
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("x".into()),
                    glossa_type: GlossaType::Number,
                },
            ],
        };
        let prog = mock_program(vec![binding]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        // x is defined. Usage check skips expressions[1] because it doesn't exist.
        // So x is unused.
        assert!(prophecies.iter().any(|p| p.message.contains("'x'")));
    }

    #[test]
    fn test_malformed_assignment() {
        // Assignment with missing value expression
        let assign = AnalyzedStatement {
            kind: StatementKind::Assignment {
                name: "x".into(),
                value_type: GlossaType::Number,
            },
            expressions: vec![
                // Only target, no value
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable("x".into()),
                    glossa_type: GlossaType::Number,
                },
            ],
        };
        let prog = mock_program(vec![assign]);
        let oracle = Oracle::new(&prog);
        let prophecies = oracle.consult();

        // Assignment doesn't define x. It tries to use x (in target) but assignment logic
        // typically treats target as definition or mutation, not read usage.
        // In collect_usages: Assignment: expressions[0] (target) is skipped, expressions[1] (value) is read.
        // Since expressions[1] is missing, NO usage is recorded.
        // Since x isn't defined here, no "unused variable" warning.
        assert!(prophecies.is_empty());
    }
}
