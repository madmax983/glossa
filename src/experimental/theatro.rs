//! Theatro (Θέατρο) - Visual Assembly Debugger
//!
//! This module provides a "Rehearsal" mode that traces the semantic assembly process step-by-step.
//! It allows users to see how the `Assembler` routes words to grammatical slots in real-time.

#![allow(clippy::collapsible_if)]

use crate::errors::GlossaError;
use crate::parser::parse;
use crate::semantic::expressions::feed_expr_to_assembler_with_context;
use crate::semantic::{AssembledStatement, Assembler, DisambiguationContext};

/// A snapshot of the assembly state at a specific point in time
#[derive(Debug, Clone)]
pub struct Snapshot {
    /// The current state of the assembled statement
    pub statement: AssembledStatement,
    /// Description of the last action (e.g., "Fed 'λόγον' (Accusative) -> Object")
    pub description: String,
    /// The word that triggered this snapshot (if any)
    pub word: Option<String>,
}

/// A complete trace of the assembly process for a program
#[derive(Debug)]
pub struct Rehearsal {
    pub snapshots: Vec<Snapshot>,
}

/// Trace the assembly of a source string
pub fn rehearse(source: &str) -> Result<Rehearsal, GlossaError> {
    let ast = parse(source)?;
    let mut snapshots = Vec::new();

    for stmt in ast.statements {
        // Skip non-regular statements for now (Type defs, etc.)
        // But we should probably trace them if possible.
        // For now, focus on regular statements which use the Assembler.
        if let crate::ast::Statement::Regular { .. } = stmt {
            trace_statement(&stmt, &mut snapshots)?;
        }
    }

    Ok(Rehearsal { snapshots })
}

fn trace_statement(
    stmt: &crate::ast::Statement,
    snapshots: &mut Vec<Snapshot>,
) -> Result<(), GlossaError> {
    let mut asm = Assembler::new();
    asm.set_query(stmt.is_query());
    asm.set_propagate(stmt.is_propagate());

    let mut context = DisambiguationContext::new();

    // Initial snapshot (empty)
    snapshots.push(Snapshot {
        statement: asm.state().clone(),
        description: "Start of statement".to_string(),
        word: None,
    });

    for clause in stmt.clauses() {
        for expr in &clause.expressions {
            flatten_and_trace(expr, &mut asm, &mut context, snapshots)?;
        }
    }

    // Finalize (check agreement)
    match asm.finalize() {
        Ok(final_stmt) => {
            snapshots.push(Snapshot {
                statement: final_stmt,
                description: "Finalized (Agreement Checks Passed)".to_string(),
                word: None,
            });
        }
        Err(e) => {
            snapshots.push(Snapshot {
                statement: asm.state().clone(),
                description: format!("Error: {}", e),
                word: None,
            });
            // Don't fail the whole rehearsal, just log the error in the trace
        }
    }

    Ok(())
}

fn flatten_and_trace(
    expr: &crate::ast::Expr,
    asm: &mut Assembler,
    context: &mut DisambiguationContext,
    snapshots: &mut Vec<Snapshot>,
) -> Result<(), GlossaError> {
    // If it's a flat phrase (sequence of words), recurse to trace individual words
    if let crate::ast::Expr::Phrase(terms) = expr {
        for term in terms {
            // Check if this is a nested phrase (parenthesized)
            // If it is, we don't flatten it, but pass it as a unit to feed_expr
            if matches!(term, crate::ast::Expr::Phrase(_)) {
                trace_single_expr(term, asm, context, snapshots)?;
            } else {
                flatten_and_trace(term, asm, context, snapshots)?;
            }
        }
    } else {
        trace_single_expr(expr, asm, context, snapshots)?;
    }
    Ok(())
}

fn trace_single_expr(
    expr: &crate::ast::Expr,
    asm: &mut Assembler,
    context: &mut DisambiguationContext,
    snapshots: &mut Vec<Snapshot>,
) -> Result<(), GlossaError> {
    match feed_expr_to_assembler_with_context(asm, expr, context) {
        Ok(_) => {
            let description = describe_expr(expr);
            snapshots.push(Snapshot {
                statement: asm.state().clone(),
                description,
                word: extract_word(expr),
            });
            Ok(())
        }
        Err(e) => {
            snapshots.push(Snapshot {
                statement: asm.state().clone(),
                description: format!("Error: {}", e),
                word: extract_word(expr),
            });
            // Stop processing this statement on error
            // But we return Ok to allow caller to continue if they want (though we stopped here)
            // Actually, we should probably return the error or stop.
            // Let's stop.
            Ok(())
        }
    }
}

