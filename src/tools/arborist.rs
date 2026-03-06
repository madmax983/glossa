//! The Arborist Tool ("Arborist")
//!
//! This module implements the "Arborist" functionality, which visualizes the
//! Abstract Syntax Tree (AST) of a ΓΛΩΣΣΑ program.
//!
//! # Purpose
//!
//! "Arborist" reveals the hierarchical structure of the analyzed code, making it
//! easier to understand how expressions and statements are nested.

use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, analyze_program};
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// Run the Arborist tool on a file
pub fn run_tree(input: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Δενδροκομία (Planting Tree)", "🌳");

    let source = crate::tools::runner::load_source(input)?;
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    let tree = generate_tree(&program);

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   T R E E".bold().green());
    println!("   {}", "Abstract Syntax Tree".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    if tree.is_empty() {
        table.set_header(vec![
            Cell::new("Status")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
        ]);
        table.add_row(vec![
            Cell::new("No AST nodes found.")
                .fg(Color::DarkGrey)
                .add_attribute(Attribute::Italic),
        ]);
        println!("{table}");
        println!();
    } else {
        table.set_header(vec![
            Cell::new("AST Visualization")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
        ]);
        table.add_row(vec![Cell::new(tree)]);

        println!("{table}");
        println!();
    }

    Ok(())
}

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};

/// Generate a tree representation of an analyzed program
pub fn generate_tree(program: &AnalyzedProgram) -> String {
    let mut output = String::new();
    output.push_str("Program\n");
    for (i, stmt) in program.statements.iter().enumerate() {
        let is_last = i == program.statements.len() - 1;
        format_statement(stmt, "", is_last, &mut output);
    }
    output
}

