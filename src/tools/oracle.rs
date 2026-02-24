//! The Oracle (ὁ Χρησμός) - Semantic Style & Logic Analysis
//!
//! This module implements "The Oracle", a tool that analyzes ΓΛΩΣΣΑ programs
//! not just for correctness (which the compiler does), but for "Wisdom".
//! It detects stylistic issues, complexity, and potential logic flaws.
//!
//! # The Prophecies
//!
//! The Oracle delivers prophecies in four categories:
//! * **Hubris (Ὕβρις)**: Excessive complexity (deep nesting, long functions).
//! * **Laziness (Ἀργία)**: Empty blocks or loops with no effect.
//! * **Barbarism (Βαρβαρισμός)**: Use of non-Greek characters (Latin) in identifiers.
//! * **Narcissus (Νάρκισσος)**: Unused variables or self-assignments.

use crate::parser::parse;
use crate::semantic::{
    AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, analyze_program,
};
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::collections::HashSet;
use std::fmt::Display;
use std::io::Write;
use std::path::Path;

/// Severity of a prophecy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// A minor suggestion (Stylistic)
    Advice,
    /// A potential issue (Warning)
    Warning,
    /// A likely bug or serious flaw (Omen)
    Omen,
}

impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Advice => write!(f, "Συμβουλή (Advice)"),
            Severity::Warning => write!(f, "Προειδοποίησις (Warning)"),
            Severity::Omen => write!(f, "Οἰωνός (Omen)"),
        }
    }
}

impl Severity {
    fn color(&self) -> Color {
        match self {
            Severity::Advice => Color::Green,
            Severity::Warning => Color::Yellow,
            Severity::Omen => Color::Red,
        }
    }
}

/// A single prophecy delivered by the Oracle
#[derive(Debug)]
pub struct Prophecy {
    pub severity: Severity,
    pub category: &'static str,
    pub message: String,
    pub location: Option<usize>, // Line number approximation (index in statement list)
}

/// The Oracle context
pub struct Oracle {
    prophecies: Vec<Prophecy>,
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

impl Oracle {
    pub fn new() -> Self {
        Self {
            prophecies: Vec::new(),
        }
    }

    /// Consult the Oracle on a file
    pub fn consult(&mut self, source: &str) -> Result<()> {
        let ast = parse(source).map_err(|e| miette::miette!("{}", e))?;
        let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

        self.analyze_program(&program);
        Ok(())
    }

    fn analyze_program(&mut self, program: &AnalyzedProgram) {
        // 1. Collect usages for Narcissus check
        let mut used_vars = HashSet::new();
        let mut declared_vars = Vec::new();

        self.collect_usages_and_decls(program, &mut used_vars, &mut declared_vars);

        // Narcissus: Check for unused variables
        for (name, line_idx) in declared_vars {
            if !used_vars.contains(&name) {
                self.prophecies.push(Prophecy {
                    severity: Severity::Advice,
                    category: "Narcissus",
                    message: format!("Variable '{}' is declared but never used.", name),
                    location: Some(line_idx),
                });
            }
        }

        // 2. Analyze statements
        for (i, stmt) in program.statements.iter().enumerate() {
            self.analyze_statement(stmt, 0, i + 1);
        }

        // 3. Analyze Global Scope Identifiers for Barbarism
        for (name, _) in program.scope.types() {
            if has_latin_chars(name) {
                self.prophecies.push(Prophecy {
                    severity: Severity::Advice,
                    category: "Barbarism",
                    message: format!("Type name '{}' contains Latin characters.", name),
                    location: None,
                });
            }
        }
        for func in program.scope.functions() {
            if has_latin_chars(&func.name) {
                self.prophecies.push(Prophecy {
                    severity: Severity::Advice,
                    category: "Barbarism",
                    message: format!("Function name '{}' contains Latin characters.", func.name),
                    location: None,
                });
            }
        }
    }

    fn collect_usages_and_decls(
        &self,
        program: &AnalyzedProgram,
        used: &mut HashSet<String>,
        declared: &mut Vec<(String, usize)>,
    ) {
        for (i, stmt) in program.statements.iter().enumerate() {
            self.collect_stmt(stmt, used, declared, i + 1);
        }
    }

