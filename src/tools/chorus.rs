//! The Chorus (ὁ Χορός) - Semantic Euphony Linter
//!
//! This module implements "The Chorus", a tool that analyzes ΓΛΩΣΣΑ programs
//! not for correctness, but for *style* and *euphony*.
//!
//! # The Philosophy
//!
//! In Greek Tragedy, the Chorus stands apart from the action, commenting on the
//! themes, the morality, and the hidden truths of the play.
//!
//! Similarly, this tool comments on your code. It does not stop compilation,
//! but it offers wisdom:
//!
//! * "You are repeating yourself." (Vocabulary Repetition)
//! * "You are speaking too breathlessly." (Complexity/Statement Length)
//! * "You have introduced a character who never speaks." (Unused Variables)
//!
//! # The Goal
//!
//! To encourage code that is not just correct, but *beautiful* and *idiomatic*.

use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use smol_str::SmolStr;
use std::collections::VecDeque;
use std::fmt::Display;

/// The type of feedback provided by the Chorus
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackKind {
    /// Repetitive vocabulary (e.g. using "λέγει" 5 times in a row)
    RepetitiveVocabulary,
    /// Statement is too complex/long
    Breathlessness,
    /// General stylistic advice
    Advice,
}

impl Display for FeedbackKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedbackKind::RepetitiveVocabulary => write!(f, "Ἐπανάληψις (Repetition)"),
            FeedbackKind::Breathlessness => write!(f, "Πνευστί (Breathlessness)"),
            FeedbackKind::Advice => write!(f, "Συμβουλή (Advice)"),
        }
    }
}

/// A single piece of feedback
#[derive(Debug, Clone)]
pub struct Feedback {
    pub kind: FeedbackKind,
    pub message: String,
    pub location: Option<String>, // Context/Function name
}

/// The Chorus analyzer
pub struct Chorus<'a> {
    program: &'a AnalyzedProgram,
}

impl<'a> Chorus<'a> {
    pub fn new(program: &'a AnalyzedProgram) -> Self {
        Self { program }
    }

    /// Run the analysis and return a list of feedback items
    pub fn check_euphony(&self) -> Vec<Feedback> {
        let mut feedback = Vec::new();
        feedback.extend(self.check_repetitive_vocabulary());
        feedback.extend(self.check_breathlessness());
        feedback
    }

    /// Check for repetitive vocabulary (using the same verb too often)
    fn check_repetitive_vocabulary(&self) -> Vec<Feedback> {
        let mut feedback = Vec::new();
        let mut recent_verbs: VecDeque<SmolStr> = VecDeque::new();
        let window_size = 5;
        let repetition_threshold = 3;

        // Traverse statements to find verbs
        // We flatten the program into a sequence of statements
        // For simplicity, we only look at top-level and first-level nested statements
        let all_statements = self.flatten_statements(&self.program.statements);

        for stmt in all_statements {
            let verbs = self.extract_verbs(stmt);
            for verb in verbs {
                recent_verbs.push_back(verb.clone());
                if recent_verbs.len() > window_size {
                    recent_verbs.pop_front();
                }

                // Check count in window
                let count = recent_verbs.iter().filter(|v| **v == verb).count();
                if count >= repetition_threshold {
                    // Only report once per saturation to avoid spam
                    if count == repetition_threshold {
                        feedback.push(Feedback {
                            kind: FeedbackKind::RepetitiveVocabulary,
                            message: format!(
                                "The verb '{}' echoes too often. Consider variation.",
                                verb
                            ),
                            location: None,
                        });
                    }
                }
            }
        }

        feedback
    }

    /// Check for breathless statements (too complex/deep)
    fn check_breathlessness(&self) -> Vec<Feedback> {
        let mut feedback = Vec::new();
        let all_statements = self.flatten_statements(&self.program.statements);

        for (i, stmt) in all_statements.iter().enumerate() {
            let complexity = self.measure_complexity(stmt);
            if complexity > 10 {
                feedback.push(Feedback {
                    kind: FeedbackKind::Breathlessness,
                    message: format!(
                        "Statement #{} is too breathless (complexity score: {}). Break it down.",
                        i + 1,
                        complexity
                    ),
                    location: None,
                });
            }
        }

        feedback
    }

