//! The Philosopher (ὁ Φιλόσοφος)
//!
//! A thematic static analyzer and linter for ΓΛΩΣΣΑ programs.
//! It evaluates code against Ancient Greek philosophical maxims, finding
//! code smells such as deep nesting, excessive mutability, or overly long functions.

#![cfg(feature = "nova")]

use crate::semantic::{AnalyzedProgram, AnalyzedStatement};
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};
use crossterm::style::Stylize;
use miette::Result;
use std::path::Path;

/// A philosophical lint rule violation
#[derive(Debug, PartialEq, Eq)]
pub struct Maxim {
    pub philosopher: String,
    pub quote: String,
    pub observation: String,
    pub location: String, // E.g., "Function 'main'", "Global Scope"
}

/// The core static analyzer that generates philosophical maxims
#[derive(Default)]
pub struct Philosopher {
    maxims: Vec<Maxim>,
}

impl Philosopher {
    pub fn new() -> Self {
        Self { maxims: Vec::new() }
    }

    /// Analyze a program and return a list of philosophical maxims
    pub fn analyze(&mut self, program: &AnalyzedProgram) -> &[Maxim] {
        self.maxims.clear();

        // Pass 1: Global Scope Analysis
        self.analyze_block(&program.statements, "Global Scope", 0, 0);

        &self.maxims
    }

    fn analyze_block(
        &mut self,
        statements: &[AnalyzedStatement],
        location: &str,
        depth: usize,
        nesting_level: usize,
    ) {
        let mut mutable_count = 0;
        let statement_count = statements.len();

        if statement_count == 0 && depth > 0 && !location.contains("Test") {
            self.maxims.push(Maxim {
                philosopher: "Diogenes".to_string(),
                quote: "I am looking for an honest block of code.".to_string(),
                observation: "This block is completely empty. It serves no purpose.".to_string(),
                location: location.to_string(),
            });
        }

        if nesting_level > 2 {
            self.maxims.push(Maxim {
                philosopher: "Daedalus".to_string(),
                quote: "The labyrinth you build may trap you as well.".to_string(),
                observation: "Deep nesting (level > 2) makes logic impenetrable. Consider extracting functions or using early returns (guard clauses).".to_string(),
                location: location.to_string(),
            });
        }

        if statement_count > 15 {
            self.maxims.push(Maxim {
                philosopher: "Aristotle".to_string(),
                quote: "The whole is greater than the sum of its parts, but this part is too great.".to_string(),
                observation: format!("Function or block contains {} statements (limit 15). Break it down into smaller, focused helpers.", statement_count),
                location: location.to_string(),
            });
        }

        for stmt in statements {
            match stmt {
                AnalyzedStatement::Binding { mutable: true, .. } => {
                    mutable_count += 1;
                }
                AnalyzedStatement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    self.analyze_block(then_body, location, depth + 1, nesting_level + 1);
                    if let Some(else_b) = else_body {
                        self.analyze_block(else_b, location, depth + 1, nesting_level + 1);
                    }
                }
                AnalyzedStatement::While { body, .. } => {
                    self.analyze_block(body, location, depth + 1, nesting_level + 1);
                }
                AnalyzedStatement::For { body, .. } => {
                    self.analyze_block(body, location, depth + 1, nesting_level + 1);
                }
                AnalyzedStatement::Match { arms, .. } => {
                    for (_, body) in arms {
                        self.analyze_block(body, location, depth + 1, nesting_level + 1);
                    }
                }
                AnalyzedStatement::FunctionDef { name, body, .. } => {
                    self.analyze_block(body, &format!("Function '{}'", name), depth + 1, 0);
                }
                AnalyzedStatement::TestDeclaration { name, body } => {
                    self.analyze_block(body, &format!("Test '{}'", name), depth + 1, 0);
                }
                _ => {}
            }
        }

        if mutable_count > 3 {
            self.maxims.push(Maxim {
                philosopher: "Heraclitus".to_string(),
                quote: "Everything flows, nothing stands still.".to_string(),
                observation: format!("Excessive mutability detected ({} mutable bindings). Too much flow washes away reason. Prefer immutable bindings where possible.", mutable_count),
                location: location.to_string(),
            });
        }
    }
}

