//! The Scribe (ὁ Γραμματεύς) - Report generation for ΓΛΩΣΣΑ
//!
//! This module provides tools to generate human-readable reports and statistics
//! for analyzed programs.
//!
//! # The Mission
//!
//! Compiler feedback should be more than just "Error" or "Success".
//! ΓΛΩΣΣΑ strives to provide **insights** into the code:
//!
//! * How complex is the program? (Cyclomatic complexity proxy via `conditional_count`)
//! * How deep is the nesting? (`max_depth`)
//! * How "functional" vs "imperative" is the style? (Expression vs Statement count)
//!
//! The [`GlossaReport`] provides a dashboard-style summary of these metrics.

use comfy_table::{Cell, Color, Table, presets};
use crossterm::style::Stylize;
use std::fmt::Display;
use std::path::PathBuf;
use std::time::Duration;

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};

/// Statistics for an analyzed program
///
/// These metrics help users understand the scale and complexity of their code.
#[derive(Debug, Default, Clone)]
pub struct ProgramStats {
    /// Total number of statements (imperative actions)
    pub statement_count: usize,
    /// Total number of expressions (values evaluated)
    pub expression_count: usize,
    /// Number of variable bindings (let x = ...)
    pub binding_count: usize,
    /// Number of functions defined in the scope
    pub function_count: usize,
    /// Number of user-defined types (structs)
    pub type_count: usize,
    /// Number of trait definitions
    #[allow(dead_code)]
    pub trait_count: usize,
    /// Number of loops (while, for)
    pub loop_count: usize,
    /// Number of conditional branches (if, match) - a proxy for complexity
    pub conditional_count: usize,
    /// Maximum nesting depth of statements
    pub max_depth: usize,
}

impl ProgramStats {
    /// Analyze a program and collect statistics
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::parser::parse;
    /// use glossa::semantic::analyze_program;
    /// use glossa::tools::report::ProgramStats;
    ///
    /// let source = "ξ πέντε ἔστω.";
    /// let ast = parse(source).unwrap();
    /// let program = analyze_program(&ast).unwrap();
    /// let stats = ProgramStats::new(&program);
    ///
    /// assert_eq!(stats.statement_count, 1);
    /// assert_eq!(stats.binding_count, 1);
    /// ```
    pub fn new(program: &AnalyzedProgram) -> Self {
        let mut stats = ProgramStats {
            function_count: program.scope.functions().count(),
            type_count: program.scope.types().count(),
            trait_count: program.scope.traits().count(),
            ..ProgramStats::default()
        };

        // Traverse statements to count structural elements
        for stmt in &program.statements {
            stats.visit_statement(stmt, 0);
        }

        stats
    }

    fn visit_statement(&mut self, stmt: &AnalyzedStatement, depth: usize) {
        self.statement_count += 1;
        self.max_depth = self.max_depth.max(depth);

        match stmt {
            AnalyzedStatement::Binding { value, .. } => {
                self.binding_count += 1;
                self.visit_expr(value);
            }
            AnalyzedStatement::Assignment { value, .. } => {
                self.visit_expr(value);
            }
            AnalyzedStatement::Print(exprs)
            | AnalyzedStatement::Expression(exprs)
            | AnalyzedStatement::Query(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.conditional_count += 1;
                self.visit_expr(condition);
                for s in then_body {
                    self.visit_statement(s, depth + 1);
                }
                if let Some(else_body) = else_body {
                    for s in else_body {
                        self.visit_statement(s, depth + 1);
                    }
                }
            }
            AnalyzedStatement::While { condition, body } => {
                self.loop_count += 1;
                self.visit_expr(condition);
                for s in body {
                    self.visit_statement(s, depth + 1);
                }
            }
            AnalyzedStatement::For { iterator, body, .. } => {
                self.loop_count += 1;
                self.visit_expr(iterator);
                for s in body {
                    self.visit_statement(s, depth + 1);
                }
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                self.conditional_count += 1;
                self.visit_expr(scrutinee);
                for (pat, body) in arms {
                    self.visit_expr(pat);
                    for s in body {
                        self.visit_statement(s, depth + 1);
                    }
                }
            }
            AnalyzedStatement::FunctionDef { body, .. } => {
                // Function definitions in statements (if any) are already counted in scope?
                // `analyze_statement` returns FunctionDef for control flow analysis
                // But it also adds to scope.
                // We shouldn't double count. Scope count is authoritative for "defined functions".
                // But we should traverse the body for complexity stats.
                for s in body {
                    self.visit_statement(s, depth + 1);
                }
            }
            AnalyzedStatement::TestDeclaration { body, .. } => {
                for s in body {
                    self.visit_statement(s, depth + 1);
                }
            }
            AnalyzedStatement::Return { value } => {
                if let Some(v) = value {
                    self.visit_expr(v);
                }
            }
            AnalyzedStatement::Break | AnalyzedStatement::Continue => {}
            AnalyzedStatement::TypeDefinition { .. } => {} // Already counted in scope
            AnalyzedStatement::TraitDefinition { .. } => {} // Already counted in scope
            AnalyzedStatement::TraitImplementation { methods, .. } => {
                // Visit trait method implementations to get complete expression coverage
                for method in methods {
                    if let Some(body) = &method.body {
                        for s in body {
                            self.visit_statement(s, depth + 1);
                        }
                    }
                }
            }
        }
    }