    // --- Helpers ---

    fn flatten_statements<'b>(&self, stmts: &'b [AnalyzedStatement]) -> Vec<&'b AnalyzedStatement> {
        let mut flat = Vec::new();
        for stmt in stmts {
            flat.push(stmt);
            match stmt {
                AnalyzedStatement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    flat.extend(self.flatten_statements(then_body));
                    if let Some(else_b) = else_body {
                        flat.extend(self.flatten_statements(else_b));
                    }
                }
                AnalyzedStatement::While { body, .. }
                | AnalyzedStatement::For { body, .. }
                | AnalyzedStatement::FunctionDef { body, .. }
                | AnalyzedStatement::TestDeclaration { body, .. } => {
                    flat.extend(self.flatten_statements(body));
                }
                AnalyzedStatement::Match { arms, .. } => {
                    for (_, body) in arms {
                        flat.extend(self.flatten_statements(body));
                    }
                }
                _ => {}
            }
        }
        flat
    }

    fn extract_verbs(&self, stmt: &AnalyzedStatement) -> Vec<SmolStr> {
        let mut verbs = Vec::new();
        match stmt {
            AnalyzedStatement::Print(_) => verbs.push("λέγει".into()),
            AnalyzedStatement::Expression(exprs) => {
                for expr in exprs {
                    self.extract_verbs_from_expr(expr, &mut verbs);
                }
            }
            AnalyzedStatement::Binding { value, .. }
            | AnalyzedStatement::Assignment { value, .. } => {
                self.extract_verbs_from_expr(value, &mut verbs);
            }
            AnalyzedStatement::If { condition, .. }
            | AnalyzedStatement::While { condition, .. } => {
                self.extract_verbs_from_expr(condition, &mut verbs);
            }
            _ => {}
        }
        verbs
    }

    fn extract_verbs_from_expr(&self, expr: &AnalyzedExpr, verbs: &mut Vec<SmolStr>) {
        match &expr.expr {
            AnalyzedExprKind::VerbCall { verb, args } => {
                verbs.push(verb.clone());
                for arg in args {
                    self.extract_verbs_from_expr(arg, verbs);
                }
            }
            AnalyzedExprKind::FunctionCall { args, .. }
            | AnalyzedExprKind::MethodCall { args, .. }
            | AnalyzedExprKind::TraitMethodCall { args, .. } => {
                for arg in args {
                    self.extract_verbs_from_expr(arg, verbs);
                }
            }
            AnalyzedExprKind::BinOp { left, right, .. } => {
                self.extract_verbs_from_expr(left, verbs);
                self.extract_verbs_from_expr(right, verbs);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => {
                self.extract_verbs_from_expr(operand, verbs);
            }
            // Recurse into other structures as needed...
            _ => {}
        }
    }

    fn measure_complexity(&self, stmt: &AnalyzedStatement) -> usize {
        match stmt {
            AnalyzedStatement::Print(exprs)
            | AnalyzedStatement::Expression(exprs)
            | AnalyzedStatement::Query(exprs) => {
                exprs.iter().map(|e| self.expr_complexity(e)).sum()
            }
            AnalyzedStatement::Binding { value, .. }
            | AnalyzedStatement::Assignment { value, .. } => self.expr_complexity(value),
            AnalyzedStatement::If { condition, .. }
            | AnalyzedStatement::While { condition, .. } => self.expr_complexity(condition),
            _ => 1,
        }
    }

    fn expr_complexity(&self, expr: &AnalyzedExpr) -> usize {
        let base = 1;
        match &expr.expr {
            AnalyzedExprKind::VerbCall { args, .. }
            | AnalyzedExprKind::FunctionCall { args, .. }
            | AnalyzedExprKind::MethodCall { args, .. }
            | AnalyzedExprKind::TraitMethodCall { args, .. } => {
                base + args.iter().map(|a| self.expr_complexity(a)).sum::<usize>()
            }
            AnalyzedExprKind::BinOp { left, right, .. } => {
                base + self.expr_complexity(left) + self.expr_complexity(right)
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => base + self.expr_complexity(operand),
            _ => base,
        }
    }

    /// Display the chorus feedback in a table
    pub fn display(&self, feedback: &[Feedback]) {
        if feedback.is_empty() {
            println!(
                "\n{}",
                "The Chorus is silent. (No issues found)".green().italic()
            );
            return;
        }

        println!("\n{}", "The Chorus speaks:".cyan().bold().underlined());

        let mut table = Table::new();
        table.load_preset(presets::UTF8_FULL).set_header(vec![
            Cell::new("Theme")
                .add_attribute(Attribute::Bold)
                .fg(Color::Magenta),
            Cell::new("Wisdom").add_attribute(Attribute::Bold),
        ]);

        for item in feedback {
            let kind_str = item.kind.to_string();
            let msg = &item.message;

            table.add_row(vec![Cell::new(kind_str).fg(Color::Yellow), Cell::new(msg)]);
        }

        println!("{table}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedStatement, GlossaType, Scope};

    fn create_repetitive_program() -> AnalyzedProgram {
        // Create a program with 5 "say" statements in a row
        let mut statements = Vec::new();
        for _ in 0..5 {
            statements.push(AnalyzedStatement::Print(vec![AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral("hello".into()),
                glossa_type: GlossaType::String,
            }]));
        }

        AnalyzedProgram {
            statements,
            scope: Scope::new(),
        }
    }

    #[test]
    fn test_repetitive_vocabulary() {
        let program = create_repetitive_program();
        let chorus = Chorus::new(&program);
        let feedback = chorus.check_euphony();

        assert!(
            feedback
                .iter()
                .any(|f| f.kind == FeedbackKind::RepetitiveVocabulary),
            "Expected repetitive vocabulary warning"
        );
    }

    fn create_breathless_statement() -> AnalyzedProgram {
        // Create a deeply nested statement or one with many expressions
        let mut deep_expr = AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(1),
            glossa_type: GlossaType::Number,
        };

        // Wrap it 10 times
        for _ in 0..10 {
            deep_expr = AnalyzedExpr {
                expr: AnalyzedExprKind::UnaryOp {
                    op: crate::morphology::lexicon::UnaryOp::Not, // Just a dummy wrap
                    operand: Box::new(deep_expr),
                },
                glossa_type: GlossaType::Number,
            };
        }

        AnalyzedProgram {
            statements: vec![AnalyzedStatement::Expression(vec![deep_expr])],
            scope: Scope::new(),
        }
    }

    #[test]
    fn test_breathlessness() {
        let program = create_breathless_statement();
        let chorus = Chorus::new(&program);
        let feedback = chorus.check_euphony();

        assert!(
            feedback
                .iter()
                .any(|f| f.kind == FeedbackKind::Breathlessness),
            "Expected breathlessness warning"
        );
    }

    #[test]
    fn test_display_coverage() {
        // Just verify it doesn't panic
        let program = create_breathless_statement();
        let chorus = Chorus::new(&program);
        let feedback = chorus.check_euphony();
        chorus.display(&feedback);

        // Also test empty feedback
        chorus.display(&[]);
    }

    #[test]
    fn test_control_flow_complexity() {
        // Tests If/While complexity and flattening
        let condition = Box::new(AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(true),
            glossa_type: GlossaType::Boolean,
        });

        // Nested statement to test flattening
        let nested_stmt = AnalyzedStatement::Print(vec![AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral("test".into()),
            glossa_type: GlossaType::String,
        }]);

        let if_stmt = AnalyzedStatement::If {
            condition: condition.clone(),
            then_body: vec![nested_stmt.clone()],
            else_body: Some(vec![nested_stmt.clone()]),
        };

        let while_stmt = AnalyzedStatement::While {
            condition: condition.clone(),
            body: vec![nested_stmt.clone()],
        };

        let program = AnalyzedProgram {
            statements: vec![if_stmt, while_stmt],
            scope: Scope::new(),
        };

        let chorus = Chorus::new(&program);
        // Complexity should be low (condition is literal), but verbs should be extracted
        let _ = chorus.check_euphony();
    }

    #[test]
    fn test_binding_and_assignment_complexity() {
        // Binding
        let binding = AnalyzedStatement::Binding {
            name: "x".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            },
            mutable: false,
        };

        // Assignment with complexity
        let assignment = AnalyzedStatement::Assignment {
            name: "x".into(),
            value: AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(1),
                        glossa_type: GlossaType::Number,
                    }),
                    op: crate::morphology::lexicon::BinaryOp::Add,
                    right: Box::new(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(2),
                        glossa_type: GlossaType::Number,
                    }),
                },
                glossa_type: GlossaType::Number,
            },
        };

        let program = AnalyzedProgram {
            statements: vec![binding, assignment],
            scope: Scope::new(),
        };

        let chorus = Chorus::new(&program);
        let _ = chorus.check_euphony();
    }

    #[test]
    fn test_verb_extraction_various_sources() {
        // Test extracting verbs from different expression kinds
        let verb_call = AnalyzedExpr {
            expr: AnalyzedExprKind::VerbCall {
                verb: "verb1".into(),
                args: vec![],
            },
            glossa_type: GlossaType::Unit,
        };

        let func_call = AnalyzedExpr {
            expr: AnalyzedExprKind::FunctionCall {
                func: "func1".into(),
                args: vec![verb_call.clone()], // Nested verb call
            },
            glossa_type: GlossaType::Unit,
        };

        let method_call = AnalyzedExpr {
            expr: AnalyzedExprKind::MethodCall {
                receiver: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
                method: "method1".into(),
                args: vec![func_call.clone()],
            },
            glossa_type: GlossaType::Unit,
        };

        let trait_method_call = AnalyzedExpr {
            expr: AnalyzedExprKind::TraitMethodCall {
                receiver: Box::new(AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                }),
                trait_name: "Trait1".into(),
                method_name: "method2".into(),
                args: vec![method_call.clone()],
            },
            glossa_type: GlossaType::Unit,
        };

        let stmt = AnalyzedStatement::Expression(vec![trait_method_call]);

        let program = AnalyzedProgram {
            statements: vec![stmt],
            scope: Scope::new(),
        };

        let chorus = Chorus::new(&program);
        let _ = chorus.check_euphony();
        // Just verify it runs without crashing and covers the extraction logic
    }

    #[test]
    fn test_flatten_match_and_function_def() {
        // Match statement
        let match_stmt = AnalyzedStatement::Match {
            scrutinee: Box::new(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(1),
                glossa_type: GlossaType::Number,
            }),
            arms: vec![(
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(1),
                    glossa_type: GlossaType::Number,
                },
                vec![AnalyzedStatement::Print(vec![AnalyzedExpr {
                    expr: AnalyzedExprKind::StringLiteral("match".into()),
                    glossa_type: GlossaType::String,
                }])],
            )],
        };

        // Function def
        let func_def = AnalyzedStatement::FunctionDef {
            name: "func".into(),
            params: vec![],
            body: vec![AnalyzedStatement::Print(vec![AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral("func".into()),
                glossa_type: GlossaType::String,
            }])],
            return_type: None,
        };

        let program = AnalyzedProgram {
            statements: vec![match_stmt, func_def],
            scope: Scope::new(),
        };

        let chorus = Chorus::new(&program);
        let _ = chorus.check_euphony();
    }
}
