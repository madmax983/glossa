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
use crate::tools::mosaic::run_mosaic_inner;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
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

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    // 1. Parse & Analyze
    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    // 2. Generate Rust Code
    let rust_code = generate_rust_file(&program);

    // 3. Generate Mosaic Table
    let mut mosaic_buffer = Vec::new();
    if let Err(e) = run_mosaic_inner(&source, &mut mosaic_buffer) {
        status.error("Σφάλμα (Error)");
        return Err(e);
    }
    let mosaic_output = String::from_utf8(mosaic_buffer).expect("comfy-table outputs valid UTF-8");

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
    if let Err(e) = fs::write(&output_path, &md).into_diagnostic() {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(e);
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   W E A V E".bold().cyan());
    println!("   {}", "Rosetta Stone Document Generated".italic().dim());

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(vec![
        Cell::new("Property")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Value")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    table.add_row(vec![
        Cell::new("Input File (ΓΛΩΣΣΑ)").add_attribute(Attribute::Bold),
        Cell::new(input.display().to_string()),
    ]);
    table.add_row(vec![
        Cell::new("Output File (Markdown)").add_attribute(Attribute::Bold),
        Cell::new(output_path.display().to_string()).fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Status").add_attribute(Attribute::Bold),
        Cell::new("✓ Success").fg(Color::Green),
    ]);

    println!("{table}");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_run_weave_success() {
        use std::io::Read;

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

        let mut f = std::fs::File::open(&output_path).unwrap();
        let mut md = String::new();
        std::io::Read::take(&mut f, 1024 * 1024 + 1)
            .read_to_string(&mut md)
            .unwrap();

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

    #[test]
    fn test_run_weave_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("parse_error.γλ");
        std::fs::write(&input_path, b"invalid syntax").unwrap();

        let result = run_weave(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_weave_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("semantic_error.γλ");
        std::fs::write(&input_path, "ψ 10 γίγνεται.").unwrap();

        let result = run_weave(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_weave_file_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("file_error.γλ");
        std::fs::write(&input_path, "ξ 10 ἔστω.").unwrap();

        // Create a directory at the expected output path so that fs::write fails
        let output_path = input_path.with_extension("md");
        std::fs::create_dir(&output_path).unwrap();

        let result = run_weave(&input_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // A generic file/IO error assertion that works across platforms
        assert!(
            err_msg.contains("Failed to write")
                || err_msg.contains("directory")
                || err_msg.contains("denied")
                || err_msg.contains("Permission")
        );
    }
}
