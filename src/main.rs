//! ΓΛΩΣΣΑ Compiler CLI
//!
//! A compiler for ΓΛΩΣΣΑ - where Ancient Greek morphology encodes programming semantics.

use clap::{Parser, Subcommand};
use comfy_table::{Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::SystemTime;

use glossa::codegen::{generate_rust_file, generate_statement_code};
use glossa::errors::GlossaError;
use glossa::parser::parse;
use glossa::semantic::{AnalyzedStatement, GlossaType, Scope, analyze_program};

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

    /// Highlight a .γλ file with semantic colors
    Highlight {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Start the interactive REPL
    Repl,
}

/// Maximum source file size (1MB) to prevent memory exhaustion
const MAX_FILE_SIZE: u64 = 1024 * 1024;

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

        Some(Commands::Highlight { input }) => {
            highlight_file(&input)?;
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}

fn compile(source: &str) -> std::result::Result<String, GlossaError> {
    let ast = parse(source)?;
    let analyzed = analyze_program(&ast)?;
    Ok(generate_rust_file(&analyzed))
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

fn build_file(input: &Path, output: Option<&Path>) -> Result<()> {
    check_file_size(input)?;

    let source = fs::read_to_string(input).into_diagnostic()?;

    let rust_code = compile(&source).map_err(|e| miette::miette!("{}", e))?;

    let output_path = output
        .map(|p| p.to_owned())
        .unwrap_or_else(|| input.with_extension("rs"));

    fs::write(&output_path, &rust_code).into_diagnostic()?;

    println!(
        "{} Ἐγράφη: {}",
        "✓".green(),
        output_path.display().to_string().bold()
    );

    Ok(())
}

fn run_file(input: &Path) -> Result<()> {
    // Validate file exists
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    check_file_size(input)?;

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
    let status = Command::new(&cached_exe).status().into_diagnostic()?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}

fn check_file(input: &Path) -> Result<()> {
    check_file_size(input)?;

    let source = fs::read_to_string(input).into_diagnostic()?;

    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let _analyzed = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    println!(
        "{} {} - ὀρθόν",
        "✓".green(),
        input.display().to_string().bold()
    );

    Ok(())
}

fn highlight_file(input: &Path) -> Result<()> {
    check_file_size(input)?;

    let source = fs::read_to_string(input).into_diagnostic()?;
    let highlighted =
        glossa::highlight::highlight(&source).map_err(|e| miette::miette!("{}", e))?;

    println!("{}", highlighted);

    Ok(())
}

fn run_repl() -> Result<()> {
    print_banner();

    let mut context = ReplContext::new();

    loop {
        print!("{}", "γλ> ".green().bold());
        use std::io::Write;
        std::io::stdout().flush().into_diagnostic()?;

        let mut input = String::new();
        let bytes = std::io::stdin().read_line(&mut input).into_diagnostic()?;

        // Fix: Handle EOF (Ctrl+D) gracefully
        if bytes == 0 {
            println!("\nΧαῖρε!");
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
                println!("{} Ἐκαθαρίσθη.", "✓".green());
                continue;
            }
            ".περιβάλλον" | ".env" => {
                print_env(&context);
                continue;
            }
            _ => {}
        }

        match context.execute(input) {
            Ok(output) => {
                // Display handles formatting (empty if None)
                print!("{}", output);
            }
            Err(e) => {
                // Use default error formatting but ensure it's visible
                eprintln!("{}", format!("× Σφάλμα: {}", e).red());
            }
        }
    }

    Ok(())
}

fn print_banner() {
    // Welcome Banner
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α".bold().cyan());
    println!("   {}", "Code as the ancients intended.".italic().dim());
    println!("   {}", version.blue());
    println!();
    println!("   {}", "Type .help for commands".dim());
    println!();
}

fn print_help() {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL).set_header(vec![
        Cell::new("Ἐντολή (Command)")
            .fg(Color::Cyan)
            .add_attribute(comfy_table::Attribute::Bold),
        Cell::new("Περιγραφή (Description)")
            .fg(Color::Cyan)
            .add_attribute(comfy_table::Attribute::Bold),
    ]);

    table.add_row(vec![
        ".βοήθεια / .help",
        "Δεῖξαι τήνδε τὴν βοήθειαν (Show this help)",
    ]);
    table.add_row(vec![".ἔξοδος / .exit", "Ἐξελθεῖν (Exit REPL)"]);
    table.add_row(vec![
        ".καθαρός / .clear",
        "Καθαρίσαι τὸ περιβάλλον (Clear history)",
    ]);
    table.add_row(vec![
        ".περιβάλλον / .env",
        "Δεῖξαι τὰς μεταβλητάς (Show variables)",
    ]);

    println!("{table}");

    println!("\n{}", "Παραδείγματα:".bold().underlined());
    println!("  «χαῖρε κόσμε» λέγε.");
    println!("  ξ πέντε ἔστω.");
}

