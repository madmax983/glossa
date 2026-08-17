//! The Automaton (ὁ Αὐτόματος) - JavaScript Exporter
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use std::path::Path;

pub fn run_automaton(input: &Path) -> miette::Result<()> {
    let source = crate::tools::runner::load_source(input)?;
    let status = crate::tools::ui::Status::start_with_symbol("Αὐτόματος (Transpiling to JS)", "🤖");
    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα");
            return Err(e);
        }
    };
    let js_code = transpile_to_js(&program);
    status.success();
    println!("\n   {}", "Γ Λ Ω Σ Σ Α   A U T O M A T O N".bold().cyan());
    println!("   {}\n", "JavaScript Transpilation Result".italic().dim());
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(vec![
        Cell::new("JavaScript Source Code")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
    ]);
    table.add_row(vec![Cell::new(&js_code)]);
    println!("{table}\n");
    Ok(())
}

pub fn transpile_to_js(program: &AnalyzedProgram) -> String {
    let mut js = String::new();
    for stmt in &program.statements {
        js.push_str(&transpile_statement(stmt));
        js.push('\n');
    }
    js
}

fn transpile_statement(stmt: &AnalyzedStatement) -> String {
    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let keyword = if *mutable { "let" } else { "const" };
            format!("{} {} = {};", keyword, name, transpile_expr(value))
        }
        AnalyzedStatement::Assignment { name, value } => {
            format!("{} = {};", name, transpile_expr(value))
        }
        AnalyzedStatement::Print(exprs) => {
            let args: Vec<String> = exprs.iter().map(transpile_expr).collect();
            format!("console.log({});", args.join(", "))
        }
        _ => String::from("/* Unsupported statement */"),
    }
}

fn transpile_expr(expr: &AnalyzedExpr) -> String {
    match &expr.expr {
        AnalyzedExprKind::NumberLiteral(n) => n.to_string(),
        AnalyzedExprKind::StringLiteral(s) => format!("\"{}\"", s),
        AnalyzedExprKind::BooleanLiteral(b) => b.to_string(),
        AnalyzedExprKind::Variable(name) => name.to_string(),
        _ => String::from("/* Unsupported expr */"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{
        AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement, GlossaType, Scope,
    };
    use smol_str::SmolStr;

    #[test]
    fn test_transpile_binding() {
        let scope = Scope::new();
        let program = AnalyzedProgram {
            statements: vec![AnalyzedStatement::Binding {
                name: SmolStr::new("x"),
                value: AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(42),
                    glossa_type: GlossaType::Number,
                },
                mutable: false,
            }],
            scope,
        };
        let js = transpile_to_js(&program);
        assert_eq!(js.trim(), "const x = 42;");
    }
}
