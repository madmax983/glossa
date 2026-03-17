use comfy_table::{presets, Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

use crate::parser::parse;
use crate::semantic::{analyze_program, AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement};

pub fn run_timeline(path: &Path) -> Result<()> {
    let source = std::fs::read_to_string(path).into_diagnostic()?;
    let ast = parse(&source)?;
    let program = analyze_program(&ast)?;

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   Χ Ρ Ο Ν Ο Σ".bold().cyan());
    println!(
        "   {}",
        "The Timeline (Static Analysis Simulator)".italic().dim()
    );
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(vec![
        Cell::new("Event").add_attribute(Attribute::Bold).fg(Color::Cyan),
        Cell::new("Target").add_attribute(Attribute::Bold),
        Cell::new("Details").add_attribute(Attribute::Bold),
        Cell::new("Type").add_attribute(Attribute::Bold),
    ]);

    traverse_statements(&mut table, &program.statements, 0);

    println!("{}", table);

    Ok(())
}

fn traverse_statements(table: &mut Table, statements: &[AnalyzedStatement], depth: usize) {
    let prefix = "  ".repeat(depth);
    for stmt in statements {
        match stmt {
            AnalyzedStatement::Binding {
                name,
                value,
                mutable,
            } => {
                let type_str = format!("{:?}", value.glossa_type);
                let mut_str = if *mutable { " (Mut)" } else { "" };
                table.add_row(vec![
                    Cell::new(format!("{}Birth", prefix)).fg(Color::Green),
                    Cell::new(name.as_str()),
                    Cell::new(format!("Initial value assigned{}", mut_str)),
                    Cell::new(type_str).fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::Assignment { name, value } => {
                let type_str = format!("{:?}", value.glossa_type);
                table.add_row(vec![
                    Cell::new(format!("{}Mutation", prefix)).fg(Color::Red),
                    Cell::new(name.as_str()),
                    Cell::new("Value reassigned"),
                    Cell::new(type_str).fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::Print(exprs) => {
                let targets = exprs.iter().map(tell_expr).collect::<Vec<_>>().join(", ");
                table.add_row(vec![
                    Cell::new(format!("{}IO Side-Effect", prefix)).fg(Color::Blue),
                    Cell::new("stdout"),
                    Cell::new(format!("Print: {}", targets)),
                    Cell::new("Print").fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                table.add_row(vec![
                    Cell::new(format!("{}Control Flow Divergence", prefix)).fg(Color::Yellow),
                    Cell::new("if"),
                    Cell::new(format!("Condition: {}", tell_expr(condition))),
                    Cell::new("Branch").fg(Color::DarkGrey),
                ]);
                traverse_statements(table, then_body, depth + 1);
                if let Some(else_branch) = else_body {
                    table.add_row(vec![
                        Cell::new(format!("{}Control Flow Divergence", prefix)).fg(Color::Yellow),
                        Cell::new("else"),
                        Cell::new("Fallback branch"),
                        Cell::new("Branch").fg(Color::DarkGrey),
                    ]);
                    traverse_statements(table, else_branch, depth + 1);
                }
            }
            AnalyzedStatement::While { condition, body } => {
                table.add_row(vec![
                    Cell::new(format!("{}Control Flow Divergence", prefix)).fg(Color::Yellow),
                    Cell::new("while"),
                    Cell::new(format!("Condition: {}", tell_expr(condition))),
                    Cell::new("Loop").fg(Color::DarkGrey),
                ]);
                traverse_statements(table, body, depth + 1);
            }
            AnalyzedStatement::For {
                variable,
                iterator,
                body,
            } => {
                table.add_row(vec![
                    Cell::new(format!("{}Control Flow Divergence", prefix)).fg(Color::Yellow),
                    Cell::new("for"),
                    Cell::new(format!("Iterating over {}", tell_expr(iterator))),
                    Cell::new("Loop").fg(Color::DarkGrey),
                ]);
                table.add_row(vec![
                    Cell::new(format!("  {}Birth", prefix)).fg(Color::Green),
                    Cell::new(variable.as_str()),
                    Cell::new("Loop iteration variable"),
                    Cell::new("Inferred").fg(Color::DarkGrey),
                ]);
                traverse_statements(table, body, depth + 1);
            }
            AnalyzedStatement::FunctionDef {
                name,
                params,
                return_type,
                ..
            } => {
                let params_str = params
                    .iter()
                    .map(|(n, _)| n.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                table.add_row(vec![
                    Cell::new(format!("{}Birth", prefix)).fg(Color::Green),
                    Cell::new(name.as_str()),
                    Cell::new(format!("Function ({})", params_str)),
                    Cell::new(format!("{:?}", return_type)).fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::TypeDefinition { name, fields } => {
                let fields_str = fields
                    .iter()
                    .map(|(n, _)| n.as_str())
                    .collect::<Vec<_>>()
                    .join(", ");
                table.add_row(vec![
                    Cell::new(format!("{}Birth", prefix)).fg(Color::Green),
                    Cell::new(name.as_str()),
                    Cell::new(format!("Struct {{ {} }}", fields_str)),
                    Cell::new("Type").fg(Color::DarkGrey),
                ]);
            }
            AnalyzedStatement::TestDeclaration { name, body } => {
                table.add_row(vec![
                    Cell::new(format!("{}Test Definition", prefix)).fg(Color::Cyan),
                    Cell::new(name.as_str()),
                    Cell::new("Test declared"),
                    Cell::new("Verification").fg(Color::DarkGrey),
                ]);
                traverse_statements(table, body, depth + 1);
            }
            _ => {
                // Ignore other statements for timeline
            }
        }
    }
}

// Simple formatter to avoid dragging in all of narrator
fn tell_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s),
        AnalyzedExprKind::NumberLiteral(n) => format!("{}", n),
        AnalyzedExprKind::BooleanLiteral(b) => format!("{}", b),
        AnalyzedExprKind::Variable(name) => format!("`{}`", name),
        AnalyzedExprKind::PropertyAccess { owner, property } => {
            format!("{}.{}", tell_expr(owner), property)
        }
        AnalyzedExprKind::VerbCall { verb, args } => {
            let args_str: Vec<String> = args.iter().map(tell_expr).collect();
            format!("{}({})", verb, args_str.join(", "))
        }
        AnalyzedExprKind::BinOp { left, op, right } => {
            format!("({} {:?} {})", tell_expr(left), op, tell_expr(right))
        }
        _ => "<expr>".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeline_variable_birth() {
        let source = "ξ 10 ἔστω.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let mut table = Table::new();
        traverse_statements(&mut table, &program.statements, 0);
        let output = table.to_string();

        assert!(output.contains("Birth"));
        assert!(output.contains('ξ'));
        assert!(output.contains("Initial value assigned"));
    }

    #[test]
    fn test_timeline_mutation() {
        let source = "ξ μετά 10 ἔστω. ξ 20 γίγνεται.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let mut table = Table::new();
        traverse_statements(&mut table, &program.statements, 0);
        let output = table.to_string();

        assert!(output.contains("Mutation"));
        assert!(output.contains("Value reassigned"));
        assert!(output.contains("Birth")); // Has birth too
    }
}
