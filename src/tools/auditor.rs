//! The Auditor (ὁ Λογιστής) - Static Analysis Tool
//!
//! This module implements the "Auditor" tool, providing basic static analysis capabilities
//! for ΓΛΩΣΣΑ programs to enforce code quality and correctness before compilation.
//!
//! # Purpose
//!
//! Just as an auditor balances the books and finds inefficiencies, this tool
//! scans the analyzed Abstract Syntax Tree (AST) to identify potential issues such as:
//! * **Unused Variables**: Variables defined with `ἔστω` but never referenced.
//! * **Unnecessary Mutability**: Variables declared as mutable (`μετά`) but never reassigned.
//!
//! # How it Works
//!
//! The [`run_auditor`](crate::tools::auditor::run_auditor) function drives the analysis:
//! 1. The code is parsed and semantically analyzed.
//! 2. A custom visitor (`AuditorVisitor`) traverses every statement and expression in the AST.
//! 3. The visitor tracks variable declarations, usages, and reassignments using HashMaps and HashSets.
//! 4. After traversal, the findings are cross-referenced to produce a final report,
//!    which is displayed in a stylized terminal table.
use crate::parser::parse;
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, analyze_program};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use rustc_hash::{FxHashMap, FxHashSet};
use smol_str::SmolStr;
use std::path::Path;

/// Runs the Auditor tool on a given Glossa source file.
///
/// The Auditor (Λογιστής) reads the provided source file, parses and analyzes it to produce an AST,
/// and then runs static analysis over the structure to detect unused variables and variables
/// declared as mutable but never reassigned.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::auditor::run_auditor;
/// use std::path::Path;
///
/// let input = Path::new("main.γλ");
/// if let Err(e) = run_auditor(&input) {
///     eprintln!("Audit failed: {}", e);
/// }
/// ```
///
/// # Errors
///
/// Returns a [`miette::Result`] if the file cannot be read, or if there is a parsing
/// or semantic analysis error during compilation.
pub fn run_auditor(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Λογιστής (Auditing Code)", "🔍");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    let ast = match parse(&source) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα συντάξεως (Syntax Error)");
            return Err(miette::miette!("Parse error: {}", e));
        }
    };

    let program = match analyze_program(&ast) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα σημασίας (Semantic Error)");
            return Err(miette::miette!("Semantic error: {}", e));
        }
    };

    status.success();

    let mut visitor = AuditorVisitor::new();
    for stmt in &program.statements {
        visitor.visit_statement(stmt);
    }

    let mut issues = 0;

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A U D I T O R".bold().cyan());
    println!("   {}", "Audit Report".italic().dim());
    println!();
    println!("   {}", format!("🔍 File: {}", input.display()).bold());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Type")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Variable")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Message")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    for (var, count) in &visitor.usage_count {
        if *count == 0 {
            table.add_row(vec![
                Cell::new("⚠️ Unused Variable").fg(Color::Yellow),
                Cell::new(var),
                Cell::new("Declared but never used"),
            ]);
            issues += 1;
        }
    }

    for (var, count) in &visitor.mutation_count {
        if *count == 0
            && visitor.mutable_vars.contains(var)
            && visitor.usage_count.get(var).unwrap_or(&0) > &0
        {
            table.add_row(vec![
                Cell::new("💡 Unnecessary Mutation").fg(Color::Blue),
                Cell::new(var),
                Cell::new("Declared mutable ('μετά') but never changed"),
            ]);
            issues += 1;
        }
    }

    if issues == 0 {
        let mut success_table = Table::new();
        success_table.load_preset(UTF8_FULL);
        success_table.add_row(vec![
            Cell::new("✨ No issues found. The code is pure.")
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
        ]);
        println!("{success_table}");
    } else {
        println!("{table}");
        println!();
        println!("   Total issues found: {}", issues);
    }

    Ok(())
}

