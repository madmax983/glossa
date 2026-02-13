//! Interactive Read-Eval-Print Loop (REPL)

use crate::codegen::generate_statement_code;
use crate::errors::GlossaError;
use crate::parser::parse;
use crate::semantic::{AnalyzedStatement, GlossaType, Scope, analyze_program};
use comfy_table::{Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::io::{BufRead, Write};

/// Maximum number of bindings to track in REPL history
pub const MAX_REPL_BINDINGS: usize = 50;
/// Maximum total source size for REPL history
pub const MAX_REPL_SOURCE_LEN: usize = 50_000;

pub fn run_repl() -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    run_repl_inner(stdin.lock(), &mut stdout)
}

/// Inner REPL loop logic for testing
pub fn run_repl_inner<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> Result<()> {
    print_banner_to(&mut writer)?;

    let mut context = ReplContext::new();

    loop {
        write!(writer, "{}", "γλ> ".green().bold()).into_diagnostic()?;
        writer.flush().into_diagnostic()?;

        let mut input = String::new();
        let bytes = reader.read_line(&mut input).into_diagnostic()?;

        // Fix: Handle EOF (Ctrl+D) gracefully
        if bytes == 0 {
            writeln!(writer, "\nΧαῖρε!").into_diagnostic()?;
            break;
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        // Handle special commands
        match input {
            ".ἔξοδος" | ".exit" | ".quit" => {
                writeln!(writer, "Χαῖρε!").into_diagnostic()?;
                break;
            }
            ".βοήθεια" | ".help" => {
                print_help_to(&mut writer)?;
                continue;
            }
            ".καθαρός" | ".clear" => {
                context = ReplContext::new();
                writeln!(writer, "{} Ἐκαθαρίσθη.", "✓".green()).into_diagnostic()?;
                continue;
            }
            ".περιβάλλον" | ".env" => {
                print_env_to(&context, &mut writer)?;
                continue;
            }
            _ => {}
        }

        match context.execute(input) {
            Ok(output) => {
                // Display handles formatting (empty if None)
                write!(writer, "{}", output).into_diagnostic()?;
            }
            Err(e) => {
                // Use default error formatting but ensure it's visible
                writeln!(writer, "{}", format!("× Σφάλμα: {}", e).red()).into_diagnostic()?;
            }
        }
    }

    Ok(())
}

fn print_banner_to<W: Write>(writer: &mut W) -> Result<()> {
    // Welcome Banner
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    writeln!(writer).into_diagnostic()?;
    writeln!(writer, "   {}", "Γ Λ Ω Σ Σ Α".bold().cyan()).into_diagnostic()?;
    writeln!(
        writer,
        "   {}",
        "Code as the ancients intended.".italic().dim()
    )
    .into_diagnostic()?;
    writeln!(writer, "   {}", version.blue()).into_diagnostic()?;
    writeln!(writer).into_diagnostic()?;
    writeln!(writer, "   {}", "Type .help for commands".dim()).into_diagnostic()?;
    writeln!(writer).into_diagnostic()?;
    Ok(())
}

pub fn print_banner() {
    let mut stdout = std::io::stdout();
    let _ = print_banner_to(&mut stdout);
}

fn print_help_to<W: Write>(writer: &mut W) -> Result<()> {
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

    writeln!(writer, "{table}").into_diagnostic()?;

    writeln!(writer, "\n{}", "Παραδείγματα:".bold().underlined()).into_diagnostic()?;
    writeln!(writer, "  «χαῖρε κόσμε» λέγε.").into_diagnostic()?;
    writeln!(writer, "  ξ πέντε ἔστω.").into_diagnostic()?;
    Ok(())
}

pub fn print_help() {
    let mut stdout = std::io::stdout();
    let _ = print_help_to(&mut stdout);
}

fn print_env_to<W: Write>(context: &ReplContext, writer: &mut W) -> Result<()> {
    if let Some(scope) = &context.last_scope {
        // Collect and sort bindings for consistent display
        let mut bindings: Vec<_> = scope.bindings().collect();
        bindings.sort_by(|a, b| a.0.cmp(b.0));

        if bindings.is_empty() {
            writeln!(
                writer,
                "{}",
                "Οὐδεμία μεταβλητή (No variables defined).".yellow()
            )
            .into_diagnostic()?;
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
        writeln!(writer, "{table}").into_diagnostic()?;
    } else {
        writeln!(
            writer,
            "{}",
            "Οὐδεμία μεταβλητή (No variables defined).".yellow()
        )
        .into_diagnostic()?;
    }
    Ok(())
}

pub fn print_env(context: &ReplContext) {
    let mut stdout = std::io::stdout();
    let _ = print_env_to(context, &mut stdout);
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

pub struct ReplContext {
    bindings: Vec<String>,
    pub last_scope: Option<Scope>,
    pub statement_count: usize,
}

impl Default for ReplContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplContext {
    pub fn new() -> Self {
        ReplContext {
            bindings: Vec::new(),
            last_scope: None,
            statement_count: 0,
        }
    }

    pub fn execute(&mut self, input: &str) -> std::result::Result<ReplOutput, GlossaError> {
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
    use std::io::Cursor;

    #[test]
    fn test_repl_inner_loop() {
        // Simulate input: binding, usage, clear, exit
        let input = "ξ πέντε ἔστω.\nξ λέγε.\n.clear\n.exit\n";
        let mut output = Vec::new();
        let reader = Cursor::new(input);

        let result = run_repl_inner(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("ξ"));
        assert!(output_str.contains("Ἐκτελέσθη"));
        assert!(output_str.contains("Ἐκαθαρίσθη"));
        assert!(output_str.contains("Χαῖρε!"));
    }

    #[test]
    fn test_repl_help_and_env() {
        let input = ".help\n.env\n.exit\n";
        let mut output = Vec::new();
        let reader = Cursor::new(input);

        let result = run_repl_inner(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Ἐντολή")); // Help header
        assert!(output_str.contains("Οὐδεμία μεταβλητή")); // Empty env
    }

    #[test]
    fn test_repl_empty_line_and_invalid_command() {
        // Empty line should be ignored, unknown command handled gracefully (as syntax error usually)
        // But "." commands are special. If not matched, they are treated as syntax?
        // No, current logic:
        // match input { ... _ => {} }
        // then execute(input)
        // So ".unknown" is sent to parser, which will likely fail.
        let input = "\n.unknown\n.exit\n";
        let mut output = Vec::new();
        let reader = Cursor::new(input);

        let result = run_repl_inner(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Σφάλμα")); // Error from parser on ".unknown"
    }

    #[test]
    fn test_repl_eof() {
        let input = ""; // Immediate EOF
        let mut output = Vec::new();
        let reader = Cursor::new(input);

        let result = run_repl_inner(reader, &mut output);
        assert!(result.is_ok());

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Χαῖρε!"));
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
        print_env(&context);

        // 6. Test clear simulation (new context)
        let context = ReplContext::new();
        assert!(context.last_scope.is_none());
        print_env(&context);
    }

    #[test]
    fn test_print_help_coverage() {
        print_help();
    }

    #[test]
    fn test_print_banner_coverage() {
        print_banner();
    }

    #[test]
    fn test_print_env_empty() {
        let context = ReplContext::new();
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
