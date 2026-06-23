import re

with open('src/tools/highlight.rs', 'r') as f:
    content = f.read()

# Fix highlight
old_run = """/// apply ANSI color codes based on the semantic role of each element.
///
/// # Errors
///
/// Returns a [`GlossaError`] if the source code cannot be parsed.
pub fn highlight"""
new_run = """/// apply ANSI color codes based on the semantic role of each element.
///
/// # Examples
///
/// ```rust
/// use glossa::tools::highlight::highlight;
///
/// let source = "«χαῖρε κόσμε» λέγε.";
/// let colored = highlight(source).unwrap();
/// assert!(colored.contains("\\x1b[")); // Contains ANSI escape codes
/// ```
///
/// # Errors
///
/// Returns a [`GlossaError`] if the source code cannot be parsed.
pub fn highlight"""
content = content.replace(old_run, new_run)

with open('src/tools/highlight.rs', 'w') as f:
    f.write(content)