    fn visit_expr(&mut self, expr: &AnalyzedExpr) {
        self.expression_count += 1;
        match &expr.expr {
            AnalyzedExprKind::PropertyAccess { owner, .. } => self.visit_expr(owner),
            AnalyzedExprKind::VerbCall { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::BinOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => self.visit_expr(operand),
            AnalyzedExprKind::Range { start, end, .. } => {
                self.visit_expr(start);
                self.visit_expr(end);
            }
            AnalyzedExprKind::ArrayLiteral(exprs) => {
                for e in exprs {
                    self.visit_expr(e);
                }
            }
            AnalyzedExprKind::Some(e)
            | AnalyzedExprKind::Ok(e)
            | AnalyzedExprKind::Err(e)
            | AnalyzedExprKind::Unwrap(e)
            | AnalyzedExprKind::Try(e) => {
                self.visit_expr(e);
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.visit_expr(array);
                self.visit_expr(index);
            }
            AnalyzedExprKind::FunctionCall { args, .. }
            | AnalyzedExprKind::StructInstantiation { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::MethodCall { receiver, args, .. }
            | AnalyzedExprKind::TraitMethodCall { receiver, args, .. } => {
                self.visit_expr(receiver);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::Lambda { body, .. } => {
                self.visit_expr(body);
            }
            AnalyzedExprKind::Assert { condition } => self.visit_expr(condition),
            AnalyzedExprKind::AssertEq { left, right } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            AnalyzedExprKind::StringLiteral(_)
            | AnalyzedExprKind::NumberLiteral(_)
            | AnalyzedExprKind::BooleanLiteral(_)
            | AnalyzedExprKind::None
            | AnalyzedExprKind::CollectionNew { .. }
            | AnalyzedExprKind::Variable(_) => {} // Leaf nodes
        }
    }
}

/// A human-readable report for an analyzed program
pub struct GlossaReport<'a> {
    program: &'a AnalyzedProgram,
    stats: ProgramStats,
    filename: String,
}

impl<'a> GlossaReport<'a> {
    /// Creates a new GlossaReport.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use glossa::parser::parse;
    /// use glossa::semantic::analyze_program;
    /// use glossa::tools::report::GlossaReport;
    ///
    /// let source = "ξ πέντε ἔστω.";
    /// let ast = parse(source).unwrap();
    /// let program = analyze_program(&ast).unwrap();
    /// let report = GlossaReport::new(&program, "main.γλ".to_string());
    ///
    /// let output = format!("{}", report);
    /// assert!(output.contains("main.γλ"));
    /// assert!(output.contains("1")); // Statement count
    /// ```
    pub fn new(program: &'a AnalyzedProgram, filename: String) -> Self {
        let stats = ProgramStats::new(program);
        Self {
            program,
            stats,
            filename,
        }
    }
}

impl Display for GlossaReport<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL).set_header(vec![
            Cell::new("Μετρική (Metric)")
                .add_attribute(comfy_table::Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Τιμή (Value)").add_attribute(comfy_table::Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Ἀρχεῖον (File)"),
            Cell::new(&self.filename).fg(Color::Green),
        ]);
        table.add_row(vec![
            Cell::new("Προτάσεις (Statements)"),
            Cell::new(self.stats.statement_count),
        ]);
        table.add_row(vec![
            Cell::new("Εκφράσεις (Expressions)"),
            Cell::new(self.stats.expression_count),
        ]);
        table.add_row(vec![
            Cell::new("Μεταβλητές (Bindings)"),
            Cell::new(self.stats.binding_count),
        ]);

