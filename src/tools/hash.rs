//! The Cryptographer (ὁ Κρυπτογράφος) - Semantic Hashing
//!
//! This module implements the "Cryptographer" tool, which generates a semantic hash (SHA-256)
//! of a ΓΛΩΣΣΑ program.
//!
//! # Purpose
//!
//! Because ΓΛΩΣΣΑ features a free word order, two programs might have completely different
//! source text but evaluate to the exact same semantic meaning. The Cryptographer proves this
//! by hashing the normalized, assembled `AnalyzedProgram` rather than the raw text.

use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use sha2::{Digest, Sha256};
use std::path::Path;

/// Run the Cryptographer tool
pub fn run_hash(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let status = Status::start_with_symbol("Κρυπτογράφος (Hashing)", "🔐");

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    status.success();

    // Generate a hash based on the debug representation of the statements
    // This ignores source code formatting and word order!
    let semantic_str = format!("{:#?}", program.statements);
    let mut hasher = Sha256::new();
    hasher.update(semantic_str.as_bytes());
    let result = hasher.finalize();
    let hash_hex = hex::encode(result);

    // Also hash the raw source text for comparison
    let mut raw_hasher = Sha256::new();
    raw_hasher.update(source.as_bytes());
    let raw_result = raw_hasher.finalize();
    let raw_hash_hex = hex::encode(raw_result);

    println!();
    println!(
        "   {}",
        "Γ Λ Ω Σ Σ Α   C R Y P T O G R A P H E R".cyan().bold()
    );
    println!("   {}", "Semantic Fingerprint Analysis".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Metric")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("SHA-256 Fingerprint").add_attribute(Attribute::Bold),
    ]);

    table.add_row(vec![
        Cell::new("Raw Source Hash").fg(Color::DarkGrey),
        Cell::new(&raw_hash_hex).fg(Color::DarkGrey),
    ]);

    table.add_row(vec![
        Cell::new("Semantic Hash")
            .fg(Color::Green)
            .add_attribute(Attribute::Bold),
        Cell::new(&hash_hex)
            .fg(Color::Green)
            .add_attribute(Attribute::Bold),
    ]);

    println!("{table}");
    println!();

    Ok(())
}
