//! The Astrolabe (ὁ Ἀστρολάβος) - String Extractor
//!
//! A CLI tool that traverses the analyzed AST and extracts all String Literals
//! into a beautiful table format, acting as an i18n localization asset extractor.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Run the Astrolabe string extractor
pub fn run_astrolabe(input: &Path) -> Result<()> {
    let source = crate::tools::runner::load_source(input)?;
    let status =
        crate::tools::ui::Status::start_with_symbol("Ἀστρολάβος (Extracting Strings)", "🧭");

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    let mut strings = Vec::new();
    for stmt in &program.statements {
        extract_strings_from_stmt(stmt, &mut strings);
    }

    status.success();

    println!("\n   {}", "Γ Λ Ω Σ Σ Α   A S T R O L A B E".cyan().bold());
    println!("   {}\n", "The String Extractor (i18n)".dim().italic());

    let mut table = Table::new();
    table.load_preset(UTF8_FULL).set_header(vec![
        Cell::new("ID").fg(Color::Cyan),
        Cell::new("Extracted String").fg(Color::Yellow),
    ]);

    if strings.is_empty() {
        println!("   No strings found in the program.\n");
        return Ok(());
    }

    for (i, s) in strings.iter().enumerate() {
        let id = format!("str_{:03}", i + 1);
        table.add_row(vec![
            Cell::new(&id).fg(Color::DarkGrey),
            Cell::new(s).fg(Color::Yellow),
        ]);
    }

    println!("{table}");
    Ok(())
}

fn extract_strings_from_stmt(stmt: &AnalyzedStatement, strings: &mut Vec<String>) {
    match stmt {
        AnalyzedStatement::Binding { value, .. } => extract_strings_from_expr(value, strings),
        AnalyzedStatement::Assignment { value, .. } => extract_strings_from_expr(value, strings),
        AnalyzedStatement::Print(exprs) => {
            for e in exprs {
                extract_strings_from_expr(e, strings);
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            for e in exprs {
                extract_strings_from_expr(e, strings);
            }
        }
        AnalyzedStatement::Query(exprs) => {
            for e in exprs {
                extract_strings_from_expr(e, strings);
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            extract_strings_from_expr(condition, strings);
            for s in then_body {
                extract_strings_from_stmt(s, strings);
            }
            if let Some(ebody) = else_body {
                for s in ebody {
                    extract_strings_from_stmt(s, strings);
                }
            }
        }
        AnalyzedStatement::While { condition, body } => {
            extract_strings_from_expr(condition, strings);
            for s in body {
                extract_strings_from_stmt(s, strings);
            }
        }
        AnalyzedStatement::For { iterator, body, .. } => {
            extract_strings_from_expr(iterator, strings);
            for s in body {
                extract_strings_from_stmt(s, strings);
            }
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            extract_strings_from_expr(scrutinee, strings);
            for (pat, body) in arms {
                extract_strings_from_expr(pat, strings);
                for s in body {
                    extract_strings_from_stmt(s, strings);
                }
            }
        }
        AnalyzedStatement::Return { value } => {
            if let Some(val) = value {
                extract_strings_from_expr(val, strings);
            }
        }
        AnalyzedStatement::FunctionDef { body, .. } => {
            for s in body {
                extract_strings_from_stmt(s, strings);
            }
        }
        AnalyzedStatement::TypeDefinition { .. } => {}
        AnalyzedStatement::TraitDefinition { .. } => {}
        AnalyzedStatement::TraitImplementation { methods, .. } => {
            for method in methods {
                for s in &method.body {
                    extract_strings_from_stmt(s, strings);
                }
            }
        }
        AnalyzedStatement::TestDeclaration { body, .. } => {
            for s in body {
                extract_strings_from_stmt(s, strings);
            }
        }
        AnalyzedStatement::Break | AnalyzedStatement::Continue => {}
    }
}

fn extract_strings_from_expr(expr: &AnalyzedExpr, strings: &mut Vec<String>) {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => strings.push(s.clone()),
        AnalyzedExprKind::PropertyAccess { owner, .. } => extract_strings_from_expr(owner, strings),
        AnalyzedExprKind::VerbCall { args, .. } => {
            for a in args {
                extract_strings_from_expr(a, strings);
            }
        }
        AnalyzedExprKind::BinOp { left, right, .. } => {
            extract_strings_from_expr(left, strings);
            extract_strings_from_expr(right, strings);
        }
        AnalyzedExprKind::UnaryOp { operand, .. } => extract_strings_from_expr(operand, strings),
        AnalyzedExprKind::Range { start, end, .. } => {
            extract_strings_from_expr(start, strings);
            extract_strings_from_expr(end, strings);
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            for e in exprs {
                extract_strings_from_expr(e, strings);
            }
        }
        AnalyzedExprKind::Some(val) => extract_strings_from_expr(val, strings),
        AnalyzedExprKind::Ok(val) => extract_strings_from_expr(val, strings),
        AnalyzedExprKind::Err(val) => extract_strings_from_expr(val, strings),
        AnalyzedExprKind::Unwrap(val) => extract_strings_from_expr(val, strings),
        AnalyzedExprKind::Try(val) => extract_strings_from_expr(val, strings),
        AnalyzedExprKind::IndexAccess { array, index } => {
            extract_strings_from_expr(array, strings);
            extract_strings_from_expr(index, strings);
        }
        AnalyzedExprKind::FunctionCall { args, .. } => {
            for a in args {
                extract_strings_from_expr(a, strings);
            }
        }
        AnalyzedExprKind::MethodCall { receiver, args, .. } => {
            extract_strings_from_expr(receiver, strings);
            for a in args {
                extract_strings_from_expr(a, strings);
            }
        }
        AnalyzedExprKind::StructInstantiation { args, .. } => {
            for a in args {
                extract_strings_from_expr(a, strings);
            }
        }
        AnalyzedExprKind::Lambda { body, .. } => {
            // body is Box<AnalyzedExpr>
            extract_strings_from_expr(body, strings);
        }
        AnalyzedExprKind::NumberLiteral(_)
        | AnalyzedExprKind::BooleanLiteral(_)
        | AnalyzedExprKind::Variable(_)
        | AnalyzedExprKind::None => {}
        // Use a fallback as required by Groundedness rule memory.
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_astrolabe_extraction() {
        let code = "
        ξ «χαῖρε» ἔστω.
        «κόσμε» λέγε.
        εἰ ἀληθές ἐστι, «ναι» λέγε.
        ";
        let ast = parse(code).unwrap();
        let program = analyze_program(&ast).unwrap();
        let mut strings = Vec::new();
        for stmt in &program.statements {
            extract_strings_from_stmt(stmt, &mut strings);
        }
        assert_eq!(strings, vec!["χαῖρε", "κόσμε", "ναι"]);
    }
}
