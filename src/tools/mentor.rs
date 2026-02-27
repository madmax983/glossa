//! The Mentor (ὁ Μέντωρ) - Interactive Tutorial Mode
//!
//! This module implements the "Mentor" tool, which guides users through learning
//! the ΓΛΩΣΣΑ language with interactive lessons and challenges.
//!
//! # The Mentor Philosophy
//!
//! Learning a new language (especially one based on Ancient Greek) can be daunting.
//! The Mentor provides a safe, guided environment where users can practice concepts
//! one by one.
//!
//! Each lesson consists of:
//! 1. **The Concept**: A brief explanation of a language feature.
//! 2. **The Challenge**: A specific coding task to perform.
//! 3. **The Verification**: Real-time analysis of the user's code to ensure they met the goal.

use crate::errors::GlossaError;
use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, AnalyzedStatement, GlossaType, analyze_program};
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::{IntoDiagnostic, Result};
use std::io::{BufRead, Write};

/// A single lesson in the tutorial
pub struct Lesson {
    pub title: &'static str,
    pub description: &'static str,
    pub challenge: &'static str,
    pub verify: fn(&AnalyzedProgram) -> bool,
}

const LESSONS: &[Lesson] = &[
    Lesson {
        title: "1. The Beginning (Ἡ Ἀρχή)",
        description: "In ΓΛΩΣΣΑ, variables are declared using the verb 'ἔστω' (let it be). \
                      Variables are typed, but the type is inferred from the value.",
        challenge: "Create a variable named 'x' (ξ) with the value 10.",
        verify: |program| {
            // Check if 'ξ' or 'x' is defined as a Number
            program
                .scope
                .lookup("ξ")
                .map(|t| *t == GlossaType::Number)
                .unwrap_or(false)
                || program
                    .scope
                    .lookup("x")
                    .map(|t| *t == GlossaType::Number)
                    .unwrap_or(false)
        },
    },
    Lesson {
        title: "2. The Proclamation (Ἡ Ῥῆσις)",
        description: "To output text to the console, use the imperative verb 'λέγε' (say).",
        challenge: "Print the string \"Hello World\" (or «Χαῖρε Κόσμε»).",
        verify: |program| {
            // Check for a Print statement with a String literal
            program.statements.iter().any(|stmt| {
                if let AnalyzedStatement::Print(exprs) = stmt {
                    exprs
                        .iter()
                        .any(|e| matches!(e.glossa_type, GlossaType::String))
                } else {
                    false
                }
            })
        },
    },
    Lesson {
        title: "3. The Logic (Ἡ Λογική)",
        description: "Conditional logic uses 'εἰ' (if). The condition must be boolean.",
        challenge: "Write an if statement that checks if true is true.",
        verify: |program| {
            program
                .statements
                .iter()
                .any(|stmt| matches!(stmt, AnalyzedStatement::If { .. }))
        },
    },
];

/// Start the interactive Mentor session
///
/// This function enters a loop where it presents lessons to the user and waits
/// for them to type code that satisfies the lesson's requirements.
pub fn run_mentor() -> Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    run_mentor_inner(&mut stdin.lock(), &mut stdout)
}

