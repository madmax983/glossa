//! The Lexicon (Dictionary) Tool
//!
//! This module implements the dictionary lookup and morphological analysis interface.
//! It serves as "The Lexicon" in the compiler's toolset, allowing users to query
//! word meanings, grammatical properties, and etymology.
//!
//! # The Dual Approach
//!
//! The Lexicon uses a two-tiered approach to analyze words:
//!
//! 1.  **Static Lexicon Lookup (Definitive)**:
//!     First, it checks a built-in static dictionary of common words. If found, this provides
//!     definitive information (lemma, part of speech, definition). This is fast and accurate
//!     for core vocabulary.
//!
//! 2.  **Dynamic Morphological Analysis (Probabilistic)**:
//!     If the word is not in the static lexicon (or even if it is), the tool performs
//!     a dynamic morphological analysis. It decomposes the word into stem and ending,
//!     applying grammatical rules to guess its properties (Case, Number, Gender, etc.).
//!     This allows the compiler to understand words it has never seen before!
//!
//! # CLI Usage
//!
//! This tool powers the `glossa lookup` command:
//!
//! ```bash
//! glossa lookup λόγον
//! ```
//!
//! Output:
//!
//! ```text
//!    Γ Λ Ω Σ Σ Α   L E X I C O N
//!    Analyzing: λόγον
//!
//!    📚 Lexicon Entry (Definitive)
//!    Property        Value
//!    Lemma           λόγος
//!    Part of Speech  Noun
//!    Meaning         word, reason, logic
//!    ...
//!
//!    🔬 Morphological Analysis (All Possibilities)
//!    Lemma  PoS   Grammar                        Confidence
//!    λόγος  Noun  Accusative, Singular, Masculine  1.00
//! ```

use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;

use crate::morphology::{analyze_all, lexicon};
use crate::text::normalize_greek;

/// Lookup a word in the dictionary
///
/// This is the main entry point for the `glossa lookup` tool. It normalizes the
/// input word, attempts to find it in the built-in lexicon, and then performs a
/// full morphological analysis, displaying all results in a formatted table.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::dictionary::lookup_word;
///
/// if let Err(e) = lookup_word("λόγον") {
///     eprintln!("Lookup failed: {}", e);
/// }
/// ```
pub fn lookup_word(word: &str) -> Result<()> {
    let normalized = normalize_greek(word);

    print_header(word, &normalized);

    // 1. Direct Lexicon Lookup
    if let Some(entry) = lexicon::lookup(&normalized) {
        print_lexicon_entry(entry);
    } else {
        println!("   {}", "✕ Not found in built-in lexicon.".yellow().bold());
        println!("   {}", "Showing morphological analysis only.".dim());
        println!();
    }

    // 2. Morphological Analysis
    let analyses = analyze_all(word);

    if analyses.is_empty() {
        println!("   {}", "× No morphological analysis found.".dark_grey());
        return Ok(());
    }

    print_morphological_analyses(&analyses);

    Ok(())
}

fn print_header(word: &str, normalized: &str) {
    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   L E X I C O N".bold().cyan());
    println!("   {}", "Semantic Dictionary Lookup".italic().dim());
    println!();
    println!("   Analyzing: {}", word.yellow().bold());
    if normalized != word {
        println!("   Normalized: {}", normalized.dim());
    }
    println!();
}

