//! The Translator Tool ("Translate")
//!
//! This module implements a reverse dictionary lookup for ΓΛΩΣΣΑ.
//! Given an English term or a Rust equivalent (like "println!" or "to calculate"),
//! it searches the lexicon and provides the corresponding Greek lemmas.

use crate::morphology::lexicon::entries;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;

/// Run the Translator tool
pub fn run_translate(query: &str) -> Result<()> {
    let status = Status::start_with_symbol(
        format!("Μεταφραστής (Translating '{}')", query),
        "🔤",
    );

    let query_lower = query.to_lowercase();

    let mut matches = Vec::new();
    for (lemma, entry) in entries() {
        if entry.meaning.to_lowercase().contains(&query_lower)
            || entry
                .rust_equiv
                .map(|r| r.to_lowercase().contains(&query_lower))
                .unwrap_or(false)
        {
            matches.push((lemma, entry));
        }
    }

    if matches.is_empty() {
        status.error("Οὐχ εὑρέθη (Not found)");
        return Err(miette::miette!(
            "No Greek equivalent found for: '{}'.",
            query
        ));
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   T R A N S L A T O R".bold().cyan());
    println!("   {}", format!("Results for '{}'", query).italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    table.set_header(vec![
        Cell::new("Lemma (Λῆμμα)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Part of Speech").add_attribute(Attribute::Bold),
        Cell::new("Meaning").add_attribute(Attribute::Bold),
        Cell::new("Rust Equivalent").add_attribute(Attribute::Bold),
    ]);

    for (lemma, entry) in matches {
        let rust_equiv = entry.rust_equiv.unwrap_or("-");
        table.add_row(vec![
            Cell::new(lemma).fg(Color::Green),
            Cell::new(format!("{:?}", entry.pos)),
            Cell::new(entry.meaning),
            Cell::new(rust_equiv).fg(Color::Yellow),
        ]);
    }

    println!("{table}");
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_found_meaning() {
        let result = run_translate("print");
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_found_rust_equiv() {
        let result = run_translate("println!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_not_found() {
        let result = run_translate("supercalifragilisticexpialidocious");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No Greek equivalent found"));
    }
}
