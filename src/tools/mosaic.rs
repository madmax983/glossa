use comfy_table::{Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::ast::Statement;
use crate::parser::parse;
use crate::semantic::{AssembledStatement, assemble_statement};

/// Run the Mosaic semantic visualizer tool on a file
pub fn run_mosaic(input_path: &PathBuf) -> Result<()> {
    let source = fs::read_to_string(input_path).into_diagnostic()?;
    run_mosaic_on_source(&source, &mut std::io::stdout())
}

/// Run the Mosaic semantic visualizer on a source string, writing output to the given writer
pub fn run_mosaic_on_source<W: Write>(source: &str, writer: &mut W) -> Result<()> {
    let program = parse(source)?;

    writeln!(writer).into_diagnostic()?;
    writeln!(writer, "   {}", "Μ Ω Σ Α Ϊ Κ Ο Ν".bold().cyan()).into_diagnostic()?;
    writeln!(writer, "   {}", "Semantic Assembly Visualizer".italic().dim()).into_diagnostic()?;
    writeln!(writer).into_diagnostic()?;

    for (i, stmt) in program.statements.iter().enumerate() {
        writeln!(writer, "{}", format!("Statement #{}", i + 1).yellow().bold()).into_diagnostic()?;

        match stmt {
            Statement::Regular { .. } => {
                match assemble_statement(stmt) {
                    Ok(assembled) => print_assembled_table(&assembled, writer)?,
                    Err(e) => writeln!(writer, "  {}: {}", "Error".red(), e).into_diagnostic()?,
                }
            }
            Statement::TypeDefinition(def) => {
                writeln!(writer, "  Type Definition: {}", def.name.original.as_str().cyan()).into_diagnostic()?;
            }
            Statement::TraitDefinition(def) => {
                writeln!(writer, "  Trait Definition: {}", def.name.original.as_str().cyan()).into_diagnostic()?;
            }
            Statement::TraitImpl(def) => {
                writeln!(writer, "  Trait Implementation: {} for {}", def.trait_name.original.as_str().cyan(), def.type_name.original.as_str().cyan()).into_diagnostic()?;
            }
            Statement::TestDeclaration(def) => {
                writeln!(writer, "  Test Declaration: {}", def.name.as_str().cyan()).into_diagnostic()?;
            }
        }
        writeln!(writer).into_diagnostic()?;
    }

    Ok(())
}

fn print_assembled_table<W: Write>(stmt: &AssembledStatement, writer: &mut W) -> Result<()> {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL).set_header(vec![
        Cell::new("Role").fg(Color::Magenta).add_attribute(comfy_table::Attribute::Bold),
        Cell::new("Original Text").add_attribute(comfy_table::Attribute::Bold),
        Cell::new("Grammar Info").fg(Color::Cyan).add_attribute(comfy_table::Attribute::Bold),
    ]);

    // Subject
    if let Some(subj) = &stmt.subject {
        table.add_row(vec![
            Cell::new("Subject (Nominative)").fg(Color::Green),
            Cell::new(&subj.original),
            Cell::new(format!("{:?} {:?}", subj.number, subj.gender)),
        ]);
    }

    // Additional Nominatives
    for nom in &stmt.nominatives {
        table.add_row(vec![
            Cell::new("Nominative (Extra)").fg(Color::Green).add_attribute(comfy_table::Attribute::Dim),
            Cell::new(&nom.original),
            Cell::new(format!("{:?} {:?}", nom.number, nom.gender)),
        ]);
    }

    // Verb
    if let Some(verb) = &stmt.verb {
        table.add_row(vec![
            Cell::new("Verb (Action)").fg(Color::Red),
            Cell::new(&verb.original),
            Cell::new(format!("{:?} {:?} {:?}", verb.person, verb.number, verb.tense)),
        ]);
    }

    // Object
    if let Some(obj) = &stmt.object {
        table.add_row(vec![
            Cell::new("Object (Accusative)").fg(Color::Blue),
            Cell::new(&obj.original),
            Cell::new(format!("{:?} {:?}", obj.number, obj.gender)),
        ]);
    }

    // Indirect Object
    if let Some(ind) = &stmt.indirect {
        table.add_row(vec![
            Cell::new("Indirect Object (Dative)").fg(Color::Yellow),
            Cell::new(&ind.original),
            Cell::new(format!("{:?} {:?}", ind.number, ind.gender)),
        ]);
    }

    // Genitives
    for genitive in &stmt.genitives {
        table.add_row(vec![
            Cell::new("Genitive (Possessor)").fg(Color::DarkYellow),
            Cell::new(&genitive.original),
            Cell::new(format!("{:?} {:?}", genitive.number, genitive.gender)),
        ]);
    }

    // Adjectives
    for adj in &stmt.adjectives {
        table.add_row(vec![
            Cell::new("Adjective (Modifier)").fg(Color::Cyan),
            Cell::new(&adj.original),
            Cell::new(format!("{:?} {:?}", adj.number, adj.gender)),
        ]);
    }

    // Participles
    for part in &stmt.participles {
        table.add_row(vec![
            Cell::new("Participle (Lambda)").fg(Color::Cyan),
            Cell::new(&part.original),
            Cell::new(format!("{:?} {:?} {:?}", part.tense, part.voice, part.case)),
        ]);
    }

    // Literals
    for lit in &stmt.literals {
        let (val, type_name) = match lit {
            crate::semantic::Literal::String(s) => (format!("«{}»", s), "String"),
            crate::semantic::Literal::Number(n) => (n.to_string(), "Number"),
            crate::semantic::Literal::Boolean(b) => (b.to_string(), "Boolean"),
        };
        table.add_row(vec![
            Cell::new(format!("Literal ({})", type_name)).fg(Color::White),
            Cell::new(val),
            Cell::new("-"),
        ]);
    }

    // Operators
    for op in &stmt.operators {
         table.add_row(vec![
            Cell::new("Operator").fg(Color::DarkGrey),
            Cell::new(format!("{:?}", op)), // BinaryOp handles Debug
            Cell::new("-"),
        ]);
    }

    writeln!(writer, "{table}").into_diagnostic()?;
    Ok(())
}
