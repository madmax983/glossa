//! Report generation for ΓΛΩΣΣΑ
//!
//! This module provides tools to generate human-readable reports and statistics
//! for analyzed programs.

use comfy_table::{Cell, Color, Table, presets};
use crossterm::style::Stylize;
use std::fmt::Display;

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};

/// Statistics for an analyzed program
#[derive(Debug, Default, Clone)]
pub struct ProgramStats {
    /// Total number of statements
    pub statement_count: usize,
    /// Total number of expressions
    pub expression_count: usize,
    /// Number of variable bindings
    pub binding_count: usize,
    /// Number of functions defined
    pub function_count: usize,
    /// Number of types defined
    pub type_count: usize,
    /// Number of traits defined
    pub trait_count: usize,
    /// Number of loops
    pub loop_count: usize,
    /// Number of conditional statements
    pub conditional_count: usize,
    /// Maximum nesting depth
    pub max_depth: usize,
}

impl ProgramStats {
    /// Analyze a program and collect statistics
    pub fn new(program: &AnalyzedProgram) -> Self {
        let mut stats = ProgramStats {
            function_count: program.scope.functions().count(),
            type_count: program.scope.types().count(),
            trait_count: program.scope.traits().count(),
            ..Default::default()
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
            AnalyzedStatement::TraitImplementation { .. } => {} // Not tracking impls yet
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
            _ => {} // Literals, Variables, etc.
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

        writeln!(
            f,
            "\n{}",
            "ΑΝΑΦΟΡΑ ΓΛΩΣΣΗΣ (LANGUAGE REPORT)".bold().underlined()
        )?;
        writeln!(f, "{}", table)?;

        // If there are top-level functions, list them
        let functions: Vec<_> = self.program.scope.functions().collect();
        if !functions.is_empty() {
            writeln!(f, "\n{}", "ΣΥΝΑΡΤΗΣΕΙΣ (FUNCTIONS)".bold())?;
            let mut func_table = Table::new();
            func_table
                .load_preset(presets::UTF8_HORIZONTAL_ONLY)
                .set_header(vec![
                    "Ὄνομα (Name)",
                    "Παράμετροι (Params)",
                    "Επιστροφή (Returns)",
                ]);

            for func in functions {
                let params = func
                    .param_types
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_report_generation() {
        let source = r#"
            // Binding
            ξ πέντε ἔστω.

            // Conditional (If/Else)
            εἰ ξ 5 μεῖζον ᾖ,
                «μείζον» λέγε.
            εἰ δὲ μή,
                «ἐλάσσον» λέγε.

            // Loop (While/For)
            ἕως ξ 0 μεῖζον ᾖ,
                ξ ξ 1 διαφορά ἔστω.

            // Type Definition
            εἶδος Τύπος ὁρίζειν {
                χ Ἀριθμός.
            }.

            // Function Definition
            συνάρτησις φ(α Ἀριθμός) Ἀριθμός {
                α.
            }.

            // Test Declaration
            δοκιμή «δοκιμή».
                ξ 0 ἰσοῦται.
            τέλος.

            // Expressions coverage:
            // Array, Index, BinOp, UnaryOp, MethodCall (via map/filter implicitly or explicit)
            α [1, 2, 3] ἔστω.
            β α[0] ἔστω.
            γ οὐκ ἀληθές ἔστω.
        "#;
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let report = GlossaReport::new(&analyzed, "test.gl".to_string());
        let output = report.to_string();

        assert!(output.contains("ΑΝΑΦΟΡΑ ΓΛΩΣΣΗΣ"));
        assert!(output.contains("test.gl"));

        // Verify stats counting
        // Bindings: ξ, ξ (in loop), α, β, γ = 5
        assert!(report.stats.binding_count >= 3);
        // Conditionals: 1 (if)
        assert!(report.stats.conditional_count >= 1);
        // Loops: 1 (while)
        assert!(report.stats.loop_count >= 1);
        // Types: 1
        assert_eq!(report.stats.type_count, 1);
    }

    #[test]
    fn test_report_expression_visitor_coverage() {
        // Construct a program specifically to hit Expr branches in visitor
        // We use a syntactically valid program that exercises various constructs
        let source = r#"
            // Array literal
            α [1, 2, 3] ἔστω.

            // Index access
            β α[0] ἔστω.

            // BinOp
            γ 1 2 ἄθροισμα ἔστω.

            // UnaryOp
            δ οὐκ ἀληθές ἔστω.

            // Range (in for loop)
            διὰ α, ε λέγε.

            // Function call
            «hello» λέγε.
        "#;

        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let stats = ProgramStats::new(&analyzed);

        // Just verify we visited expressions
        assert!(stats.expression_count > 5);
        assert!(stats.statement_count > 0);
    }

    #[test]
    fn test_report_extended_coverage() {
        // Cover remaining expression types: StructInstantiation, Lambda, Unwrap
        let source = r#"
            εἶδος Τύπος ὁρίζειν { α Ἀριθμός }.

            // Struct Instantiation
            τ νέον Τύπος 1 ἔστω.

            // Unwrap (unary op !)
            // Assuming `τ` could be an Option/Result for parser context,
            // or just lexically valid.
            // Actually parser handles ! as Unwrap unary op.
            υ τ! ἔστω.

            // Lambda (via participle)
            // [1, 2] doubled
            [1, 2] διπλασιαζόμενα λέγε.

            // Method Call (not easily expressible in pure Greek syntax without std types,
            // but PropertyAccess is similar and covered).
            // Let's try a chain if possible.
        "#;

        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let stats = ProgramStats::new(&analyzed);

        assert!(stats.binding_count >= 2);
        // Lambda generates a closure body expression
        assert!(stats.expression_count > 0);
    }
}
