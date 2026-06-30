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
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};
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

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    status.success();

    let mut usage_count = FxHashMap::default();
    let mut mutation_count = FxHashMap::default();
    let mut mutable_vars = FxHashSet::default();
    for stmt in &program.statements {
        visit_statement(
            stmt,
            &mut usage_count,
            &mut mutation_count,
            &mut mutable_vars,
        );
    }

    let mut issues = 0;

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A U D I T O R".cyan().bold());
    println!(
        "   {}",
        format!("Audit Report for {}", input.display())
            .italic()
            .dim()
    );
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Type").add_attribute(Attribute::Bold),
        Cell::new("Variable").add_attribute(Attribute::Bold),
        Cell::new("Message").add_attribute(Attribute::Bold),
    ]);

    for (var, count) in &usage_count {
        if *count == 0 {
            table.add_row(vec![
                Cell::new("⚠️ Unused Variable").fg(Color::Yellow),
                Cell::new(var),
                Cell::new("Declared but never used"),
            ]);
            issues += 1;
        }
    }

    for (var, count) in &mutation_count {
        if *count == 0 && mutable_vars.contains(var) && usage_count.get(var).unwrap_or(&0) > &0 {
            table.add_row(vec![
                Cell::new("💡 Unnecessary Mutation").fg(Color::Blue),
                Cell::new(var),
                Cell::new("Declared mutable ('μετά') but never changed"),
            ]);
            issues += 1;
        }
    }

    if issues == 0 {
        println!(
            "   {}",
            "✨ No issues found. The code is pure.".green().bold()
        );
        println!();
    } else {
        println!("{table}");
        println!("\n   Total issues found: {}", issues);
    }

    Ok(())
}

fn visit_if_statement(
    condition: &AnalyzedExpr,
    then_body: &[AnalyzedStatement],
    else_body: &Option<Vec<AnalyzedStatement>>,
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    visit_expr(condition, usage_count);
    for s in then_body {
        visit_statement(s, usage_count, mutation_count, mutable_vars);
    }
    if let Some(else_stmts) = else_body {
        for s in else_stmts {
            visit_statement(s, usage_count, mutation_count, mutable_vars);
        }
    }
}

fn visit_while_loop(
    condition: &AnalyzedExpr,
    body: &[AnalyzedStatement],
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    visit_expr(condition, usage_count);
    for s in body {
        visit_statement(s, usage_count, mutation_count, mutable_vars);
    }
}

fn visit_for_loop(
    variable: &smol_str::SmolStr,
    iterator: &AnalyzedExpr,
    body: &[AnalyzedStatement],
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    usage_count.insert(variable.clone(), 0);
    visit_expr(iterator, usage_count);
    for s in body {
        visit_statement(s, usage_count, mutation_count, mutable_vars);
    }
}

fn visit_match_statement(
    scrutinee: &AnalyzedExpr,
    arms: &[(AnalyzedExpr, Vec<AnalyzedStatement>)],
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    visit_expr(scrutinee, usage_count);
    for (expr, stmts) in arms {
        visit_expr(expr, usage_count);
        for s in stmts {
            visit_statement(s, usage_count, mutation_count, mutable_vars);
        }
    }
}

fn visit_function_def(
    params: &[(smol_str::SmolStr, Option<crate::semantic::GlossaType>)],
    body: &[AnalyzedStatement],
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    for (param_name, _) in params {
        usage_count.insert(param_name.clone(), 0);
    }
    for s in body {
        visit_statement(s, usage_count, mutation_count, mutable_vars);
    }
}

