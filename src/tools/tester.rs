//! The Judge (Test Runner) Tool
//!
//! This module implements the `glossa test` command, allowing developers to write and run
//! unit tests directly in ΓΛΩΣΣΑ.
//!
//! # The Strategy: Harness Generation
//!
//! Unlike interpreted languages that run tests in a VM, or compiled languages that need
//! complex test discovery, ΓΛΩΣΣΑ leverages the existing Rust test infrastructure.
//!
//! 1.  **Parse & Analyze**: The source file is parsed and analyzed as usual.
//! 2.  **Generate Test Harness**: The `codegen` module produces a Rust file where
//!     `δοκιμή` (test) declarations are converted to `#[test]` functions.
//! 3.  **Compile with `--test`**: The generated Rust file is compiled using `rustc --test`.
//!     This produces a standalone test executable.
//! 4.  **Execute & Capture**: The executable is run, and its output (stdout/stderr) is captured.
//! 5.  **Parse & Report**: The raw output is parsed to provide a stylized, emoji-rich report.
//!
//! # Writing Tests
//!
//! Tests are declared using the `δοκιμή` keyword:
//!
//! ```glossa
//! δοκιμή "Addition works" {
//!     ξ 10 ἔστω.
//!     ψ 20 ἔστω.
//!     assert_eq(ξ + ψ, 30).
//! }
//! ```
//!
//! This compiles to:
//!
//! ```rust,ignore
//! #[test]
//! fn test_addition_works() {
//!     let g_x = 10;
//!     let g_y = 20;
//!     assert_eq!(g_x + g_y, 30);
//! }
//! ```

use crate::codegen::generate_rust_file;
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, CellAlignment, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::Builder;

#[derive(Debug, PartialEq)]
enum TestStatus {
    Ok,
    Failed,
    Ignored,
}

#[derive(Debug)]
struct TestResult {
    name: String,
    status: TestStatus,
}

fn parse_test_output(output: &str) -> Vec<TestResult> {
    output
        .lines()
        .filter_map(|line| {
            // Standard rustc test output line format: "test test_name ... status"
            let rest = line.strip_prefix("test ")?;
            let idx = rest.rfind(" ... ")?;

            let name = rest[..idx].trim();
            if name.is_empty() {
                return None;
            }

            let status = match &rest[idx + 5..] {
                "ok" => TestStatus::Ok,
                "FAILED" => TestStatus::Failed,
                "ignored" => TestStatus::Ignored,
                _ => return None,
            };

            Some(TestResult {
                name: name.to_string(),
                status,
            })
        })
        .collect()
}

/// Extracts failed tests and their output from `rustc --test` output.
///
/// ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<&str>>()` allocation.
/// This parses the output stream in-place using a `Peekable` iterator, reducing memory
/// allocations when test outputs are large.
fn extract_failures(output: &str) -> Vec<(String, String)> {
    // ⚡ Bolt Optimization: Uses `Vec::with_capacity` based on heuristic estimation.
    let mut failures = Vec::with_capacity(4);
    let mut lines = output.lines().peekable();

    // Skip until "failures:"
    for line in lines.by_ref() {
        if line.trim() == "failures:" {
            break;
        }
    }

    // Scan for "---- <name> stdout ----"
    while let Some(line) = lines.next() {
        let Some(name) = parse_failure_name(line) else {
            continue;
        };

        let message = capture_failure_message(&mut lines);
        failures.push((name, message));
    }

    failures
}

fn parse_failure_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("----") || !trimmed.ends_with("stdout ----") {
        return None;
    }

    let name_part = trimmed
        .trim_start_matches("---- ")
        .trim_end_matches(" stdout ----");

    // Remove module path if present for cleaner display
    Some(
        name_part
            .split("::")
            .last()
            .unwrap_or(name_part)
            .to_string(),
    )
}

fn capture_failure_message(lines: &mut std::iter::Peekable<std::str::Lines>) -> String {
    let mut message = String::new();
    while let Some(current) = lines.peek() {
        let trimmed = current.trim();
        if (trimmed.starts_with("----") && trimmed.ends_with("stdout ----"))
            || trimmed == "failures:"
        {
            break;
        }

        // Hide backtrace hint
        if current.starts_with("note: run with `RUST_BACKTRACE=1`") {
            lines.next();
            continue;
        }

        // Stop before stack trace
        if trimmed.starts_with("stack backtrace:")
            || current.starts_with("note: Some details are omitted")
        {
            lines.next();
            while let Some(next) = lines.peek() {
                let next_trimmed = next.trim();
                if next_trimmed.starts_with("----") && next_trimmed.ends_with("stdout ----")
                    || next_trimmed == "failures:"
                {
                    break;
                }
                lines.next();
            }
            continue;
        }

        if let Some(clean_panic) = clean_panic_message(current) {
            message.push_str(&clean_panic);
            message.push('\n');
            lines.next();
            continue;
        }

        message.push_str(current);
        message.push('\n');
        lines.next();
    }
    message.trim().to_string()
}

