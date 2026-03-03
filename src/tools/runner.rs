use crate::codegen::generate_rust_file;
use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, analyze_program};
use crate::tools::cache::Cache;
use crate::tools::highlight::highlight;
use crate::tools::narrator::tell_tale;
use crate::tools::report::{CompilationReport, GlossaReport, ProgramStats};
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

/// Maximum source file size (1MB) to prevent memory exhaustion
const MAX_FILE_SIZE: u64 = 1024 * 1024;

/// Parse and semantically analyze a source string
///
/// This helper runs the first two phases of the compiler pipeline:
/// 1. **Parsing**: Converts source text to AST
/// 2. **Semantic Analysis**: Resolves names, types, and statement structure
fn analyze_source(source: &str) -> Result<AnalyzedProgram> {
    let ast = parse(source).map_err(|e| miette::miette!("{}", e))?;
    analyze_program(&ast).map_err(|e| miette::miette!("{}", e))
}

/// Compile a source string directly to Rust code
///
/// Runs the full pipeline: Parse -> Analyze -> Codegen.
/// Returns the generated Rust source code as a string.
fn compile(source: &str) -> Result<String> {
    let analyzed = analyze_source(source)?;
    Ok(generate_rust_file(&analyzed))
}

/// Check file size to prevent DoS
fn check_file_size(input: &Path) -> Result<()> {
    let metadata = fs::metadata(input).into_diagnostic()?;
    if metadata.len() > MAX_FILE_SIZE {
        return Err(miette::miette!(
            "Ἀρχεῖον λίαν μέγα (File too large): {} > {} bytes",
            metadata.len(),
            MAX_FILE_SIZE
        ));
    }
    Ok(())
}

/// Load source code from a file with strict size limits
///
/// This function enforces a strict 1MB size limit to prevent Denial of Service (DoS)
/// attacks via memory exhaustion. It uses `take()` to limit the read operation,
/// ensuring we never read more than `MAX_FILE_SIZE` bytes even from infinite streams
/// (like `/dev/zero`).
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist.
/// - The file metadata indicates it is too large.
/// - The file content exceeds the 1MB limit.
fn load_source(input: &Path) -> Result<String> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }
    check_file_size(input)?;

    let file = fs::File::open(input).into_diagnostic()?;
    let mut content = String::new();

    // Use take to limit the read, preventing OOM on infinite streams (e.g. /dev/zero)
    file.take(MAX_FILE_SIZE + 1)
        .read_to_string(&mut content)
        .into_diagnostic()?;

    if content.len() as u64 > MAX_FILE_SIZE {
        return Err(miette::miette!(
            "Ἀρχεῖον λίαν μέγα (File too large): > {} bytes",
            MAX_FILE_SIZE
        ));
    }

    Ok(content)
}

/// Build a ΓΛΩΣΣΑ file to Rust source (without running it)
///
/// This function executes the compiler pipeline up to the code generation phase
/// and writes the resulting Rust code to a file (defaulting to `input.rs`).
///
/// # Steps
///
/// 1. **Load**: Reads the source file (with size limits).
/// 2. **Analyze**: Parses and performs semantic analysis.
/// 3. **Codegen**: Generates valid Rust code.
/// 4. **Write**: Saves the Rust code to the output path.
/// 5. **Report**: Prints a compilation report with statistics.
///
/// ## Errors
///
/// Returns an error if the input file does not exist, exceeds the size limit,
/// or contains syntax/semantic errors. Also returns an error if writing to
/// the output path fails.
///
/// ## Examples
///
/// ```no_run
/// use glossa::tools::runner::build_file;
/// use std::path::Path;
///
/// let input = Path::new("main.γλ");
/// let output = Path::new("main.rs");
/// // Compiles main.γλ to main.rs
/// build_file(input, Some(output)).unwrap();
/// ```
pub fn build_file(input: &Path, output: Option<&Path>) -> Result<()> {
    let status = Status::start_with_symbol("Μεταγλώττισις (Compiling)", "🏗️");
    let start = std::time::Instant::now();
    let source = load_source(input)?;
    let input_size = source.len() as u64;

    // Split compile to get stats
    let analyzed = analyze_source(&source)?;
    let rust_code = generate_rust_file(&analyzed);

    let output_path = output
        .map(|p| p.to_owned())
        .unwrap_or_else(|| input.with_extension("rs"));

    fs::write(&output_path, &rust_code).into_diagnostic()?;

    let output_size = fs::metadata(&output_path).into_diagnostic()?.len();
    let duration = start.elapsed();
    let stats = ProgramStats::new(&analyzed);

    status.success();

    let report = CompilationReport {
        input_path: input.to_path_buf(),
        output_path,
        input_size,
        output_size,
        duration,
        stats,
    };

    println!("{}", report);

    Ok(())
}