fn format_statement(stmt: &AnalyzedStatement, prefix: &str, is_last: bool, output: &mut String) {
    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let mutability = if *mutable { " (mutable)" } else { "" };
            output.push_str(&format!(
                "{}{}{}Binding: name: {}{}\n",
                prefix,
                connector,
                "● ".cyan(),
                name,
                mutability
            ));
            format_expr(value, &child_prefix, true, output);
        }
        AnalyzedStatement::Assignment { name, value } => {
            output.push_str(&format!(
                "{}{}{}Assignment: name: {}\n",
                prefix,
                connector,
                "● ".cyan(),
                name
            ));
            format_expr(value, &child_prefix, true, output);
        }
        AnalyzedStatement::Print(exprs) => {
            output.push_str(&format!("{}{}{}Print\n", prefix, connector, "● ".cyan()));
            for (i, expr) in exprs.iter().enumerate() {
                format_expr(expr, &child_prefix, i == exprs.len() - 1, output);
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            output.push_str(&format!(
                "{}{}{}Expression\n",
                prefix,
                connector,
                "● ".cyan()
            ));
            for (i, expr) in exprs.iter().enumerate() {
                format_expr(expr, &child_prefix, i == exprs.len() - 1, output);
            }
        }
        AnalyzedStatement::Query(exprs) => {
            output.push_str(&format!("{}{}{}Query\n", prefix, connector, "● ".cyan()));
            for (i, expr) in exprs.iter().enumerate() {
                format_expr(expr, &child_prefix, i == exprs.len() - 1, output);
            }
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            output.push_str(&format!("{}{}{}If\n", prefix, connector, "● ".cyan()));
            format_expr(condition, &child_prefix, false, output);

            output.push_str(&format!("{}├── Then\n", child_prefix));
            let then_prefix = format!("{}│   ", child_prefix);
            for (i, s) in then_body.iter().enumerate() {
                format_statement(s, &then_prefix, i == then_body.len() - 1, output);
            }

            if let Some(else_stmts) = else_body {
                output.push_str(&format!("{}└── Else\n", child_prefix));
                let else_prefix = format!("{}    ", child_prefix);
                for (i, s) in else_stmts.iter().enumerate() {
                    format_statement(s, &else_prefix, i == else_stmts.len() - 1, output);
                }
            } else {
                // If there's no else body, we need to make sure the last item in `Then` was actually printed with `└──`?
                // Actually, the structure above handles it by having `Then` itself be a branch.
                // It's fine for simple output.
            }
        }
        AnalyzedStatement::While { condition, body } => {
            output.push_str(&format!("{}{}{}While\n", prefix, connector, "● ".cyan()));
            format_expr(condition, &child_prefix, false, output);

            output.push_str(&format!("{}└── Body\n", child_prefix));
            let body_prefix = format!("{}    ", child_prefix);
            for (i, s) in body.iter().enumerate() {
                format_statement(s, &body_prefix, i == body.len() - 1, output);
            }
        }
        AnalyzedStatement::For {
            variable,
            iterator,
            body,
        } => {
            output.push_str(&format!(
                "{}{}{}For: {}\n",
                prefix,
                connector,
                "● ".cyan(),
                variable
            ));
            format_expr(iterator, &child_prefix, false, output);

            output.push_str(&format!("{}└── Body\n", child_prefix));
            let body_prefix = format!("{}    ", child_prefix);
            for (i, s) in body.iter().enumerate() {
                format_statement(s, &body_prefix, i == body.len() - 1, output);
            }
        }
        AnalyzedStatement::Match { scrutinee, arms } => {
            output.push_str(&format!("{}{}{}Match\n", prefix, connector, "● ".cyan()));
            format_expr(scrutinee, &child_prefix, arms.is_empty(), output);

            for (i, (pat, body)) in arms.iter().enumerate() {
                let is_last_arm = i == arms.len() - 1;
                let arm_connector = if is_last_arm {
                    "└── "
                } else {
                    "├── "
                };
                let arm_prefix = format!(
                    "{}{}",
                    child_prefix,
                    if is_last_arm { "    " } else { "│   " }
                );

                output.push_str(&format!(
                    "{}{}{}Arm\n",
                    child_prefix,
                    arm_connector,
                    "○ ".blue()
                ));
                format_expr(pat, &arm_prefix, false, output);

                output.push_str(&format!("{}└── Body\n", arm_prefix));
                let body_prefix = format!("{}    ", arm_prefix);
                for (j, s) in body.iter().enumerate() {
                    format_statement(s, &body_prefix, j == body.len() - 1, output);
                }
            }
        }
        AnalyzedStatement::Break => {
            output.push_str(&format!("{}{}{}Break\n", prefix, connector, "● ".cyan()));
        }
        AnalyzedStatement::Continue => {
            output.push_str(&format!("{}{}{}Continue\n", prefix, connector, "● ".cyan()));
        }
        AnalyzedStatement::Return { value } => {
            output.push_str(&format!("{}{}{}Return\n", prefix, connector, "● ".cyan()));
            if let Some(v) = value {
                format_expr(v, &child_prefix, true, output);
            }
        }
        AnalyzedStatement::FunctionDef {
            name,
            params,
            body,
            return_type,
        } => {
            let ret_str = return_type
                .as_ref()
                .map(|t| format!(" -> {:?}", t))
                .unwrap_or_default();
            let params_str: Vec<String> = params.iter().map(|(n, _)| n.to_string()).collect();
            output.push_str(&format!(
                "{}{}{}FunctionDef: {}({}){}\n",
                prefix,
                connector,
                "● ".cyan(),
                name,
                params_str.join(", "),
                ret_str
            ));

            for (i, s) in body.iter().enumerate() {
                format_statement(s, &child_prefix, i == body.len() - 1, output);
            }
        }
        AnalyzedStatement::TypeDefinition { name, fields } => {
            let fields_str: Vec<String> = fields.iter().map(|(n, _)| n.to_string()).collect();
            output.push_str(&format!(
                "{}{}{}TypeDefinition: {} {{{}}}\n",
                prefix,
                connector,
                "● ".cyan(),
                name,
                fields_str.join(", ")
            ));
        }
        AnalyzedStatement::TraitDefinition { name, methods } => {
            let methods_str: Vec<String> = methods.iter().map(|m| m.name.to_string()).collect();
            output.push_str(&format!(
                "{}{}{}TraitDefinition: {} {{{}}}\n",
                prefix,
                connector,
                "● ".cyan(),
                name,
                methods_str.join(", ")
            ));
        }
        AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            methods,
        } => {
            let methods_str: Vec<String> = methods.iter().map(|m| m.name.to_string()).collect();
            output.push_str(&format!(
                "{}{}{}TraitImplementation: {} for {} {{{}}}\n",
                prefix,
                connector,
                "● ".cyan(),
                trait_name,
                type_name,
                methods_str.join(", ")
            ));
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            output.push_str(&format!(
                "{}{}{}TestDeclaration: {}\n",
                prefix,
                connector,
                "● ".cyan(),
                name
            ));
            for (i, s) in body.iter().enumerate() {
                format_statement(s, &child_prefix, i == body.len() - 1, output);
            }
        }
    }
}

