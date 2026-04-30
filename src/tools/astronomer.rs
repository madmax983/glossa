//! The Astronomer (ὁ Ἀστρονόμος) - AST Tree Visualizer
//!
//! This module implements a CLI tool to visualize the `AnalyzedProgram` Abstract Syntax Tree.
//! It prints a beautifully formatted, colored, tree-like structure in the terminal using
//! box-drawing characters, similar to the Linux `tree` command.
//!
//! # Purpose
//!
//! "The Astronomer maps the constellations of the AST in the night sky of the terminal."
//! It allows developers to quickly inspect the parsed semantic structure of a ΓΛΩΣΣΑ program
//! without needing external tools like Graphviz.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Run the Astronomer tool on a file
pub fn run_astronomer(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἀστρονόμος (Mapping Constellations)", "🔭");

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
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    status.success();

    println!();
    println!(
        "   {}",
        "🌌 Γ Λ Ω Σ Σ Α   A S T R O N O M E R".bold().cyan()
    );
    println!("   {}", "🔭 Semantic AST Constellation".italic().dim());
    println!();

    println!("{}", "Program".bold().cyan());

    let stmts_len = program.statements.len();
    for (i, stmt) in program.statements.iter().enumerate() {
        print_statement(stmt, "", i == stmts_len - 1);
    }

    println!();
    Ok(())
}

