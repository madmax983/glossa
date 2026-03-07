//! The Weave Tool ("Weave")
//!
//! This module implements the "Weave" functionality, which acts as an exporter
//! that generates a 'Rosetta Stone' Markdown document combining Glossa source code,
//! semantic assembly logic, and generated Rust code.
//!
//! # Purpose
//!
//! "Weave" enables users to export their codebase into a structured, readable Markdown format,
//! making it easy to see how Greek syntax maps to semantic meaning and compiled Rust code.
//! It is especially useful for documentation and education.

use crate::codegen::generate_rust_file;
use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::tools::mosaic::run_mosaic_inner;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::Path;

/// Run the Weave tool on a file
///
/// Reads the source file, compiles it, generates the mosaic, and writes out a Markdown file.
pub fn run_weave(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ὕφανσις (Weaving)", "🕸️");

    let source = load_source(input)?;

    // 1. Parse & Analyze
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    // 2. Generate Rust Code
    let rust_code = generate_rust_file(&program);

    // 3. Generate Mosaic Table
    let mut mosaic_buffer = Vec::new();
    run_mosaic_inner(&source, &mut mosaic_buffer)?;
    let mosaic_output = String::from_utf8(mosaic_buffer).into_diagnostic()?;

    // 4. Format Markdown
    let mut md = String::new();

    let filename = input.file_name().unwrap_or_default().to_string_lossy();

    md.push_str(&format!("# Rosetta Stone: `{}`\n\n", filename));

    md.push_str("## 📜 ΓΛΩΣΣΑ Source\n\n");
    md.push_str("```glossa\n");
    md.push_str(&source);
    if !source.ends_with('\n') {
        md.push('\n');
    }
    md.push_str("```\n\n");

    md.push_str("## 🧩 Semantic Assembly (Mosaic)\n\n");
    // Indent the mosaic output or just put it in a block to preserve formatting
    md.push_str(&mosaic_output);
    md.push('\n');

    md.push_str("## 🦀 Generated Rust Code\n\n");
    md.push_str("```rust\n");
    md.push_str(&rust_code);
    if !rust_code.ends_with('\n') {
        md.push('\n');
    }
    md.push_str("```\n");

    // 5. Write to file or print
    let output_path = input.with_extension("md");
    fs::write(&output_path, &md).into_diagnostic()?;

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   W E A V E".bold().cyan());
    println!("   {}", "Rosetta Stone Document Generated".italic().dim());
    println!();
    println!(
        "   {} {}",
        "Saved to:".bold(),
        output_path.display().to_string().cyan()
    );
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_run_weave_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("weave_test.γλ");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«χαῖρε κόσμε» λέγε.\n".as_bytes()).unwrap();
        }

        let result = run_weave(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("md");
        assert!(output_path.exists());

        let md = fs::read_to_string(&output_path).unwrap();

        // Assertions for expected content
        assert!(md.contains("# Rosetta Stone"));
        assert!(md.contains("```glossa"));
        assert!(md.contains("«χαῖρε κόσμε» λέγε."));
        assert!(md.contains("## 🧩 Semantic Assembly (Mosaic)"));
        assert!(md.contains("## 🦀 Generated Rust Code"));
        assert!(md.contains("```rust"));
        assert!(md.contains("println"));
    }

    #[test]
    fn test_run_weave_file_not_found() {
        let path = Path::new("non_existent_file.γλ");
        let result = run_weave(path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_run_weave_file_too_large() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("too_large.γλ");

        // Create a file larger than MAX_FILE_SIZE (1MB)
        let max_size = 1024 * 1024;
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            let data = vec![0u8; max_size + 1];
            f.write_all(&data).unwrap();
        }

        let result = run_weave(&input_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Ἀρχεῖον λίαν μέγα")
        );
    }
}