/// Run the Philosopher tool on a given file
pub fn run_philosopher(input: &Path) -> Result<()> {
    use crate::parser::parse;
    use crate::semantic::analyze_program;
    use crate::tools::runner::load_source;
    use crate::tools::ui::Status;

    let status = Status::start_with_symbol("Στοχασμός (Philosophizing)", "🦉");

    let source = load_source(input)?;
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    let mut philosopher = Philosopher::new();
    let maxims = philosopher.analyze(&program);

    status.success();

    if maxims.is_empty() {
        println!(
            "\n{}",
            "✨ The Philosopher nods in approval. Your logic is sound and balanced."
                .green()
                .bold()
        );
        return Ok(());
    }

    println!("\n{}", "🦉 The Philosopher speaks...".yellow().bold());
    println!(
        "{}",
        "Heed these ancient maxims to improve your code:\n".dim()
    );

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table.set_header(vec![
        Cell::new("Philosopher")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Maxim")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
        Cell::new("Observation")
            .add_attribute(Attribute::Bold)
            .fg(Color::White),
        Cell::new("Location")
            .add_attribute(Attribute::Bold)
            .fg(Color::Magenta),
    ]);

    for maxim in maxims {
        table.add_row(vec![
            Cell::new(&maxim.philosopher)
                .fg(Color::Cyan)
                .add_attribute(Attribute::Bold),
            Cell::new(format!("\"{}\"", maxim.quote))
                .fg(Color::Yellow)
                .add_attribute(Attribute::Italic),
            Cell::new(&maxim.observation).fg(Color::White),
            Cell::new(&maxim.location).fg(Color::Magenta),
        ]);
    }

    println!("{}", table);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, GlossaType, Scope};
    use smol_str::SmolStr;

    fn dummy_expr() -> AnalyzedExpr {
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        }
    }

    #[test]
    fn test_philosopher_heraclitus() {
        let mut statements = Vec::new();
        for i in 0..4 {
            statements.push(AnalyzedStatement::Binding {
                name: SmolStr::new(format!("var{}", i)),
                value: dummy_expr(),
                mutable: true,
            });
        }

        let program = AnalyzedProgram {
            statements,
            scope: Scope::new(),
        };

        let mut phil = Philosopher::new();
        let maxims = phil.analyze(&program);

        assert_eq!(maxims.len(), 1);
        assert_eq!(maxims[0].philosopher, "Heraclitus");
    }

    #[test]
    fn test_philosopher_daedalus() {
        // Build nested ifs: if -> if -> if (nesting level 3)
        let inner_if = AnalyzedStatement::If {
            condition: Box::new(dummy_expr()),
            then_body: vec![AnalyzedStatement::Expression(vec![dummy_expr()])],
            else_body: None,
        };

        let middle_if = AnalyzedStatement::If {
            condition: Box::new(dummy_expr()),
            then_body: vec![inner_if],
            else_body: None,
        };

        let outer_if = AnalyzedStatement::If {
            condition: Box::new(dummy_expr()),
            then_body: vec![middle_if],
            else_body: None,
        };

        let program = AnalyzedProgram {
            statements: vec![outer_if],
            scope: Scope::new(),
        };

        let mut phil = Philosopher::new();
        let maxims = phil.analyze(&program);

        assert!(maxims.iter().any(|m| m.philosopher == "Daedalus"));
    }

    #[test]
    fn test_philosopher_diogenes() {
        let empty_func = AnalyzedStatement::FunctionDef {
            name: "do_nothing".into(),
            params: vec![],
            return_type: None,
            body: vec![], // Empty body -> depth 1, len 0
        };

        let program = AnalyzedProgram {
            statements: vec![empty_func],
            scope: Scope::new(),
        };

        let mut phil = Philosopher::new();
        let maxims = phil.analyze(&program);

        assert!(maxims.iter().any(|m| m.philosopher == "Diogenes"));
    }

    #[test]
    fn test_philosopher_empty_block() {
        // Build an empty while loop to cover the empty block check natively
        let empty_while = AnalyzedStatement::While {
            condition: Box::new(dummy_expr()),
            body: vec![],
        };

        let program = AnalyzedProgram {
            statements: vec![empty_while],
            scope: Scope::new(),
        };

        let mut phil = Philosopher::new();
        let maxims = phil.analyze(&program);

        assert!(maxims.iter().any(|m| m.philosopher == "Diogenes"));
    }

    #[test]
    fn test_philosopher_match_and_for_coverage() {
        // Need to cover For and Match arms to bump coverage
        let for_loop = AnalyzedStatement::For {
            variable: "x".into(),
            iterator: Box::new(dummy_expr()),
            body: vec![AnalyzedStatement::Binding {
                name: "y".into(),
                value: dummy_expr(),
                mutable: true,
            }],
        };

        let match_expr = AnalyzedStatement::Match {
            scrutinee: Box::new(dummy_expr()),
            arms: vec![(
                dummy_expr(),
                vec![AnalyzedStatement::Expression(vec![dummy_expr()])],
            )],
        };

        let test_decl = AnalyzedStatement::TestDeclaration {
            name: "coverage_test".into(),
            body: vec![AnalyzedStatement::Expression(vec![dummy_expr()])],
        };

        let program = AnalyzedProgram {
            statements: vec![for_loop, match_expr, test_decl],
            scope: Scope::new(),
        };

        let mut phil = Philosopher::new();
        let _maxims = phil.analyze(&program);
        // We just want to ensure it traversed these correctly without crashing.
    }

    #[test]
    fn test_philosopher_aristotle() {
        let mut statements = Vec::new();
        for _ in 0..16 {
            statements.push(AnalyzedStatement::Expression(vec![dummy_expr()]));
        }

        let func = AnalyzedStatement::FunctionDef {
            name: "long_func".into(),
            params: vec![],
            return_type: None,
            body: statements,
        };

        let program = AnalyzedProgram {
            statements: vec![func],
            scope: Scope::new(),
        };

        let mut phil = Philosopher::new();
        let maxims = phil.analyze(&program);

        assert!(maxims.iter().any(|m| m.philosopher == "Aristotle"));
    }
}