fn print_statement(stmt: &AnalyzedStatement, prefix: &str, is_last: bool) {
    let current_prefix = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let mut_str = if *mutable { " (mut)" } else { "" };
            println!(
                "{}{}{}{}",
                prefix,
                current_prefix,
                "Binding".bold().green(),
                format!(" {}{}", name, mut_str).yellow()
            );
            print_expr(value, &child_prefix, true);
        }
        AnalyzedStatement::Assignment { name, value } => {
            println!(
                "{}{}{}{}",
                prefix,
                current_prefix,
                "Assignment".bold().green(),
                format!(" {}", name).yellow()
            );
            print_expr(value, &child_prefix, true);
        }
        AnalyzedStatement::Print(exprs) => {
            println!("{}{}{}", prefix, current_prefix, "Print".bold().green());
            for (i, expr) in exprs.iter().enumerate() {
                print_expr(expr, &child_prefix, i == exprs.len() - 1);
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            println!(
                "{}{}{}",
                prefix,
                current_prefix,
                "ExpressionStatement".bold().green()
            );
            for (i, expr) in exprs.iter().enumerate() {
                print_expr(expr, &child_prefix, i == exprs.len() - 1);
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            println!("{}{}{}", prefix, current_prefix, "If".bold().green());
            print_expr(condition, &child_prefix, false);

            println!("{}├── {}", child_prefix, "Then".cyan());
            let then_prefix = format!("{}│   ", child_prefix);
            for (i, s) in then_body.iter().enumerate() {
                print_statement(s, &then_prefix, i == then_body.len() - 1);
            }

            if let Some(eb) = else_body {
                println!("{}└── {}", child_prefix, "Else".cyan());
                let else_prefix = format!("{}    ", child_prefix);
                let eb_slice: &[AnalyzedStatement] = eb.as_slice();
                for (i, s) in eb_slice.iter().enumerate() {
                    print_statement(s, &else_prefix, i == eb.len() - 1);
                }
            }
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            println!(
                "{}{}{}{}",
                prefix,
                current_prefix,
                "For".bold().green(),
                format!(" {}", variable).yellow()
            );
            print_expr(iterator, &child_prefix, false);
            println!("{}└── {}", child_prefix, "Body".cyan());
            let body_prefix = format!("{}    ", child_prefix);
            for (i, s) in body.iter().enumerate() {
                print_statement(s, &body_prefix, i == body.len() - 1);
            }
        }
        AnalyzedStatement::While { condition, body } => {
            println!("{}{}{}", prefix, current_prefix, "While".bold().green());
            print_expr(condition, &child_prefix, false);
            println!("{}└── {}", child_prefix, "Body".cyan());
            let body_prefix = format!("{}    ", child_prefix);
            for (i, s) in body.iter().enumerate() {
                print_statement(s, &body_prefix, i == body.len() - 1);
            }
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            println!("{}{}{}", prefix, current_prefix, "Match".bold().green());
            print_expr(scrutinee, &child_prefix, false);

            for (i, (pattern, arm_body)) in arms.iter().enumerate() {
                let is_last_arm = i == arms.len() - 1;
                let arm_prefix = if is_last_arm {
                    "└── "
                } else {
                    "├── "
                };
                let inner_prefix = format!(
                    "{}{}",
                    child_prefix,
                    if is_last_arm { "    " } else { "│   " }
                );

                println!("{}{}{}", child_prefix, arm_prefix, "Arm".cyan());
                print_expr(pattern, &inner_prefix, false);

                println!("{}└── {}", inner_prefix, "Body".cyan());
                let body_prefix = format!("{}    ", inner_prefix);
                for (j, s) in arm_body.iter().enumerate() {
                    print_statement(s, &body_prefix, j == arm_body.len() - 1);
                }
            }
        }
        AnalyzedStatement::Return { value } => {
            println!("{}{}{}", prefix, current_prefix, "Return".bold().green());
            if let Some(v) = value {
                print_expr(v, &child_prefix, true);
            }
        }
        AnalyzedStatement::Break => {
            println!("{}{}{}", prefix, current_prefix, "Break".bold().green());
        }
        AnalyzedStatement::Continue => {
            println!("{}{}{}", prefix, current_prefix, "Continue".bold().green());
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => {
            let func_def_name = name;
            let func_def_body = body;
            let func_def_params = params;
            let func_def_return_type = return_type;
            println!(
                "{}{}{}{}",
                prefix,
                current_prefix,
                "FunctionDef".bold().green(),
                format!(" {}", func_def_name).yellow()
            );

            let has_body = !func_def_body.is_empty();
            let has_returns = func_def_return_type.is_some();

            for (i, (param_name, param_type)) in func_def_params.iter().enumerate() {
                let is_last_param = !has_returns && !has_body && i == func_def_params.len() - 1;
                let branch = if is_last_param {
                    "└── "
                } else {
                    "├── "
                };
                let p_type = param_type
                    .as_ref()
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "?".to_string());
                println!(
                    "{}{}{} {}: {}",
                    child_prefix,
                    branch,
                    "Param".cyan(),
                    param_name,
                    p_type
                );
            }

            if let Some(ret_type) = &func_def_return_type {
                let branch = if !has_body {
                    "└── "
                } else {
                    "├── "
                };
                println!(
                    "{}{}{} {}",
                    child_prefix,
                    branch,
                    "Returns".cyan(),
                    ret_type
                );
            }

            if has_body {
                println!("{}└── {}", child_prefix, "Body".cyan());
                let body_prefix = format!("{}    ", child_prefix);
                for (i, s) in func_def_body.iter().enumerate() {
                    print_statement(s, &body_prefix, i == func_def_body.len() - 1);
                }
            }
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            println!(
                "{}{}{}{}",
                prefix,
                current_prefix,
                "TypeDef".bold().green(),
                format!(" {}", name).yellow()
            );
            for (i, (f_name, f_type)) in fields.iter().enumerate() {
                let branch = if i == fields.len() - 1 {
                    "└── "
                } else {
                    "├── "
                };
                println!(
                    "{}{}{} {}: {}",
                    child_prefix,
                    branch,
                    "Field".cyan(),
                    f_name,
                    f_type
                );
            }
        }
        AnalyzedStatement::TraitDefinition { name, methods } => {
            let trait_def_name = name;
            let trait_def_methods = methods;
            println!(
                "{}{}{}{}",
                prefix,
                current_prefix,
                "TraitDef".bold().green(),
                format!(" {}", trait_def_name).yellow()
            );
            for (i, method) in trait_def_methods.iter().enumerate() {
                let is_last_method = i == trait_def_methods.len() - 1;
                let branch = if is_last_method {
                    "└── "
                } else {
                    "├── "
                };
                let inner_prefix = format!(
                    "{}{}",
                    child_prefix,
                    if is_last_method { "    " } else { "│   " }
                );

                println!(
                    "{}{}{} {}",
                    child_prefix,
                    branch,
                    "Method".cyan(),
                    method.name
                );

                for (j, (param_name, param_type)) in method.params.iter().enumerate() {
                    let is_last_param =
                        method.return_type.is_none() && j == method.params.len() - 1;
                    let m_branch = if is_last_param {
                        "└── "
                    } else {
                        "├── "
                    };
                    let p_type = param_type.to_string();
                    println!(
                        "{}{}{} {}: {}",
                        inner_prefix,
                        m_branch,
                        "Param".cyan(),
                        param_name,
                        p_type
                    );
                }
            }
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods,
        } => {
            println!(
                "{}{}{}{}",
                prefix,
                current_prefix,
                "TraitImpl".bold().green(),
                format!(" {} for {}", trait_name, type_name).yellow()
            );
            for (i, method) in methods.iter().enumerate() {
                let is_last_method = i == methods.len() - 1;
                let branch = if is_last_method {
                    "└── "
                } else {
                    "├── "
                };
                let inner_prefix = format!(
                    "{}{}",
                    child_prefix,
                    if is_last_method { "    " } else { "│   " }
                );

                println!(
                    "{}{}{} {}",
                    child_prefix,
                    branch,
                    "Method".cyan(),
                    method.name
                );

                if let Some(body) = &method.body {
                    println!("{}└── {}", inner_prefix, "Body".cyan());
                    let body_prefix = format!("{}    ", inner_prefix);
                    for (j, s) in body.iter().enumerate() {
                        print_statement(s, &body_prefix, j == body.len() - 1);
                    }
                }
            }
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            println!(
                "{}{}{}{}",
                prefix,
                current_prefix,
                "TestDecl".bold().green(),
                format!(" \"{}\"", name).yellow()
            );
            println!("{}└── {}", child_prefix, "Body".cyan());
            let body_prefix = format!("{}    ", child_prefix);
            for (i, s) in body.iter().enumerate() {
                print_statement(s, &body_prefix, i == body.len() - 1);
            }
        }
        AnalyzedStatement::Query(exprs) => {
            println!("{}{}{}", prefix, current_prefix, "Query".bold().green());
            for (i, expr) in exprs.iter().enumerate() {
                print_expr(expr, &child_prefix, i == exprs.len() - 1);
            }
        }
    }
}