/// Compile and run a ΓΛΩΣΣΑ file
///
/// This is the "all-in-one" command that developers use most often.
/// It orchestrates the entire lifecycle of a program from Greek source to execution.
///
/// # The Pipeline
///
/// 1. **Validation**: Checks file existence and strict size limits to prevent DoS.
/// 2. **Caching**: Calculates a hash of the input. If a binary exists for this hash,
///    compilation is skipped entirely (The "Hot Path").
/// 3. **Compilation**: Runs the ΓΛΩΣΣΑ compiler to produce Rust source code.
/// 4. **Build**: Invokes `rustc` (the Rust compiler) to produce a native executable.
///    This inherits Rust's optimizations (set to `-O` level).
/// 5. **Execution**: Spawns the resulting binary as a child process, inheriting
///    stdin/stdout/stderr so it feels like a native script.
///
/// ## Errors
///
/// Returns an error if the input file does not exist, exceeds the size limit,
/// or contains syntax/semantic errors. Also returns an error if `rustc` fails to
/// compile the generated code.
///
/// ## Examples
///
/// ```no_run
/// use glossa::tools::runner::run_file;
/// use std::path::Path;
///
/// let input = Path::new("main.γλ");
/// // Compiles and immediately executes the file
/// run_file(input).unwrap();
/// ```
pub fn run_file(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }
    check_file_size(input)?;

    // Set up cache
    let cache = Cache::new();
    cache.init().into_diagnostic()?;

    let (cached_rs, cached_exe) = cache.get_paths(input);

    // Check if we can use cached binary
    if cache.is_valid(input, &cached_exe) && cached_exe.exists() {
        // Run cached binary directly
        let exit_status = Command::new(&cached_exe).status().into_diagnostic()?;

        if !exit_status.success() {
            std::process::exit(exit_status.code().unwrap_or(1));
        }
        return Ok(());
    }

    let mut status = Status::start_with_symbol("Μεταγλώττισις (Compiling)", "🚀");

    // Compile source
    let source = load_source(input)?;

    let rust_code = match compile(&source) {
        Ok(code) => code,
        Err(e) => {
            status.error("Σφάλμα μεταγλωττίσεως");
            return Err(e);
        }
    };

    // Write Rust source to cache
    fs::write(&cached_rs, &rust_code).into_diagnostic()?;

    status.update("Οἰκοδόμησις (Building)");

    // Compile with rustc (hide output)
    let rustc_output = Command::new("rustc")
        .arg(&cached_rs)
        .arg("-o")
        .arg(&cached_exe)
        .arg("-O") // Optimize for speed
        .arg("--color=always")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .into_diagnostic()?;

    if !rustc_output.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_output.stderr);
        status.error("Σφάλμα κώδικος (Codegen Error)");

        // Format the error nicely
        let error_msg = format!(
            "\n{}\n{}\n{}\n",
            "╔══════════════════════════════════════════════════════════════╗".red(),
            "║  INTERNAL COMPILER ERROR (Codegen Failed)                    ║"
                .red()
                .bold(),
            "╚══════════════════════════════════════════════════════════════╝".red()
        );

        let help_msg = format!(
            "{}\n{}",
            "This indicates a bug in the Glossa compiler's code generation.",
            "Please report this issue with the following details:"
        )
        .yellow();

        return Err(miette::miette!("{}\n{}\n\n{}", error_msg, help_msg, stderr));
    }

    status.success();

    // Run the compiled program
    let exit_status = Command::new(&cached_exe).status().into_diagnostic()?;

    if !exit_status.success() {
        std::process::exit(exit_status.code().unwrap_or(1));
    }

    Ok(())
}