fn describe_expr(expr: &crate::ast::Expr) -> String {
    match expr {
        crate::ast::Expr::Word(w) => format!("Fed word '{}'", w.original),
        crate::ast::Expr::NumberLiteral(n) => format!("Fed number '{}'", n),
        crate::ast::Expr::StringLiteral(s) => format!("Fed string \"{}\"", s),
        crate::ast::Expr::BooleanLiteral(b) => format!("Fed boolean '{}'", b),
        crate::ast::Expr::Phrase(_) => "Fed phrase".to_string(),
        _ => "Fed complex expression".to_string(),
    }
}

fn extract_word(expr: &crate::ast::Expr) -> Option<String> {
    match expr {
        crate::ast::Expr::Word(w) => Some(w.original.to_string()),
        _ => None,
    }
}

#[cfg(feature = "nova")]
mod tui {
    use super::*;
    use crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::{
        prelude::*,
        widgets::{Block, Borders, Paragraph, Row, Table},
    };
    use std::io;

    pub fn run(rehearsal: Rehearsal) -> Result<(), io::Error> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // App State
        let mut index = 0;
        let max_index = rehearsal.snapshots.len().saturating_sub(1);

        loop {
            let snapshot = &rehearsal.snapshots[index];

            terminal.draw(|f| ui(f, snapshot, index, max_index))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if let Some(new_index) = handle_key(key.code, index, max_index) {
                            index = new_index;
                        } else {
                            // Quit signal
                            break;
                        }
                    }
                }
            }
        }

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    pub(super) fn handle_key(code: KeyCode, current: usize, max: usize) -> Option<usize> {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => None, // Signal to quit
            KeyCode::Right | KeyCode::Char('l') => {
                if current < max {
                    Some(current + 1)
                } else {
                    Some(current)
                }
            }
            KeyCode::Left | KeyCode::Char('h') => Some(current.saturating_sub(1)),
            _ => Some(current),
        }
    }

    fn ui(f: &mut Frame, snapshot: &Snapshot, index: usize, total: usize) {
        draw_ui(f, snapshot, index, total);
    }

    pub(super) fn draw_ui(f: &mut Frame, snapshot: &Snapshot, index: usize, total: usize) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(1),    // Main content
                Constraint::Length(3), // Footer
            ])
            .split(f.area());

        // Header
        let title = Paragraph::new(format!("Theatro - Step {}/{}", index + 1, total + 1))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Main Content (Split into Left: Description/History, Right: Slots)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[1]);

        // Left: Description
        let desc_text = format!(
            "Action:\n{}\n\nWord:\n{}",
            snapshot.description,
            snapshot.word.as_deref().unwrap_or("-")
        );
        let description = Paragraph::new(desc_text)
            .block(Block::default().title("Trace Log").borders(Borders::ALL));
        f.render_widget(description, main_chunks[0]);

        // Right: Slots Table
        let stmt = &snapshot.statement;
        let rows = vec![
            Row::new(vec![
                String::from("Subject (Nom)"),
                format_constituent(&stmt.subject),
            ]),
            Row::new(vec![String::from("Verb"), format_verb(&stmt.verb)]),
            Row::new(vec![
                String::from("Object (Acc)"),
                format_constituent(&stmt.object),
            ]),
            Row::new(vec![
                String::from("Indirect (Dat)"),
                format_constituent(&stmt.indirect),
            ]),
            Row::new(vec![
                String::from("Literals"),
                format!("{:?}", stmt.literals),
            ]),
        ];

        let table = Table::new(
            rows,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        )
        .block(
            Block::default()
                .title("Assembled Slots")
                .borders(Borders::ALL),
        )
        .header(
            Row::new(vec!["Slot", "Content"]).style(Style::default().add_modifier(Modifier::BOLD)),
        );

        f.render_widget(table, main_chunks[1]);

        // Footer
        let help =
            Paragraph::new("←/→: Navigate | Q: Quit").block(Block::default().borders(Borders::ALL));
        f.render_widget(help, chunks[2]);
    }

    fn format_constituent(c: &Option<crate::semantic::Constituent>) -> String {
        match c {
            Some(c) => format!("{} ({:?})", c.original, c.case),
            None => "-".to_string(),
        }
    }

    fn format_verb(c: &Option<crate::semantic::VerbConstituent>) -> String {
        match c {
            Some(c) => format!(
                "{} ({:?})",
                c.original,
                c.tense
                    .as_ref()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or("?".into())
            ),
            None => "-".to_string(),
        }
    }
}