fn print_env(context: &ReplContext) {
    if let Some(scope) = &context.last_scope {
        // Collect and sort bindings for consistent display
        let mut bindings: Vec<_> = scope.bindings().collect();
        bindings.sort_by(|a, b| a.0.cmp(b.0));

        if bindings.is_empty() {
            println!("{}", "Οὐδεμία μεταβλητή (No variables defined).".yellow());
            return;
        }

        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL).set_header(vec![
            Cell::new("Μεταβλητή (Var)")
                .fg(Color::Magenta)
                .add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Τύπος (Type)").add_attribute(comfy_table::Attribute::Bold),
            Cell::new("Μεταβλητότης (Mut)").add_attribute(comfy_table::Attribute::Bold),
        ]);

        for (name, binding) in bindings {
            let mut_str = if binding.mutable {
                "Ναί (Yes)"
            } else {
                "Οὔ (No)"
            };

            let mut_cell = Cell::new(mut_str);
            let mut_cell = if binding.mutable {
                mut_cell.fg(Color::Yellow)
            } else {
                mut_cell.fg(Color::DarkGrey)
            };

            table.add_row(vec![
                Cell::new(name).fg(Color::Green),
                Cell::new(&binding.glossa_type),
                mut_cell,
            ]);
        }
        println!("{table}");
    } else {
        println!("{}", "Οὐδεμία μεταβλητή (No variables defined).".yellow());
    }
}

/// REPL Output variants
#[derive(Debug)]
pub enum ReplOutput {
    /// A new variable binding
    Binding {
        name: String,
        type_: GlossaType,
        mutable: bool,
    },
    /// Code execution (compilation)
    Statement { code: String },
    /// No output (e.g. empty line)
    None,
}

impl std::fmt::Display for ReplOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplOutput::Binding {
                name,
                type_,
                mutable,
            } => {
                write!(
                    f,
                    "{} {}: {}{}",
                    "✓".green(),
                    name.as_str().cyan().bold(),
                    type_,
                    if *mutable {
                        " (mutable)".yellow().to_string()
                    } else {
                        "".to_string()
                    }
                )?;
                writeln!(f)
            }
            ReplOutput::Statement { code } => {
                writeln!(f, "{}", "✓ Ἐκτελέσθη (Executed)".green())?;
                if !code.is_empty() {
                    writeln!(f, "  {}", code.as_str().dim())?;
                }
                Ok(())
            }
            ReplOutput::None => Ok(()),
        }
    }
}

/// Maximum number of bindings to track in REPL history
const MAX_REPL_BINDINGS: usize = 50;
/// Maximum total source size for REPL history
const MAX_REPL_SOURCE_LEN: usize = 50_000;

struct ReplContext {
    bindings: Vec<String>,
    last_scope: Option<Scope>,
    statement_count: usize,
}

impl ReplContext {
    fn new() -> Self {
        ReplContext {
            bindings: Vec::new(),
            last_scope: None,
            statement_count: 0,
        }
    }