/// Verifies the syntax and semantics of a ΓΛΩΣΣΑ file.
///
/// This function loads the source code, parses it, and performs semantic analysis
/// without generating any output code or binaries. It prints a [`GlossaReport`]
/// summarizing the program's statistics if successful.
///
/// ## Errors
///
/// Returns an error if the input file does not exist, exceeds the size limit,
/// or contains any syntax or semantic errors.
///
/// ## Examples
///
/// ```no_run
/// use glossa::tools::runner::check_file;
/// use std::path::Path;
///
/// let input = Path::new("main.γλ");
/// // Checks the file for errors without compiling it
/// check_file(input).unwrap();
/// ```
pub fn check_file(input: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Ἔλεγχος (Checking)", "🔍");
    let source = load_source(input)?;

    let analyzed = analyze_source(&source)?;

    let filename = input
        .file_name()
        .unwrap_or(input.as_os_str())
        .to_string_lossy()
        .to_string();
    let report = GlossaReport::new(&analyzed, filename);

    status.success();
    println!("{}", report);

    Ok(())
}

/// Semantically highlights a ΓΛΩΣΣΑ file and prints it to the terminal.
///
/// Uses the [`highlight`] function to parse
/// and colorize the source code based on grammatical roles (e.g., Subjects are blue,
/// Objects are red, Verbs are green).
///
/// ## Errors
///
/// Returns an error if the input file does not exist, exceeds the size limit,
/// or contains syntax errors that prevent highlighting.
///
/// ## Examples
///
/// ```no_run
/// use glossa::tools::runner::highlight_file;
/// use std::path::Path;
///
/// let input = Path::new("main.γλ");
/// // Prints the highlighted source code to stdout
/// highlight_file(input).unwrap();
/// ```
pub fn highlight_file(input: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Χρωματισμός (Highlighting)", "🎨");
    let source = load_source(input)?;
    let highlighted = highlight(&source).map_err(|e| miette::miette!("{}", e))?;

    status.success();
    println!("{}", highlighted);

    Ok(())
}

