//! The Philosopher Tool (ὁ Φιλόσοφος)
//!
//! This module implements a thematic static analyzer for ΓΛΩΣΣΑ.
//! It traverses the `AnalyzedProgram` AST to identify code smells
//! (like deep nesting or overly long functions) and presents them
//! as Ancient Greek philosophical maxims.

use crate::semantic::{AnalyzedProgram, AnalyzedStatement};
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

pub fn run_philosopher(input_path: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Φιλόσοφος (Philosopher)", "🦉");

    let source = crate::tools::runner::load_source(input_path)?;
    let ast = crate::parser::parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = crate::semantic::analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    let mut smells = Vec::new();
    analyze_program_smells(&program, &mut smells);

    status.success();

    if smells.is_empty() {
        println!(
            "\n✨ {} - {}",
            "Σοφία (Wisdom)".bold().cyan(),
            "Your code is perfectly balanced. Aristotle would be proud."
                .italic()
                .dim()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Maxim")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Location")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
            Cell::new("Critique")
                .add_attribute(Attribute::Bold)
                .fg(Color::Red),
        ]);

    for smell in smells {
        table.add_row(vec![
            Cell::new(&smell.maxim).fg(Color::Cyan),
            Cell::new(&smell.location).fg(Color::Yellow),
            Cell::new(&smell.critique).fg(Color::Red),
        ]);
    }

    println!("\n{}\n", table);

    Ok(())
}

struct CodeSmell {
    maxim: String,
    location: String,
    critique: String,
}

fn analyze_program_smells(program: &AnalyzedProgram, smells: &mut Vec<CodeSmell>) {
    for stmt in &program.statements {
        analyze_statement(stmt, 0, "Global", smells);
    }
}

fn analyze_statement(
    stmt: &AnalyzedStatement,
    depth: usize,
    context: &str,
    smells: &mut Vec<CodeSmell>,
) {
    if depth > 3 {
        smells.push(CodeSmell {
            maxim: "Λαβύρινθος (Labyrinth)".to_string(),
            location: context.to_string(),
            critique: format!(
                "Nesting depth is {}, which is too complex. Simplify your logic.",
                depth
            ),
        });
    }

    match stmt {
        AnalyzedStatement::FunctionDef {
            name, params, body, ..
        } => {
            if params.len() > 3 {
                smells.push(CodeSmell {
                    maxim: "Μηδὲν ἄγαν (Nothing in excess)".to_string(),
                    location: format!("Function '{}'", name),
                    critique: format!("Function has {} parameters. Limit is 3.", params.len()),
                });
            }
            if body.len() > 10 {
                smells.push(CodeSmell {
                    maxim: "Μέτρον ἄριστον (Moderation is best)".to_string(),
                    location: format!("Function '{}'", name),
                    critique: format!("Function is {} statements long. Break it down.", body.len()),
                });
            }
            for s in body {
                analyze_statement(s, depth + 1, &format!("Function '{}'", name), smells);
            }
        }
        AnalyzedStatement::If {
            then_body,
            else_body,
            ..
        } => {
            for s in then_body {
                analyze_statement(s, depth + 1, context, smells);
            }
            if let Some(else_b) = else_body {
                for s in else_b {
                    analyze_statement(s, depth + 1, context, smells);
                }
            }
        }
        AnalyzedStatement::While { body, .. } | AnalyzedStatement::For { body, .. } => {
            for s in body {
                analyze_statement(s, depth + 1, context, smells);
            }
        }
        AnalyzedStatement::Match { arms, .. } => {
            for (_, body) in arms {
                for s in body {
                    analyze_statement(s, depth + 1, context, smells);
                }
            }
        }
        AnalyzedStatement::TraitImplementation { methods, .. } => {
            for m in methods {
                if let Some(body) = &m.body {
                    if m.params.len() > 3 {
                        smells.push(CodeSmell {
                            maxim: "Μηδὲν ἄγαν (Nothing in excess)".to_string(),
                            location: format!("Method '{}'", m.name),
                            critique: format!(
                                "Method has {} parameters. Limit is 3.",
                                m.params.len()
                            ),
                        });
                    }
                    if body.len() > 10 {
                        smells.push(CodeSmell {
                            maxim: "Μέτρον ἄριστον (Moderation is best)".to_string(),
                            location: format!("Method '{}'", m.name),
                            critique: format!(
                                "Method is {} statements long. Break it down.",
                                body.len()
                            ),
                        });
                    }
                    for s in body {
                        analyze_statement(s, depth + 1, &format!("Method '{}'", m.name), smells);
                    }
                }
            }
        }
        AnalyzedStatement::TestDeclaration { name, body } => {
            if body.len() > 15 {
                smells.push(CodeSmell {
                    maxim: "Μέτρον ἄριστον (Moderation is best)".to_string(),
                    location: format!("Test '{}'", name),
                    critique: format!(
                        "Test is {} statements long. Keep tests focused.",
                        body.len()
                    ),
                });
            }
            for s in body {
                analyze_statement(s, depth + 1, &format!("Test '{}'", name), smells);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::AnalyzedProgram;
    use crate::semantic::AnalyzedStatement;
    use crate::semantic::Scope;

    #[test]
    fn test_philosopher_smells() {
        let mut program = AnalyzedProgram {
            statements: vec![],
            scope: Scope::new(),
        };

        // Inject a function smell
        program.statements.push(AnalyzedStatement::FunctionDef {
            name: "test_func".into(),
            params: vec![
                ("a".into(), None),
                ("b".into(), None),
                ("c".into(), None),
                ("d".into(), None), // > 3 params triggers "Nothing in excess"
            ],
            body: vec![],
            return_type: None,
        });

        let mut smells = Vec::new();
        analyze_program_smells(&program, &mut smells);
        assert_eq!(smells.len(), 1);
        assert_eq!(smells[0].maxim, "Μηδὲν ἄγαν (Nothing in excess)");
    }
}
