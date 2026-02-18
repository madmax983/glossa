use crate::codegen::generate_rust_file;
use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::Builder;

/// Run tests defined in a Glossa file
///
/// This tool compiles the Glossa source to Rust, but instead of building a regular binary,
/// it compiles it with `rustc --test`. This creates a test harness that runs all functions
/// marked with `#[test]` (which `codegen` generates for `TestDeclaration` nodes).
pub fn run_tests(input: &Path) -> Result<()> {
    // 1. Validation
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let mut status = Status::start("Δοκιμασία (Testing)");

    // 2. Compilation (Lex -> Parse -> Analyze -> Codegen)
    let source = fs::read_to_string(input).into_diagnostic()?;
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let analyzed = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;
    let rust_code = generate_rust_file(&analyzed);

    // 3. Create temporary file for Rust source
    let mut temp_file = Builder::new()
        .prefix("glossa_test_")
        .suffix(".rs")
        .tempfile()
        .into_diagnostic()?;

    write!(temp_file, "{}", rust_code).into_diagnostic()?;
    let temp_path = temp_file.path().to_owned();

    // 4. Determine output path for the test binary
    // We use the temp directory to avoid polluting the user's workspace
    // Use the temp file name to ensure uniqueness
    let exe_name = temp_path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let exe_path = temp_path.parent().unwrap().join(if cfg!(windows) {
        format!("{}.exe", exe_name)
    } else {
        exe_name
    });

    // 5. Compile with rustc --test
    let rustc_output = Command::new("rustc")
        .arg("--test")
        .arg(&temp_path)
        .arg("-o")
        .arg(&exe_path)
        .output()
        .into_diagnostic()?;

    if !rustc_output.status.success() {
        let stderr = String::from_utf8_lossy(&rustc_output.stderr);
        status.error("Σφάλμα μεταγλωττίσεως δοκιμῶν (Test Compilation Error)");
        return Err(miette::miette!("{}\n{}", "Rustc Error:".red(), stderr));
    }

    // 6. Run the test binary
    status.update("Ἐκτέλεσις (Running)");

    let test_output = Command::new(&exe_path)
        .output() // Capture output to display it nicely
        .into_diagnostic()?;

    // Cleanup: The temp_file (source) is auto-deleted when dropped.
    // The executable must be deleted manually.
    let _ = fs::remove_file(&exe_path);

    // 7. Report results
    if test_output.status.success() {
        status.success();
        println!();
        println!(
            "{}",
            "✓ Πᾶσαι αἱ δοκιμασίαι ἐπέτυχαν! (All tests passed)"
                .green()
                .bold()
        );
        println!("{}", String::from_utf8_lossy(&test_output.stdout));
    } else {
        status.error("Ἀποτυχία (Failure)");
        println!();
        println!(
            "{}",
            "✕ Τινὲς δοκιμασίαι ἀπέτυχαν (Some tests failed)"
                .red()
                .bold()
        );
        println!("{}", String::from_utf8_lossy(&test_output.stdout));
        if !test_output.stderr.is_empty() {
            println!("{}", String::from_utf8_lossy(&test_output.stderr).red());
        }
        return Err(miette::miette!("Tests failed"));
    }

    Ok(())
}
