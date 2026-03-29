//! The Simulator (ὁ Προσομοιωτής) - Time-Traveling Execution Engine
//!
//! A specialized execution mode that tracks variable state changes over time
//! and produces a simulation timeline of the program execution.
//! This allows developers to see exactly how variables mutate without having
//! to insert print statements manually.

use crate::semantic::{AnalyzedProgram, AnalyzedStatement};
use crate::tools::interpreter::Interpreter;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;

pub fn run_simulation(program: &AnalyzedProgram) -> Result<()> {
    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S I M U L A T O R".bold().cyan());
    println!("   {}", "Time-Traveling Execution Engine".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(vec![
        Cell::new("Step")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Action")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
        Cell::new("State Changes")
            .add_attribute(Attribute::Bold)
            .fg(Color::Green),
    ]);

    let mut cumulative_stmts = Vec::new();

    for (i, stmt) in program.statements.iter().enumerate() {
        cumulative_stmts.push(stmt.clone());

        let action_desc = match stmt {
            AnalyzedStatement::Binding { name, .. } => format!("Bind `{}`", name),
            AnalyzedStatement::Assignment { name, .. } => format!("Mutate `{}`", name),
            AnalyzedStatement::Expression(_) => "Evaluate Expression".to_string(),
            AnalyzedStatement::Print(_) => "Print".to_string(),
            _ => "Execute Statement".to_string(),
        };

        let current_prog = AnalyzedProgram {
            statements: cumulative_stmts.clone(),
            scope: program.scope.clone(),
        };

        let mut interp = Interpreter::new();
        let result = interp.run(&current_prog);

        let state_desc = match result {
            Ok(_) => {
                match stmt {
                    AnalyzedStatement::Binding { name, .. }
                    | AnalyzedStatement::Assignment { name, .. } => {
                        // Extract state via probing with a dummy print statement
                        let mut probe_stmts = cumulative_stmts.clone();
                        probe_stmts.push(AnalyzedStatement::Print(vec![
                            crate::semantic::AnalyzedExpr {
                                expr: crate::semantic::AnalyzedExprKind::Variable(name.clone()),
                                glossa_type: crate::semantic::GlossaType::Unknown,
                            },
                        ]));
                        let probe_prog = AnalyzedProgram {
                            statements: probe_stmts,
                            scope: program.scope.clone(),
                        };
                        let mut probe_interp = Interpreter::new();
                        if probe_interp.run(&probe_prog).is_ok() {
                            let out = probe_interp.get_output();
                            let lines: Vec<&str> =
                                out.split('\n').filter(|s| !s.is_empty()).collect();
                            let last_out = lines.last().unwrap_or(&"Unknown");
                            format!("{} = {}", name, last_out)
                        } else {
                            "Unknown".to_string()
                        }
                    }
                    AnalyzedStatement::Print(_) => {
                        let out = interp.get_output();
                        let lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
                        let last_out = lines.last().unwrap_or(&"");
                        format!("Printed: {}", last_out)
                    }
                    _ => "Ok".to_string(),
                }
            }
            Err(e) => format!("Error: {:?}", e),
        };

        table.add_row(vec![
            Cell::new(format!("{}", i + 1)),
            Cell::new(action_desc),
            Cell::new(state_desc),
        ]);
    }

    println!("{table}");
    Ok(())
}

use std::path::Path;
pub fn run_simulation_from_file(input: &Path) -> Result<()> {
    let source = crate::tools::runner::load_source(input)?;
    let ast = crate::parser::parse(&source).map_err(|e| miette::miette!("Parse error: {}", e))?;
    let program = crate::semantic::analyze_program(&ast)
        .map_err(|e| miette::miette!("Semantic error: {}", e))?;
    run_simulation(&program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_simulator() {
        let source = "μετά ξ πέντε ἔστω.\nξ δέκα γίγνεται.\nξ λέγε.";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();

        let result = run_simulation(&program);
        assert!(result.is_ok());
    }
}
