//! The Arena (ἡ Ἀρένα) - TUI AST Explorer
//!
//! This module implements a Terminal User Interface (TUI) to interactively
//! explore the compiled structure of a ΓΛΩΣΣΑ program.

use miette::{IntoDiagnostic, Result};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{self, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use std::io::stdout;
use std::path::Path;

use crate::codegen::generate_rust_file;
use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::tools::runner::load_source;

/// Run the Arena tool on a file
pub fn run_arena(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let rust_code;
    let mut mosaic = String::new();

    // Attempt to parse and analyze
    match parse(&source) {
        Ok(ast) => match analyze_program(&ast) {
            Ok(program) => {
                rust_code = generate_rust_file(&program);
                let mut mosaic_buffer = Vec::new();
                if crate::tools::mosaic::run_mosaic_inner(&source, &mut mosaic_buffer).is_ok() {
                    mosaic = String::from_utf8_lossy(&mosaic_buffer).into_owned();
                } else {
                    mosaic = "Failed to generate mosaic".to_string();
                }
            }
            Err(e) => {
                rust_code = format!("Semantic Error:\n{}", e);
            }
        },
        Err(e) => {
            rust_code = format!("Parse Error:\n{}", e);
        }
    }

    // Setup terminal
    enable_raw_mode().into_diagnostic()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).into_diagnostic()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).into_diagnostic()?;

    let res = run_app(&mut terminal, &source, &rust_code, &mosaic);

    // Restore terminal
    disable_raw_mode().into_diagnostic()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).into_diagnostic()?;
    terminal.show_cursor().into_diagnostic()?;

    res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    source: &str,
    rust_code: &str,
    mosaic: &str,
) -> Result<()> {
    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(f.area());

                let right_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(chunks[1]);

                let source_block = Paragraph::new(source)
                    .block(
                        Block::default()
                            .title("ΓΛΩΣΣΑ (Source - Press 'q' to quit)")
                            .borders(Borders::ALL),
                    )
                    .wrap(Wrap { trim: false });
                f.render_widget(source_block, chunks[0]);

                let rust_block = Paragraph::new(rust_code)
                    .block(
                        Block::default()
                            .title("Rust (Generated)")
                            .borders(Borders::ALL),
                    )
                    .wrap(Wrap { trim: false });
                f.render_widget(rust_block, right_chunks[0]);

                let mosaic_block = Paragraph::new(mosaic)
                    .block(
                        Block::default()
                            .title("Mosaic (Semantic)")
                            .borders(Borders::ALL),
                    )
                    .wrap(Wrap { trim: false });
                f.render_widget(mosaic_block, right_chunks[1]);
            })
            .into_diagnostic()?;

        let event = event::read().into_diagnostic()?;
        match event {
            Event::Key(key) if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc => {
                return Ok(());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run_app;

    #[test]
    fn test_arena_scaffold() {
        // Just verify imports work
        let _run = run_app as *const ();
    }
}
