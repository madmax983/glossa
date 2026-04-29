//! The REPL (Read-Eval-Print Loop) - "The Playground"
//!
//! This module implements the interactive shell for ΓΛΩΣΣΑ.
//! It allows users to experiment with the language, test snippets, and explore
//! the type system in real-time.
//!
//! # The Playground Philosophy
//!
//! The REPL is designed to be forgiving and helpful. It maintains a persistent
//! session state, allowing users to define variables and functions incrementally.
//!
//! # Features
//!
//! * **Persistence**: Variables defined with `ἔστω` stay in scope.
//! * **Incremental Compilation**: Each line is compiled with the previous context.
//! * **Safety Limits**: Prevents memory exhaustion with strict binding and size limits.
//! * **Commands**: Built-in commands for managing the environment.
//!
//! # Commands
//!
//! * `.βοήθεια` / `.help` - Show available commands.
//! * `.ἔξοδος` / `.exit` - Exit the REPL.
//! * `.καθαρός` / `.clear` - Clear the session history (reset scope).
//! * `.περιβάλλον` / `.env` - Show all defined variables and their types.

use comfy_table::{Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::io::{BufRead, Write};

use crate::codegen::generate_statement_code;
use crate::errors::GlossaError;
use crate::semantic::{AnalyzedStatement, GlossaType, Scope};

/// Maximum number of bindings to track in REPL history
const MAX_REPL_BINDINGS: usize = 50;
/// Maximum total source size for REPL history
const MAX_REPL_SOURCE_LEN: usize = 50_000;

/// Entry point for the REPL using stdin/stdout
///
/// Starts the interactive loop, reading from standard input and writing to standard output.
///
/// # Example Usage
///
/// ```text
/// $ glossa repl
///
///    Γ Λ Ω Σ Σ Α
///    Code as the ancients intended.
///    v0.1.0
///
/// γλ> ξ πέντε ἔστω.
/// ✓ ξ: Ἀριθμός
/// γλ> ξ λέγε.
/// ✓ Ἐκτελέσθη
///   println!("{}", ξ);
/// ```
pub fn run_repl() -> Result<()> {
    print_banner();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    run_repl_inner(&mut stdin.lock(), &mut stdout)
}

/// Internal REPL loop that can be tested with arbitrary streams
fn run_repl_inner<R: BufRead, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    let mut context = ReplContext::new();

    loop {
        // We ignore write errors for the prompt as they might happen in non-interactive tests
        let _ = write!(output, "{}", "γλ> ".green().bold());
        let _ = output.flush();

        let mut line = String::new();
        let bytes = input.read_line(&mut line).into_diagnostic()?;

        // Handle EOF
        if bytes == 0 {
            writeln!(output, "\nΧαῖρε!").into_diagnostic()?;
            break;
        }

        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Handle special commands
        match trimmed {
            ".ἔξοδος" | ".exit" | ".quit" => {
                writeln!(output, "Χαῖρε!").into_diagnostic()?;
                break;
            }
            ".βοήθεια" | ".help" => {
                print_help(output)?;
                continue;
            }
            ".καθαρός" | ".clear" => {
                context = ReplContext::new();
                writeln!(output, "{} Ἐκαθαρίσθη.", "✓".green()).into_diagnostic()?;
                continue;
            }
            ".περιβάλλον" | ".env" => {
                print_env(&context, output)?;
                continue;
            }
            _ => {}
        }

        match context.execute(trimmed) {
            Ok(repl_output) => {
                // Display handles formatting (empty if None)
                write!(output, "{}", repl_output).into_diagnostic()?;
            }
            Err(e) => {
                // Use default error formatting but ensure it's visible
                // The '×' symbol provides visual indication of error, so we don't need
                // to prefix with "Σφάλμα: " which often leads to redundancy.
                writeln!(output, "{}", format!("× {}", e).red()).into_diagnostic()?;
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

fn print_help<W: Write>(w: &mut W) -> Result<()> {
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

    writeln!(w, "{table}").into_diagnostic()?;
    writeln!(w, "\n{}", "Παραδείγματα:".bold().underlined()).into_diagnostic()?;
    writeln!(w, "  «χαῖρε κόσμε» λέγε.").into_diagnostic()?;
    writeln!(w, "  ξ πέντε ἔστω.").into_diagnostic()?;
    Ok(())
}

fn print_env<W: Write>(context: &ReplContext, w: &mut W) -> Result<()> {
    if let Some(scope) = &context.last_scope {
        // Collect and sort bindings for consistent display
        let mut bindings: Vec<_> = scope.bindings().collect();
        bindings.sort_by(|a, b| a.0.cmp(b.0));

        if bindings.is_empty() {
            let mut empty_table = Table::new();
            empty_table.load_preset(presets::UTF8_FULL);
            empty_table.add_row(vec![
                Cell::new("Οὐδεμία μεταβλητή (No variables defined).")
                    .fg(Color::Yellow)
                    .set_alignment(comfy_table::CellAlignment::Center),
            ]);
            writeln!(w, "{}", empty_table).into_diagnostic()?;
            return Ok(());
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
        writeln!(w, "{table}").into_diagnostic()?;
    } else {
        let mut empty_table = Table::new();
        empty_table.load_preset(presets::UTF8_FULL);
        empty_table.add_row(vec![
            Cell::new("Οὐδεμία μεταβλητή (No variables defined).")
                .fg(Color::Yellow)
                .set_alignment(comfy_table::CellAlignment::Center),
        ]);
        writeln!(w, "{}", empty_table).into_diagnostic()?;
    }
    Ok(())
}

/// REPL Output variants
#[derive(Debug)]
pub enum ReplOutput {
    /// A new variable binding
    Binding {
        /// The identifier the user has chosen to represent this piece of reality.
        name: String,
        /// The philosophical form (`εἶδος`) that governs what this identifier can do.
        type_: GlossaType,
        /// Indicates if the user allows this reality to change (`true`) or demands it remain constant (`false`).
        mutable: bool,
    },
    /// Code execution (compilation)
    Statement {
        /// The translated Rust syntax ready to be executed by the underlying machine.
        code: String,
    },
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
        // Safety: Prevent memory exhaustion from infinite binding history
        // The REPL re-compiles the entire history on every line, so we must limit it.
        if self.bindings.len() > MAX_REPL_BINDINGS {
            return Err(GlossaError::semantic(
                "REPL binding limit exceeded (50). Please use .καθαρός (.clear)",
            ));
        }

        // 1. Construct the Virtual Source File
        // We simulate a single persistent file by concatenating previous valid bindings
        // with the new input. This allows variable references to resolve correctly.
        let mut full_source = self.bindings.join("\n");
        if !full_source.is_empty() {
            full_source.push('\n');
        }
        full_source.push_str(input);

        // Safety: Check total size limit
        if full_source.len() > MAX_REPL_SOURCE_LEN {
            return Err(GlossaError::semantic(
                "REPL source size limit exceeded (50KB). Please use .καθαρός (.clear)",
            ));
        }

        // 2. Compile the Virtual File
        // If this fails (parse error, type error), the history remains unchanged.
        let analyzed = crate::tools::runner::analyze_source(&full_source)
            .map_err(|e| GlossaError::semantic(e.to_string()))?;

        // 3. Detect New Activity
        // If the new input didn't add any executable statements (e.g. it was just a comment),
        // we don't need to do anything.
        let new_count = analyzed.statements.len();
        if new_count <= self.statement_count {
            return Ok(ReplOutput::None);
        }

        // 4. Update State
        // The compilation succeeded, so we update our snapshot of the scope and statement count.
        self.last_scope = Some(analyzed.scope.clone());
        self.statement_count = new_count;

        // 5. Analyze Result
        // We only care about the *last* statement, because previous statements have already
        // been executed/processed in previous turns.
        let last_stmt = match analyzed.statements.last() {
            Some(stmt) => stmt,
            None => return Ok(ReplOutput::None),
        };
        match last_stmt {
            AnalyzedStatement::Binding { name, mutable, .. } => {
                // Lookup type from scope since it's no longer in the binding statement
                let value_type = analyzed
                    .scope
                    .lookup(name)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown);

                // Persistence: Variable definitions MUST be saved to history
                // so they can be referenced in future lines.
                self.bindings.push(input.to_string());

                Ok(ReplOutput::Binding {
                    name: name.to_string(),
                    type_: value_type,
                    mutable: *mutable,
                })
            }
            _ => {
                // Persistence Logic:
                // - Side effects (print, expressions) are NOT saved. We don't want to
                //   print "Hello" 50 times just because we defined a variable later.
                // - Structural definitions (Types, Functions, Traits) MUST be saved.
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
        let mut buf = Vec::new();
        print_env(&context, &mut buf).unwrap();

        // 2. Execute binding
        let _ = context.execute("ξ πέντε ἔστω.").unwrap();

        // 3. Verify scope captured
        assert!(context.last_scope.is_some());
        let scope = context.last_scope.as_ref().unwrap();
        assert!(scope.lookup("ξ").is_some());

        // 4. Add a mutable binding to cover that branch
        let _ = context.execute("μετά ψ δέκα ἔστω.").unwrap();

        // 5. Run print_env to cover the table generation code
        print_env(&context, &mut buf).unwrap();

        // 6. Test clear simulation (new context)
        let context = ReplContext::new();
        assert!(context.last_scope.is_none());
        print_env(&context, &mut buf).unwrap();
    }

    #[test]
    fn test_print_help_coverage() {
        // Just verify it doesn't panic
        let mut buf = Vec::new();
        print_help(&mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("βοήθεια"));
    }

    #[test]
    fn test_print_banner_coverage() {
        // Just verify it doesn't panic
        print_banner();
    }

    #[test]
    fn test_print_env_empty() {
        let mut context = ReplContext::new();
        let mut buf = Vec::new();

        // Context with no last_scope
        print_env(&context, &mut buf).unwrap();
        let s = String::from_utf8(buf.clone()).unwrap();
        assert!(s.contains("No variables defined"));

        // Execute a statement that does not create a binding
        // so last_scope becomes Some(scope) but bindings is empty
        buf.clear();
        let _ = context.execute("«χαῖρε» λέγε.").unwrap();

        assert!(context.last_scope.is_some());
        print_env(&context, &mut buf).unwrap();
        let s2 = String::from_utf8(buf).unwrap();
        assert!(s2.contains("No variables defined"));
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

    #[test]
    fn test_run_repl_inner_workflow() {
        let input_data = "\
ξ πέντε ἔστω.\n\
.env\n\
.clear\n\
.env\n\
.exit\n";
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        let result = run_repl_inner(&mut input, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        // Verify prompts
        assert!(output_str.contains("γλ>"));
        // Verify binding execution
        assert!(output_str.contains("ξ"));
        assert!(output_str.contains("Ἀριθμός"));
        // Verify clear
        assert!(output_str.contains("Ἐκαθαρίσθη"));
        // Verify env after clear (should be empty)
        assert!(output_str.contains("No variables defined"));
        // Verify exit
        assert!(output_str.contains("Χαῖρε"));
    }

    #[test]
    fn test_run_repl_inner_help() {
        let input_data = ".help\n.exit\n";
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        let result = run_repl_inner(&mut input, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("βοήθεια"));
        assert!(output_str.contains("Περιγραφή"));
    }

    #[test]
    fn test_run_repl_inner_eof() {
        // Empty input simulates immediate EOF
        let input_data = "";
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        let result = run_repl_inner(&mut input, &mut output);
        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Χαῖρε"));
    }

    #[test]
    fn test_run_repl_inner_error_output() {
        // Test that the error output is formatted correctly (with '×')
        // Using invalid syntax to trigger a ParseError
        let input_data = "invalid syntax\n.exit\n";
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        let result = run_repl_inner(&mut input, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        // Should contain the error indicator
        assert!(
            output_str.contains("×"),
            "Output should contain error indicator '×'"
        );
        // Should contain the error message content
        assert!(
            output_str.contains("Parse error"),
            "Output should contain 'Parse error'"
        );
        // Should NOT contain the redundant "Σφάλμα: " prefix if the error itself starts with it
        // The error message from GlossaError::ParseError starts with "Σφάλμα συντάξεως: ..."
        // Our formatting prints "× error_string".
        // We just want to ensure it printed.
    }
}