fn clean_panic_message(current: &str) -> Option<String> {
    if !current.starts_with("thread '") {
        return None;
    }

    let panicked_idx = current.find("panicked at")?;
    let mut clean_panic = format!("{}panicked", &current[..panicked_idx]);

    // Remove the "(pid)" thread id
    #[allow(clippy::collapsible_if)]
    if let Some(idx1) = clean_panic.find(" (") {
        if let Some(idx2) = clean_panic[idx1..].find(") ") {
            clean_panic.replace_range(idx1..idx1 + idx2 + 2, " ");
        }
    }

    Some(clean_panic)
}

/// Run tests defined in a Glossa file
///
/// This tool compiles the Glossa source to Rust, but instead of building a regular binary,
/// it compiles it with `rustc --test`. This creates a test harness that runs all functions
/// marked with `#[test]` (which `codegen` generates for `TestDeclaration` nodes).
fn compile_test_harness(temp_path: &Path, exe_path: &Path, status: Status) -> Result<Status> {
    let rustc_cmd = std::env::var("GLOSSA_RUSTC_CMD").unwrap_or_else(|_| "rustc".to_string());

    let rustc_output = Command::new(rustc_cmd)
        .arg("--test")
        .arg(temp_path)
        .arg("-o")
        .arg(exe_path)
        .output()
        .map_err(|e| miette::miette!("Failed to start rustc. Is Rust installed? Detail: {}", e))?;

    if !rustc_output.status.success() {
        let raw_stderr = String::from_utf8_lossy(&rustc_output.stderr);

        let mut clean_stderr = String::new();
        let mut prev_empty = false;
        for line in raw_stderr.lines() {
            // Skip file location lines
            if line.starts_with(" --> ") {
                continue;
            }

            let trimmed = line.trim_start();

            // Check if the line starts with a line number followed by |
            if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) && trimmed.contains(" | ")
            {
                continue;
            }

            // Skip the underline carets and empty pipes
            if trimmed.starts_with('|') {
                continue;
            }

            let is_empty = line.trim().is_empty();
            if is_empty && prev_empty {
                continue;
            }

            clean_stderr.push_str(line);
            clean_stderr.push('\n');
            prev_empty = is_empty;
        }

        status.error("Σφάλμα μεταγλωττίσεως δοκιμῶν (Test Compilation Error)");
        return Err(miette::miette!(
            "{}\n{}",
            "Rustc Error:".red(),
            clean_stderr.trim()
        ));
    }

    Ok(status)
}

fn execute_test_binary(exe_path: &Path, status: &mut Status) -> Result<std::process::Output> {
    status.update("Ἐκτέλεσις (Running)");

    let test_output = Command::new(exe_path)
        .output() // Capture output to display it nicely
        .into_diagnostic()?;

    // Cleanup: The executable must be deleted manually.
    let _ = fs::remove_file(exe_path);

    Ok(test_output)
}

