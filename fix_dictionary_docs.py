import re

with open('src/tools/dictionary.rs', 'r') as f:
    content = f.read()

if not content.startswith('//!'):
    content = '''//! The Dictionary (τὸ Λεξικόν) - Lexicon Query Tool
//!
//! This module implements the "Dictionary" tool, which allows users to query
//! the built-in lexicon for specific words to see how the compiler analyzes them.
//!
//! # Purpose
//!
//! It provides a way to verify morphology, stem resolution, and part-of-speech
//! identification without needing to compile a full program.

''' + content

# Fix lookup_word
old_run = "/// Lookup a word in the dictionary\npub fn lookup_word"
new_run = """/// Looks up a word in the built-in dictionary.
///
/// This function normalizes the input word, queries the internal lexicon for exact
/// matches, and performs morphological analysis (declension/conjugation) to find
/// potential stems and features. It prints the results to standard output.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::dictionary::lookup_word;
///
/// if let Err(e) = lookup_word("λέγε") {
///     eprintln!("Lookup failed: {}", e);
/// }
/// ```
///
/// # Errors
///
/// Returns a [`miette::Result`] if an error occurs during morphological analysis.
pub fn lookup_word"""
content = content.replace(old_run, new_run)

with open('src/tools/dictionary.rs', 'w') as f:
    f.write(content)