    fn collect_stmt(
        &self,
        stmt: &AnalyzedStatement,
        used: &mut HashSet<String>,
        declared: &mut Vec<(String, usize)>,
        line_idx: usize,
    ) {
        match stmt {
            AnalyzedStatement::Binding { name, value, .. } => {
                declared.push((name.to_string(), line_idx));
                self.collect_expr(value, used);
            }
            AnalyzedStatement::Assignment { value, .. } => {
                // Assignment counts as usage? Or update?
                // Usually assignment is not usage. But reading is.
                // However, we track variable *names*.
                // If I assign to `x`, I must have declared `x`.
                // If I only assign and never read, it's effectively unused (write-only).
                // So I won't add to `used` here, unless the RHS uses it.
                // But wait, `name` here is the target.
                self.collect_expr(value, used);
            }
            AnalyzedStatement::Expression(exprs)
            | AnalyzedStatement::Print(exprs)
            | AnalyzedStatement::Query(exprs) => {
                for e in exprs {
                    self.collect_expr(e, used);
                }
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.collect_expr(condition, used);
                for s in then_body {
                    self.collect_stmt(s, used, declared, line_idx);
                }
                if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        self.collect_stmt(s, used, declared, line_idx);
                    }
                }
            }
            AnalyzedStatement::While { condition, body } => {
                self.collect_expr(condition, used);
                for s in body {
                    self.collect_stmt(s, used, declared, line_idx);
                }
            }
            AnalyzedStatement::For {
                variable,
                iterator,
                body,
            } => {
                declared.push((variable.to_string(), line_idx));
                self.collect_expr(iterator, used);
                for s in body {
                    self.collect_stmt(s, used, declared, line_idx);
                }
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                self.collect_expr(scrutinee, used);
                for (pat, body) in arms {
                    self.collect_expr(pat, used);
                    for s in body {
                        self.collect_stmt(s, used, declared, line_idx);
                    }
                }
            }
            AnalyzedStatement::FunctionDef { body, .. } => {
                for s in body {
                    self.collect_stmt(s, used, declared, line_idx);
                }
            }
            AnalyzedStatement::TestDeclaration { body, .. } => {
                for s in body {
                    self.collect_stmt(s, used, declared, line_idx);
                }
            }
            AnalyzedStatement::Return { value: Some(v) } => {
                self.collect_expr(v, used);
            }
            _ => {}
        }
    }

    fn collect_expr(&self, expr: &AnalyzedExpr, used: &mut HashSet<String>) {
        match &expr.expr {
            AnalyzedExprKind::Variable(name) => {
                used.insert(name.to_string());
            }
            AnalyzedExprKind::BinOp { left, right, .. } => {
                self.collect_expr(left, used);
                self.collect_expr(right, used);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => {
                self.collect_expr(operand, used);
            }
            AnalyzedExprKind::FunctionCall { args, .. }
            | AnalyzedExprKind::StructInstantiation { args, .. }
            | AnalyzedExprKind::VerbCall { args, .. } => {
                for arg in args {
                    self.collect_expr(arg, used);
                }
            }
            AnalyzedExprKind::MethodCall { receiver, args, .. }
            | AnalyzedExprKind::TraitMethodCall { receiver, args, .. } => {
                self.collect_expr(receiver, used);
                for arg in args {
                    self.collect_expr(arg, used);
                }
            }
            AnalyzedExprKind::PropertyAccess { owner, .. } => {
                self.collect_expr(owner, used);
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.collect_expr(array, used);
                self.collect_expr(index, used);
            }
            AnalyzedExprKind::ArrayLiteral(exprs) => {
                for e in exprs {
                    self.collect_expr(e, used);
                }
            }
            AnalyzedExprKind::Some(e)
            | AnalyzedExprKind::Ok(e)
            | AnalyzedExprKind::Err(e)
            | AnalyzedExprKind::Unwrap(e)
            | AnalyzedExprKind::Try(e)
            | AnalyzedExprKind::Assert { condition: e } => {
                self.collect_expr(e, used);
            }
            AnalyzedExprKind::AssertEq { left, right } => {
                self.collect_expr(left, used);
                self.collect_expr(right, used);
            }
            AnalyzedExprKind::Lambda { body, .. } => {
                self.collect_expr(body, used);
            }
            AnalyzedExprKind::Range { start, end, .. } => {
                self.collect_expr(start, used);
                self.collect_expr(end, used);
            }
            _ => {}
        }
    }

    fn analyze_statement(&mut self, stmt: &AnalyzedStatement, depth: usize, line_idx: usize) {
        // Hubris: Check depth
        if depth > 3 {
            self.prophecies.push(Prophecy {
                severity: Severity::Warning,
                category: "Hubris",
                message: format!(
                    "Deep nesting detected (depth: {}). Consider refactoring.",
                    depth
                ),
                location: Some(line_idx),
            });
        }

        match stmt {
            AnalyzedStatement::If {
                then_body,
                else_body,
                ..
            } => {
                if then_body.is_empty() {
                    self.prophecies.push(Prophecy {
                        severity: Severity::Advice,
                        category: "Laziness",
                        message: "Empty 'If' block found.".to_string(),
                        location: Some(line_idx),
                    });
                }
                for s in then_body {
                    self.analyze_statement(s, depth + 1, line_idx);
                }
                if let Some(else_stmts) = else_body {
                    if else_stmts.is_empty() {
                        self.prophecies.push(Prophecy {
                            severity: Severity::Advice,
                            category: "Laziness",
                            message: "Empty 'Else' block found.".to_string(),
                            location: Some(line_idx),
                        });
                    }
                    for s in else_stmts {
                        self.analyze_statement(s, depth + 1, line_idx);
                    }
                }
            }
            AnalyzedStatement::While { body, .. } => {
                if body.is_empty() {
                    self.prophecies.push(Prophecy {
                        severity: Severity::Warning,
                        category: "Laziness",
                        message: "Empty 'While' loop found. This may be an infinite loop."
                            .to_string(),
                        location: Some(line_idx),
                    });
                }
                for s in body {
                    self.analyze_statement(s, depth + 1, line_idx);
                }
            }
            AnalyzedStatement::For { body, .. } => {
                if body.is_empty() {
                    self.prophecies.push(Prophecy {
                        severity: Severity::Advice,
                        category: "Laziness",
                        message: "Empty 'For' loop found.".to_string(),
                        location: Some(line_idx),
                    });
                }
                for s in body {
                    self.analyze_statement(s, depth + 1, line_idx);
                }
            }
            AnalyzedStatement::Match { arms, .. } => {
                if arms.is_empty() {
                    self.prophecies.push(Prophecy {
                        severity: Severity::Warning,
                        category: "Laziness",
                        message: "Empty 'Match' statement found.".to_string(),
                        location: Some(line_idx),
                    });
                }
                for (_, body) in arms {
                    for s in body {
                        self.analyze_statement(s, depth + 1, line_idx);
                    }
                }
            }
            AnalyzedStatement::FunctionDef { name, body, .. } => {
                if has_latin_chars(name) {
                    self.prophecies.push(Prophecy {
                        severity: Severity::Advice,
                        category: "Barbarism",
                        message: format!("Function name '{}' contains Latin characters.", name),
                        location: Some(line_idx),
                    });
                }
                for s in body {
                    self.analyze_statement(s, depth + 1, line_idx);
                }
            }
            AnalyzedStatement::TestDeclaration { body, .. } => {
                for s in body {
                    self.analyze_statement(s, depth + 1, line_idx);
                }
            }

            // Barbarism Check (Latin characters in variable names)
            AnalyzedStatement::Binding { name, .. }
            | AnalyzedStatement::Assignment { name, .. } => {
                if has_latin_chars(name) {
                    self.prophecies.push(Prophecy {
                        severity: Severity::Advice,
                        category: "Barbarism",
                        message: format!(
                            "Identifier '{}' contains Latin characters. Use Greek!",
                            name
                        ),
                        location: Some(line_idx),
                    });
                }
            }

            AnalyzedStatement::TypeDefinition { name, .. } => {
                if has_latin_chars(name) {
                    self.prophecies.push(Prophecy {
                        severity: Severity::Advice,
                        category: "Barbarism",
                        message: format!("Type name '{}' contains Latin characters.", name),
                        location: Some(line_idx),
                    });
                }
            }

            AnalyzedStatement::TraitDefinition { name, .. } => {
                if has_latin_chars(name) {
                    self.prophecies.push(Prophecy {
                        severity: Severity::Advice,
                        category: "Barbarism",
                        message: format!("Trait name '{}' contains Latin characters.", name),
                        location: Some(line_idx),
                    });
                }
            }

            _ => {}
        }
    }
}