fn print_test_results(results: &[TestResult], test_output: &std::process::Output, stdout: &str) {
    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   T E S T E R".bold().cyan());
    println!("   {}", "Unit Test Results".italic().dim());
    println!();

    if test_output.status.success() {
        if !results.is_empty() {
            let mut success_table = Table::new();
            success_table.load_preset(presets::UTF8_FULL);
            success_table.add_row(vec![
                Cell::new(" ✓ Πᾶσαι αἱ δοκιμασίαι ἐπέτυχαν! (All tests passed) ")
                    .bg(Color::DarkGreen)
                    .fg(Color::White)
                    .add_attribute(Attribute::Bold),
            ]);
            println!("{success_table}");
            println!();
        }
    } else {
        let mut failure_table = Table::new();
        failure_table.load_preset(presets::UTF8_FULL);
        failure_table.add_row(vec![
            Cell::new(" ✕ Τινὲς δοκιμασίαι ἀπέτυχαν (Some tests failed) ")
                .bg(Color::DarkRed)
                .fg(Color::White)
                .add_attribute(Attribute::Bold),
        ]);
        println!("{failure_table}");
        println!();
    }

    if !results.is_empty() {
        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL);

        table.set_header(vec![
            Cell::new("Test Case")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Status").add_attribute(Attribute::Bold),
        ]);

        for result in results {
            let status_cell = match result.status {
                TestStatus::Ok => Cell::new("PASSED").fg(Color::Green),
                TestStatus::Failed => Cell::new("FAILED")
                    .fg(Color::Red)
                    .add_attribute(Attribute::Bold),
                TestStatus::Ignored => Cell::new("IGNORED").fg(Color::Yellow),
            };

            // Clean up test name (remove module prefix if any)
            // e.g., "tests::test_name" -> "test_name"
            let display_name = result.name.split("::").last().unwrap_or(&result.name);

            table.add_row(vec![Cell::new(display_name), status_cell]);
        }
        println!("{table}");
    } else {
        let mut empty_table = Table::new();
        empty_table.load_preset(presets::UTF8_FULL);
        empty_table.set_header(vec![
            Cell::new("Status")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
        ]);
        empty_table.add_row(vec![
            Cell::new("No tests found.")
                .fg(Color::DarkGrey)
                .add_attribute(Attribute::Italic)
                .set_alignment(CellAlignment::Center),
        ]);
        println!("{empty_table}");
    }

    // If there were failures, try to extract and print them nicely
    if !test_output.status.success() {
        println!();
        println!("{}", "--- 📜 Λεπτoμέρειες (Details) ---".dim());

        let failures = extract_failures(stdout);

        if !failures.is_empty() {
            for (name, msg) in failures {
                let mut header_table = Table::new();
                header_table.load_preset(presets::UTF8_FULL);
                header_table.add_row(vec![
                    Cell::new(format!(" FAILED: {} ", name))
                        .bg(Color::DarkRed)
                        .fg(Color::White)
                        .add_attribute(Attribute::Bold),
                ]);
                println!("{header_table}");

                // Create a box for the error message using comfy_table
                let mut error_table = Table::new();
                error_table.load_preset(presets::UTF8_FULL);
                error_table.add_row(vec![Cell::new(format!("\n{}\n", msg)).fg(Color::Red)]);
                println!("{error_table}");
                println!();
            }
        } else {
            // Fallback to raw output if extraction failed but tests failed
            println!("{}", stdout);
            if !test_output.stderr.is_empty() {
                println!("{}", String::from_utf8_lossy(&test_output.stderr).red());
            }
        }
    }
}