        if self.stats.function_count > 0 {
            table.add_row(vec![
                Cell::new("Συναρτήσεις (Functions)"),
                Cell::new(self.stats.function_count),
            ]);
        }

        if self.stats.type_count > 0 {
            table.add_row(vec![
                Cell::new("Τύποι (Types)"),
                Cell::new(self.stats.type_count),
            ]);
        }

        if self.stats.loop_count > 0 {
            table.add_row(vec![
                Cell::new("Βρόχοι (Loops)"),
                Cell::new(self.stats.loop_count),
            ]);
        }

        if self.stats.max_depth > 0 {
            table.add_row(vec![
                Cell::new("Βάθος (Max Depth)"),
                Cell::new(self.stats.max_depth),
            ]);
        }

        writeln!(f)?;
        writeln!(f, "   {}", "Γ Λ Ω Σ Σ Α   R E P O R T".bold().cyan())?;
        writeln!(f, "   {}", "Language Metrics Dashboard".italic().dim())?;
        writeln!(f)?;
        writeln!(f, "{}", table)?;

        // If there are top-level functions, list them
        let functions: Vec<_> = self.program.scope.functions().collect();
        if !functions.is_empty() {
            writeln!(f, "\n{}", "ΣΥΝΑΡΤΗΣΕΙΣ (FUNCTIONS)".bold())?;
            let mut func_table = Table::new();
            func_table.load_preset(presets::UTF8_FULL).set_header(vec![
                "Ὄνομα (Name)",
                "Παράμετροι (Params)",
                "Επιστροφή (Returns)",
            ]);

            for func in functions {
                // ⚡ Bolt Optimization: Avoids intermediate heap allocations from `.collect::<Vec<_>>()` and `.join(", ")`
                let mut params = String::with_capacity(func.param_types.len() * 10);
                for (i, param_type) in func.param_types.iter().enumerate() {
                    if i > 0 {
                        params.push_str(", ");
                    }
                    use std::fmt::Write;
                    let _ = write!(&mut params, "{}", param_type);
                }

                let ret = func
                    .return_type
                    .as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "Οὐδέν".to_string());

                func_table.add_row(vec![
                    Cell::new(&func.name).fg(Color::Cyan),
                    Cell::new(if params.is_empty() { "-" } else { &params }),
                    Cell::new(ret).fg(Color::Yellow),
                ]);
            }
            writeln!(f, "{}", func_table)?;
        }

        Ok(())
    }
}

/// A comprehensive report for the compilation process
///
/// ## Examples
///
/// ```rust
/// use glossa::parser::parse;
/// use glossa::semantic::analyze_program;
/// use glossa::tools::report::{CompilationReport, ProgramStats};
/// use std::path::PathBuf;
/// use std::time::Duration;
///
/// let source = "ξ πέντε ἔστω.";
/// let ast = parse(source).unwrap();
/// let program = analyze_program(&ast).unwrap();
/// let stats = ProgramStats::new(&program);
///
/// let report = CompilationReport {
///     input_path: PathBuf::from("main.γλ"),
///     output_path: PathBuf::from("main.rs"),
///     input_size: 15,
///     output_size: 150,
///     duration: Duration::from_millis(10),
///     stats,
/// };
///
/// let output = format!("{}", report);
/// assert!(output.contains("main.γλ"));
/// assert!(output.contains("15 bytes"));
/// ```
pub struct CompilationReport {
    /// The location of the original ΓΛΩΣΣΑ scroll that was read.
    pub input_path: PathBuf,
    /// The destination where the newly forged Rust code was inscribed.
    pub output_path: PathBuf,
    /// The physical weight (in bytes) of the original thoughts.
    pub input_size: u64,
    /// The resulting weight (in bytes) of the generated machine instructions.
    pub output_size: u64,
    /// The fleeting moments spent translating human intent into machine action.
    pub duration: Duration,
    /// A deeper look into the complexity and structure of the logic itself.
    pub stats: ProgramStats,
}

