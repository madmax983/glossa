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
use crate::parser::parse;
use crate::semantic::analyze_program;
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
    let mut results = Vec::new();
    for line in output.lines() {
        // Standard rustc test output line format: "test test_name ... status"
        if line.starts_with("test ")
            && (line.ends_with(" ... ok")
                || line.ends_with(" ... FAILED")
                || line.ends_with(" ... ignored"))
        {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Expected parts: ["test", "test_name", "...", "status"]
            // Sometimes there might be extra info, but name is usually at index 1
            if parts.len() >= 4 {
                #[allow(clippy::collapsible_if)]
                if let [_, name, .., status_str] = parts.as_slice() {
                    let status = match *status_str {
                        "ok" => TestStatus::Ok,
                        "FAILED" => TestStatus::Failed,
                        "ignored" => TestStatus::Ignored,
                        _ => continue,
                    };
                    results.push(TestResult {
                        name: name.to_string(),
                        status,
                    });
                }
            }
        }
    }
    results
}

/// Extracts failed tests and their output from `rustc --test` output.
///
/// ⚡ Bolt Optimization: Removed intermediate `.collect::<Vec<&str>>()` allocation.
/// This parses the output stream in-place using a `Peekable` iterator, reducing memory
/// allocations when test outputs are large.
fn extract_failures(output: &str) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    let mut lines = output.lines().peekable();

    // Skip until "failures:"
    for line in lines.by_ref() {
        if line.trim() == "failures:" {
            break;
        }
    }

    // Scan for "---- <name> stdout ----"
    while let Some(line) = lines.next() {
        if line.trim().starts_with("----") && line.trim().ends_with("stdout ----") {
            // Extract name: "---- test_name stdout ----"
            let trimmed = line.trim();
            let name_part = trimmed
                .trim_start_matches("---- ")
                .trim_end_matches(" stdout ----");
            // Remove module path if present for cleaner display
            let name = name_part
                .split("::")
                .last()
                .unwrap_or(name_part)
                .to_string();

            // Capture output until next "----" or empty line followed by "failures:"
            let mut message = String::new();
            while let Some(current) = lines.peek() {
                if current.trim().starts_with("----") && current.trim().ends_with("stdout ----") {
                    // Start of next failure
                    break;
                }
                if current.trim() == "failures:" {
                    // End of details section
                    break;
                }
                message.push_str(current);
                message.push('\n');
                lines.next();
            }
            failures.push((name, message.trim().to_string()));
        }
    }

    failures
}

/// Run tests defined in a Glossa file
///
/// This tool compiles the Glossa source to Rust, but instead of building a regular binary,
/// it compiles it with `rustc --test`. This creates a test harness that runs all functions
/// marked with `#[test]` (which `codegen` generates for `TestDeclaration` nodes).
pub fn run_tests(input: &Path) -> Result<()> {
    // 1 & 2. Validation & Compilation (Lex -> Parse -> Analyze -> Codegen)
    let source = crate::tools::runner::load_source(input)?;
    let mut status = Status::start_with_symbol("Δοκιμασία (Testing)", "🧪");

    let ast = match parse(&source).map_err(|e| miette::miette!("{}", e)) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα συντάξεως (Syntax Error)");
            return Err(e);
        }
    };

    let analyzed = match analyze_program(&ast).map_err(|e| miette::miette!("{}", e)) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα σημασιολογίας (Semantic Error)");
            return Err(e);
        }
    };
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

    // 7. Parse and Report results
    let stdout = String::from_utf8_lossy(&test_output.stdout);
    let results = parse_test_output(&stdout);

    if test_output.status.success() {
        status.success();
    } else {
        status.error("Ἀποτυχία (Failure)");
    }

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   T E S T E R".bold().cyan());
    println!("   {}", "Unit Test Results".italic().dim());
    println!();

    if test_output.status.success() {
        if !results.is_empty() {
            println!(
                "   {}",
                "✓ Πᾶσαι αἱ δοκιμασίαι ἐπέτυχαν! (All tests passed)"
                    .green()
                    .bold()
            );
            println!();
        }
    } else {
        println!(
            "   {}",
            "✕ Τινὲς δοκιμασίαι ἀπέτυχαν (Some tests failed)"
                .red()
                .bold()
        );
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

        for result in &results {
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

        let failures = extract_failures(&stdout);

        if !failures.is_empty() {
            for (name, msg) in failures {
                println!(
                    "{} {}",
                    "FAILED:".red().bold(),
                    name.cyan().bold().underlined()
                );
                // Create a box for the error message
                let border_top =
                    "╭───────────────────────────────────────────────────────────────────╮".red();
                let border_bottom =
                    "╰───────────────────────────────────────────────────────────────────╯".red();

                println!("{}", border_top);
                for line in msg.lines() {
                    // Wrap extremely long lines if needed, but for now simple print
                    println!("{} {}", "│".red(), line);
                }
                println!("{}", border_bottom);
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

    if !test_output.status.success() {
        return Err(miette::miette!("Tests failed"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
