//! The Tracer Tool (ὁ Ἰχνηλάτης)
//!
//! A hybrid of "The Simulator" (Interpreter) and "The Bard" (Narrator).
//! It executes a ΓΛΩΣΣΑ program step-by-step, explaining the semantic meaning
//! of each statement in English before it runs, and then showing the resulting
//! changes to the runtime environment (variables).
//!
//! # Philosophy
//!
//! **The Mashup**: Combines `Interpreter` and `Narrator`.
//! "We know what the code *means*, and we know how to *run* it. What if we
//! watched it think?"

use crate::semantic::AnalyzedProgram;
use crossterm::style::Stylize;
use miette::Result;

use crate::tools::interpreter::Interpreter;
use crate::tools::narrator::describe_statement;
use std::io::Write;

pub fn run_trace(program: &AnalyzedProgram) -> Result<()> {
    run_trace_inner(program, &mut std::io::stdout())
}

pub fn run_trace_inner<W: Write>(program: &AnalyzedProgram, output: &mut W) -> Result<()> {
    let mut interpreter = Interpreter::new();

    writeln!(
        output,
        "{}",
        "🔍 Starting Trace (The Simulator + The Bard)".bold().cyan()
    )
    .unwrap();
    writeln!(output, "=============================================\n").unwrap();

    for (i, stmt) in program.statements.iter().enumerate() {
        // 1. The Bard tells us what will happen
        let (_, description, _) = describe_statement(stmt);
        writeln!(
            output,
            "{} {} {}",
            "Step".yellow().bold(),
            (i + 1).to_string().yellow().bold(),
            "-".dark_grey()
        )
        .unwrap();
        writeln!(output, "  {} {}", "Intent:".cyan(), description).unwrap();

        // Save old env state conceptually (since we can't easily deep clone interpreter env)
        // Here we just print the result after.

        // 2. The Simulator executes it
        if let Err(e) = interpreter.eval_statement_public(stmt) {
            writeln!(output, "  {} {:?}", "Error:".red().bold(), e).unwrap();
            break;
        }

        // Check if there was any captured stdout
        let out = interpreter.take_output();
        if !out.is_empty() {
            writeln!(output, "  {} {}", "Stdout:".green(), out).unwrap();
        }

        // 3. Inspect the environment (simplified)
        if let Some(scope) = interpreter.global_env() {
            let mut vars: Vec<_> = scope.iter().collect();
            vars.sort_by_key(|(k, _)| *k);
            if !vars.is_empty() {
                writeln!(output, "  {} ", "State:".magenta()).unwrap();
                for (name, val) in vars {
                    writeln!(output, "    `{}` = {}", name, val).unwrap();
                }
            }
        }
        writeln!(output).unwrap();
    }

    writeln!(output, "{}", "✅ Trace Complete".bold().green()).unwrap();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_tracer_red_green() {
        let source = "ξ πέντε ἔστω.\nξ λέγε.";
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let mut output = Vec::new();
        run_trace_inner(&analyzed, &mut output).unwrap();

        let output_str = String::from_utf8(output).unwrap();
        println!("{}", output_str); // Log for debugging

        // Verify Bard's description is present
        assert!(output_str.contains("Intent:\u{1b}[39m Let `ξ` be 5."));
        // Verify execution output is captured
        assert!(output_str.contains("Stdout:\u{1b}[39m 5"));
        // Verify state is inspected
        assert!(output_str.contains("`ξ` = 5"));
    }
}
