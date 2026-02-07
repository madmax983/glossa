use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, StatementKind,
};
use std::fmt::Write;

/// The Scribe: A storyteller for your code.
pub struct Scribe {
    output: String,
    indent_level: usize,
}

impl Scribe {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
        }
    }
}

impl Default for Scribe {
    fn default() -> Self {
        Self::new()
    }
}

impl Scribe {
    pub fn describe(program: &AnalyzedProgram) -> String {
        let mut scribe = Scribe::new();
        scribe.write_header("The Chronicles of Code");
        scribe.write_line("Here begins the tale of the program.");
        scribe.write_line("");

        for stmt in &program.statements {
            scribe.describe_statement(stmt);
        }

        scribe.write_line("");
        scribe.write_line("Thus ends the tale.");
        scribe.output
    }

    fn write_header(&mut self, text: &str) {
        writeln!(self.output, "# {}\n", text).unwrap();
    }

    fn write_line(&mut self, text: &str) {
        let indent = "  ".repeat(self.indent_level);
        writeln!(self.output, "{}{}", indent, text).unwrap();
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn describe_statement(&mut self, stmt: &AnalyzedStatement) {
        match &stmt.kind {
            StatementKind::Binding {
                name,
                value_type,
                mutable,
            } => {
                let mut desc = format!(
                    "Let there be a {} variable named **{}**",
                    if *mutable { "mutable" } else { "constant" },
                    name
                );

                if let GlossaType::Unknown = value_type {
                    // Type inferred
                } else {
                    write!(desc, ", of type *{:?}*", value_type).unwrap();
                }

                // The second expression is the value (index 1), the first is the variable itself
                if let Some(expr) = stmt.expressions.get(1) {
                    write!(desc, ", initialized to {}", self.describe_expr_inline(expr)).unwrap();
                } else if let Some(expr) = stmt.expressions.first() {
                    // Fallback if only one expression exists (shouldn't happen in standard binding)
                    write!(desc, ", initialized to {}", self.describe_expr_inline(expr)).unwrap();
                }

                desc.push('.');
                self.write_line(&desc);
            }
            StatementKind::Assignment { name, .. } => {
                // The second expression is the value (index 1)
                let val = if let Some(expr) = stmt.expressions.get(1) {
                    self.describe_expr_inline(expr)
                } else if let Some(expr) = stmt.expressions.first() {
                    self.describe_expr_inline(expr)
                } else {
                    "a value".to_string()
                };
                self.write_line(&format!("Update **{}** to become {}.", name, val));
            }
            StatementKind::Print => {
                if let Some(expr) = stmt.expressions.first() {
                    self.write_line(&format!(
                        "The program shall proclaim: {}.",
                        self.describe_expr_inline(expr)
                    ));
                }
            }
            StatementKind::If {
                condition,
                then_body,
                else_body,
            } => {
                self.write_line(&format!(
                    "If indeed {}, then shall the following occur:",
                    self.describe_expr_inline(condition)
                ));
                self.indent();
                for s in then_body {
                    self.describe_statement(s);
                }
                self.dedent();

                if let Some(else_stmts) = else_body {
                    self.write_line("Otherwise:");
                    self.indent();
                    for s in else_stmts {
                        self.describe_statement(s);
                    }
                    self.dedent();
                }
            }
            StatementKind::While { condition, body } => {
                self.write_line(&format!(
                    "So long as {}, repeat these labors:",
                    self.describe_expr_inline(condition)
                ));
                self.indent();
                for s in body {
                    self.describe_statement(s);
                }
                self.dedent();
            }
            StatementKind::For {
                variable,
                iterator,
                body,
            } => {
                self.write_line(&format!(
                    "For each **{}** in {}, perform the following:",
                    variable,
                    self.describe_expr_inline(iterator)
                ));
                self.indent();
                for s in body {
                    self.describe_statement(s);
                }
                self.dedent();
            }
            StatementKind::FunctionDef {
                name,
                params,
                body,
                return_type,
            } => {
                let params_desc: Vec<String> = params
                    .iter()
                    .map(|(n, t)| {
                        if let Some(ty) = t {
                            format!("**{}** ({:?})", n, ty)
                        } else {
                            format!("**{}**", n)
                        }
                    })
                    .collect();

                let ret_desc = if let Some(rt) = return_type {
                    format!(" returning *{:?}*", rt)
                } else {
                    "".to_string()
                };

                self.write_line(&format!(
                    "Define a function named **{}** accepting {}{}:",
                    name,
                    if params_desc.is_empty() {
                        "nothing".to_string()
                    } else {
                        params_desc.join(", ")
                    },
                    ret_desc
                ));

                self.indent();
                for s in body {
                    self.describe_statement(s);
                }
                self.dedent();
            }
            StatementKind::Expression => {
                if let Some(expr) = stmt.expressions.first() {
                    self.write_line(&format!("Evaluate: {}.", self.describe_expr_inline(expr)));
                }
            }
            // Fallback for other kinds
            _ => {
                self.write_line(&format!("Perform a statement of kind: {:?}", stmt.kind));
            }
        }
    }

    fn describe_expr_inline(&self, expr: &AnalyzedExpr) -> String {
        match &expr.expr {
            AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s),
            AnalyzedExprKind::NumberLiteral(n) => n.to_string(),
            AnalyzedExprKind::BooleanLiteral(b) => b.to_string(),
            AnalyzedExprKind::Variable(name) => format!("**{}**", name),
            AnalyzedExprKind::BinOp { left, op, right } => {
                format!(
                    "({} {:?} {})",
                    self.describe_expr_inline(left),
                    op,
                    self.describe_expr_inline(right)
                )
            }
            AnalyzedExprKind::FunctionCall { func, args } => {
                let args_desc: Vec<String> =
                    args.iter().map(|a| self.describe_expr_inline(a)).collect();
                format!("call **{}** with ({})", func, args_desc.join(", "))
            }
            AnalyzedExprKind::MethodCall {
                receiver,
                method,
                args,
            } => {
                let args_desc: Vec<String> =
                    args.iter().map(|a| self.describe_expr_inline(a)).collect();
                format!(
                    "{}.{}({})",
                    self.describe_expr_inline(receiver),
                    method,
                    args_desc.join(", ")
                )
            }
            AnalyzedExprKind::VerbCall { verb, args } => {
                let args_desc: Vec<String> =
                    args.iter().map(|a| self.describe_expr_inline(a)).collect();
                format!("verb **{}** acting on ({})", verb, args_desc.join(", "))
            }
            _ => "complex expression".to_string(),
        }
    }
}