fn run_mentor_inner<R: BufRead, W: Write>(input: &mut R, output: &mut W) -> Result<()> {
    print_banner(output)?;

    for (i, lesson) in LESSONS.iter().enumerate() {
        print_lesson(output, i + 1, lesson)?;

        loop {
            // Prompt
            write!(output, "{}", "mentor> ".green().bold()).into_diagnostic()?;
            output.flush().into_diagnostic()?;

            let mut line = String::new();
            let bytes = input.read_line(&mut line).into_diagnostic()?;

            if bytes == 0 {
                return Ok(()); // EOF
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed == ".exit" || trimmed == ".quit" {
                return Ok(());
            }

            if trimmed == ".skip" {
                writeln!(output, "{}", "Skipping lesson...".yellow()).into_diagnostic()?;
                break;
            }

            // Attempt to parse and analyze
            match process_submission(trimmed) {
                Ok(program) => {
                    if (lesson.verify)(&program) {
                        writeln!(output, "\n{}", "✓ Correct! Well done.".green().bold())
                            .into_diagnostic()?;
                        writeln!(output, "{}", "Press Enter to continue...".dim())
                            .into_diagnostic()?;
                        let _ = input.read_line(&mut String::new());
                        break; // Next lesson
                    } else {
                        writeln!(
                            output,
                            "{}",
                            "✓ Syntax is valid, but the goal was not met.".yellow()
                        )
                        .into_diagnostic()?;
                        writeln!(output, "  Goal: {}", lesson.challenge).into_diagnostic()?;
                    }
                }
                Err(e) => {
                    let mut buf = String::new();
                    let _ = miette::GraphicalReportHandler::new().render_report(&mut buf, &e);
                    writeln!(output, "{}", buf).into_diagnostic()?;
                }
            }
        }
    }

    writeln!(
        output,
        "\n{}",
        "🎓 Graduation! You have completed the basic tutorial."
            .green()
            .bold()
            .underlined()
    )
    .into_diagnostic()?;
    Ok(())
}

fn process_submission(source: &str) -> Result<AnalyzedProgram, GlossaError> {
    let ast = parse(source)?;
    analyze_program(&ast)
}

fn print_banner<W: Write>(w: &mut W) -> Result<()> {
    writeln!(w).into_diagnostic()?;
    writeln!(w, "   {}", "Γ Λ Ω Σ Σ Α   M E N T O R".bold().cyan()).into_diagnostic()?;
    writeln!(w, "   {}", "Interactive Tutorial Mode".italic().dim()).into_diagnostic()?;
    writeln!(w).into_diagnostic()?;
    Ok(())
}

fn print_lesson<W: Write>(w: &mut W, index: usize, lesson: &Lesson) -> Result<()> {
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL).set_header(vec![
        Cell::new(format!("Lesson {}", index))
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new(lesson.title).add_attribute(Attribute::Bold),
    ]);

    table.add_row(vec![
        Cell::new("Description").add_attribute(Attribute::Bold),
        Cell::new(lesson.description),
    ]);

    table.add_row(vec![
        Cell::new("Challenge")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
        Cell::new(lesson.challenge).fg(Color::Yellow),
    ]);

    writeln!(w, "{}", table).into_diagnostic()?;
    writeln!(
        w,
        "\nType your code below (or .skip to skip, .exit to quit):"
    )
    .into_diagnostic()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lesson_1_verification() {
        let lesson = &LESSONS[0]; // Variable
        // Correct
        let p1 = process_submission("ξ 10 ἔστω.").unwrap();
        assert!((lesson.verify)(&p1));

        // Incorrect (wrong name)
        let p2 = process_submission("ψ 10 ἔστω.").unwrap();
        assert!(!((lesson.verify)(&p2)));

        // Incorrect (wrong type - String)
        let p3 = process_submission("ξ «10» ἔστω.").unwrap();
        assert!(!((lesson.verify)(&p3)));
    }

    #[test]
    fn test_lesson_2_verification() {
        let lesson = &LESSONS[1]; // Print
        // Correct
        let p1 = process_submission("«Hello» λέγε.").unwrap();
        assert!((lesson.verify)(&p1));

        // Incorrect (no print)
        let p2 = process_submission("ξ 10 ἔστω.").unwrap();
        assert!(!((lesson.verify)(&p2)));
    }

    #[test]
    fn test_run_mentor_inner_flow() {
        // Simulate completing first lesson
        let input_data = "ξ 10 ἔστω.\n\n.exit\n"; // Code -> Enter (continue) -> Exit
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        run_mentor_inner(&mut input, &mut output).unwrap();

        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("The Beginning"));
        assert!(s.contains("Correct!"));
    }

    #[test]
    fn test_lesson_3_verification() {
        let lesson = &LESSONS[2]; // If
        // Correct
        let p1 = process_submission("εἰ ἀληθές, «ναί» λέγε.").unwrap();
        assert!((lesson.verify)(&p1));

        // Incorrect (no if)
        let p2 = process_submission("«ναί» λέγε.").unwrap();
        assert!(!((lesson.verify)(&p2)));
    }

    #[test]
    fn test_mentor_skip_command() {
        let input_data = ".skip\n.exit\n";
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        run_mentor_inner(&mut input, &mut output).unwrap();

        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Skipping lesson"));
    }

    #[test]
    fn test_mentor_quit_command() {
        let input_data = ".quit\n";
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        run_mentor_inner(&mut input, &mut output).unwrap();

        // Should return without error
    }

    #[test]
    fn test_mentor_eof() {
        let input_data = ""; // Immediate EOF
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        run_mentor_inner(&mut input, &mut output).unwrap();
        // Just finishes loop
    }

    #[test]
    fn test_mentor_syntax_error() {
        let input_data = "invalid syntax\n.exit\n";
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        run_mentor_inner(&mut input, &mut output).unwrap();

        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Error"), "Expected error message in output, got:\n{}", s);
    }

    #[test]
    fn test_mentor_semantic_failure() {
        let input_data = "ψ 10 ἔστω.\n.exit\n"; // Valid syntax, but wrong variable name (for lesson 1)
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        run_mentor_inner(&mut input, &mut output).unwrap();

        let s = String::from_utf8(output).unwrap();
        assert!(s.contains("Syntax is valid, but the goal was not met"));
    }

    #[test]
    fn test_mentor_empty_input() {
        let input_data = "\n.exit\n";
        let mut input = std::io::Cursor::new(input_data);
        let mut output = Vec::new();

        run_mentor_inner(&mut input, &mut output).unwrap();
        // Should just continue loop and then exit
    }
}
