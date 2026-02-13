//! REPL implementation for ΓΛΩΣΣΑ
//!
//! Provides an interactive Read-Eval-Print Loop for executing ΓΛΩΣΣΑ statements.

use comfy_table::{Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::io::{BufRead, Write};

use crate::codegen::generate_statement_code;
use crate::errors::GlossaError;
use crate::parser::parse;
use crate::semantic::{AnalyzedStatement, GlossaType, Scope, analyze_program};

/// Maximum number of bindings to track in REPL history
const MAX_REPL_BINDINGS: usize = 50;
/// Maximum total source size for REPL history
const MAX_REPL_SOURCE_LEN: usize = 50_000;

/// Run the REPL loop using stdin and stdout
pub fn run_repl() -> Result<()> {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut stdout = std::io::stdout();

    print_banner(&mut stdout)?;
    run_repl_inner(&mut handle, &mut stdout)
}

/// Internal REPL loop logic, decoupled from I/O for testing
fn run_repl_inner<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> Result<()> {
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

        let trimmed = input.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Handle special commands
        match trimmed {
            ".ἔξοδος" | ".exit" | ".quit" => {
                writeln!(writer, "Χαῖρε!").into_diagnostic()?;
                break;
            }
            ".βοήθεια" | ".help" => {
                print_help(writer)?;
                continue;
            }
            ".καθαρός" | ".clear" => {
                context = ReplContext::new();
                writeln!(writer, "{} Ἐκαθαρίσθη.", "✓".green()).into_diagnostic()?;
                continue;
            }
            ".περιβάλλον" | ".env" => {
                print_env(writer, &context)?;
                continue;
            }
            _ => {}
        }

        match context.execute(trimmed) {
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

fn print_banner<W: Write>(w: &mut W) -> Result<()> {
    // Welcome Banner
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    writeln!(w).into_diagnostic()?;
    writeln!(w, "   {}", "Γ Λ Ω Σ Σ Α".bold().cyan()).into_diagnostic()?;
    writeln!(w, "   {}", "Code as the ancients intended.".italic().dim()).into_diagnostic()?;
    writeln!(w, "   {}", version.blue()).into_diagnostic()?;
    writeln!(w).into_diagnostic()?;
    writeln!(w, "   {}", "Type .help for commands".dim()).into_diagnostic()?;
    writeln!(w).into_diagnostic()?;
    Ok(())
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

fn print_env<W: Write>(w: &mut W, context: &ReplContext) -> Result<()> {
    if let Some(scope) = &context.last_scope {
        // Collect and sort bindings for consistent display
        let mut bindings: Vec<_> = scope.bindings().collect();
        bindings.sort_by(|a, b| a.0.cmp(b.0));

        if bindings.is_empty() {
            writeln!(
                w,
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
        writeln!(w, "{table}").into_diagnostic()?;
    } else {
        writeln!(
            w,
            "{}",
            "Οὐδεμία μεταβλητή (No variables defined).".yellow()
        )
        .into_diagnostic()?;
    }
    Ok(())
}

/// REPL Output variants
#[derive(Debug)]
enum ReplOutput {
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
        let mut buf = Vec::new();

        // 1. Initial state: No scope
        assert!(context.last_scope.is_none());
        print_env(&mut buf, &context).unwrap(); // Should print "No variables"
        assert!(String::from_utf8(buf.clone()).unwrap().contains("Οὐδεμία"));
        buf.clear();

        // 2. Execute binding
        let _ = context.execute("ξ πέντε ἔστω.").unwrap();

        // 3. Verify scope captured
        assert!(context.last_scope.is_some());
        let scope = context.last_scope.as_ref().unwrap();
        assert!(scope.lookup("ξ").is_some());

        // 4. Add a mutable binding to cover that branch
        let _ = context.execute("μετά ψ δέκα ἔστω.").unwrap();

        // 5. Run print_env to cover the table generation code
        print_env(&mut buf, &context).unwrap();
        let output = String::from_utf8(buf.clone()).unwrap();
        assert!(output.contains("ξ"));
        assert!(output.contains("ψ"));
        buf.clear();

        // 6. Test clear simulation (new context)
        let context = ReplContext::new();
        assert!(context.last_scope.is_none());
        print_env(&mut buf, &context).unwrap();
        assert!(String::from_utf8(buf).unwrap().contains("Οὐδεμία"));
    }

    #[test]
    fn test_print_help_coverage() {
        let mut buf = Vec::new();
        print_help(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("βοήθεια"));
        assert!(output.contains("ἔξοδος"));
    }

    #[test]
    fn test_print_banner_coverage() {
        let mut buf = Vec::new();
        print_banner(&mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("Γ Λ Ω Σ Σ Α"));
    }

    #[test]
    fn test_print_env_empty() {
        let context = ReplContext::new();
        let mut buf = Vec::new();
        print_env(&mut buf, &context).unwrap();
        assert!(String::from_utf8(buf).unwrap().contains("Οὐδεμία"));
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
    fn test_repl_session() {
        // Simulate a full REPL session
        let input_commands = [
            ".help",
            "ξ πέντε ἔστω.",
            "ξ λέγε.",
            ".env",
            ".clear",
            ".env",
            ".exit",
        ];
        let input_str = input_commands.join("\n");
        let mut input = std::io::Cursor::new(input_str);
        let mut output = Vec::new();

        run_repl_inner(&mut input, &mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();

        // Verify key interactions
        assert!(output_str.contains("γλ>"), "Should prompt");
        assert!(output_str.contains("βοήθεια"), "Should show help");
        assert!(output_str.contains("ξ"), "Should echo binding name");
        assert!(output_str.contains("println"), "Should show generated code");
        assert!(output_str.contains("Ἐκαθαρίσθη"), "Should confirm clear");
        assert!(output_str.contains("Χαῖρε"), "Should confirm exit");

        // Count occurrences of prompt to ensure loop ran correct number of times
        // 7 commands + 1 final prompt (maybe, depending on implementation detail of break)
        // Actually, break happens inside loop, so prompt is printed before read.
        // Commands:
        // 1. .help -> continue -> prompt
        // 2. binding -> execute -> prompt
        // 3. statement -> execute -> prompt
        // 4. .env -> continue -> prompt
        // 5. .clear -> continue -> prompt
        // 6. .env -> continue -> prompt
        // 7. .exit -> break
        // So prompt should appear 7 times.
        let prompt_count = output_str.matches("γλ>").count();
        assert_eq!(prompt_count, 7, "Prompt count mismatch");
    }

    #[test]
    fn test_repl_error_handling() {
        // Simulate an error
        let input_str = "κακή εντολή\n.exit";
        let mut input = std::io::Cursor::new(input_str);
        let mut output = Vec::new();

        run_repl_inner(&mut input, &mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Σφάλμα"), "Should report error");
    }

    struct BrokenReader;
    impl std::io::Read for BrokenReader {
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Broken"))
        }
    }
    impl std::io::BufRead for BrokenReader {
        fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Broken"))
        }
        fn consume(&mut self, _amt: usize) {}
    }

    #[test]
    fn test_read_error() {
        let mut reader = BrokenReader;
        let mut output = Vec::new();
        let result = run_repl_inner(&mut reader, &mut output);
        assert!(result.is_err());
    }

    struct BrokenWriter;
    impl std::io::Write for BrokenWriter {
        fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Broken"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Err(std::io::Error::new(std::io::ErrorKind::Other, "Broken"))
        }
    }

    #[test]
    fn test_flush_error() {
        let input = b".exit";
        let mut reader = std::io::Cursor::new(input);
        let mut writer = BrokenWriter;
        let result = run_repl_inner(&mut reader, &mut writer);
        assert!(result.is_err());
    }

    #[test]
    fn test_repl_none_output() {
        let none = ReplOutput::None;
        let s = none.to_string();
        assert_eq!(s, "");
    }
}