#[cfg(feature = "nova")]
pub fn start_theatro(source: &str) -> Result<(), GlossaError> {
    let rehearsal = rehearse(source)?;
    tui::run(rehearsal).map_err(|e| GlossaError::semantic(format!("TUI Error: {}", e)))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rehearse_simple_svo() {
        // Use τιμήν (value) to avoid participle ambiguity with λόγον
        let source = "ὁ ἄνθρωπος τὴν τιμήν λέγει.";
        let rehearsal = rehearse(source).expect("Failed to rehearse");

        assert!(!rehearsal.snapshots.is_empty());

        for (i, snap) in rehearsal.snapshots.iter().enumerate() {
            println!("{}: {} -> {:?}", i, snap.description, snap.statement);
        }

        let last = rehearsal.snapshots.last().unwrap();
        assert_eq!(last.description, "Finalized (Agreement Checks Passed)");
        assert!(last.statement.subject.is_some());
        assert!(last.statement.object.is_some());
        assert!(last.statement.verb.is_some());
    }

    #[test]
    fn test_rehearse_literals() {
        let source = "42 λέγε.\n«χαῖρε» λέγε.\nἀληθές λέγε.";
        let rehearsal = rehearse(source).expect("Failed to rehearse");

        let has_number = rehearsal.snapshots.iter().any(|s| s.description.contains("Fed number '42'"));
        assert!(has_number, "Should trace number literal");

        let has_string = rehearsal.snapshots.iter().any(|s| s.description.contains("Fed string \"χαῖρε\""));
        assert!(has_string, "Should trace string literal");

        let has_bool = rehearsal.snapshots.iter().any(|s| s.description.contains("Fed boolean 'true'"));
        assert!(has_bool, "Should trace boolean literal");
    }

    #[test]
    fn test_rehearse_nested_phrases() {
        // (1) λέγε. -> Nested phrase
        let source = "(1) λέγε.";
        let rehearsal = rehearse(source).expect("Failed to rehearse");

        // Should trace the nested expression "1"
        let has_nested = rehearsal.snapshots.iter().any(|s| s.description.contains("Fed number '1'"));
        assert!(has_nested, "Should trace nested expression");
    }

    #[test]
    fn test_rehearse_error() {
        // Invalid syntax: Double verb "λέγει γράφει."
        let source = "λέγει γράφει.";
        let rehearsal = rehearse(source).expect("Rehearsal should not panic on semantic error");

        // Should contain an error snapshot
        let has_error = rehearsal.snapshots.iter().any(|s| s.description.contains("Error:"));
        assert!(has_error, "Should capture semantic error in trace");
    }

    #[test]
    #[cfg(feature = "nova")]
    fn test_tui_rendering() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        use crate::experimental::theatro::tui::draw_ui;

        // Create a dummy snapshot
        let stmt = AssembledStatement {
            subject: Some(crate::semantic::Constituent {
                lemma: "Subject".into(),
                original: "Subject".into(),
                case: crate::morphology::Case::Nominative,
                number: None,
                gender: None,
                person: None,
            }),
            verb: Some(crate::semantic::VerbConstituent {
                lemma: "Verb".into(),
                original: "Verb".into(),
                person: None,
                number: None,
                tense: None,
                mood: None,
                voice: None,
            }),
            ..Default::default()
        };

        let snapshot = Snapshot {
            statement: stmt,
            description: "Test Description".to_string(),
            word: Some("TestWord".to_string()),
        };

        // Setup test terminal
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();

        // Render UI
        terminal.draw(|f| {
            draw_ui(f, &snapshot, 0, 10);
        }).unwrap();

        // Verify content
        let buffer = terminal.backend().buffer();

        // Convert buffer to string for checking
        let mut content = String::new();
        for y in 0..20 {
            for x in 0..100 {
                content.push(buffer[(x, y)].symbol().chars().next().unwrap_or(' '));
            }
            content.push('\n');
        }

        assert!(content.contains("Theatro - Step 1/11"), "Header missing");
        assert!(content.contains("Test Description"), "Description missing");
        assert!(content.contains("TestWord"), "Trigger word missing");
        assert!(content.contains("Subject (Nom)"), "Subject label missing");
        assert!(content.contains("Verb"), "Verb label missing");
    }

    #[test]
    #[cfg(feature = "nova")]
    fn test_tui_key_handling() {
        use crossterm::event::KeyCode;
        use crate::experimental::theatro::tui::handle_key;

        // Test navigation
        assert_eq!(handle_key(KeyCode::Right, 0, 10), Some(1));
        assert_eq!(handle_key(KeyCode::Char('l'), 0, 10), Some(1));
        assert_eq!(handle_key(KeyCode::Left, 1, 10), Some(0));
        assert_eq!(handle_key(KeyCode::Char('h'), 1, 10), Some(0));

        // Test boundaries
        assert_eq!(handle_key(KeyCode::Right, 10, 10), Some(10)); // Max
        assert_eq!(handle_key(KeyCode::Left, 0, 10), Some(0)); // Min

        // Test quit
        assert_eq!(handle_key(KeyCode::Char('q'), 5, 10), None);
        assert_eq!(handle_key(KeyCode::Esc, 5, 10), None);

        // Test ignore
        assert_eq!(handle_key(KeyCode::Char('a'), 5, 10), Some(5));
    }
}
