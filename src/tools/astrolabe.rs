//! The Astrolabe (ὁ Ἀστρολάβος) - Variable and Type Scope Visualizer
//!
//! This tool extracts all variables, functions, and types from the semantic
//! analyzer's Scope and displays them in an easy-to-read table.

use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

pub fn run_astrolabe(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἀστρολάβος (Mapping Scope)", "🔭");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A S T R O L A B E".bold().cyan());
    println!("   {}", "Scope and Variable Environment Map".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Name")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("GlossaType")
            .add_attribute(Attribute::Bold)
            .fg(Color::Magenta),
        Cell::new("Mutable")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
    ]);

    let mut found = false;
    for (name, binding) in program.scope.bindings() {
        found = true;
        let mut mut_cell = Cell::new(if binding.mutable { "Yes" } else { "No" });
        if binding.mutable {
            mut_cell = mut_cell.fg(Color::Red);
        } else {
            mut_cell = mut_cell.fg(Color::Green);
        }
        table.add_row(vec![
            Cell::new(name.as_str()),
            Cell::new(format!("{}", binding.glossa_type)),
            mut_cell,
        ]);
    }

    if found {
        println!("{table}");
    } else {
        println!("   {}", "No variables found in the global scope.".yellow());
    }
    println!();

    Ok(())
}