impl Display for CompilationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL).set_header(vec![
            Cell::new("Μετρική (Metric)")
                .add_attribute(comfy_table::Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Τιμή (Value)").add_attribute(comfy_table::Attribute::Bold),
        ]);

        // Input File
        table.add_row(vec![
            Cell::new("Εἴσοδος (Input)"),
            Cell::new(self.input_path.display().to_string()).fg(Color::Yellow),
        ]);

        // Output File
        table.add_row(vec![
            Cell::new("Ἔξοδος (Output)"),
            Cell::new(self.output_path.display().to_string()).fg(Color::Green),
        ]);

        // Time
        table.add_row(vec![
            Cell::new("Χρόνος (Time)"),
            Cell::new(format!("{:.2?}", self.duration)),
        ]);

        // Sizes
        table.add_row(vec![
            Cell::new("Μέγεθος Εἰσόδου (Input Size)"),
            Cell::new(format!("{} bytes", self.input_size)),
        ]);
        table.add_row(vec![
            Cell::new("Μέγεθος Ἐξόδου (Output Size)"),
            Cell::new(format!("{} bytes", self.output_size)),
        ]);

        // Stats summary
        table.add_row(vec![
            Cell::new("Προτάσεις (Statements)"),
            Cell::new(self.stats.statement_count),
        ]);
        table.add_row(vec![
            Cell::new("Συναρτήσεις (Functions)"),
            Cell::new(self.stats.function_count),
        ]);

        writeln!(f)?;
        writeln!(f, "   {}", "Γ Λ Ω Σ Σ Α   R E P O R T".bold().cyan())?;
        writeln!(f, "   {}", "Compilation Metrics Dashboard".italic().dim())?;
        writeln!(f)?;
        writeln!(f, "{}", table)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{
        AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
    };

    fn create_dummy_program() -> AnalyzedProgram {
        let mut scope = Scope::new();
        // Add a function to scope
        scope.define_function("test_func", vec![], None);

        let mut statements = Vec::new();

        // Add a binding statement
        let binding = AnalyzedStatement::Binding {
            name: "x".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(42),
                glossa_type: GlossaType::Number,
            },
            mutable: false,
        };
        statements.push(binding);

        // Add an if statement
        let if_stmt = AnalyzedStatement::If {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
            then_body: vec![AnalyzedStatement::Print(vec![AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral("test".into()),
                glossa_type: GlossaType::String,
            }])],
            else_body: Some(vec![AnalyzedStatement::Query(vec![AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(10),
                glossa_type: GlossaType::Number,
            }])]),
        };
        statements.push(if_stmt);

        AnalyzedProgram { statements, scope }
    }

    #[test]
    fn test_program_stats_coverage() {
        let mut program = create_dummy_program();

        // Add more statements to hit missed branches
        program.statements.push(AnalyzedStatement::For {
            variable: "x".into(),
            iterator: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::Range {
                    start: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
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
            body: vec![],
        });

        program.statements.push(AnalyzedStatement::While {
            condition: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(true),
                glossa_type: GlossaType::Boolean,
            }),
            body: vec![],
        });

        program.statements.push(AnalyzedStatement::Match {
            scrutinee: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }),
            arms: vec![(
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                },
                vec![AnalyzedStatement::Break],
            )],
        });

        program.statements.push(AnalyzedStatement::FunctionDef {
            name: "inner".into(),
            params: vec![],
            body: vec![AnalyzedStatement::Continue],
            return_type: None,
        });

        program.statements.push(AnalyzedStatement::TestDeclaration {
            name: "test".into(),
            body: vec![],
        });

        program.statements.push(AnalyzedStatement::Return {
            value: Some(Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            })),
        });

        program.statements.push(AnalyzedStatement::TypeDefinition {
            name: "Type".into(),
            fields: vec![],
        });

        program.statements.push(AnalyzedStatement::TraitDefinition {
            name: "Trait".into(),
            methods: vec![],
        });

        program
            .statements
            .push(AnalyzedStatement::TraitImplementation {
                trait_name: "Trait".into(),
                type_name: "Type".into(),
                methods: vec![crate::semantic::AnalyzedMethod {
                    name: "method".into(),
                    params: vec![],
                    body: Some(vec![AnalyzedStatement::Break]),
                    return_type: None,
                }],
            });

        // Exprs
        program.statements.push(AnalyzedStatement::Expression(vec![
            AnalyzedExpr {
                expr: AnalyzedExprKind::UnaryOp {
                    op: crate::morphology::lexicon::UnaryOp::Not,
                    operand: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::BooleanLiteral(true),
                        glossa_type: GlossaType::Boolean,
                    }),
                },
                glossa_type: GlossaType::Boolean,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::ArrayLiteral(vec![]),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                })),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Ok(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                })),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Err(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                })),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    })),
                    glossa_type: GlossaType::Unknown,
                })),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    })),
                    glossa_type: GlossaType::Unknown,
                })),
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::IndexAccess {
                    array: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::ArrayLiteral(vec![]),
                        glossa_type: GlossaType::Unknown,
                    }),
                    index: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(0),
                        glossa_type: GlossaType::Number,
                    }),
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::FunctionCall {
                    func: "func".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::StructInstantiation {
                    type_name: "Type".into(),
                    fields: vec![],
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    method: "method".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
            AnalyzedExpr {
                expr: AnalyzedExprKind::TraitMethodCall {
                    receiver: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    trait_name: "Trait".into(),
                    method_name: "method".into(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown,
            },
        ]));

        let stats = ProgramStats::new(&program);

        assert_eq!(stats.statement_count, 17); // 4 + 9 newly added nodes + 1 Expr node containing multiple elements + 3 loop/match nested statements
        assert_eq!(stats.binding_count, 1);
        assert_eq!(stats.conditional_count, 2);
        assert_eq!(stats.function_count, 1);
        assert_eq!(stats.loop_count, 2);
        assert!(stats.max_depth >= 1);
        assert!(stats.expression_count > 0);
    }

    #[test]
    fn test_report_generation_coverage() {
        let mut program = create_dummy_program();
        // Add another function to test param rendering and return type formatting
        program.scope.define_function(
            "complex_func",
            vec![GlossaType::Number, GlossaType::String],
            Some(GlossaType::Boolean),
        );

        let report = GlossaReport::new(&program, "test.gl".to_string());
        let output = format!("{}", report);

        assert!(output.contains("R E P O R T"));
        assert!(output.contains("test.gl"));
        assert!(output.contains("test_func")); // Function list
        assert!(output.contains("complex_func")); // Function list
        assert!(output.contains("Οὐδέν")); // None return mapping
        // Statement count is back to 4 because we didn't add the extra exhaustive coverage nodes
        // to `test_report_generation_coverage` - we only added them to `test_program_stats_coverage`
        assert!(output.contains("4"));
    }

    #[test]
    fn test_compilation_report_coverage() {
        let program = create_dummy_program();
        let stats = ProgramStats::new(&program);
        let report = CompilationReport {
            input_path: PathBuf::from("input.gl"),
            output_path: PathBuf::from("output.rs"),
            input_size: 100,
            output_size: 200,
            duration: Duration::from_millis(123),
            stats,
        };

        let output = format!("{}", report);

        assert!(output.contains("R E P O R T"));
        assert!(output.contains("input.gl"));
        assert!(output.contains("output.rs"));
        assert!(output.contains("123")); // Time
        assert!(output.contains("100 bytes")); // Input size
        assert!(output.contains("200 bytes")); // Output size
        assert!(output.contains("4")); // Statements
    }

    #[test]
    fn test_report_manual_ast_coverage() {
        // Create AST nodes that might be hard to trigger via parser but need coverage
        let scope = Scope::new();
        let statements = vec![
            // While loop
            AnalyzedStatement::While {
                condition: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::BooleanLiteral(true),
                    glossa_type: GlossaType::Boolean,
                }),
                body: vec![],
            },
            // Loop (for)
            AnalyzedStatement::For {
                variable: "i".into(),
                iterator: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::Range {
                        start: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(1),
                            glossa_type: GlossaType::Number,
                        }),
                        end: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(10),
                            glossa_type: GlossaType::Number,
                        }),
                        inclusive: false,
                    },
                    glossa_type: GlossaType::Number,
                }),
                body: vec![],
            },
            // Match
            AnalyzedStatement::Match {
                scrutinee: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
                arms: vec![],
            },
            // Assignment
            AnalyzedStatement::Assignment {
                name: "x".into(),
                value: AnalyzedExpr {
                    expr: AnalyzedExprKind::BinOp {
                        left: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(1),
                            glossa_type: GlossaType::Number,
                        }),
                        op: crate::morphology::lexicon::BinaryOp::Add,
                        right: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(2),
                            glossa_type: GlossaType::Number,
                        }),
                    },
                    glossa_type: GlossaType::Number,
                },
            },
            // Return
            AnalyzedStatement::Return {
                value: Some(Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::FunctionCall {
                        func: "test_func".into(),
                        args: vec![],
                    },
                    glossa_type: GlossaType::Unit,
                })),
            },
            // Trait Definition (coverage for empty branch)
            AnalyzedStatement::TraitDefinition {
                name: "TestTrait".into(),
                methods: vec![],
            },
            // Trait Implementation (coverage for empty branch and methods)
            AnalyzedStatement::TraitImplementation {
                trait_name: "TestTrait".into(),
                type_name: "TestType".into(),
                methods: vec![crate::semantic::AnalyzedMethod {
                    name: "test_method".into(),
                    params: vec![],
                    body: Some(vec![AnalyzedStatement::Break]),
                    return_type: None,
                }],
            },
            // Type Definition (coverage for empty branch)
            AnalyzedStatement::TypeDefinition {
                name: "TestType".into(),
                fields: vec![],
            },
            // Test Declaration
            AnalyzedStatement::TestDeclaration {
                name: "test_decl".into(),
                body: vec![],
            },
            // Function Definition in statement (not scope)
            AnalyzedStatement::FunctionDef {
                name: "inner_func".into(),
                params: vec![],
                return_type: None,
                body: vec![AnalyzedStatement::Break, AnalyzedStatement::Continue],
            },
            // Expression Statement with Unwrap, Assert, Some, Ok, Err
            AnalyzedStatement::Expression(vec![
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Assert {
                        condition: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::Unwrap(Box::new(AnalyzedExpr {
                                expr: AnalyzedExprKind::BooleanLiteral(true),
                                glossa_type: GlossaType::Boolean,
                            })),
                            glossa_type: GlossaType::Boolean,
                        }),
                    },
                    glossa_type: GlossaType::Unit,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Some(Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    })),
                    glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Ok(Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    })),
                    glossa_type: GlossaType::Result(
                        Box::new(GlossaType::Number),
                        Box::new(GlossaType::String),
                    ),
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Err(Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::StringLiteral("error".into()),
                        glossa_type: GlossaType::String,
                    })),
                    glossa_type: GlossaType::Result(
                        Box::new(GlossaType::Number),
                        Box::new(GlossaType::String),
                    ),
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::IndexAccess {
                        array: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::ArrayLiteral(vec![]),
                            glossa_type: GlossaType::List(Box::new(GlossaType::Number)),
                        }),
                        index: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(0),
                            glossa_type: GlossaType::Number,
                        }),
                    },
                    glossa_type: GlossaType::Number,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::StructInstantiation {
                        type_name: "MyStruct".into(),
                        fields: vec![],
                        args: vec![],
                    },
                    glossa_type: GlossaType::Unknown,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::CollectionNew {
                        collection_type: "HashSet".into(),
                    },
                    glossa_type: GlossaType::Unknown,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Try(Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    })),
                    glossa_type: GlossaType::Number,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::None,
                    glossa_type: GlossaType::Option(Box::new(GlossaType::Number)),
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::MethodCall {
                        receiver: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(1),
                            glossa_type: GlossaType::Number,
                        }),
                        method: "abs".into(),
                        args: vec![],
                    },
                    glossa_type: GlossaType::Number,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::TraitMethodCall {
                        receiver: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(1),
                            glossa_type: GlossaType::Number,
                        }),
                        trait_name: "Num".into(),
                        method_name: "abs".into(),
                        args: vec![],
                    },
                    glossa_type: GlossaType::Number,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Lambda {
                        params: vec![],
                        body: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(1),
                            glossa_type: GlossaType::Number,
                        }),
                        capture_mode: crate::semantic::CaptureMode::Borrow,
                    },
                    glossa_type: GlossaType::Unknown,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::AssertEq {
                        left: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(1),
                            glossa_type: GlossaType::Number,
                        }),
                        right: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::NumberLiteral(1),
                            glossa_type: GlossaType::Number,
                        }),
                    },
                    glossa_type: GlossaType::Unit,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::PropertyAccess {
                        owner: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::StringLiteral("test".into()),
                            glossa_type: GlossaType::String,
                        }),
                        property: "len".into(),
                    },
                    glossa_type: GlossaType::Number,
                },
                AnalyzedExpr {
                    expr: AnalyzedExprKind::VerbCall {
                        verb: "print".into(),
                        args: vec![AnalyzedExpr {
                            expr: AnalyzedExprKind::StringLiteral("test".into()),
                            glossa_type: GlossaType::String,
                        }],
                    },
                    glossa_type: GlossaType::Unit,
                },
            ]),
        ];

        let program = AnalyzedProgram { statements, scope };
        let stats = ProgramStats::new(&program);

        // Cover Debug derive
        println!("{:?}", stats);

        assert_eq!(stats.loop_count, 2);
        assert_eq!(stats.conditional_count, 1); // Match counts as conditional
        assert!(stats.expression_count >= 5); // Should count sub-expressions
    }
}
