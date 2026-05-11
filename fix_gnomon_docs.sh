cat src/tools/gnomon.rs | sed '/\/\/\/ Just as a gnomon casts a shadow to indicate time, this visitor casts a shadow/,/\/\/\/ Recursively visits a statement and updates loop depth metrics./c\
/// Recursively visits a statement and updates loop depth metrics.' > src/tools/gnomon.rs.new

cat << 'RUST' > patch.txt
/// Analyzes a ΓΛΩΣΣΑ source file and estimates its Big-O time complexity.
///
/// This function coordinates the parsing, semantic analysis, and AST traversal
/// using the `visit_statement` function. The result is presented to the user in a
/// stylized terminal table.
///
/// # Errors
///
/// Returns a `miette::Result` if:
/// - The specified file cannot be found.
/// - The source file contains syntax or semantic errors.
///
/// # Examples
///
/// ```rust,no_run
/// use glossa::tools::gnomon::run_gnomon;
/// use std::path::Path;
///
/// let input = Path::new("algorithm.γλ");
/// if let Err(e) = run_gnomon(&input) {
///     eprintln!("Failed to estimate complexity: {}", e);
/// }
/// ```
pub fn run_gnomon
RUST

cat src/tools/gnomon.rs.new | sed '/pub fn run_gnomon/r patch.txt' | sed '/pub fn run_gnomon/d' > src/tools/gnomon.rs.final
mv src/tools/gnomon.rs.final src/tools/gnomon.rs
