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
    let source = crate::tools::runner::load_source(input_path)?;

    let status = Status::start_with_symbol("Ψηφιδωτόν (Mosaic)", "🧩");

    // Create a buffer for the table
    let mut buffer = Vec::new();
    if let Err(e) = run_mosaic_inner(&source, &mut buffer) {
        status.error("Σφάλμα (Error)");
        return Err(e);
    }
    let output = String::from_utf8(buffer).expect("comfy-table outputs valid UTF-8");

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
        match stmt {
            crate::ast::Statement::Regular { .. } => match assemble_statement(stmt) {
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
            },
            crate::ast::Statement::TypeDefinition(_) => {
                let type_name = "🏛️ Ὁρισμός Εἴδους (Type Definition)";
                table.add_row(vec![
                    Cell::new(format!("{}", i + 1)),
                    Cell::new(type_name)
                        .fg(Color::Cyan)
                        .add_attribute(Attribute::Bold)
                        .add_attribute(Attribute::Italic),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                ]);
            }
            crate::ast::Statement::TraitDefinition(_) => {
                let type_name = "📜 Ὁρισμός Χαρακτῆρος (Trait Definition)";
                table.add_row(vec![
                    Cell::new(format!("{}", i + 1)),
                    Cell::new(type_name)
                        .fg(Color::Cyan)
                        .add_attribute(Attribute::Bold)
                        .add_attribute(Attribute::Italic),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                ]);
            }
            crate::ast::Statement::TraitImpl(_) => {
                let type_name = "⚙️ Ἐφαρμογὴ Χαρακτῆρος (Trait Implementation)";
                table.add_row(vec![
                    Cell::new(format!("{}", i + 1)),
                    Cell::new(type_name)
                        .fg(Color::Cyan)
                        .add_attribute(Attribute::Bold)
                        .add_attribute(Attribute::Italic),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                ]);
            }
            crate::ast::Statement::TestDeclaration(_) => {
                let type_name = "🧪 Δοκιμασία (Test Declaration)";
                table.add_row(vec![
                    Cell::new(format!("{}", i + 1)),
                    Cell::new(type_name)
                        .fg(Color::Cyan)
                        .add_attribute(Attribute::Bold)
                        .add_attribute(Attribute::Italic),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                    Cell::new(""),
                ]);
            }
        }
    }

    writeln!(writer, "{}", table).into_diagnostic()?;
    Ok(())
}

fn format_subject(asm: &AssembledStatement) -> String {
    let subject = asm
        .subject
        .as_ref()
        .map(fmt_constituent)
        .unwrap_or_default();

    // Combine nominatives if subject is present or if there are multiple
    // ⚡ Bolt Optimization: Avoid intermediate `Vec` allocations by constructing
    // the joined string directly with `String::with_capacity` and a loop.
    let mut extra_noms = String::with_capacity(asm.nominatives.len() * 10);
    for (i, nom) in asm.nominatives.iter().enumerate() {
        if i > 0 {
            extra_noms.push_str(", ");
        }
        extra_noms.push_str(&fmt_constituent(nom));
    }

    if !extra_noms.is_empty() {
        if !subject.is_empty() {
            format!("{} (+ {})", subject, extra_noms)
        } else {
            extra_noms
        }
    } else {
        subject
    }
}

fn format_collections(asm: &AssembledStatement, other: &mut Vec<String>) {
    // Genitives
    if !asm.genitives.is_empty() {
        // ⚡ Bolt Optimization: Avoid intermediate `Vec` allocations by constructing
        // the joined string directly with `String::with_capacity` and a loop.
        let mut gens = String::with_capacity(asm.genitives.len() * 10);
        for (i, g) in asm.genitives.iter().enumerate() {
            if i > 0 {
                gens.push_str(", ");
            }
            gens.push_str(&fmt_constituent(g));
        }
        other.push(format!("Gen: [{}]", gens));
    }

    // Adjectives
    if !asm.adjectives.is_empty() {
        // ⚡ Bolt Optimization: Avoid intermediate `Vec` allocations by constructing
        // the joined string directly with `String::with_capacity` and a loop.
        let mut adjs = String::with_capacity(asm.adjectives.len() * 10);
        for (i, adj) in asm.adjectives.iter().enumerate() {
            if i > 0 {
                adjs.push_str(", ");
            }
            adjs.push_str(&fmt_constituent(adj));
        }
        other.push(format!("Adj: [{}]", adjs));
    }

    // Participles
    if !asm.participles.is_empty() {
        // ⚡ Bolt Optimization: Avoid intermediate `Vec` allocations by constructing
        // the joined string directly with `String::with_capacity` and a loop.
        let mut parts = String::with_capacity(asm.participles.len() * 10);
        for (i, part) in asm.participles.iter().enumerate() {
            if i > 0 {
                parts.push_str(", ");
            }
            parts.push_str(part.original.as_str());
        }
        other.push(format!("Participles: [{}]", parts));
    }
}