fn format_expr(expr: &AnalyzedExpr, prefix: &str, is_last: bool, output: &mut String) {
    let connector = if is_last { "└── " } else { "├── " };
    let child_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => {
            output.push_str(&format!(
                "{}{}{}StringLiteral: \"{}\"\n",
                prefix,
                connector,
                "○ ".green(),
                s
            ));
        }
        AnalyzedExprKind::NumberLiteral(n) => {
            output.push_str(&format!(
                "{}{}{}NumberLiteral: {}\n",
                prefix,
                connector,
                "○ ".green(),
                n
            ));
        }
        AnalyzedExprKind::BooleanLiteral(b) => {
            output.push_str(&format!(
                "{}{}{}BooleanLiteral: {}\n",
                prefix,
                connector,
                "○ ".green(),
                b
            ));
        }
        AnalyzedExprKind::Variable(name) => {
            output.push_str(&format!(
                "{}{}{}Variable: {}\n",
                prefix,
                connector,
                "○ ".green(),
                name
            ));
        }
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            output.push_str(&format!(
                "{}{}{}PropertyAccess: .{}\n",
                prefix,
                connector,
                "○ ".green(),
                property
            ));
            format_expr(owner, &child_prefix, true, output);
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            output.push_str(&format!(
                "{}{}{}VerbCall: {}\n",
                prefix,
                connector,
                "○ ".green(),
                verb
            ));
            for (i, arg) in args.iter().enumerate() {
                format_expr(arg, &child_prefix, i == args.len() - 1, output);
            }
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            output.push_str(&format!(
                "{}{}{}BinOp: {:?}\n",
                prefix,
                connector,
                "○ ".green(),
                op
            ));
            format_expr(left, &child_prefix, false, output);
            format_expr(right, &child_prefix, true, output);
        }
        AnalyzedExprKind::UnaryOp { op, operand } => {
            output.push_str(&format!(
                "{}{}{}UnaryOp: {:?}\n",
                prefix,
                connector,
                "○ ".green(),
                op
            ));
            format_expr(operand, &child_prefix, true, output);
        }
        AnalyzedExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let op = if *inclusive { "..=" } else { ".." };
            output.push_str(&format!(
                "{}{}{}Range: {}\n",
                prefix,
                connector,
                "○ ".green(),
                op
            ));
            format_expr(start, &child_prefix, false, output);
            format_expr(end, &child_prefix, true, output);
        }
        AnalyzedExprKind::ArrayLiteral(exprs) => {
            output.push_str(&format!(
                "{}{}{}ArrayLiteral\n",
                prefix,
                connector,
                "○ ".green()
            ));
            for (i, e) in exprs.iter().enumerate() {
                format_expr(e, &child_prefix, i == exprs.len() - 1, output);
            }
        }
        AnalyzedExprKind::Some(e) => {
            output.push_str(&format!("{}{}{}Some\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::None => {
            output.push_str(&format!("{}{}{}None\n", prefix, connector, "○ ".green()));
        }
        AnalyzedExprKind::Ok(e) => {
            output.push_str(&format!("{}{}{}Ok\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::Err(e) => {
            output.push_str(&format!("{}{}{}Err\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::Unwrap(e) => {
            output.push_str(&format!("{}{}{}Unwrap\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::Try(e) => {
            output.push_str(&format!("{}{}{}Try\n", prefix, connector, "○ ".green()));
            format_expr(e, &child_prefix, true, output);
        }
        AnalyzedExprKind::IndexAccess { array, index } => {
            output.push_str(&format!(
                "{}{}{}IndexAccess\n",
                prefix,
                connector,
                "○ ".green()
            ));
            format_expr(array, &child_prefix, false, output);
            format_expr(index, &child_prefix, true, output);
        }
        AnalyzedExprKind::FunctionCall { func, args } => {
            output.push_str(&format!(
                "{}{}{}FunctionCall: {}\n",
                prefix,
                connector,
                "○ ".green(),
                func
            ));
            for (i, arg) in args.iter().enumerate() {
                format_expr(arg, &child_prefix, i == args.len() - 1, output);
            }
        }
        AnalyzedExprKind::MethodCall {
            receiver,
            method,
            args,
        } => {
            output.push_str(&format!(
                "{}{}{}MethodCall: {}\n",
                prefix,
                connector,
                "○ ".green(),
                method
            ));
            format_expr(receiver, &child_prefix, args.is_empty(), output);
            for (i, arg) in args.iter().enumerate() {
                format_expr(arg, &child_prefix, i == args.len() - 1, output);
            }
        }
        AnalyzedExprKind::TraitMethodCall {
            receiver,
            trait_name,
            method_name,
            args,
        } => {
            output.push_str(&format!(
                "{}{}{}TraitMethodCall: {}::{}\n",
                prefix,
                connector,
                "○ ".green(),
                trait_name,
                method_name
            ));
            format_expr(receiver, &child_prefix, args.is_empty(), output);
            for (i, arg) in args.iter().enumerate() {
                format_expr(arg, &child_prefix, i == args.len() - 1, output);
            }
        }
        AnalyzedExprKind::StructInstantiation {
            type_name,
            fields,
            args,
        } => {
            output.push_str(&format!(
                "{}{}{}StructInstantiation: {}\n",
                prefix,
                connector,
                "○ ".green(),
                type_name
            ));
            // Print fields as parallel array to args
            for (i, arg) in args.iter().enumerate() {
                let field_name = fields.get(i).map(|s| s.as_str()).unwrap_or("?");
                output.push_str(&format!("{}├── Field: {}\n", child_prefix, field_name));
                format_expr(
                    arg,
                    &format!("{}│   ", child_prefix),
                    i == args.len() - 1,
                    output,
                );
            }
        }
        AnalyzedExprKind::Lambda {
            params,
            body,
            capture_mode,
        } => {
            let params_str = params.join(", ");
            output.push_str(&format!(
                "{}{}{}Lambda: |{}| (mode: {:?})\n",
                prefix,
                connector,
                "○ ".green(),
                params_str,
                capture_mode
            ));
            format_expr(body, &child_prefix, true, output);
        }
        AnalyzedExprKind::CollectionNew { collection_type } => {
            output.push_str(&format!(
                "{}{}{}CollectionNew: {}\n",
                prefix,
                connector,
                "○ ".green(),
                collection_type
            ));
        }
        AnalyzedExprKind::Assert { condition } => {
            output.push_str(&format!("{}{}{}Assert\n", prefix, connector, "○ ".green()));
            format_expr(condition, &child_prefix, true, output);
        }
        AnalyzedExprKind::AssertEq { left, right } => {
            output.push_str(&format!(
                "{}{}{}AssertEq\n",
                prefix,
                connector,
                "○ ".green()
            ));
            format_expr(left, &child_prefix, false, output);
            format_expr(right, &child_prefix, true, output);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_generate_tree_basic() {
        let source = "ξ 10 ἔστω.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();
        let tree = generate_tree(&program);

        assert!(tree.contains("Program"));
        assert!(tree.contains("Binding"));
        assert!(tree.contains("name: ξ"));
        assert!(tree.contains("NumberLiteral: 10"));
    }
}
