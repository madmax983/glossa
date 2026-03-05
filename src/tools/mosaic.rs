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

use crate::parser::parse;
use crate::semantic::{AssembledStatement, Constituent, assemble_statement};
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::path::Path;

/// Run the Mosaic tool on a file
///
/// Reads the source file, parses it, and prints the semantic assembly table to stdout.
pub fn run_mosaic(input_path: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Ψηφιδωτόν (Mosaic)", "🧩");

    let source = crate::tools::runner::load_source(input_path)?;

    // Create a buffer for the table
    let mut buffer = Vec::new();
    run_mosaic_inner(&source, &mut buffer)?;
    let output = String::from_utf8(buffer).into_diagnostic()?;

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   M O S A I C".bold().cyan());
    println!("   {}", "Semantic Sentence Structure".italic().dim());
    println!();
    println!("{}", output);

    Ok(())
}

/// Internal implementation of Mosaic logic
///
/// Separated for testing purposes (allows injecting a writer).
pub fn run_mosaic_inner<W: std::io::Write>(source: &str, writer: &mut W) -> Result<()> {
    let program = parse(source)?;

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

    // Literals
    if !asm.literals.is_empty() {
        other.push(format!("Literals: {}", asm.literals.len()));
    }

    // Operators
    if !asm.operators.is_empty() {
        other.push(format!("Ops: {:?}", asm.operators));
    }

    // Genitives
    if !asm.genitives.is_empty() {
        let gens = asm
            .genitives
            .iter()
            .map(fmt_constituent)
            .collect::<Vec<_>>()
            .join(", ");
        other.push(format!("Gen: [{}]", gens));
    }

    // Adjectives
    if !asm.adjectives.is_empty() {
        let adjs = asm
            .adjectives
            .iter()
            .map(fmt_constituent)
            .collect::<Vec<_>>()
            .join(", ");
        other.push(format!("Adj: [{}]", adjs));
    }

    // Participles
    if !asm.participles.is_empty() {
        let parts = asm
            .participles
            .iter()
            .map(|p| p.original.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        other.push(format!("Participles: [{}]", parts));
    }

    // New Fields (Arrays, Index Accesses, Properties, Blocks, Phrases, Unwraps)
    if !asm.arrays.is_empty() {
        other.push(format!("Arrays: {}", asm.arrays.len()));
    }
    if !asm.index_accesses.is_empty() {
        other.push(format!("Index Accesses: {}", asm.index_accesses.len()));
    }
    if !asm.property_accesses.is_empty() {
        let props = asm
            .property_accesses
            .iter()
            .map(|(o, p)| format!("{}.{}", o, p))
            .collect::<Vec<_>>()
            .join(", ");
        other.push(format!("Properties: [{}]", props));
    }
    if !asm.blocks.is_empty() {
        other.push(format!("Blocks: {}", asm.blocks.len()));
    }
    if !asm.nested_phrases.is_empty() {
        other.push(format!("Phrases: {}", asm.nested_phrases.len()));
    }
    if !asm.unwraps.is_empty() {
        other.push(format!("Unwraps: {}", asm.unwraps.len()));
    }

    // String Method
    if let Some((method, delim)) = &asm.string_method {
        other.push(format!("Method: {}({})", method, delim));
    }

    // Flags
    let mut flags = Vec::new();
    if asm.is_query {
        flags.push("Query (?)");
    }
    if asm.is_propagate {
        flags.push("Propagate (;)");
    }
    if asm.has_mutable_marker {
        flags.push("Mut (μετά)");
    }
    if asm.has_containment_preposition {
        flags.push("In (ἐν)");
    }
    if asm.has_delimiter_preposition {
        flags.push("By (κατά)");
    }

    if !flags.is_empty() {
        other.push(format!("Flags: [{}]", flags.join(", ")));
    }

    table.add_row(vec![
        Cell::new(format!("{}", line)),
        Cell::new(full_subject),
        Cell::new(verb),
        Cell::new(object),
        Cell::new(indirect),
        Cell::new(other.join("\n")).fg(Color::DarkGrey),
    ]);
}

fn fmt_constituent(c: &Constituent) -> String {
    c.original.to_string()
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
    fn test_run_mosaic() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_run.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            f.write_all("ὁ ἄνθρωπος τὸν λόγον λέγει.".as_bytes())
                .unwrap();
        }

        let result = run_mosaic(&file_path);
        assert!(result.is_ok());
    }
}
