//! The Translator (ὁ Ἑρμηνεύς) Tool
//!
//! This module implements a reverse-dictionary lookup. It allows users
//! to search for English keywords, semantics, or concepts and find the
//! corresponding Ancient Greek words in the Glossa language.

use crate::morphology::lexicon::entries;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;

pub fn run_translator(english: &str) -> Result<()> {
    let search_term = english.to_lowercase();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   T R A N S L A T O R".bold().cyan());
    println!("   {}", "Reverse Semantic Lookup".italic().dim());
    println!();
    println!("   Searching for: {}", english.yellow().bold());
    println!();

    let mut matches = Vec::new();

    for (_key, entry) in entries() {
        let meaning_lower = entry.meaning.to_lowercase();
        let rust_lower = entry.rust_equiv.unwrap_or("").to_lowercase();

        if meaning_lower.contains(&search_term) || rust_lower.contains(&search_term) {
            matches.push(entry);
        }
    }

    if matches.is_empty() {
        println!("   {}", "× No matching words found.".dark_grey());
        return Ok(());
    }

    // Sort matches alphabetically by lemma
    matches.sort_by(|a, b| a.lemma.cmp(b.lemma));
    // Deduplicate by lemma in case multiple lexicon entries point to the same lemma
    matches.dedup_by(|a, b| a.lemma == b.lemma);

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL).set_header(vec![
        Cell::new("Glossa (Lemma)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Green),
        Cell::new("Part of Speech")
            .add_attribute(Attribute::Bold)
            .fg(Color::Magenta),
        Cell::new("English Meaning")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
    ]);

    for entry in matches {
        table.add_row(vec![
            Cell::new(entry.lemma).add_attribute(Attribute::Bold),
            Cell::new(format!("{:?}", entry.pos)),
            Cell::new(entry.meaning),
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
    fn test_translate_known_word() {
        // "say" should find something, run_translator just returns Ok(())
        let result = run_translator("say");
        assert!(result.is_ok());
    }

    #[test]
    fn test_translate_unknown_word() {
        let result = run_translator("unknown_garbage_xyz");
        assert!(result.is_ok());
    }
}
