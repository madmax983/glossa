//! The Weaver (ὁ Ὑφαντής)
//!
//! This module implements the `Weave` tool, an exporter that generates a
//! "Rosetta Stone" Markdown file for a given ΓΛΩΣΣΑ program.
//!
//! # Concept
//!
//! It weaves together:
//! 1. The original source code
//! 2. The morphological analysis & semantic assembly
//! 3. The generated Rust code

use crate::codegen::generate_rust_file;
use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Runs the Weaver tool on a given input file path
pub fn run_weave(input_path: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Ὑφαίνων (Weaving)", "🧵");

    // 1. Load source
    let source = load_source(input_path)?;

    // 2. Parse & Analyze
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    // 3. Generate Rust
    let rust_code = generate_rust_file(&program);

    // 4. Create the Markdown Content
    let mut md_content = String::new();
    let file_stem = input_path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    md_content.push_str(&format!("# ΓΛΩΣΣΑ Weave Report: `{}`\n\n", file_stem));

    // Section 1: Original Source
    md_content.push_str("## Original Source\n\n");
    md_content.push_str("```glossa\n");
    md_content.push_str(&source);
    if !source.ends_with('\n') {
        md_content.push('\n');
    }
    md_content.push_str("```\n\n");

    // Section 2: AST Assembly (like Mosaic)
    md_content.push_str("## Semantic Assembly\n\n");

    // Create the markdown table
    let mut table = String::new();
    table.push_str(
        "| Line | Subject (Nom) | Verb (Action) | Object (Acc) | Indirect (Dat) | Other |\n",
    );
    table.push_str("|---|---|---|---|---|---|\n");

    for (i, stmt) in ast.statements.iter().enumerate() {
        if let crate::ast::Statement::Regular { .. } = stmt {
            match crate::semantic::assemble_statement(stmt) {
                Ok(asm) => {
                    let subject = asm
                        .subject
                        .as_ref()
                        .map(|c| c.original.to_string())
                        .unwrap_or_default();
                    let extra_noms = asm
                        .nominatives
                        .iter()
                        .map(|c| c.original.to_string())
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
                    let object = asm
                        .object
                        .as_ref()
                        .map(|c| c.original.to_string())
                        .unwrap_or_default();
                    let indirect = asm
                        .indirect
                        .as_ref()
                        .map(|c| c.original.to_string())
                        .unwrap_or_default();

                    let mut other = Vec::new();
                    if !asm.literals.is_empty() {
                        other.push(format!("Literals: {}", asm.literals.len()));
                    }
                    if !asm.genitives.is_empty() {
                        let gens = asm
                            .genitives
                            .iter()
                            .map(|c| c.original.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        other.push(format!("Gen: [{}]", gens));
                    }
                    if !asm.adjectives.is_empty() {
                        let adjs = asm
                            .adjectives
                            .iter()
                            .map(|c| c.original.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        other.push(format!("Adj: [{}]", adjs));
                    }

                    let other_str = other.join("<br>");

                    table.push_str(&format!(
                        "| {} | {} | {} | {} | {} | {} |\n",
                        i + 1,
                        escape_md(&full_subject),
                        escape_md(&verb),
                        escape_md(&object),
                        escape_md(&indirect),
                        escape_md(&other_str)
                    ));
                }
                Err(e) => {
                    table.push_str(&format!(
                        "| {} | Error | | | | {} |\n",
                        i + 1,
                        escape_md(&e.to_string())
                    ));
                }
            }
        } else {
            let type_name = match stmt {
                crate::ast::Statement::TypeDefinition(_) => "Type Definition",
                crate::ast::Statement::TraitDefinition(_) => "Trait Definition",
                crate::ast::Statement::TraitImpl(_) => "Trait Implementation",
                crate::ast::Statement::TestDeclaration(_) => "Test Declaration",
                _ => "Unknown",
            };
            table.push_str(&format!("| {} | *{}* | | | | |\n", i + 1, type_name));
        }
    }

    md_content.push_str(&table);
    md_content.push_str("\n\n");

    // Section 3: Generated Rust
    md_content.push_str("## Generated Rust\n\n");
    md_content.push_str("```rust\n");
    md_content.push_str(&rust_code);
    if !rust_code.ends_with('\n') {
        md_content.push('\n');
    }
    md_content.push_str("```\n");

    // 5. Write to File
    let output_path = input_path.with_extension("md");
    let mut file = File::create(&output_path).into_diagnostic()?;
    file.write_all(md_content.as_bytes()).into_diagnostic()?;

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   W E A V E R".bold().cyan());
    println!("   {}", "Rosetta Stone Generated".italic().dim());
    println!("   Output: {}", output_path.display().to_string().yellow());
    println!();

    Ok(())
}

fn escape_md(s: &str) -> String {
    s.replace("|", "\\|")
}
