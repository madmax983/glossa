//! The Panoptes (ὁ Πανόπτης) - Memory Visualizer
//!
//! This tool runs a script through the simulator and displays the final
//! state of all variables in a terminal grid.

use crate::tools::interpreter::Interpreter;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use std::path::Path;

pub fn run_panoptes(input: &Path) -> miette::Result<()> {
    let source = crate::tools::runner::load_source(input)?;
    let status = crate::tools::ui::Status::start_with_symbol("Πανόπτης (Visualizing Memory)", "👁️");
    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(&program) {
        status.error("Σφάλμα Εκτέλεσης (Runtime Error)");
        eprintln!("{}", e);
        return Err(miette::miette!("{}", e));
    }
    status.success();

    println!("\n   {}", "Γ Λ Ω Σ Σ Α   P A N O P T E S".bold().cyan());
    println!("   {}", "Final Interpreter Memory State".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(vec![
        Cell::new("Scope")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Variable (μεταβλητή)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Value (τιμή)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
    ]);

    for (i, scope) in interp.get_environment().iter().enumerate() {
        for (name, val) in scope {
            table.add_row(vec![
                Cell::new(i.to_string()).fg(Color::DarkGrey),
                Cell::new(name).fg(Color::Yellow),
                Cell::new(val.to_string()).fg(Color::Green),
            ]);
        }
    }
    println!("{table}");
    Ok(())
}
