//! The Archimedes (ὁ Ἀρχιμήδης) - Variable Scope Inspector
//!
//! This module implements the "Archimedes" tool, which visualizes the final
//! variable scope of a ΓΛΩΣΣΑ program after interpreting it.

use crate::tools::interpreter::{Interpreter, Value};
use crate::tools::runner::analyze_source;
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

pub fn run_archimedes(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἀρχιμήδης (Inspecting Scope)", "🔍");

    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(miette::miette!("{}", e));
        }
    };

    let program = match analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    let mut interpreter = Interpreter::new();
    if let Err(e) = interpreter.run(&program) {
        status.error("Σφάλμα ἐκτελέσεως (Execution Error)");
        return Err(miette::miette!("{}", e));
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A R C H I M E D E S".cyan().bold());
    println!(
        "   {}",
        format!("Final Scope for {}", input.display())
            .italic()
            .dim()
    );
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        Cell::new("Variable (Ὄνομα)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Value (Τιμή)").add_attribute(Attribute::Bold),
        Cell::new("Type (Εἶδος)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Magenta),
    ]);

    let mut scope = interpreter.env_globals().iter().collect::<Vec<_>>();
    scope.sort_by(|a, b| a.0.cmp(b.0));

    for (name, val) in scope {
        let (val_str, ty_str) = match val {
            Value::Number(n) => (n.to_string(), "Number (Ἀριθμός)"),
            Value::String(s) => (format!("«{}»", s), "String (Ὀνόματος)"),
            Value::Boolean(b) => (b.to_string(), "Boolean (Ἀληθές/Ψευδές)"),
            Value::Unit => ("()".to_string(), "Unit (Οὐδέν)"),
        };
        table.add_row(vec![
            Cell::new(name.as_str()),
            Cell::new(val_str),
            Cell::new(ty_str).fg(Color::Magenta),
        ]);
    }

    for line in table.to_string().lines() {
        println!("   {}", line);
    }
    println!();

    Ok(())
}