    fn execute(&mut self, input: &str) -> std::result::Result<ReplOutput, GlossaError> {
        // Check binding count limit
        if self.bindings.len() > MAX_REPL_BINDINGS {
            return Err(GlossaError::semantic(
                "REPL binding limit exceeded (50). Please use .καθαρός (.clear)",
            ));
        }

        // Build full program with previous bindings
        let mut full_source = self.bindings.join("\n");
        if !full_source.is_empty() {
            full_source.push('\n');
        }
        full_source.push_str(input);

        // Check total size limit
        if full_source.len() > MAX_REPL_SOURCE_LEN {
            return Err(GlossaError::semantic(
                "REPL source size limit exceeded (50KB). Please use .καθαρός (.clear)",
            ));
        }

        // Try to compile
        let ast = parse(&full_source)?;
        let analyzed = analyze_program(&ast)?;

        let new_count = analyzed.statements.len();
        if new_count <= self.statement_count {
            return Ok(ReplOutput::None);
        }

        // Update scope and count
        self.last_scope = Some(analyzed.scope.clone());
        self.statement_count = new_count;

        // Analyze what happened in the last statement
        // We know it exists because new_count > old_count >= 0
        let last_stmt = analyzed.statements.last().unwrap();
        match last_stmt {
            AnalyzedStatement::Binding { name, mutable, .. } => {
                // Lookup type from scope since it's no longer in the binding statement
                let value_type = analyzed
                    .scope
                    .lookup(name)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown);

                // Add to bindings list so it persists
                self.bindings.push(input.to_string());

                Ok(ReplOutput::Binding {
                    name: name.to_string(),
                    type_: value_type,
                    mutable: *mutable,
                })
            }
            _ => {
                // For non-binding statements, we don't save them to bindings list
                // because we don't want to re-execute side effects (print) every time.
                // BUT: If the user defines a function or type, we SHOULD save it.
                if matches!(
                    last_stmt,
                    AnalyzedStatement::FunctionDef { .. }
                        | AnalyzedStatement::TypeDefinition { .. }
                        | AnalyzedStatement::TraitDefinition { .. }
                        | AnalyzedStatement::TraitImplementation { .. }
                ) {
                    self.bindings.push(input.to_string());
                }

                Ok(ReplOutput::Statement {
                    code: generate_statement_code(last_stmt),
                })
            }
        }
    }
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
    fn test_repl_binding_limit() {
        let mut context = ReplContext::new();

        // Add max bindings
        for i in 0..MAX_REPL_BINDINGS {
            context.execute(&format!("ξ{} πέντε ἔστω.", i)).unwrap();
        }

        // Add one more (this puts us over the limit of 50 -> 51)
        // The check is `> 50`, so 50 is allowed, 51 triggers error on NEXT call
        context.execute("υπερβολή πέντε ἔστω.").unwrap();

        // This one should be blocked
        let result = context.execute("τέλος πέντε ἔστω.");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("binding limit"));
    }

    #[test]
    fn test_repl_source_limit() {
        let mut context = ReplContext::new();

        // Create a huge input that exceeds the limit immediately
        // The check happens before parsing, so content validity doesn't matter much
        // as long as it triggers the length check.
        let huge_input = " ".repeat(MAX_REPL_SOURCE_LEN + 1);

        let result = context.execute(&huge_input);

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("source size limit")
        );
    }

    #[test]
    fn test_repl_env_coverage() {
        let mut context = ReplContext::new();

        // 1. Initial state: No scope
        assert!(context.last_scope.is_none());
        print_env(&context); // Should print "No variables"

        // 2. Execute binding
        let _ = context.execute("ξ πέντε ἔστω.").unwrap();

        // 3. Verify scope captured
        assert!(context.last_scope.is_some());
        let scope = context.last_scope.as_ref().unwrap();
        assert!(scope.lookup("ξ").is_some());

        // 4. Add a mutable binding to cover that branch
        let _ = context.execute("μετά ψ δέκα ἔστω.").unwrap();

        // 5. Run print_env to cover the table generation code
        // We aren't capturing stdout, but this ensures the code runs without panicking
        print_env(&context);

        // 6. Test clear simulation (new context)
        let context = ReplContext::new();
        assert!(context.last_scope.is_none());
        print_env(&context);
    }

    #[test]
    fn test_print_help_coverage() {
        // Just verify it doesn't panic
        print_help();
    }

    #[test]
    fn test_print_banner_coverage() {
        // Just verify it doesn't panic
        print_banner();
    }

    #[test]
    fn test_print_env_empty() {
        let context = ReplContext::new();
        // Should print "No variables defined"
        print_env(&context);
    }

    #[test]
    fn test_repl_execute_variants() {
        let mut context = ReplContext::new();

        // Test Binding execution
        let output = context.execute("ξ πέντε ἔστω.").unwrap();
        if let ReplOutput::Binding {
            name,
            type_,
            mutable,
        } = output
        {
            assert_eq!(name, "ξ");
            assert_eq!(type_, GlossaType::Number);
            assert!(!mutable);
        } else {
            panic!("Expected Binding, got {:?}", output);
        }

        // Test Statement execution
        let output = context.execute("ξ λέγε.").unwrap();
        if let ReplOutput::Statement { code } = output {
            assert!(code.contains("println"));
        } else {
            panic!("Expected Statement, got {:?}", output);
        }

        // Test None execution (no new statement, e.g. comment or empty)
        // Comments are stripped by parser usually, but let's try an empty input or just re-running without change
        // Actually, execute appends input.
        // Let's try an input that parses but produces no statements (e.g. just whitespace)
        // But parse likely fails on empty.
        // Let's try a comment if parser supports it.
        // "Using strict parser..."
        // If we can't easily generate 0 statements from valid input, we can check that
        // the statement count logic works by manually manipulating if possible,
        // but since we can't, let's trust the logic if we can verify state updates.
        // Let's rely on the logic that if we add a statement, count increases.
        assert_eq!(context.statement_count, 2);
    }

    #[test]
    fn test_repl_output_display() {
        // Test Binding display
        let binding = ReplOutput::Binding {
            name: "x".to_string(),
            type_: GlossaType::Number,
            mutable: false,
        };
        let output = binding.to_string();
        // Check content without worrying about specific color codes
        assert!(output.contains("x"));
        assert!(output.contains("Ἀριθμός"));
        assert!(!output.contains("mutable")); // Not mutable

        let binding_mut = ReplOutput::Binding {
            name: "y".to_string(),
            type_: GlossaType::String,
            mutable: true,
        };
        let output_mut = binding_mut.to_string();
        assert!(output_mut.contains("y"));
        assert!(output_mut.contains("Ὄνομα"));
        assert!(output_mut.contains("mutable"));

        // Test Statement display
        let stmt = ReplOutput::Statement {
            code: "println!(\"hello\")".to_string(),
        };
        let output_stmt = stmt.to_string();
        assert!(output_stmt.contains("Ἐκτελέσθη"));
        assert!(output_stmt.contains("println"));

        // Test None display
        let none = ReplOutput::None;
        assert_eq!(none.to_string(), "");
    }
}
