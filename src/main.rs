//! ΓΛΩΣΣΑ Compiler CLI
//!
//! A compiler for ΓΛΩΣΣΑ - where Ancient Greek morphology encodes programming semantics.

use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

mod cli_ui;
use cli_ui::GlossaUi;

use glossa::ast::build_ast;
use glossa::codegen::{generate_rust, generate_rust_file};
use glossa::errors::GlossaError;
use glossa::ir::lower_to_hir;
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
            run_repl()?;
        }
    }

    Ok(())
}

fn compile(source: &str) -> std::result::Result<String, GlossaError> {
    let ast = build_ast(source)?;
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

    let ui = GlossaUi::new();
    ui.success(&format!("Ἐγράφη: {}", output_path.display()));

    Ok(())
}

fn run_file(input: &Path) -> Result<()> {
    // Validate file exists
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let ui = GlossaUi::new();

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
        ui.info(&format!("Τρέχων (cached): {}", input.display()));
        // Run cached binary directly
        let status = Command::new(&cached_exe).status().into_diagnostic()?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
        return Ok(());
    }

    // Compile source
    let source = fs::read_to_string(input).into_diagnostic()?;
    let rust_code = compile(&source).map_err(|e| miette::miette!("{}", e))?;

    // Write Rust source to cache
    fs::write(&cached_rs, &rust_code).into_diagnostic()?;

    // Compile with rustc (hide output)
    ui.step("Compiling native binary", || {
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
        Ok(())
    })?;

    // Run the compiled program
    ui.info(&format!("Τρέχων: {}", input.display()));
    let status = Command::new(&cached_exe).status().into_diagnostic()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn check_file(input: &Path) -> Result<()> {
    let source = fs::read_to_string(input).into_diagnostic()?;

    let ast = build_ast(&source).map_err(|e| miette::miette!("{}", e))?;
    let _analyzed = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    let ui = GlossaUi::new();
    ui.success(&format!("{} - ὀρθόν", input.display()));

    Ok(())
}

fn run_repl() -> Result<()> {
    let ui = GlossaUi::new();
    println!("ΓΛΩΣΣΑ v{}", env!("CARGO_PKG_VERSION"));
    println!("Γράψον .βοήθεια διὰ βοήθειαν, .ἔξοδος διὰ ἔξοδον.");
    println!();

    let mut context = ReplContext::new();

    loop {
        ui.prompt();

        let mut input = String::new();
        let bytes = std::io::stdin().read_line(&mut input).into_diagnostic()?;

        // Handle EOF (Ctrl+D)
        if bytes == 0 {
            println!();
            break;
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle special commands
        match input {
            ".ἔξοδος" | ".exit" | ".quit" => {
                println!("Χαῖρε!");
                break;
            }
            ".βοήθεια" | ".help" => {
                print_help();
                continue;
            }
            ".καθαρός" | ".clear" => {
                context = ReplContext::new();
                println!("Ἐκαθαρίσθη.");
                continue;
            }
            _ => {}
        }

        match context.execute(input) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("Σφάλμα: {}", e);
            }
        }
    }

    Ok(())
}

fn print_help() {
    println!("Ἐντολαί:");
    println!("  .βοήθεια  - δεῖξαι τήνδε τὴν βοήθειαν");
    println!("  .ἔξοδος   - ἐξελθεῖν");
    println!("  .καθαρός  - καθαρίσαι τὸ περιβάλλον");
    println!();
    println!("Παραδείγματα:");
    println!("  «χαῖρε κόσμε» λέγε.");
    println!("  ξ πέντε ἔστω.");
    println!("  ξ λέγε.");
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
        let ast = build_ast(&full_source)?;
        let analyzed = analyze_program(&ast)?;
        let hir = lower_to_hir(&analyzed);

        // Check if input contains a binding
        if input.contains("ἔστω") || input.contains("εστω") {
            self.bindings.push(input.to_string());
        }

        // Generate and return the code (for now, just show the Rust)
        let rust_code = generate_rust(&hir);
        Ok(format!(
            "→ {}",
            rust_code
                .lines()
                .skip(1)
                .take(5)
                .collect::<Vec<_>>()
                .join("\n")
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
}
