//! The Catalog (ὁ Κατάλογος) - Lexicon Explorer
//!
//! A CLI tool to visually explore the built-in vocabulary of the compiler.
//! Displays all known words, categorized by their part of speech, along with
//! their internal semantics and rust equivalents.

use crate::morphology::models::PartOfSpeech;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;

/// Runs the Catalog tool to explore the compiler vocabulary.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::catalog::run_catalog;
///
/// if let Err(e) = run_catalog() {
///     eprintln!("Catalog failed: {}", e);
/// }
/// ```
pub fn run_catalog() -> Result<()> {
    // ⚡ Bolt Optimization: Uses `rustc_hash::FxHashMap` instead of the standard `HashMap`
    // since the keys are internal `PartOfSpeech` enums and are not exposed to HashDoS attacks.
    let mut entries_by_pos: rustc_hash::FxHashMap<
        PartOfSpeech,
        Vec<(&str, &crate::morphology::lexicon::LexiconEntry)>,
    > = rustc_hash::FxHashMap::default();

    for (word, entry) in crate::morphology::lexicon::entries() {
        entries_by_pos
            .entry(entry.pos)
            .or_default()
            .push((word, entry));
    }

    let mut pos_keys: Vec<_> = entries_by_pos.keys().copied().collect();
    // Sort keys based on debug formatting since they don't implement Ord natively
    pos_keys.sort_by_key(|k| format!("{:?}", k));

    println!("\n   {}", "Γ Λ Ω Σ Σ Α   C A T A L O G".cyan().bold());
    println!("   {}\n", "The Lexicon Explorer".dim().italic());

    for pos in pos_keys {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL).set_header(vec![
            Cell::new("Word").fg(Color::Cyan),
            Cell::new("Lemma").fg(Color::Cyan),
            Cell::new("Meaning").fg(Color::Cyan),
            Cell::new("Rust Equivalent").fg(Color::Cyan),
        ]);

        let mut pos_entries = entries_by_pos.get(&pos).unwrap().clone();
        pos_entries.sort_by_key(|(word, _)| *word);

        for (word, entry) in pos_entries {
            table.add_row(vec![
                Cell::new(word).fg(Color::Yellow),
                Cell::new(entry.lemma).fg(Color::White),
                Cell::new(entry.meaning).fg(Color::Green),
                Cell::new(entry.rust_equiv.unwrap_or("-")).fg(Color::Magenta),
            ]);
        }

        println!("\n  {}", format!("{:?} Lexicon", pos).yellow().bold());
        println!("{table}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_does_not_panic() {
        // Run the catalog and ensure it returns Ok(()) and doesn't panic on the built-in lexicon.
        assert!(run_catalog().is_ok());
    }
}