pub fn visit_statement(
    stmt: &AnalyzedStatement,
    usage_count: &mut FxHashMap<SmolStr, usize>,
    mutation_count: &mut FxHashMap<SmolStr, usize>,
    mutable_vars: &mut FxHashSet<SmolStr>,
) {
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            usage_count.insert(name.clone(), 0);
            mutation_count.insert(name.clone(), 0);
            if *mutable {
                mutable_vars.insert(name.clone());
            }
            visit_expr(value, usage_count);
        }
        AnalyzedStatement::Assignment { name, value } => {
            if let Some(count) = mutation_count.get_mut(name) {
                *count += 1;
            }
            if let Some(count) = usage_count.get_mut(name) {
                *count += 1;
            }
            visit_expr(value, usage_count);
        }
        AnalyzedStatement::Print(exprs) => {
            for expr in exprs {
                visit_expr(expr, usage_count);
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            for expr in exprs {
                visit_expr(expr, usage_count);
            }
        }
        AnalyzedStatement::Query(exprs) => {
            for expr in exprs {
                visit_expr(expr, usage_count);
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            visit_if_statement(
                condition,
                then_body,
                else_body,
                usage_count,
                mutation_count,
                mutable_vars,
            );
        }
        AnalyzedStatement::While { condition, body } => {
            visit_while_loop(condition, body, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            visit_for_loop(
                variable,
                iterator,
                body,
                usage_count,
                mutation_count,
                mutable_vars,
            );
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            visit_match_statement(scrutinee, arms, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedStatement::FunctionDef { params, body, .. } => {
            visit_function_def(params, body, usage_count, mutation_count, mutable_vars);
        }
        AnalyzedStatement::Return { value } => {
            if let Some(v) = value {
                visit_expr(v, usage_count);
            }
        }
        AnalyzedStatement::TestDeclaration { body, .. } => {
            for s in body {
                visit_statement(s, usage_count, mutation_count, mutable_vars);
            }
        }
        AnalyzedStatement::Break
        | AnalyzedStatement::Continue
        | AnalyzedStatement::TypeDefinition { .. }
        | AnalyzedStatement::TraitDefinition { .. }
        | AnalyzedStatement::TraitImplementation { .. } => {}
    }
}

pub fn visit_expr(expr: &AnalyzedExpr, usage_count: &mut FxHashMap<SmolStr, usize>) {
    match &expr.expr {
        AnalyzedExprKind::Variable(name) => {
            if let Some(count) = usage_count.get_mut(name) {
                *count += 1;
            }
        }
        AnalyzedExprKind::BinOp { left, right, .. } => {
            visit_expr(left, usage_count);
            visit_expr(right, usage_count);
        }
        AnalyzedExprKind::UnaryOp { operand, .. } => {
            visit_expr(operand, usage_count);
        }
        AnalyzedExprKind::PropertyAccess { owner, .. } => {
            visit_expr(owner, usage_count);
        }
        AnalyzedExprKind::MethodCall { receiver, args, .. } => {
            visit_expr(receiver, usage_count);
            for arg in args {
                visit_expr(arg, usage_count);
            }
        }
        AnalyzedExprKind::FunctionCall { args, .. } => {
            for arg in args {
                visit_expr(arg, usage_count);
            }
        }
        AnalyzedExprKind::VerbCall { args, .. } => {
            for arg in args {
                visit_expr(arg, usage_count);
            }
        }
        AnalyzedExprKind::StructInstantiation { args, .. } => {
            for arg in args {
                visit_expr(arg, usage_count);
            }
        }
        AnalyzedExprKind::ArrayLiteral(elements) => {
            for el in elements {
                visit_expr(el, usage_count);
            }
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            visit_expr(array, usage_count);
            visit_expr(index, usage_count);
        }
        AnalyzedExprKind::Lambda { body, .. } => {
            visit_expr(body, usage_count);
        }
        AnalyzedExprKind::Some(inner) => {
            visit_expr(inner, usage_count);
        }
        AnalyzedExprKind::Ok(inner) | AnalyzedExprKind::Err(inner) => {
            visit_expr(inner, usage_count);
        }
        AnalyzedExprKind::Unwrap(inner) | AnalyzedExprKind::Try(inner) => {
            visit_expr(inner, usage_count);
        }
        AnalyzedExprKind::Assert { condition } => {
            visit_expr(condition, usage_count);
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            visit_expr(left, usage_count);
            visit_expr(right, usage_count);
        }
        AnalyzedExprKind::Range { start, end, .. } => {
            visit_expr(start, usage_count);
            visit_expr(end, usage_count);
        }
        AnalyzedExprKind::NumberLiteral(_)
        | AnalyzedExprKind::StringLiteral(_)
        | AnalyzedExprKind::BooleanLiteral(_)
        | AnalyzedExprKind::None
        | AnalyzedExprKind::CollectionNew { .. } => {}
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
        let mut usage_count = FxHashMap::default();
        let mut mutation_count = FxHashMap::default();
        let mut mutable_vars = FxHashSet::default();

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
            visit_statement(
                &stmt,
                &mut usage_count,
                &mut mutation_count,
                &mut mutable_vars,
            );
        }
    }

    #[test]
    fn test_auditor_visitor_coverage_expressions() {
        let mut usage_count = FxHashMap::default();

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
            visit_expr(&expr, &mut usage_count);
        }
    }

    #[test]
    fn test_auditor_error_paths() {
        // Test file not found error path
        let result = run_auditor(Path::new("nonexistent.gl"));
        assert!(result.is_err());

        // Test syntax/semantic error path
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("error.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("invalid syntax that fails analysis\n".as_bytes())
                .unwrap();
        }
        let result = run_auditor(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_auditor_output_success_and_issues() {
        let dir = tempfile::tempdir().unwrap();

        // 1. Test code with NO issues (pure)
        let pure_path = dir.path().join("pure.γλ");
        {
            let mut f = std::fs::File::create(&pure_path).unwrap();
            f.write_all("«pure» λέγε.\n".as_bytes()).unwrap();
        }
        let result = run_auditor(&pure_path);
        assert!(result.is_ok());

        // 2. Test code with unused variable issue
        let issue_path = dir.path().join("issue.γλ");
        {
            let mut f = std::fs::File::create(&issue_path).unwrap();
            f.write_all("ξ 5 ἔστω.\n".as_bytes()).unwrap(); // declared, unused
        }
        let result = run_auditor(&issue_path);
        assert!(result.is_ok());

        // 3. Test code with unnecessary mutation issue
        let mut_path = dir.path().join("mut.γλ");
        {
            let mut f = std::fs::File::create(&mut_path).unwrap();
            // Declared mutable but never reassigned, though used
            f.write_all("μετὰ ξ 5 ἔστω.\nξ λέγε.\n".as_bytes()).unwrap();
        }
        let result = run_auditor(&mut_path);
        assert!(result.is_ok());
    }
}
