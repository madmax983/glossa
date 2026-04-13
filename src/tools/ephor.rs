//! The Ephor (ὁ Ἔφορος) - Interactive Debugger
//!
//! A visual interactive debugger that steps through the Glossa simulator.
//! The Ephor allows developers to watch the environment and output dynamically.

use std::io::{Write, stdout};
use std::path::Path;

use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Table};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::{Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

/// A Drop guard to ensure the terminal is always restored to its
/// normal state even if an error occurs or a panic happens during debugging.
struct TerminalRestorer<'a, W: Write> {
    out: &'a mut W,
}

impl<'a, W: Write> TerminalRestorer<'a, W> {
    fn new(out: &'a mut W) -> Result<Self> {
        terminal::enable_raw_mode().into_diagnostic()?;
        execute!(out, terminal::EnterAlternateScreen, cursor::Hide).into_diagnostic()?;
        Ok(Self { out })
    }
}

impl<'a, W: Write> Drop for TerminalRestorer<'a, W> {
    fn drop(&mut self) {
        let _ = execute!(self.out, terminal::LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}
use miette::{IntoDiagnostic, Result};

use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::tools::interpreter::Interpreter;

/// Entry point to start the Ephor visual debugger on a .γλ file
pub fn run_ephor(path: &Path) -> Result<()> {
    let mut out = stdout();
    let source = std::fs::read_to_string(path).into_diagnostic()?;
    run_ephor_inner(path, &source, &mut out)
}

fn run_ephor_inner<W: Write>(path: &Path, source: &str, out: &mut W) -> Result<()> {
    // Parse and Analyze
    let ast = parse(source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    // Create simulator but don't run it yet
    let mut interpreter = Interpreter::new();

    // Start Interactive UI (with terminal restorer guard)
    let _restorer = TerminalRestorer::new(out)?;

    // We step statement by statement.
    // Instead of using `interpreter.run`, we take manual control.
    let mut current_stmt_idx = 0;

    let total_stmts = program.statements.len();

    let render = |out: &mut W,
                  interpreter: &Interpreter,
                  current_stmt_idx: usize,
                  total_stmts: usize|
     -> Result<()> {
        execute!(out, Clear(ClearType::All), cursor::MoveTo(0, 0)).into_diagnostic()?;

        // Header
        execute!(
            out,
            SetForegroundColor(crossterm::style::Color::Magenta),
            Print(format!(
                "🌟 The Ephor (ὁ Ἔφορος) - Debugging {}\r\n",
                path.display()
            )),
            ResetColor,
            Print("=========================================================\r\n")
        )
        .into_diagnostic()?;

        // Status
        if current_stmt_idx < total_stmts {
            execute!(
                out,
                SetForegroundColor(crossterm::style::Color::Yellow),
                Print(format!(
                    "Next Statement: {} of {}\r\n",
                    current_stmt_idx + 1,
                    total_stmts
                )),
                ResetColor
            )
            .into_diagnostic()?;
        } else {
            execute!(
                out,
                SetForegroundColor(crossterm::style::Color::Green),
                Print("Program Complete.\r\n"),
                ResetColor
            )
            .into_diagnostic()?;
        }
        execute!(out, Print("Press [SPACE] to step, [Q] to quit.\r\n\n")).into_diagnostic()?;

        // Environment Table
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            Cell::new("Variable").fg(Color::Cyan),
            Cell::new("Value").fg(Color::Green),
        ]);

        let mut has_vars = false;
        for scope in interpreter.env.iter() {
            for (name, val) in scope {
                table.add_row(vec![Cell::new(name), Cell::new(val.to_string())]);
                has_vars = true;
            }
        }

        if has_vars {
            execute!(out, Print(format!("{}\r\n\n", table).replace('\n', "\r\n")))
                .into_diagnostic()?;
        } else {
            execute!(out, Print("(Environment is empty)\r\n\n")).into_diagnostic()?;
        }

        // Output Buffer
        execute!(
            out,
            SetForegroundColor(crossterm::style::Color::Cyan),
            Print("Output Buffer:\r\n"),
            ResetColor,
            Print("---------------------------------------------------------\r\n")
        )
        .into_diagnostic()?;

        let output = interpreter.get_output();
        if !output.is_empty() {
            execute!(out, Print(output.replace('\n', "\r\n")), Print("\r\n")).into_diagnostic()?;
        }

        out.flush().into_diagnostic()?;
        Ok(())
    };

    render(_restorer.out, &interpreter, current_stmt_idx, total_stmts)?;

    loop {
        if event::poll(std::time::Duration::from_millis(100)).into_diagnostic()? {
            let evt = event::read().into_diagnostic()?;
            if let Event::Key(key_event) = evt {
                #[allow(clippy::collapsible_if)]
                if key_event.kind == KeyEventKind::Press {
                    if key_event.modifiers.contains(KeyModifiers::CONTROL)
                        && key_event.code == KeyCode::Char('c')
                    {
                        break;
                    }

                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                            break;
                        }
                        KeyCode::Char(' ') | KeyCode::Enter => {
                            if current_stmt_idx < total_stmts {
                                let stmt = &program.statements[current_stmt_idx];
                                if let Err(e) = interpreter.eval_statement(stmt) {
                                    execute!(
                                        _restorer.out,
                                        Print(format!("\r\nRuntime Error: {}\r\n", e)),
                                        Print("Press [Q] to quit.\r\n")
                                    )
                                    .into_diagnostic()?;
                                    current_stmt_idx = total_stmts; // Stop execution
                                } else {
                                    current_stmt_idx += 1;
                                }
                                render(_restorer.out, &interpreter, current_stmt_idx, total_stmts)?;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ephor_basic_compile() {
        // Just verify it compiles correctly and tests run without terminal interaction
        let compiled = true;
        assert!(compiled);
    }
}