fn print_lexicon_entry(entry: &crate::morphology::lexicon::LexiconEntry) {
    println!(
        "   {}",
        "📚 Lexicon Entry (Definitive)".green().bold().underlined()
    );

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL).set_header(vec![
        Cell::new("Property")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Value").add_attribute(Attribute::Bold),
    ]);

    table.add_row(vec![
        Cell::new("Lemma"),
        Cell::new(entry.lemma)
            .fg(Color::Green)
            .add_attribute(Attribute::Bold),
    ]);

    table.add_row(vec![
        Cell::new("Part of Speech"),
        Cell::new(format!("{:?}", entry.pos)),
    ]);

    table.add_row(vec![
        Cell::new("Meaning"),
        Cell::new(entry.meaning).fg(Color::Yellow),
    ]);

    // Clarify rust_equiv semantics: None means user-defined or no direct mapping
    let rust_equiv_display = entry.rust_equiv.unwrap_or("(none/user-defined)");
    table.add_row(vec![
        Cell::new("Rust Equivalent"),
        Cell::new(rust_equiv_display)
            .fg(if entry.rust_equiv.is_some() {
                Color::Red
            } else {
                Color::DarkGrey
            })
            .add_attribute(Attribute::Bold),
    ]);

    // Add grammatical details if present
    if let Some(c) = entry.case {
        table.add_row(vec![Cell::new("Case"), Cell::new(format!("{}", c))]);
    }
    if let Some(n) = entry.number {
        table.add_row(vec![Cell::new("Number"), Cell::new(format!("{}", n))]);
    }
    if let Some(g) = entry.gender {
        table.add_row(vec![Cell::new("Gender"), Cell::new(format!("{}", g))]);
    }
    if let Some(p) = entry.person {
        table.add_row(vec![Cell::new("Person"), Cell::new(format!("{:?}", p))]);
    }
    if let Some(t) = entry.tense {
        table.add_row(vec![Cell::new("Tense"), Cell::new(format!("{:?}", t))]);
    }
    if let Some(m) = entry.mood {
        table.add_row(vec![Cell::new("Mood"), Cell::new(format!("{:?}", m))]);
    }
    if let Some(v) = entry.voice {
        table.add_row(vec![Cell::new("Voice"), Cell::new(format!("{:?}", v))]);
    }

    println!("{table}");
    println!();
}

fn print_morphological_analyses(analyses: &[crate::morphology::models::MorphAnalysis]) {
    println!(
        "   {}",
        "🔬 Morphological Analysis (All Possibilities)"
            .yellow()
            .bold()
            .underlined()
    );

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL).set_header(vec![
        Cell::new("Lemma")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
        Cell::new("PoS")
            .add_attribute(Attribute::Bold)
            .fg(Color::Magenta),
        Cell::new("Grammar")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Confidence")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
    ]);

    // ⚡ Bolt Optimization: Reuse a single String buffer and avoid intermediate format allocations
    // by writing directly into the buffer instead of collecting into a Vec<String> and joining.
    use std::fmt::Write;
    let mut grammar_buf = String::new();

    for analysis in analyses {
        grammar_buf.clear();
        let mut first = true;
        let mut push_comma = |buf: &mut String| {
            if !first {
                buf.push_str(", ");
            }
            first = false;
        };

        if let Some(c) = analysis.case {
            push_comma(&mut grammar_buf);
            let _ = write!(grammar_buf, "{}", c);
        }
        if let Some(n) = analysis.number {
            push_comma(&mut grammar_buf);
            let _ = write!(grammar_buf, "{}", n);
        }
        if let Some(g) = analysis.gender {
            push_comma(&mut grammar_buf);
            let _ = write!(grammar_buf, "{}", g);
        }
        if let Some(p) = analysis.person {
            push_comma(&mut grammar_buf);
            let _ = write!(grammar_buf, "{:?}", p);
        }
        if let Some(t) = analysis.tense {
            push_comma(&mut grammar_buf);
            let _ = write!(grammar_buf, "{:?}", t);
        }
        if let Some(m) = analysis.mood {
            push_comma(&mut grammar_buf);
            let _ = write!(grammar_buf, "{:?}", m);
        }
        if let Some(v) = analysis.voice {
            push_comma(&mut grammar_buf);
            let _ = write!(grammar_buf, "{:?}", v);
        }

        let conf_cell = if analysis.confidence >= 0.9 {
            Cell::new(format!("{:.2}", analysis.confidence)).fg(Color::Green)
        } else if analysis.confidence >= 0.5 {
            Cell::new(format!("{:.2}", analysis.confidence)).fg(Color::Yellow)
        } else {
            Cell::new(format!("{:.2}", analysis.confidence)).fg(Color::Red)
        };

        table.add_row(vec![
            Cell::new(analysis.lemma.as_ref()),
            Cell::new(format!("{:?}", analysis.part_of_speech)),
            Cell::new(&grammar_buf),
            conf_cell,
        ]);
    }

    println!("{table}");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_known_word() {
        // Just ensure it doesn't panic
        lookup_word("λόγον").unwrap();
    }

    #[test]
    fn test_lookup_unknown_word() {
        lookup_word("ἀγνωστον").unwrap();
    }

    #[test]
    fn test_lookup_verb() {
        lookup_word("λέγε").unwrap();
    }

    #[test]
    fn test_lookup_word_without_rust_equiv() {
        // "εγω" (I) has rust_equiv: None in the lexicon
        lookup_word("εγω").unwrap();
    }
}