/// Run unit tests defined within a ΓΛΩΣΣΑ file.
///
/// This tool allows developers to execute `δοκιμή` (test) blocks directly from their source
/// files. It works by orchestrating the compilation pipeline, generating a Rust file with
/// `#[test]` attributes, compiling it using `rustc --test`, and then executing the resulting
/// test harness binary. It finally parses the output to present a clean, language-specific
/// test report to the user.
///
/// ## Examples
///
/// ```rust
/// use glossa::tools::tester::run_tests;
/// use std::path::PathBuf;
/// use std::fs;
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// let input = dir.path().join("tests.γλ");
///
/// // Create a temporary Glossa file with a test declaration
/// fs::write(&input, "δοκιμή «example».\n  ξ 5 ἔστω.\nτέλος.").unwrap();
///
/// // Execute the test harness
/// run_tests(&input).unwrap();
/// ```
pub fn run_tests(input: &Path) -> Result<()> {
    // 1 & 2. Validation & Compilation (Lex -> Parse -> Analyze -> Codegen)
    let source = crate::tools::runner::load_source(input)?;

    let status = Status::start_with_symbol("Δοκιμασία (Testing)", "🧪");

    let analyzed = match crate::tools::runner::analyze_source(&source) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };
    let rust_code = generate_rust_file(&analyzed);

    // 3. Create temporary file for Rust source
    let mut temp_file = Builder::new()
        .prefix("glossa_test_")
        .suffix(".rs")
        .tempfile()
        .map_err(|e| miette::miette!("Failed to create temporary file for test: {}", e))?;

    write!(temp_file, "{}", rust_code)
        .map_err(|e| miette::miette!("Failed to write to temporary test file: {}", e))?;
    let temp_path = temp_file.path().to_owned();

    // 4. Determine output path for the test binary
    let exe_name = temp_path
        .file_stem()
        .ok_or_else(|| miette::miette!("Σφάλμα: Could not extract file stem from temp path"))?
        .to_string_lossy()
        .into_owned();
    let exe_path = temp_path
        .parent()
        .ok_or_else(|| {
            miette::miette!("Σφάλμα: Could not extract parent directory from temp path")
        })?
        .join(if cfg!(windows) {
            format!("{}.exe", exe_name)
        } else {
            exe_name
        });

    // 5. Compile with rustc --test
    let mut status = compile_test_harness(&temp_path, &exe_path, status)?;

    // 6. Run the test binary
    let test_output = execute_test_binary(&exe_path, &mut status)?;

    // 7. Parse and Report results
    let stdout = String::from_utf8_lossy(&test_output.stdout);
    let results = parse_test_output(&stdout);

    if test_output.status.success() {
        status.success();
    } else {
        status.error("Ἀποτυχία (Failure)");
    }

    print_test_results(&results, &test_output, &stdout);

    if !test_output.status.success() {
        return Err(miette::miette!("Tests failed"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_tests_rustc_missing() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("test_rustc_missing.gl");
        std::fs::write(&input_path, "δοκιμή «test» { «ok» λέγε. }.").unwrap();

        // Spawn a child process so we don't mutate the global PATH/env.
        let bin_path = std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
            let llvm_cov_path = "target/llvm-cov-target/debug/glossa";
            if std::path::Path::new(llvm_cov_path).exists() {
                llvm_cov_path.to_string()
            } else {
                "target/debug/glossa".to_string()
            }
        });
        let mut cmd = std::process::Command::new(bin_path);
        let output = cmd
            .arg("test")
            .arg(&input_path)
            .env("GLOSSA_RUSTC_CMD", "nonexistent_rustc_binary")
            .output()
            .expect("Failed to execute glossa binary");

        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("Failed to start rustc. Is Rust installed?"));
    }

    #[test]
    fn test_parse_output_basic() {
        let output = "
running 2 tests
test test_one ... ok
test test_two ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
";
        let results = parse_test_output(output);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "test_one");
        assert_eq!(results[0].status, TestStatus::Ok);
        assert_eq!(results[1].name, "test_two");
        assert_eq!(results[1].status, TestStatus::Ok);
    }

    #[test]
    fn test_parse_output_failure() {
        let output = "
running 2 tests
test test_one ... FAILED
test test_two ... ok

failures:

---- test_one stdout ----
thread 'test_one' panicked at 'assertion failed: `(left == right)`
  left: `5`,
 right: `4`', src/lib.rs:2:5

failures:
    test_one

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
";
        let results = parse_test_output(output);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "test_one");
        assert_eq!(results[0].status, TestStatus::Failed);
        assert_eq!(results[1].name, "test_two");
        assert_eq!(results[1].status, TestStatus::Ok);
    }

    #[test]
    fn test_parse_output_ignored() {
        let output = "
running 1 test
test test_ignore ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
";
        let results = parse_test_output(output);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "test_ignore");
        assert_eq!(results[0].status, TestStatus::Ignored);
    }

    #[test]
    fn test_extract_failures_basic() {
        let output = "
running 1 test
test my_test ... FAILED

failures:

---- my_test stdout ----
Error message here
Another line

failures:
    my_test

test result: FAILED...
";
        let failures = extract_failures(output);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].0, "my_test");
        assert!(failures[0].1.contains("Error message here"));
        assert!(failures[0].1.contains("Another line"));
    }

    #[test]
    fn test_extract_failures_multiple() {
        let output = "
failures:

---- test1 stdout ----
Error 1

---- test2 stdout ----
Error 2

failures:
    test1
    test2
";
        let failures = extract_failures(output);
        assert_eq!(failures.len(), 2);
        assert_eq!(failures[0].0, "test1");
        assert!(failures[0].1.contains("Error 1"));
        assert_eq!(failures[1].0, "test2");
        assert!(failures[1].1.contains("Error 2"));
    }

    #[test]
    fn test_extract_failures_edge_cases() {
        struct TestCase {
            name: &'static str,
            input: &'static str,
            expected_count: usize,
        }

        let test_cases = vec![
            TestCase {
                name: "No failures block",
                input: "
running 1 test
test my_test ... FAILED

test result: FAILED...
",
                expected_count: 0,
            },
            TestCase {
                name: "Malformed stdout block",
                input: "
failures:

---- my_test_without_end_block
Error message here

failures:
    my_test_without_end_block
",
                expected_count: 0,
            },
            TestCase {
                name: "Missing end of test details block",
                input: "
failures:

---- my_test_missing_end stdout ----
Error message here
",
                expected_count: 1,
            },
            TestCase {
                name: "Missing next test block",
                input: "
failures:

---- my_test_next stdout ----
Error 1
---- my_test_next_missing stdout ----
Error 2
",
                expected_count: 2,
            },
            TestCase {
                name: "Thread panicked internal rust string stripped",
                input: "
failures:

---- test_panicked stdout ----
thread 'test_panicked' panicked at 'internal panic message', src/lib.rs:1:1
   0: std::backtrace::Backtrace::create
   1: core::panicking::panic_fmt
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
",
                expected_count: 1, // It should still capture the failure but strip some internal messages
            },
        ];

        for case in test_cases {
            let failures = extract_failures(case.input);
            assert_eq!(
                failures.len(),
                case.expected_count,
                "Failed case: {}",
                case.name
            );
        }
    }

    #[test]
    fn test_parse_test_output_edge_cases() {
        struct TestCase {
            name: &'static str,
            input: &'static str,
            expected_count: usize,
        }

        let test_cases = vec![
            TestCase {
                name: "Empty parts (less than 4)",
                input: "
running 1 test
test ... ok
",
                expected_count: 0,
            },
            TestCase {
                name: "Missing test prefix",
                input: "my_test ... ok",
                expected_count: 0,
            },
            TestCase {
                name: "Unknown status",
                input: "test my_test ... WEIRD_STATUS",
                expected_count: 0,
            },
            TestCase {
                name: "Just test prefix",
                input: "test ",
                expected_count: 0,
            },
            TestCase {
                name: "Extraneous info but valid format",
                input: "test my_test has extra words before ... ok",
                expected_count: 1,
            },
        ];

        for case in test_cases {
            let results = parse_test_output(case.input);
            assert_eq!(
                results.len(),
                case.expected_count,
                "Failed case: {}",
                case.name
            );
        }
    }
}