struct AuditorVisitor {
    /// ⚡ Bolt Optimization: Uses `FxHashMap` instead of the standard `HashMap`
    /// to reduce cryptographic hashing overhead for small string keys (`SmolStr`).
    usage_count: FxHashMap<SmolStr, usize>,
    mutation_count: FxHashMap<SmolStr, usize>,
    mutable_vars: FxHashSet<SmolStr>,
}

impl AuditorVisitor {
    fn new() -> Self {
        Self {
            usage_count: FxHashMap::default(),
            mutation_count: FxHashMap::default(),
            mutable_vars: FxHashSet::default(),
        }
    }

    fn visit_statement(&mut self, stmt: &AnalyzedStatement) {
        match stmt {
            AnalyzedStatement::Binding {
                name,
                value,
                mutable,
            } => {
                self.usage_count.insert(name.clone(), 0);
                self.mutation_count.insert(name.clone(), 0);
                if *mutable {
                    self.mutable_vars.insert(name.clone());
                }
                self.visit_expr(value);
            }
            AnalyzedStatement::Assignment { name, value } => {
                if let Some(count) = self.mutation_count.get_mut(name) {
                    *count += 1;
                }
                if let Some(count) = self.usage_count.get_mut(name) {
                    *count += 1;
                }
                self.visit_expr(value);
            }
            AnalyzedStatement::Print(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            AnalyzedStatement::Expression(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            AnalyzedStatement::Query(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.visit_expr(condition);
                for s in then_body {
                    self.visit_statement(s);
                }
                if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        self.visit_statement(s);
                    }
                }
            }
            AnalyzedStatement::While { condition, body } => {
                self.visit_expr(condition);
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::For {
                variable,
                iterator,
                body,
            } => {
                self.usage_count.insert(variable.clone(), 0);
                self.visit_expr(iterator);
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                self.visit_expr(scrutinee);
                for (expr, stmts) in arms {
                    self.visit_expr(expr);
                    for s in stmts {
                        self.visit_statement(s);
                    }
                }
            }
            AnalyzedStatement::FunctionDef { params, body, .. } => {
                for (param_name, _) in params {
                    self.usage_count.insert(param_name.clone(), 0);
                }
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::Return { value } => {
                if let Some(v) = value {
                    self.visit_expr(v);
                }
            }
            AnalyzedStatement::TestDeclaration { body, .. } => {
                for s in body {
                    self.visit_statement(s);
                }
            }
            AnalyzedStatement::Break
            | AnalyzedStatement::Continue
            | AnalyzedStatement::TypeDefinition { .. }
            | AnalyzedStatement::TraitDefinition { .. }
            | AnalyzedStatement::TraitImplementation { .. } => {}
        }
    }

    fn visit_expr(&mut self, expr: &AnalyzedExpr) {
        match &expr.expr {
            AnalyzedExprKind::Variable(name) => {
                if let Some(count) = self.usage_count.get_mut(name) {
                    *count += 1;
                }
            }
            AnalyzedExprKind::BinOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => {
                self.visit_expr(operand);
            }
            AnalyzedExprKind::StructInstantiation { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::PropertyAccess { owner, .. } => {
                self.visit_expr(owner);
            }
            AnalyzedExprKind::MethodCall { receiver, args, .. } => {
                self.visit_expr(receiver);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::FunctionCall { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::VerbCall { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::ArrayLiteral(exprs) => {
                for e in exprs {
                    self.visit_expr(e);
                }
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.visit_expr(array);
                self.visit_expr(index);
            }
            AnalyzedExprKind::Lambda { body, .. } => {
                self.visit_expr(body);
            }
            AnalyzedExprKind::Some(inner) => {
                self.visit_expr(inner);
            }
            AnalyzedExprKind::Ok(inner) => {
                self.visit_expr(inner);
            }
            AnalyzedExprKind::Err(inner) => {
                self.visit_expr(inner);
            }
            AnalyzedExprKind::Unwrap(inner) => {
                self.visit_expr(inner);
            }
            AnalyzedExprKind::Try(inner) => {
                self.visit_expr(inner);
            }
            AnalyzedExprKind::Assert { condition } => {
                self.visit_expr(condition);
            }
            AnalyzedExprKind::AssertEq { left, right } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            AnalyzedExprKind::Range { start, end, .. } => {
                self.visit_expr(start);
                self.visit_expr(end);
            }
            AnalyzedExprKind::NumberLiteral(_)
            | AnalyzedExprKind::StringLiteral(_)
            | AnalyzedExprKind::BooleanLiteral(_)
            | AnalyzedExprKind::None
            | AnalyzedExprKind::CollectionNew { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_auditor_unused_var() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("unused.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ 10 ἔστω.\n".as_bytes()).unwrap();
        }

        let result = run_auditor(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_auditor_unused_mut() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("mut.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("μετά ξ 10 ἔστω. ξ λέγε.\n".as_bytes()).unwrap();
        }

        let result = run_auditor(&input_path);
        assert!(result.is_ok());
    }

    fn dummy_expr() -> AnalyzedExpr {
        AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: crate::semantic::GlossaType::Boolean,
        }
    }

    #[test]
    fn test_auditor_visitor_coverage_statements() {
        let mut visitor = AuditorVisitor::new();

        let statements = vec![
            AnalyzedStatement::Binding {
                name: "x".into(),
                value: dummy_expr(),
                mutable: true,
            },
            AnalyzedStatement::Binding {
                name: "x_immut".into(),
                value: dummy_expr(),
                mutable: false,
            },
            AnalyzedStatement::Assignment {
                name: "x".into(),
                value: dummy_expr(),
            },
            AnalyzedStatement::Assignment {
                name: "unbound".into(),
                value: dummy_expr(),
            },
            AnalyzedStatement::Print(vec![dummy_expr()]),
            AnalyzedStatement::Print(vec![]),
            AnalyzedStatement::Expression(vec![dummy_expr()]),
            AnalyzedStatement::Expression(vec![]),
            AnalyzedStatement::Query(vec![dummy_expr()]),
            AnalyzedStatement::Query(vec![]),
            AnalyzedStatement::If {
                condition: Box::new(dummy_expr()),
                then_body: vec![AnalyzedStatement::Break],
                else_body: Some(vec![AnalyzedStatement::Continue]),
            },
            AnalyzedStatement::If {
                condition: Box::new(dummy_expr()),
                then_body: vec![],
                else_body: Some(vec![]),
            },
            AnalyzedStatement::If {
                condition: Box::new(dummy_expr()),
                then_body: vec![AnalyzedStatement::Break],
                else_body: None,
            },
            AnalyzedStatement::While {
                condition: Box::new(dummy_expr()),
                body: vec![AnalyzedStatement::Break],
            },
            AnalyzedStatement::While {
                condition: Box::new(dummy_expr()),
                body: vec![],
            },
            AnalyzedStatement::For {
                variable: "y".into(),
                iterator: Box::new(dummy_expr()),
                body: vec![AnalyzedStatement::Break],
            },
            AnalyzedStatement::For {
                variable: "z".into(),
                iterator: Box::new(dummy_expr()),
                body: vec![],
            },
            AnalyzedStatement::Match {
                scrutinee: Box::new(dummy_expr()),
                arms: vec![(dummy_expr(), vec![AnalyzedStatement::Break])],
            },
            AnalyzedStatement::Match {
                scrutinee: Box::new(dummy_expr()),
                arms: vec![],
            },
            AnalyzedStatement::FunctionDef {
                name: "f".into(),
                params: vec![("p".into(), Some(crate::semantic::GlossaType::Number))],
                body: vec![AnalyzedStatement::Break],
                return_type: None,
            },
            AnalyzedStatement::FunctionDef {
                name: "g".into(),
                params: vec![],
                body: vec![],
                return_type: None,
            },
            AnalyzedStatement::Return {
                value: Some(Box::new(dummy_expr())),
            },
            AnalyzedStatement::Return { value: None },
            AnalyzedStatement::TestDeclaration {
                name: "t".into(),
                body: vec![AnalyzedStatement::Break],
            },
            AnalyzedStatement::TestDeclaration {
                name: "t_empty".into(),
                body: vec![],
            },
            AnalyzedStatement::TypeDefinition {
                name: "T".into(),
                fields: vec![],
            },
            AnalyzedStatement::TraitDefinition {
                name: "Tr".into(),
                methods: vec![],
            },
            AnalyzedStatement::TraitImplementation {
                trait_name: "Tr".into(),
                type_name: "T".into(),
                methods: vec![],
            },
        ];

        for stmt in statements {
            visitor.visit_statement(&stmt);
        }
    }

    #[test]
    fn test_auditor_visitor_coverage_expressions() {
        let mut visitor = AuditorVisitor::new();

        let exprs = vec![
            AnalyzedExprKind::Variable("x".into()),
            AnalyzedExprKind::Variable("unbound".into()),
            AnalyzedExprKind::BinOp {
                left: Box::new(dummy_expr()),
                op: crate::morphology::lexicon::BinaryOp::Add,
                right: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::UnaryOp {
                op: crate::morphology::lexicon::UnaryOp::Not,
                operand: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::StructInstantiation {
                type_name: "T".into(),
                fields: vec![],
                args: vec![dummy_expr()],
            },
            AnalyzedExprKind::StructInstantiation {
                type_name: "T".into(),
                fields: vec![],
                args: vec![],
            },
            AnalyzedExprKind::PropertyAccess {
                owner: Box::new(dummy_expr()),
                property: "p".into(),
            },
            AnalyzedExprKind::MethodCall {
                receiver: Box::new(dummy_expr()),
                method: "m".into(),
                args: vec![dummy_expr()],
            },
            AnalyzedExprKind::MethodCall {
                receiver: Box::new(dummy_expr()),
                method: "m".into(),
                args: vec![],
            },
            AnalyzedExprKind::FunctionCall {
                func: "f".into(),
                args: vec![dummy_expr()],
            },
            AnalyzedExprKind::FunctionCall {
                func: "f".into(),
                args: vec![],
            },
            AnalyzedExprKind::VerbCall {
                verb: "v".into(),
                args: vec![dummy_expr()],
            },
            AnalyzedExprKind::VerbCall {
                verb: "v".into(),
                args: vec![],
            },
            AnalyzedExprKind::ArrayLiteral(vec![dummy_expr()]),
            AnalyzedExprKind::ArrayLiteral(vec![]),
            AnalyzedExprKind::IndexAccess {
                array: Box::new(dummy_expr()),
                index: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::Lambda {
                params: vec![],
                body: Box::new(dummy_expr()),
                capture_mode: crate::semantic::CaptureMode::Borrow,
            },
            AnalyzedExprKind::Some(Box::new(dummy_expr())),
            AnalyzedExprKind::Ok(Box::new(dummy_expr())),
            AnalyzedExprKind::Err(Box::new(dummy_expr())),
            AnalyzedExprKind::Unwrap(Box::new(dummy_expr())),
            AnalyzedExprKind::Try(Box::new(dummy_expr())),
            AnalyzedExprKind::Assert {
                condition: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::AssertEq {
                left: Box::new(dummy_expr()),
                right: Box::new(dummy_expr()),
            },
            AnalyzedExprKind::Range {
                start: Box::new(dummy_expr()),
                end: Box::new(dummy_expr()),
                inclusive: false,
            },
            AnalyzedExprKind::NumberLiteral(1),
            AnalyzedExprKind::StringLiteral("s".into()),
            AnalyzedExprKind::BooleanLiteral(true),
            AnalyzedExprKind::None,
            AnalyzedExprKind::CollectionNew {
                collection_type: "HashMap".into(),
            },
        ];

        for kind in exprs {
            let expr = AnalyzedExpr {
                expr: kind,
                glossa_type: crate::semantic::GlossaType::Boolean,
            };
            visitor.visit_expr(&expr);
        }
    }
}