fn print_expr(expr: &AnalyzedExpr, prefix: &str, is_last: bool) {
    let current_prefix = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    let kind_str = match &expr.expr {
        AnalyzedExprKind::NumberLiteral(n) => format!("NumberLiteral ({})", n),
        AnalyzedExprKind::StringLiteral(s) => format!("StringLiteral (\"{}\")", s),
        AnalyzedExprKind::BooleanLiteral(b) => format!("BooleanLiteral ({})", b),
        AnalyzedExprKind::Variable(v) => format!("Variable ({})", v),
        AnalyzedExprKind::PropertyAccess { owner: _, property } => {
            format!("PropertyAccess ({})", property)
        }
        AnalyzedExprKind::VerbCall { verb, args: _ } => format!("VerbCall ({})", verb),
        AnalyzedExprKind::FunctionCall { func, args: _ } => format!("FunctionCall ({})", func),
        AnalyzedExprKind::MethodCall {
            receiver: _,
            method,
            args: _,
        } => format!("MethodCall ({})", method),
        AnalyzedExprKind::BinOp {
            left: _,
            op,
            right: _,
        } => format!("BinOp ({:?})", op),
        AnalyzedExprKind::UnaryOp { op, operand: _ } => format!("UnaryOp ({:?})", op),
        AnalyzedExprKind::Range { inclusive, .. } => format!("Range (inclusive: {})", inclusive),
        AnalyzedExprKind::ArrayLiteral(_) => "ArrayLiteral".to_string(),
        AnalyzedExprKind::Some(_) => "Some".to_string(),
        AnalyzedExprKind::None => "None".to_string(),
        AnalyzedExprKind::Ok(_) => "Ok".to_string(),
        AnalyzedExprKind::Err(_) => "Err".to_string(),
        AnalyzedExprKind::Unwrap(_) => "Unwrap".to_string(),
        AnalyzedExprKind::Try(_) => "Try".to_string(),
        AnalyzedExprKind::IndexAccess { .. } => "IndexAccess".to_string(),
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields: _,
            args: _,
        } => format!("StructInstantiation ({})", type_name),
        AnalyzedExprKind::Lambda {
            params,
            capture_mode,
            ..
        } => format!("Lambda (|{}| {:?})", params.join(", "), capture_mode),
        AnalyzedExprKind::CollectionNew { collection_type } => {
            format!("CollectionNew ({})", collection_type)
        }
        AnalyzedExprKind::Assert { .. } => "Assert".to_string(),
        AnalyzedExprKind::AssertEq { .. } => "AssertEq".to_string(),
    };

    println!("{}{}{}", prefix, current_prefix, kind_str.blue());

    match &expr.expr {
        AnalyzedExprKind::PropertyAccess { owner, .. } => {
            print_expr(owner, &child_prefix, true);
        }
        AnalyzedExprKind::VerbCall { args, .. } => {
            for (i, arg) in args.iter().enumerate() {
                print_expr(arg, &child_prefix, i == args.len() - 1);
            }
        }
        AnalyzedExprKind::FunctionCall { args, .. } => {
            for (i, arg) in args.iter().enumerate() {
                print_expr(arg, &child_prefix, i == args.len() - 1);
            }
        }
        AnalyzedExprKind::MethodCall { receiver, args, .. } => {
            print_expr(receiver, &child_prefix, args.is_empty());
            for (i, arg) in args.iter().enumerate() {
                print_expr(arg, &child_prefix, i == args.len() - 1);
            }
        }
        AnalyzedExprKind::BinOp { left, right, .. } => {
            print_expr(left, &child_prefix, false);
            print_expr(right, &child_prefix, true);
        }
        AnalyzedExprKind::UnaryOp { operand, .. } => {
            print_expr(operand, &child_prefix, true);
        }
        AnalyzedExprKind::Range { start, end, .. } => {
            print_expr(start, &child_prefix, false);
            print_expr(end, &child_prefix, true);
        }
        AnalyzedExprKind::ArrayLiteral(args) => {
            for (i, arg) in args.iter().enumerate() {
                print_expr(arg, &child_prefix, i == args.len() - 1);
            }
        }
        AnalyzedExprKind::Some(inner)
        | AnalyzedExprKind::Ok(inner)
        | AnalyzedExprKind::Err(inner)
        | AnalyzedExprKind::Unwrap(inner)
        | AnalyzedExprKind::Try(inner)
        | AnalyzedExprKind::Assert { condition: inner } => {
            print_expr(inner, &child_prefix, true);
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            print_expr(array, &child_prefix, false);
            print_expr(index, &child_prefix, true);
        }
        AnalyzedExprKind::StructInstantiation { args, .. } => {
            for (i, arg) in args.iter().enumerate() {
                print_expr(arg, &child_prefix, i == args.len() - 1);
            }
        }
        AnalyzedExprKind::Lambda { body, .. } => {
            print_expr(body, &child_prefix, true);
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            print_expr(left, &child_prefix, false);
            print_expr(right, &child_prefix, true);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_astronomer_basic() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.γλ");
        std::fs::write(&file_path, "ξ 10 ἔστω.\n«hello» λέγε.\n").unwrap();

        let result = run_astronomer(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_astronomer_file_not_found() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_astronomer(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_astronomer_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("parse_error.γλ");
        std::fs::write(&input_path, b"invalid syntax").unwrap();

        let result = run_astronomer(&input_path);
        assert!(result.is_err());
    }
}
