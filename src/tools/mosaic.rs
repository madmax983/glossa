//! The Mosaic Tool ("Mosaic")
//!
//! This module implements the "Mosaic" functionality, which visualizes the internal
//! "Assembled Statement" structure of a ΓΛΩΣΣΑ program.
//!
//! # Purpose
//!
//! Ancient Greek is a language of "free word order". The meaning is determined by
//! case endings, not position.
//!
//! "Mosaic" reveals how the compiler's [`Assembler`](crate::semantic::Assembler)
//! deconstructs this freedom. It shows exactly which words land in which grammatical
//! slot (Subject, Object, Verb, etc.), proving that `SOV`, `VSO`, and `OVS` all map
//! to the same semantic structure.
//!
//! # The Output
//!
//! The output is a table where each row represents a single statement.
//!
//! | Slot | Color | Role |
//! |------|-------|------|
//! | **Subject** | Cyan | The Agent (Nominative case) |
//! | **Verb** | Yellow | The Action |
//! | **Object** | Green | The Patient (Accusative case) |
//! | **Indirect** | Magenta | The Recipient (Dative case) |
//! | **Other** | Grey | Modifiers, Genitives, Literals |

use crate::morphology::lexicon::BinaryOp;
use crate::parser::parse;
use crate::semantic::assembly_model::Literal;
use crate::semantic::{assemble_statement, AssembledStatement, Constituent};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

/// Run the Mosaic tool on a file
///
/// Reads the source file, parses it, and prints the semantic assembly table to stdout.
pub fn run_mosaic(input_path: &PathBuf) -> Result<()> {
    let source = std::fs::read_to_string(input_path).into_diagnostic()?;
    run_mosaic_inner(&source, &mut std::io::stdout())
}

/// Internal implementation of Mosaic logic
///
/// Separated for testing purposes (allows injecting a writer).
pub fn run_mosaic_inner<W: std::io::Write>(source: &str, writer: &mut W) -> Result<()> {
    let program = parse(source)?;

    writeln!(writer, "\n🎨  Mosaic: Semantic Assembly Visualization").into_diagnostic()?;
    writeln!(
        writer,
        "    Shows how Greek morphology maps to semantic slots.\n"
    )
    .into_diagnostic()?;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Line").add_attribute(Attribute::Bold),
            Cell::new("Subject (Nom)")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Verb (Action)")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
            Cell::new("Object (Acc)")
                .add_attribute(Attribute::Bold)
                .fg(Color::Green),
            Cell::new("Indirect (Dat)")
                .add_attribute(Attribute::Bold)
                .fg(Color::Magenta),
            Cell::new("Other")
                .add_attribute(Attribute::Bold)
                .fg(Color::DarkGrey),
        ]);

    for (i, stmt) in program.statements.iter().enumerate() {
        // Only assemble regular statements (others like TypeDef don't go through Assembler in the same way)
        // Check if it's a regular statement
        if let crate::ast::Statement::Regular { .. } = stmt {
            match assemble_statement(stmt) {
                Ok(assembled) => {
                    add_row(&mut table, i + 1, &assembled);
                }
                Err(e) => {
                    table.add_row(vec![
                        Cell::new(format!("{}", i + 1)),
                        Cell::new(format!("Error: {}", e)).fg(Color::Red),
                        Cell::new(""),
                        Cell::new(""),
                        Cell::new(""),
                        Cell::new(""),
                    ]);
                }
            }
        } else {
            // For non-regular statements, just print the type
            let type_name = match stmt {
                crate::ast::Statement::TypeDefinition(_) => "Type Definition",
                crate::ast::Statement::TraitDefinition(_) => "Trait Definition",
                crate::ast::Statement::TraitImpl(_) => "Trait Implementation",
                crate::ast::Statement::TestDeclaration(_) => "Test Declaration",
                _ => "Unknown",
            };
            table.add_row(vec![
                Cell::new(format!("{}", i + 1)),
                Cell::new(type_name)
                    .fg(Color::Blue)
                    .add_attribute(Attribute::Italic),
                Cell::new(""),
                Cell::new(""),
                Cell::new(""),
                Cell::new(""),
            ]);
        }
    }

    writeln!(writer, "{}", table).into_diagnostic()?;
    Ok(())
}