fn has_latin_chars(s: &str) -> bool {
    s.chars().any(|c| c.is_ascii_alphabetic())
}

/// Run the Oracle tool on a file
pub fn run_oracle(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let source = std::fs::read_to_string(input).into_diagnostic()?;
    run_oracle_inner(&source, &mut std::io::stdout())
}

/// Internal implementation (for testing)
pub fn run_oracle_inner<W: Write>(source: &str, writer: &mut W) -> Result<()> {
    writeln!(writer).into_diagnostic()?;
    writeln!(writer, "   {}", "Γ Λ Ω Σ Σ Α   O R A C L E".bold().cyan()).into_diagnostic()?;
    writeln!(writer, "   {}", "Consulting the spirits...".italic().dim()).into_diagnostic()?;
    writeln!(writer).into_diagnostic()?;

    let mut oracle = Oracle::new();
    match oracle.consult(source) {
        Ok(_) => {
            if oracle.prophecies.is_empty() {
                writeln!(
                    writer,
                    "   {}",
                    "✓ The Oracle is silent. Your code is pure.".green().bold()
                )
                .into_diagnostic()?;
            } else {
                writeln!(
                    writer,
                    "   {}",
                    "The Oracle has spoken:".bold().underlined()
                )
                .into_diagnostic()?;
                writeln!(writer).into_diagnostic()?;

                let mut table = Table::new();
                table.load_preset(presets::UTF8_FULL).set_header(vec![
                    Cell::new("Severity").add_attribute(Attribute::Bold),
                    Cell::new("Category").add_attribute(Attribute::Bold),
                    Cell::new("Prophecy").add_attribute(Attribute::Bold),
                ]);

                // Sort prophecies by severity (Omen > Warning > Advice)
                oracle
                    .prophecies
                    .sort_by(|a, b| b.severity.cmp(&a.severity));

                for p in oracle.prophecies {
                    table.add_row(vec![
                        Cell::new(p.severity).fg(p.severity.color()),
                        Cell::new(p.category).fg(Color::Cyan),
                        Cell::new(p.message),
                    ]);
                }
                writeln!(writer, "{}", table).into_diagnostic()?;
            }
        }
        Err(e) => {
            writeln!(
                writer,
                "   {}",
                "× The Oracle is confused (Compilation Error).".red()
            )
            .into_diagnostic()?;
            writeln!(writer, "   {}", e).into_diagnostic()?;
        }
    }

    Ok(())
}
