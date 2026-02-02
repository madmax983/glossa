//! ΓΛΩΣΣΑ Compiler CLI
//!
//! A compiler for ΓΛΩΣΣΑ - where Ancient Greek morphology encodes programming semantics.

use clap::{Parser, Subcommand};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

use glossa::codegen::{generate_rust, generate_rust_file};
use glossa::errors::GlossaError;
use glossa::ir::lower_to_hir;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[derive(Parser)]
#[command(name = "glossa")]
#[command(about = "ΓΛΩΣΣΑ - Ancient Greek morphology as programming semantics")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run a .γλ file directly (default action)
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a .γλ file (default)
    Run {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Compile a .γλ file to Rust source
    Build {
        /// Input file (.γλ)
        input: PathBuf,

        /// Output file (.rs)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Check a .γλ file without running
    Check {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Start the interactive REPL
    Repl,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    execute_cli(cli)
}

fn execute_cli(cli: Cli) -> Result<()> {
    // If a file is provided without a subcommand, run it
    if let Some(file) = cli.file {
        return run_file(&file);
    }

    match cli.command {
        Some(Commands::Run { input }) => {
            run_file(&input)?;
        }

        Some(Commands::Build { input, output }) => {
            build_file(&input, output.as_deref())?;
        }

        Some(Commands::Check { input }) => {
            check_file(&input)?;
        }

        Some(Commands::Repl) | None => {
            run_repl(std::io::stdin().lock(), std::io::stdout())?;
        }
    }

    Ok(())
}

fn compile(source: &str) -> std::result::Result<String, GlossaError> {
    let ast = parse(source)?;
    let analyzed = analyze_program(&ast)?;
    let hir = lower_to_hir(&analyzed);
    Ok(generate_rust_file(&hir))
}

/// Get the cache directory for compiled programs
fn cache_dir() -> PathBuf {
    let base = dirs_next::cache_dir()
        .or_else(dirs_next::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    base.join(".glossa").join("cache")
}

/// Generate a cache key from source file path
fn cache_key(input: &Path) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let canonical = input.canonicalize().unwrap_or_else(|_| input.to_path_buf());
    let mut hasher = DefaultHasher::new();
    canonical.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Check if cached binary is still valid (source not modified since compile)
fn cache_valid(input: &Path, cached_exe: &Path) -> bool {
    let source_modified = fs::metadata(input)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let exe_modified = fs::metadata(cached_exe)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);

    exe_modified > source_modified
}

fn build_file(input: &Path, output: Option<&Path>) -> Result<()> {
    let source = fs::read_to_string(input).into_diagnostic()?;

    let rust_code = compile(&source).map_err(|e| miette::miette!("{}", e))?;

    let output_path = output
        .map(|p| p.to_owned())
        .unwrap_or_else(|| input.with_extension("rs"));

    fs::write(&output_path, &rust_code).into_diagnostic()?;

    println!("{}", format!("✓ Ἐγράφη: {}", output_path.display()).green());

    Ok(())
}

fn run_file(input: &Path) -> Result<()> {
    // Validate file exists
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    // Set up cache directory
    let cache = cache_dir();
    fs::create_dir_all(&cache).into_diagnostic()?;

    let key = cache_key(input);
    let cached_rs = cache.join(format!("{}.rs", key));
    let cached_exe = cache.join(format!(
        "{}{}",
        key,
        if cfg!(windows) { ".exe" } else { "" }
    ));

    // Check if we can use cached binary
    if cache_valid(input, &cached_exe) && cached_exe.exists() {
        // Run cached binary directly
        println!("{}", "🚀 Εκτέλεσις...".dim());
        let status = Command::new(&cached_exe).status().into_diagnostic()?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        return Ok(());
    }

    // Compile source
    println!("{}", "🔨 Μεταγλώττισις...".dim());
    let source = fs::read_to_string(input).into_diagnostic()?;
    let rust_code = compile(&source).map_err(|e| miette::miette!("{}", e))?;

    // Write Rust source to cache
    fs::write(&cached_rs, &rust_code).into_diagnostic()?;

    // Compile with rustc (hide output)
    let rustc_output = Command::new("rustc")
        .arg(&cached_rs)
        .arg("-o")
        .arg(&cached_exe)
        .arg("-O") // Optimize for speed
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .into_diagnostic()?;

    if !rustc_output.status.success() {
        // Show rustc errors only on failure
        let stderr = String::from_utf8_lossy(&rustc_output.stderr);
        return Err(miette::miette!("Σφάλμα μεταγλωττίσεως:\n{}", stderr));
    }

    // Run the compiled program
    println!("{}", "🚀 Εκτέλεσις...".dim());
    let status = Command::new(&cached_exe).status().into_diagnostic()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn check_file(input: &Path) -> Result<()> {
    let source = fs::read_to_string(input).into_diagnostic()?;

    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let _analyzed = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    println!("{}", format!("✓ {} - ὀρθόν", input.display()).green());

    Ok(())
}

fn run_repl<R: std::io::BufRead, W: std::io::Write>(
    mut input_reader: R,
    mut output: W,
) -> Result<()> {
    writeln!(
        output,
        "{}",
        format!("ΓΛΩΣΣΑ v{}", env!("CARGO_PKG_VERSION"))
            .cyan()
            .bold()
    )
    .into_diagnostic()?;
    writeln!(
        output,
        "{}",
        "Γράψον .βοήθεια διὰ βοήθειαν, .ἔξοδος διὰ ἔξοδον.".dark_grey()
    )
    .into_diagnostic()?;
    writeln!(output).into_diagnostic()?;

    let mut context = ReplContext::new();

    loop {
        write!(output, "{}", "γλ> ".green().bold()).into_diagnostic()?;
        output.flush().into_diagnostic()?;

        let mut input = String::new();
        if input_reader.read_line(&mut input).into_diagnostic()? == 0 {
            // EOF
            break;
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle special commands
        match input {
            ".ἔξοδος" | ".exit" | ".quit" => {
                writeln!(output, "{}", "Χαῖρε!".cyan()).into_diagnostic()?;
                break;
            }
            ".βοήθεια" | ".help" => {
                print_help(&mut output)?;
                continue;
            }
            ".καθαρός" | ".clear" => {
                context = ReplContext::new();
                writeln!(output, "{}", "Ἐκαθαρίσθη.".green()).into_diagnostic()?;
                continue;
            }
            _ => {}
        }

        match context.execute(input) {
            Ok(result) => {
                if !result.is_empty() {
                    writeln!(output, "{}", result).into_diagnostic()?;
                }
            }
            Err(e) => {
                // In a REPL, errors should go to the output stream (user interface),
                // but std::eprintln goes to stderr. For testing, we might want to capture this too.
                // For now, we'll write to the output stream to make it testable and unified.
                writeln!(output, "{}", format!("Σφάλμα: {}", e).red()).into_diagnostic()?;
            }
        }
    }

    Ok(())
}

fn print_help<W: std::io::Write>(output: &mut W) -> Result<()> {
    writeln!(output, "{}", "Ἐντολαί:".bold()).into_diagnostic()?;
    writeln!(output, "  .βοήθεια  - δεῖξαι τήνδε τὴν βοήθειαν").into_diagnostic()?;
    writeln!(output, "  .ἔξοδος   - ἐξελθεῖν").into_diagnostic()?;
    writeln!(output, "  .καθαρός  - καθαρίσαι τὸ περιβάλλον").into_diagnostic()?;
    writeln!(output).into_diagnostic()?;
    writeln!(output, "{}", "Παραδείγματα:".bold()).into_diagnostic()?;
    writeln!(output, "  «χαῖρε κόσμε» λέγε.").into_diagnostic()?;
    writeln!(output, "  ξ πέντε ἔστω.").into_diagnostic()?;
    writeln!(output, "  ξ λέγε.").into_diagnostic()?;
    Ok(())
}

struct ReplContext {
    bindings: Vec<String>,
}

impl ReplContext {
    fn new() -> Self {
        ReplContext {
            bindings: Vec::new(),
        }
    }

    fn execute(&mut self, input: &str) -> std::result::Result<String, GlossaError> {
        // Build full program with previous bindings
        let mut full_source = self.bindings.join("\n");
        if !full_source.is_empty() {
            full_source.push('\n');
        }
        full_source.push_str(input);

        // Try to compile
        let ast = parse(&full_source)?;
        let analyzed = analyze_program(&ast)?;
        let hir = lower_to_hir(&analyzed);

        // Check if input contains a binding
        if input.contains("ἔστω") || input.contains("εστω") {
            self.bindings.push(input.to_string());
        }

        // Generate and return the code (for now, just show the Rust)
        let rust_code = generate_rust(&hir);
        Ok(format!(
            "{} {}",
            "→".blue().bold(),
            rust_code.lines().take(5).collect::<Vec<_>>().join("\n")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_repl_execute() {
        let mut context = ReplContext::new();
        let input = "«δοκιμή» λέγε.";
        let result = context.execute(input);
        assert!(result.is_ok());
        let output = result.unwrap();
        // Check for the blue arrow and some content
        assert!(output.contains("→"));
        assert!(output.contains("println"));
    }

    #[test]
    fn test_repl_binding_and_use() {
        let mut context = ReplContext::new();
        let binding = "ξ πέντε ἔστω.";
        let _ = context.execute(binding);

        let usage = "ξ λέγε.";
        let result = context.execute(usage);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("let xi = 5"));
        assert!(output.contains("println"));
    }

    #[test]
    fn test_repl_error() {
        let mut context = ReplContext::new();
        let input = "λάθος"; // Syntax error
        let result = context.execute(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_cache_key_consistency() {
        let path1 = PathBuf::from("test.gl");
        let path2 = PathBuf::from("test.gl");
        let path3 = PathBuf::from("other.gl");

        // Same path should produce same key
        assert_eq!(cache_key(&path1), cache_key(&path2));

        // Different paths should produce different keys
        assert_ne!(cache_key(&path1), cache_key(&path3));
    }

    #[test]
    fn test_cache_validity() {
        use filetime::{FileTime, set_file_mtime};
        use std::fs::File;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("source.gl");
        let exe_path = temp_dir.path().join("source.exe");

        // Create files
        File::create(&source_path).unwrap();
        File::create(&exe_path).unwrap();

        // Set exe time > source time
        let t1 = FileTime::from_unix_time(1000, 0);
        let t2 = FileTime::from_unix_time(2000, 0);

        set_file_mtime(&source_path, t1).unwrap();
        set_file_mtime(&exe_path, t2).unwrap();

        assert!(cache_valid(&source_path, &exe_path));

        // Set source time > exe time
        set_file_mtime(&source_path, t2).unwrap();
        set_file_mtime(&exe_path, t1).unwrap();

        assert!(!cache_valid(&source_path, &exe_path));
    }

    #[test]
    fn test_check_file_valid() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("valid.gl");
        fs::write(&file_path, "«χαῖρε» λέγε.").unwrap();

        assert!(check_file(&file_path).is_ok());
    }

    #[test]
    fn test_check_file_invalid() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("invalid.gl");
        fs::write(&file_path, "λάθος").unwrap();

        assert!(check_file(&file_path).is_err());
    }

    #[test]
    fn test_build_file() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("test.gl");
        let output_path = temp_dir.path().join("test.rs");

        fs::write(&input_path, "«χαῖρε» λέγε.").unwrap();

        assert!(build_file(&input_path, Some(&output_path)).is_ok());
        assert!(output_path.exists());

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("println"));
    }

    #[test]
    fn test_run_file_not_found() {
        let path = PathBuf::from("non_existent.gl");
        let result = run_file(&path);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Ἀρχεῖον οὐχ εὑρέθη")
        );
    }

    #[test]
    fn test_build_file_error() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("error.gl");

        fs::write(&input_path, "λάθος").unwrap();

        let result = build_file(&input_path, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_cli_check() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("check.gl");
        fs::write(&input_path, "«χαῖρε» λέγε.").unwrap();

        let cli = Cli {
            command: Some(Commands::Check { input: input_path }),
            file: None,
        };

        assert!(execute_cli(cli).is_ok());
    }

    #[test]
    fn test_execute_cli_build() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("build.gl");
        fs::write(&input_path, "«χαῖρε» λέγε.").unwrap();

        let cli = Cli {
            command: Some(Commands::Build {
                input: input_path,
                output: None,
            }),
            file: None,
        };

        assert!(execute_cli(cli).is_ok());
    }

    #[test]
    fn test_execute_cli_run() {
        // Since run involves compiling and executing a binary,
        // we test the error case (file not found) to be safe and fast
        let input_path = PathBuf::from("does_not_exist.gl");
        let cli = Cli {
            command: Some(Commands::Run { input: input_path }),
            file: None,
        };

        assert!(execute_cli(cli).is_err());
    }

    #[test]
    fn test_execute_cli_default_file() {
        // Test providing a file without a subcommand (like `glossa file.gl`)
        // Again, testing error case for safety/speed
        let input_path = PathBuf::from("does_not_exist.gl");
        let cli = Cli {
            command: None,
            file: Some(input_path),
        };

        assert!(execute_cli(cli).is_err());
    }

    #[test]
    fn test_run_repl_io() {
        let input_data = b".help\n.clear\n.exit\n";
        let mut output = Vec::new();

        let result = run_repl(&input_data[..], &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("ΓΛΩΣΣΑ v"));
        assert!(output_str.contains("Παραδείγματα:")); // from .help
        assert!(output_str.contains("Ἐκαθαρίσθη.")); // from .clear
        assert!(output_str.contains("Χαῖρε!")); // from .exit
    }

    #[test]
    fn test_run_repl_empty_line() {
        let input_data = b"\n.exit\n";
        let mut output = Vec::new();

        let result = run_repl(&input_data[..], &mut output);
        assert!(result.is_ok());
        // Should just prompt again and exit
    }

    #[test]
    fn test_run_repl_compile_error() {
        // Use byte string literal for UTF-8 input
        let input_data = "λάθος\n.exit\n".as_bytes();
        let mut output = Vec::new();

        let result = run_repl(&input_data[..], &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Σφάλμα"));
    }

    #[test]
    fn test_run_file_success() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let input_path = temp_dir.path().join("run_success.gl");
        fs::write(&input_path, "«χαῖρε» λέγε.").unwrap();

        // This actually compiles and runs the binary, so it requires rustc to be available
        // which it is in the environment.
        let result = run_file(&input_path);

        // If this fails (e.g. no rustc), we might need to skip or mock,
        // but for now let's assume it works as build_file worked.
        assert!(result.is_ok());
    }
}