#[cfg(test)]
mod additional_sentry_tests {
    use super::*;

    // Since we're in tools/tester.rs, let's verify edge case parsing
    #[test]
    fn test_parse_test_output_empty_parts() {
        let output = "test \n test"; // Not enough parts
        let results = parse_test_output(output);
        assert_eq!(results.len(), 0);
    }
}

#[cfg(test)]
mod tests_failures {
    use super::*;

    #[test]
    fn test_extract_failures_no_matching_start() {
        let output = "
failures:

some random text
no dashed lines here

failures:
    test_one
";
        let failures = extract_failures(output);
        assert_eq!(failures.len(), 0);
    }

    #[test]
    fn test_parse_test_output_various_parts() {
        let output = "
test   ... ok
test name ... ok
test name with spaces ... ok
";
        let results = parse_test_output(output);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_extract_failures_with_empty_output() {
        let output = "";
        let failures = extract_failures(output);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_extract_failures_with_stack_backtrace() {
        let output = "failures:\n\n---- test_1 stdout ----\nstack backtrace:\n  1: std::panicking::begin_panic\n\nfailures:\n    test_1\n";
        let failures = extract_failures(output);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].0, "test_1");
        assert!(!failures[0].1.contains("stack backtrace:"));
    }

    #[test]
    fn test_extract_failures_with_note_omitted() {
        let output = "failures:\n\n---- test_1 stdout ----\nnote: Some details are omitted\n  1: std::panicking::begin_panic\n\nfailures:\n    test_1\n";
        let failures = extract_failures(output);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].0, "test_1");
        assert!(!failures[0].1.contains("note: Some details are omitted"));
    }

    #[test]
    fn test_extract_failures_without_actual_failures() {
        let output = "failures:\n\n\n\n";
        let failures = extract_failures(output);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_extract_failures_simple_panic() {
        let output = "failures:\n\n---- test_1 stdout ----\nthread 'test_1' panicked\n\nfailures:\n    test_1\n";
        let failures = extract_failures(output);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].0, "test_1");
        assert!(failures[0].1.contains("panicked"));
    }

    #[test]
    fn test_run_tests_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("parse_error.gl");
        std::fs::write(&input_path, b"invalid syntax").unwrap();

        let result = run_tests(&input_path);
        assert!(result.is_err());
        // The underlying error bubbles up. We just need to know it failed.
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_run_tests_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("semantic_error.gl");
        std::fs::write(&input_path, "ψ 10 γίγνεται.").unwrap();

        let result = run_tests(&input_path);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        // The underlying error bubbles up.
        assert!(err_msg.contains("Semantic error") || err_msg.contains("Σφάλμα"));
    }
}
