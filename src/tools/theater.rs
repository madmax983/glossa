//! The Dramatist (ὁ Δραματουργός) - Theater Script Generator
//!
//! This module implements the "Theater" tool, which transposes a ΓΛΩΣΣΑ program
//! into a theatrical play script.
//!
//! # Purpose
//!
//! Code execution is a performance. By translating semantic AST nodes into theatrical
//! constructs (characters, scenes, props, and dialogue), "The Dramatist" helps demystify
//! execution flow through a narrative, artistic format.

use crate::semantic::{AnalyzedExprKind, AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Runs the Theater tool to generate a Markdown play script from Glossa code.
pub fn run_theater(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Δραματουργία (Generating Play Script)", "🎭");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    let mut play = String::new();
    let title = input.file_name().unwrap_or_default().to_string_lossy();

    play.push_str(&format!("# {}\n", title.to_uppercase()));
    play.push_str("*(A Tragedy of Logic and Syntax)*\n\n");

    // Dramatis Personae (Types/Structs)
    let types: Vec<_> = program.scope.types().collect();
    if !types.is_empty() {
        play.push_str("## DRAMATIS PERSONAE\n\n");
        for (name, type_def) in types {
            play.push_str(&format!("**{}**, a defined form.\n", name.to_uppercase()));
            #[allow(clippy::collapsible_if)]
            if let GlossaType::Struct { fields, .. } = type_def {
                if !fields.is_empty() {
                    play.push_str("  *(Possessing: ");
                    for (i, (field_name, _)) in fields.iter().enumerate() {
                        if i > 0 {
                            play.push_str(", ");
                        }
                        play.push_str(field_name.as_str());
                    }
                    play.push_str(")*\n");
                }
            }
            play.push('\n');
        }
    } else {
        play.push_str("## DRAMATIS PERSONAE\n\n");
        play.push_str("**THE CHORUS**, observers of the execution.\n\n");
    }

    play.push_str(
        "---\n\n## SCENE 1: The Main Thread\n\n*(The stage is dark. The execution begins.)*\n\n",
    );

    let mut act_counter = 2;

    for stmt in &program.statements {
        transpile_statement_to_script(stmt, &mut play, &mut act_counter, 0);
    }

    play.push_str("\n*(The process exits. The stage goes black.)*\n");
    play.push_str("\n**[CURTAIN]**\n");

    let output_path = input.with_extension("play.md");
    if let Err(e) = std::fs::write(&output_path, &play) {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(miette::miette!("Failed to write script file: {}", e));
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   T H E A T E R".bold().cyan());
    println!("   {}", "Theatrical Play Script Generated".italic().dim());
    println!();
    println!(
        "   {} {}",
        "Script saved to:".bold(),
        output_path.display().to_string().cyan()
    );
    println!();

    Ok(())
}

fn transpile_statement_to_script(
    stmt: &AnalyzedStatement,
    play: &mut String,
    act_counter: &mut usize,
    indent: usize,
) {
    let ind = "  ".repeat(indent);

    match stmt {
        AnalyzedStatement::Binding {
            name,
            value,
            mutable,
        } => {
            let val_str = crate::tools::narrator::tell_expr(value);
            let state = if *mutable { "restless" } else { "immutable" };
            let _ = writeln!(
                play,
                "{}*[Enter {} ({}), carrying {}]*",
                ind,
                name.to_uppercase(),
                state,
                val_str
            );
        }
        AnalyzedStatement::Assignment { name, value } => {
            let val_str = crate::tools::narrator::tell_expr(value);
            let _ = writeln!(
                play,
                "{}*[{} changes shape, now grasping {}]*",
                ind,
                name.to_uppercase(),
                val_str
            );
        }
        AnalyzedStatement::Print(exprs) => {
            let _ = writeln!(play, "\n{}**CHORUS:**", ind);
            for expr in exprs {
                let val_str = crate::tools::narrator::tell_expr(expr);
                if let AnalyzedExprKind::StringLiteral(s) = &expr.expr {
                    let _ = writeln!(play, "{}  \"{}\"", ind, s);
                } else {
                    let _ = writeln!(play, "{}  \"Behold, {}!\"", ind, val_str);
                }
            }
            let _ = writeln!(play);
        }
        AnalyzedStatement::Query(exprs) => {
            let _ = writeln!(play, "\n{}**CHORUS:**", ind);
            for expr in exprs {
                let val_str = crate::tools::narrator::tell_expr(expr);
                let _ = writeln!(play, "{}  \"What is the truth of {}?\"", ind, val_str);
            }
            let _ = writeln!(play);
        }
        AnalyzedStatement::FunctionDef {
            name, params, body, ..
        } => {
            let _ = writeln!(play, "\n---\n");
            let _ = writeln!(
                play,
                "{}## SCENE {}: The Summoning of {}\n",
                ind,
                *act_counter,
                name.to_uppercase()
            );
            *act_counter += 1;

            if !params.is_empty() {
                let _ = writeln!(play, "{}*(The scene requires tributes: ", ind);
                for (i, (p_name, _)) in params.iter().enumerate() {
                    if i > 0 {
                        let _ = write!(play, ", ");
                    }
                    let _ = write!(play, "{}", p_name);
                }
                let _ = writeln!(play, ")*\n");
            }

            for b_stmt in body {
                transpile_statement_to_script(b_stmt, play, act_counter, indent);
            }
            let _ = writeln!(play, "\n{}*(The scene concludes)*", ind);
        }
        AnalyzedStatement::If {
            condition,
            then_body,
            else_body,
        } => {
            let cond_str = crate::tools::narrator::tell_expr(condition);
            let _ = writeln!(play, "\n{}*[A tension arises: Is {} true?]*", ind, cond_str);
            let _ = writeln!(play, "{}**FATE:**", ind);
            let _ = writeln!(play, "{}  \"If so...\"", ind);
            for b_stmt in then_body {
                transpile_statement_to_script(b_stmt, play, act_counter, indent + 1);
            }
            if let Some(else_b) = else_body {
                let _ = writeln!(play, "{}**FATE:**", ind);
                let _ = writeln!(play, "{}  \"Otherwise...\"", ind);
                for b_stmt in else_b {
                    transpile_statement_to_script(b_stmt, play, act_counter, indent + 1);
                }
            }
            let _ = writeln!(play, "{}*[The tension resolves]*", ind);
        }
        AnalyzedStatement::While { condition, body } => {
            let cond_str = crate::tools::narrator::tell_expr(condition);
            let _ = writeln!(
                play,
                "\n{}*[An endless cycle begins, enduring while {} is true]*",
                ind, cond_str
            );
            for b_stmt in body {
                transpile_statement_to_script(b_stmt, play, act_counter, indent + 1);
            }
            let _ = writeln!(play, "{}*[The cycle breaks]*", ind);
        }
        AnalyzedStatement::Return { value } => {
            if let Some(val) = value {
                let val_str = crate::tools::narrator::tell_expr(val);
                let _ = writeln!(
                    play,
                    "{}*[The scene offers {} to the heavens and fades]*",
                    ind, val_str
                );
            } else {
                let _ = writeln!(play, "{}*[The scene fades into nothingness]*", ind);
            }
        }
        AnalyzedStatement::Expression(exprs) => {
            for expr in exprs {
                let val_str = crate::tools::narrator::tell_expr(expr);
                let _ = writeln!(play, "{}*[An action occurs: {}]*", ind, val_str);
            }
        }
        _ => {
            // Unimplemented features
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_theater_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("theater_test.γλ");
        std::fs::write(&input_path, "ξ 10 ἔστω. «χαῖρε» λέγε.\n").unwrap();

        let result = run_theater(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("play.md");
        assert!(output_path.exists());

        let md = std::fs::read_to_string(&output_path).unwrap();

        assert!(md.contains("# THEATER_TEST.ΓΛ"));
        assert!(md.contains("## DRAMATIS PERSONAE"));
        assert!(md.contains("SCENE 1: The Main Thread"));
        assert!(md.contains("*[Enter Ξ (immutable)"));
        assert!(md.contains("**CHORUS:**"));
        assert!(md.contains("χαῖρε"));
    }

    #[test]
    fn test_run_theater_file_not_found() {
        let result = run_theater(Path::new("missing.gl"));
        assert!(result.is_err());
    }

    #[test]
    fn test_run_theater_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("parse_error.γλ");
        std::fs::write(&input_path, b"invalid syntax").unwrap();

        let result = run_theater(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_run_theater_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("semantic_error.γλ");
        std::fs::write(&input_path, "ψ 10 γίγνεται.").unwrap(); // Reassigning undefined var

        let result = run_theater(&input_path);
        assert!(result.is_err());
    }
}