fn format_structural_elements(asm: &AssembledStatement, other: &mut Vec<String>) {
    // New Fields (Arrays, Index Accesses, Properties, Blocks, Phrases, Unwraps)
    if !asm.arrays.is_empty() {
        other.push(format!("Arrays: {}", asm.arrays.len()));
    }
    if !asm.index_accesses.is_empty() {
        other.push(format!("Index Accesses: {}", asm.index_accesses.len()));
    }
    if !asm.property_accesses.is_empty() {
        // ⚡ Bolt Optimization: Avoid intermediate `Vec` allocations by constructing
        // the joined string directly with `String::with_capacity` and a loop.
        // Also uses `write!` to avoid secondary `String` allocations from `format!`.
        use std::fmt::Write;
        let mut props = String::with_capacity(asm.property_accesses.len() * 15);
        for (i, (o, p)) in asm.property_accesses.iter().enumerate() {
            if i > 0 {
                props.push_str(", ");
            }
            let _ = write!(props, "{}.{}", o, p);
        }
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
}

fn format_flags(asm: &AssembledStatement, other: &mut Vec<String>) {
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
}

fn format_other_column(asm: &AssembledStatement) -> String {
    let mut other = Vec::new();

    // Literals
    if !asm.literals.is_empty() {
        other.push(format!("Literals: {}", asm.literals.len()));
    }

    // Operators (for future proofing, assembler doesn't currently emit raw operators to mosaic output,
    // they are resolved during statement conversion, but the field exists on the struct)
    if !asm.operators.is_empty() {
        other.push(format!("Ops: {:?}", asm.operators));
    }

    format_collections(asm, &mut other);
    format_structural_elements(asm, &mut other);
    format_flags(asm, &mut other);

    other.join("\n")
}

fn add_row(table: &mut Table, line: usize, asm: &AssembledStatement) {
    let full_subject = format_subject(asm);

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

    let other_column = format_other_column(asm);

    table.add_row(vec![
        Cell::new(format!("{}", line)),
        Cell::new(full_subject),
        Cell::new(verb),
        Cell::new(object),
        Cell::new(indirect),
        Cell::new(other_column).fg(Color::DarkGrey),
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
    fn test_mosaic_multiple_elements_and_other_decls() {
        // This test ensures `i > 0` paths (comma joining) in add_row are covered
        let source = "
            ὁ μέγας καὶ καλὸς ἄνθρωπος τὸν λόγον λέγει.
            τοῦ πατρὸς τοῦ θεοῦ ὁ λόγος.
            ὁ ἄνθρωπος ὁ ἰδὼν καὶ ἀκούσας λέγει.
            γ μῆκος μέγεθος λέγε.

            // Other decls
            χαρακτήρ Τ ὁρίζειν { }.
            εἶδος Χ τῷ Τ ἐμπίπτειν { }.
            δοκιμή «τ» .
                1 1 ἰσοῦται.
            τέλος.
        ";
        let mut buffer = Vec::new();
        run_mosaic_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Assert we hit the commas for multiple adjectives, genitives, and participles
        assert!(output.contains("μέγας"));
        assert!(output.contains("καλὸς"));
        assert!(output.contains("πατρὸς"));
        assert!(output.contains("θεοῦ"));
        assert!(output.contains("ἰδὼν"));
        assert!(output.contains("ἀκούσας"));

        assert!(output.contains("Trait Definition"));
        assert!(output.contains("Trait Implementation"));
        assert!(output.contains("Test Declaration"));
    }

    #[test]
    fn test_mosaic_error_and_missing_subject() {
        use crate::morphology::{Case, Gender, Number};
        use crate::semantic::{AssembledStatement, Constituent};

        // Let's use `add_row` directly with multiple items to cover loops mapping commas
        let mut asm = AssembledStatement::default();

        let add_cons = |c: &mut Vec<Constituent>| {
            c.push(Constituent {
                lemma: "extra1".into(),
                original: "extra1".into(),
                normalized: "extra1".into(),
                case: Case::Nominative,
                number: Some(Number::Singular),
                gender: Some(Gender::Masculine),
                person: None,
            });
            c.push(Constituent {
                lemma: "extra2".into(),
                original: "extra2".into(),
                normalized: "extra2".into(),
                case: Case::Nominative,
                number: Some(Number::Singular),
                gender: Some(Gender::Masculine),
                person: None,
            });
        };
        add_cons(&mut asm.nominatives);
        add_cons(&mut asm.genitives);
        add_cons(&mut asm.adjectives);

        asm.participles
            .push(crate::semantic::assembly::ParticipleConstituent {
                verb_lemma: "part1".into(),
                original: "part1".into(),
                normalized: "part1".into(),
                tense: crate::morphology::Tense::Present,
                voice: crate::morphology::Voice::Active,
                case: Case::Nominative,
                gender: Gender::Masculine,
                number: Number::Singular,
            });
        asm.participles
            .push(crate::semantic::assembly::ParticipleConstituent {
                verb_lemma: "part2".into(),
                original: "part2".into(),
                normalized: "part2".into(),
                tense: crate::morphology::Tense::Present,
                voice: crate::morphology::Voice::Active,
                case: Case::Nominative,
                gender: Gender::Masculine,
                number: Number::Singular,
            });

        asm.property_accesses
            .push(("owner1".into(), "prop1".into()));
        asm.property_accesses
            .push(("owner2".into(), "prop2".into()));

        // Add an operator to cover line 289
        asm.operators
            .push(crate::morphology::lexicon::BinaryOp::Add);

        let mut table = Table::new();
        add_row(&mut table, 1, &asm);
        let table_str = table.to_string();
        assert!(table_str.contains("extra1, extra2"));
        assert!(table_str.contains("Ops: "));

        // To directly hit `Err(e)` in `assemble_statement`, we need a structure that parses
        // cleanly but fails the strict semantic Assembler logic.
        // Multiple verbs trigger `DoubleVerb` assembly error.
        let source_err = "ἡμῖν τὸ διδάσκειν ἡμῖν χρῄζει.";
        let mut buffer_err = Vec::new();
        let _ = run_mosaic_inner(source_err, &mut buffer_err);
        let output_err = String::from_utf8(buffer_err).unwrap();
        assert!(output_err.contains("Error: "));
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

    #[test]
    fn test_run_mosaic_error() {
        use std::io::Write;
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test_error.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            // Invalid syntax to trigger early return
            f.write_all(b"not valid syntax").unwrap();
        }

        let result = run_mosaic(&file_path);
        assert!(result.is_err());
    }
}