fn add_row(table: &mut Table, line: usize, asm: &AssembledStatement) {
    let subject = asm
        .subject
        .as_ref()
        .map(fmt_constituent)
        .unwrap_or_default();

    // Combine nominatives if subject is present or if there are multiple
    let extra_noms = asm
        .nominatives
        .iter()
        .map(fmt_constituent)
        .collect::<Vec<_>>()
        .join(", ");
    let full_subject = if !extra_noms.is_empty() {
        if !subject.is_empty() {
            format!("{} (+ {})", subject, extra_noms)
        } else {
            extra_noms
        }
    } else {
        subject
    };

    let verb = asm
        .verb
        .as_ref()
        .map(|v| v.original.to_string())
        .unwrap_or_default();
    let object = asm.object.as_ref().map(fmt_constituent).unwrap_or_default();
    let indirect = asm
        .indirect
        .as_ref()
        .map(fmt_constituent)
        .unwrap_or_default();

    // Collect other interesting things
    let mut other = Vec::new();

    if !asm.literals.is_empty() {
        let lits = asm
            .literals
            .iter()
            .map(fmt_literal)
            .collect::<Vec<_>>()
            .join(", ");
        other.push(format!("Vals: [{}]", lits));
    }

    if !asm.operators.is_empty() {
        let ops = asm
            .operators
            .iter()
            .map(fmt_operator)
            .collect::<Vec<_>>()
            .join(" ");
        other.push(format!("Ops: [{}]", ops));
    }

    if !asm.genitives.is_empty() {
        let gens = asm
            .genitives
            .iter()
            .map(fmt_constituent)
            .collect::<Vec<_>>()
            .join(", ");
        other.push(format!("Gen: [{}]", gens));
    }
    if !asm.adjectives.is_empty() {
        let adjs = asm
            .adjectives
            .iter()
            .map(fmt_constituent)
            .collect::<Vec<_>>()
            .join(", ");
        other.push(format!("Adj: [{}]", adjs));
    }
    if !asm.participles.is_empty() {
        let parts = asm
            .participles
            .iter()
            .map(|p| p.original.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        other.push(format!("Participles: [{}]", parts));
    }
    if asm.is_query {
        other.push("Query (?)".to_string());
    }

    table.add_row(vec![
        Cell::new(format!("{}", line)),
        Cell::new(full_subject),
        Cell::new(verb),
        Cell::new(object),
        Cell::new(indirect),
        Cell::new(other.join(", ")),
    ]);
}

fn fmt_constituent(c: &Constituent) -> String {
    c.original.to_string()
}

fn fmt_literal(l: &Literal) -> String {
    match l {
        Literal::String(s) => format!("\"{}\"", s),
        Literal::Number(n) => n.to_string(),
        Literal::Boolean(b) => b.to_string(),
    }
}

fn fmt_operator(op: &BinaryOp) -> String {
    match op {
        BinaryOp::Add => "+".to_string(),
        BinaryOp::Sub => "-".to_string(),
        BinaryOp::Mul => "*".to_string(),
        BinaryOp::Div => "/".to_string(),
        BinaryOp::Mod => "%".to_string(),
        BinaryOp::Eq => "==".to_string(),
        BinaryOp::Ne => "!=".to_string(),
        BinaryOp::Lt => "<".to_string(),
        BinaryOp::Le => "<=".to_string(),
        BinaryOp::Gt => ">".to_string(),
        BinaryOp::Ge => ">=".to_string(),
        BinaryOp::And => "&&".to_string(),
        BinaryOp::Or => "||".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mosaic_output() {
        let source = "ὁ ἄνθρωπος τὸν λόγον λέγει.";
        let mut buffer = Vec::new();
        run_mosaic_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("ἄνθρωπος")); // Subject
        assert!(output.contains("λόγον")); // Object
        assert!(output.contains("λέγει")); // Verb
    }

    #[test]
    fn test_mosaic_non_regular() {
        let source = "εἶδος Χ ὁρίζειν { x ἀριθμοῦ. }.";
        let mut buffer = Vec::new();
        run_mosaic_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("Type Definition"));
    }

    #[test]
    fn test_mosaic_literals_and_ops() {
        let source = "ξ 10 ἔστω. εἰ ξ 5 μεῖζον ᾖ, «ναι» λέγε.";
        let mut buffer = Vec::new();
        run_mosaic_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Check for literals formatting
        assert!(output.contains("Vals: [10]"));
        assert!(output.contains("Vals: [5, \"ναι\"]")); // Depending on how they are grouped per line

        // Check for operator formatting
        assert!(output.contains("Ops: [>]"));
    }
}