/// Narrates the logic of a ΓΛΩΣΣΑ file in plain English.
///
/// Uses the "Bard" tool to parse, analyze, and translate the semantic
/// meaning of the program into a readable English narrative ("The Scroll of Logic").
///
/// ## Errors
///
/// Returns an error if the input file does not exist, exceeds the size limit,
/// or contains syntax/semantic errors.
///
/// ## Examples
///
/// ```no_run
/// use glossa::tools::runner::bard_file;
/// use std::path::Path;
///
/// let input = Path::new("main.γλ");
/// // Prints the English narrative of the program's logic
/// bard_file(input).unwrap();
/// ```
pub fn bard_file(input: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Ἀφήγησις (Narrating)", "📜");
    let source = load_source(input)?;
    let analyzed = analyze_source(&source)?;

    let tale = tell_tale(&analyzed);
    status.success();
    println!("{}", tale);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_compile_hello() {
        let source = "«χαῖρε κόσμε» λέγε.";
        let result = compile(source);
        assert!(result.is_ok());
        let code = result.unwrap();
        // quote! generates `println !` with space
        assert!(code.contains("println"), "Expected println in: {}", code);
    }

    #[test]
    fn test_compile_binding() {
        let source = "ξ πέντε ἔστω.";
        let result = compile(source);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("let"));
    }

    #[test]
    fn test_compile_full_program() {
        let source = "ξ πέντε ἔστω. ξ λέγε.";
        let result = compile(source);
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_size_check_internal() {
        // Create large file
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("large_internal.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            let data = vec![0u8; (MAX_FILE_SIZE + 1) as usize];
            f.write_all(&data).unwrap();
        }

        // Call check_file_size directly
        let result = check_file_size(&file_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Ἀρχεῖον λίαν μέγα")
        );
    }

    #[test]
    fn test_build_file_size_limit() {
        // Create large file
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("large_build.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            let data = vec![0u8; (MAX_FILE_SIZE + 1) as usize];
            f.write_all(&data).unwrap();
        }

        // Call build_file
        let result = build_file(&file_path, None);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Ἀρχεῖον λίαν μέγα")
        );
    }

    #[test]
    fn test_build_file_success() {
        // Create a temporary input file
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("test.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«test» λέγε.".as_bytes()).unwrap();
        }

        // Call build_file
        let result = build_file(&input_path, None);
        assert!(result.is_ok());

        // Verify output file exists
        let output_path = input_path.with_extension("rs");
        assert!(output_path.exists());

        // Output size is > 0
        let metadata = std::fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_run_file_size_limit() {
        // Create large file
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("large_run.gl");
        {
            let mut f = std::fs::File::create(&file_path).unwrap();
            let data = vec![0u8; (MAX_FILE_SIZE + 1) as usize];
            f.write_all(&data).unwrap();
        }

        // Call run_file (should fail at size check before running rustc)
        let result = run_file(&file_path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Ἀρχεῖον λίαν μέγα")
        );
    }

    #[test]
    fn test_run_file_success() {
        // 1. Create a temporary Glossa file
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("run_test.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("«test» λέγε.".as_bytes()).unwrap();
        }

        // 2. Run it
        // This exercises: run_file -> check_file_size -> cache_dir -> cache_key -> compile -> rustc -> execution
        let result = run_file(&input_path);

        // Note: this test requires `rustc` to be in the path, which is true for `cargo test`.
        assert!(result.is_ok());

        // 3. Verify cache exists
        let cache = Cache::new();
        // We can't easily check internal dir existence without exposing it,
        // but we can check if the exe exists via get_paths
        let (_, cached_exe) = cache.get_paths(&input_path);
        assert!(cached_exe.exists());

        // 4. Run again to test cache hit path
        let result_cached = run_file(&input_path);
        assert!(result_cached.is_ok());
    }

    #[test]
    fn test_run_compile_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("error.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("invalid syntax".as_bytes()).unwrap();
        }

        let result = run_file(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Σφάλμα"));
    }

    #[test]
    fn test_run_rustc_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("rustc_error.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            // This is valid Glossa but invalid Rust (redefining String)
            // Memory says: εἶδος String ὁρίζειν...
            f.write_all("εἶδος String ὁρίζειν { }. τέλος.".as_bytes())
                .unwrap();
        }

        let result = run_file(&input_path);
        assert!(result.is_err());
        // Verify it hits the rustc error path
        assert!(result.unwrap_err().to_string().contains("Codegen Failed"));
    }

    #[test]
    fn test_check_file_valid() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("check.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ πέντε ἔστω.".as_bytes()).unwrap();
        }

        let result = check_file(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_highlight_file_valid() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("highlight.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ πέντε ἔστω.".as_bytes()).unwrap();
        }

        // We can't easily capture stdout here without a lot of plumbing,
        // but we can ensure it doesn't error.
        let result = highlight_file(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_bard_file_valid() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("bard.gl");
        {
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ πέντε ἔστω.".as_bytes()).unwrap();
        }

        let result = bard_file(&input_path);
        assert!(result.is_ok());
    }
}