/// Public API to describe a program
pub fn describe_program(program: &AnalyzedProgram) -> String {
    Scribe::describe(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{Scope, StatementKind};
    use smol_str::SmolStr;
    use crate::morphology::lexicon::BinaryOp;

    fn create_program(statements: Vec<AnalyzedStatement>) -> AnalyzedProgram {
        AnalyzedProgram {
            statements,
            scope: Scope::new(),
        }
    }

    fn simple_expr(kind: AnalyzedExprKind) -> AnalyzedExpr {
        AnalyzedExpr {
            expr: kind,
            glossa_type: GlossaType::Unknown,
        }
    }

    #[test]
    fn test_describe_binding() {
        let stmt = AnalyzedStatement {
            kind: StatementKind::Binding {
                name: SmolStr::new("x"),
                value_type: GlossaType::Number,
                mutable: false,
            },
            expressions: vec![
                simple_expr(AnalyzedExprKind::Variable(SmolStr::new("x"))),
                simple_expr(AnalyzedExprKind::NumberLiteral(42)),
            ],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("Let there be a constant variable named **x**"));
        assert!(desc.contains("of type *Number*"));
        assert!(desc.contains("initialized to 42"));
    }

    #[test]
    fn test_describe_binding_mutable() {
        let stmt = AnalyzedStatement {
            kind: StatementKind::Binding {
                name: SmolStr::new("y"),
                value_type: GlossaType::String,
                mutable: true,
            },
            expressions: vec![
                simple_expr(AnalyzedExprKind::Variable(SmolStr::new("y"))),
                simple_expr(AnalyzedExprKind::StringLiteral("test".to_string())),
            ],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("Let there be a mutable variable named **y**"));
        assert!(desc.contains("of type *String*"));
    }

    #[test]
    fn test_describe_assignment() {
        let stmt = AnalyzedStatement {
            kind: StatementKind::Assignment {
                name: SmolStr::new("x"),
                value_type: GlossaType::Number,
            },
            expressions: vec![
                simple_expr(AnalyzedExprKind::Variable(SmolStr::new("x"))),
                simple_expr(AnalyzedExprKind::NumberLiteral(100)),
            ],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("Update **x** to become 100."));
    }

    #[test]
    fn test_describe_print() {
        let stmt = AnalyzedStatement {
            kind: StatementKind::Print,
            expressions: vec![simple_expr(AnalyzedExprKind::StringLiteral("Hello".to_string()))],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("The program shall proclaim: \"Hello\"."));
    }

    #[test]
    fn test_describe_if() {
        let condition = simple_expr(AnalyzedExprKind::BooleanLiteral(true));
        let then_stmt = AnalyzedStatement {
            kind: StatementKind::Print,
            expressions: vec![simple_expr(AnalyzedExprKind::StringLiteral("True".to_string()))],
        };

        let stmt = AnalyzedStatement {
            kind: StatementKind::If {
                condition: Box::new(condition),
                then_body: vec![then_stmt],
                else_body: None,
            },
            expressions: vec![],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("If indeed true, then shall the following occur:"));
        assert!(desc.contains("The program shall proclaim: \"True\"."));
    }

    #[test]
    fn test_describe_if_else() {
        let condition = simple_expr(AnalyzedExprKind::BooleanLiteral(false));
        let then_stmt = AnalyzedStatement {
            kind: StatementKind::Print,
            expressions: vec![simple_expr(AnalyzedExprKind::StringLiteral("True".to_string()))],
        };
        let else_stmt = AnalyzedStatement {
            kind: StatementKind::Print,
            expressions: vec![simple_expr(AnalyzedExprKind::StringLiteral("False".to_string()))],
        };

        let stmt = AnalyzedStatement {
            kind: StatementKind::If {
                condition: Box::new(condition),
                then_body: vec![then_stmt],
                else_body: Some(vec![else_stmt]),
            },
            expressions: vec![],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("Otherwise:"));
        assert!(desc.contains("The program shall proclaim: \"False\"."));
    }

    #[test]
    fn test_describe_while() {
        let condition = simple_expr(AnalyzedExprKind::BooleanLiteral(true));
        let body_stmt = AnalyzedStatement {
            kind: StatementKind::Print,
            expressions: vec![simple_expr(AnalyzedExprKind::StringLiteral("Loop".to_string()))],
        };

        let stmt = AnalyzedStatement {
            kind: StatementKind::While {
                condition: Box::new(condition),
                body: vec![body_stmt],
            },
            expressions: vec![],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("So long as true, repeat these labors:"));
        assert!(desc.contains("The program shall proclaim: \"Loop\"."));
    }

    #[test]
    fn test_describe_for() {
        let iterator = simple_expr(AnalyzedExprKind::Variable(SmolStr::new("items")));
        let body_stmt = AnalyzedStatement {
            kind: StatementKind::Print,
            expressions: vec![simple_expr(AnalyzedExprKind::StringLiteral("Item".to_string()))],
        };

        let stmt = AnalyzedStatement {
            kind: StatementKind::For {
                variable: SmolStr::new("item"),
                iterator: Box::new(iterator),
                body: vec![body_stmt],
            },
            expressions: vec![],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("For each **item** in **items**, perform the following:"));
        assert!(desc.contains("The program shall proclaim: \"Item\"."));
    }

    #[test]
    fn test_describe_function_def() {
        let body_stmt = AnalyzedStatement {
            kind: StatementKind::Print,
            expressions: vec![simple_expr(AnalyzedExprKind::StringLiteral("Fn".to_string()))],
        };

        let stmt = AnalyzedStatement {
            kind: StatementKind::FunctionDef {
                name: SmolStr::new("my_func"),
                params: vec![
                    (SmolStr::new("p1"), Some(GlossaType::Number)),
                    (SmolStr::new("p2"), None),
                ],
                body: vec![body_stmt],
                return_type: Some(GlossaType::Boolean),
            },
            expressions: vec![],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("Define a function named **my_func** accepting **p1** (Number), **p2** returning *Boolean*:"));
    }

    #[test]
    fn test_describe_expression_statement() {
        let stmt = AnalyzedStatement {
            kind: StatementKind::Expression,
            expressions: vec![simple_expr(AnalyzedExprKind::NumberLiteral(123))],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("Evaluate: 123."));
    }

    #[test]
    fn test_describe_expressions_complex() {
        let bin_op = simple_expr(AnalyzedExprKind::BinOp {
            left: Box::new(simple_expr(AnalyzedExprKind::NumberLiteral(1))),
            op: BinaryOp::Add,
            right: Box::new(simple_expr(AnalyzedExprKind::NumberLiteral(2))),
        });

        let func_call = simple_expr(AnalyzedExprKind::FunctionCall {
            func: SmolStr::new("call_me"),
            args: vec![bin_op],
        });

        let stmt = AnalyzedStatement {
            kind: StatementKind::Expression,
            expressions: vec![func_call],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("Evaluate: call **call_me** with ((1 Add 2))."));
    }

    #[test]
    fn test_describe_method_call() {
        let method_call = simple_expr(AnalyzedExprKind::MethodCall {
            receiver: Box::new(simple_expr(AnalyzedExprKind::Variable(SmolStr::new("obj")))),
            method: SmolStr::new("method"),
            args: vec![],
        });

        let stmt = AnalyzedStatement {
            kind: StatementKind::Expression,
            expressions: vec![method_call],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("**obj**.method()"));
    }

    #[test]
    fn test_describe_verb_call() {
        let verb_call = simple_expr(AnalyzedExprKind::VerbCall {
            verb: SmolStr::new("do_action"),
            args: vec![simple_expr(AnalyzedExprKind::NumberLiteral(5))],
        });

        let stmt = AnalyzedStatement {
            kind: StatementKind::Expression,
            expressions: vec![verb_call],
        };

        let desc = describe_program(&create_program(vec![stmt]));
        assert!(desc.contains("verb **do_action** acting on (5)"));
    }

    #[test]
    fn test_default_impl() {
        let scribe = Scribe::default();
        assert_eq!(scribe.indent_level, 0);
    }
}
