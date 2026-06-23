import re

with open('src/tools/haruspex.rs', 'r') as f:
    content = f.read()

# Fix run_haruspex
old_run = "/// Runs the Haruspex tool to generate a Graphviz DOT representation of the AST.\npub fn run_haruspex"
new_run = """/// Runs the Haruspex tool on a given Glossa source file.
///
/// This function reads the provided source file, parses and semantically analyzes it,
/// and then prints a Graphviz DOT format representation of the Abstract Syntax Tree (AST)
/// to the standard output.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::haruspex::run_haruspex;
/// use std::path::Path;
///
/// let input = Path::new("main.γλ");
/// if let Err(e) = run_haruspex(&input) {
///     eprintln!("Haruspex failed: {}", e);
/// }
/// ```
///
/// # Errors
///
/// Returns a [`miette::Result`] if the file cannot be read, or if there is a parsing
/// or semantic error during compilation.
pub fn run_haruspex"""
content = content.replace(old_run, new_run)

with open('src/tools/haruspex.rs', 'w') as f:
    f.write(content)
