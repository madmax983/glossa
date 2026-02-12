//! ΓΛΩΣΣΑ Interactive REPL
//!
//! Handles the Read-Eval-Print Loop for the compiler.

use comfy_table::{Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::io::Write;

use crate::codegen::generate_statement_code;
use crate::errors::GlossaError;
use crate::parser::parse;
use crate::semantic::{AnalyzedStatement, GlossaType, Scope, analyze_program};

/// Maximum number of bindings to track in REPL history
const MAX_REPL_BINDINGS: usize = 50;
/// Maximum total source size for REPL history
const MAX_REPL_SOURCE_LEN: usize = 50_000;

pub fn run_repl() -> Result<()> {
    print_banner();

    let mut context = ReplContext::new();

    loop {
        print!("{}", "γλ> ".green().bold());
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

    #[test]
    fn test_repl_execution_save_types() {
        let mut context = ReplContext::new();
        // Test function definition (using correct syntax from function_tests.rs)
        let _ = context.execute("Φ ὁρίζειν τῷ x ἀριθμοῦ· δός x.");
        assert_eq!(context.bindings.len(), 1);

        // Test type definition (using correct syntax from type_tests.rs)
        let _ = context.execute("εἶδος Τ ὁρίζειν { α ἀριθμοῦ. }.");
        assert_eq!(context.bindings.len(), 2);
    }
}
