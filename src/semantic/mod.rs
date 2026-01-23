//! Semantic analysis for ΓΛΩΣΣΑ
//!
//! This module handles:
//! - Name resolution and scope tracking
//! - Gender/number/case agreement checking
//! - Type inference from morphology

mod resolver;
mod agreement;
mod types;

pub use resolver::*;
pub use agreement::*;
pub use types::*;

use crate::ast::{Program, Statement, Expr};
use crate::errors::GlossaError;

/// Perform semantic analysis on a program
pub fn analyze_program(program: &Program) -> Result<AnalyzedProgram, GlossaError> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(program)
}

/// Analyzed program with resolved names and types
#[derive(Debug, Clone)]
pub struct AnalyzedProgram {
    pub statements: Vec<AnalyzedStatement>,
    pub scope: Scope,
}

/// Analyzed statement
#[derive(Debug, Clone)]
pub struct AnalyzedStatement {
    pub kind: StatementKind,
    pub expressions: Vec<AnalyzedExpr>,
}

/// The kind of statement
#[derive(Debug, Clone)]
pub enum StatementKind {
    /// Variable binding: ξ πέντε ἔστω
    Binding { name: String, value_type: GlossaType },
    /// Print statement: «χαῖρε» λέγε
    Print,
    /// Expression statement
    Expression,
    /// Query: ξ?
    Query,
}

/// Analyzed expression with type information
#[derive(Debug, Clone)]
pub struct AnalyzedExpr {
    pub expr: AnalyzedExprKind,
    pub glossa_type: GlossaType,
}

/// Kind of analyzed expression
#[derive(Debug, Clone)]
pub enum AnalyzedExprKind {
    StringLiteral(String),
    NumberLiteral(i64),
    BooleanLiteral(bool),
    Variable(String),
    PropertyAccess { owner: Box<AnalyzedExpr>, property: String },
    VerbCall { verb: String, args: Vec<AnalyzedExpr> },
}

/// Semantic analyzer state
pub struct SemanticAnalyzer {
    scope: Scope,
    errors: Vec<GlossaError>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        SemanticAnalyzer {
            scope: Scope::new(),
            errors: Vec::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<AnalyzedProgram, GlossaError> {
        let mut analyzed_statements = Vec::new();

        for stmt in &program.statements {
            match self.analyze_statement(stmt) {
                Ok(analyzed) => analyzed_statements.push(analyzed),
                Err(e) => self.errors.push(e),
            }
        }

        if !self.errors.is_empty() {
            return Err(self.errors[0].clone());
        }

        Ok(AnalyzedProgram {
            statements: analyzed_statements,
            scope: self.scope.clone(),
        })
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<AnalyzedStatement, GlossaError> {
        // Determine statement kind by analyzing expressions
        let exprs = &stmt.expressions;

        if exprs.is_empty() {
            return Ok(AnalyzedStatement {
                kind: StatementKind::Expression,
                expressions: vec![],
            });
        }

        // Analyze the first (and usually only) expression
        let first_expr = &exprs[0];
        let (kind, analyzed_exprs) = self.analyze_expression_list(first_expr, stmt.is_query)?;

        Ok(AnalyzedStatement {
            kind,
            expressions: analyzed_exprs,
        })
    }

    fn analyze_expression_list(
        &mut self,
        expr: &Expr,
        is_query: bool,
    ) -> Result<(StatementKind, Vec<AnalyzedExpr>), GlossaError> {
        match expr {
            Expr::Phrase(terms) => {
                self.analyze_phrase(terms, is_query)
            }
            _ => {
                let analyzed = self.analyze_single_expr(expr)?;
                let kind = if is_query {
                    StatementKind::Query
                } else {
                    StatementKind::Expression
                };
                Ok((kind, vec![analyzed]))
            }
        }
    }

    fn analyze_phrase(
        &mut self,
        terms: &[Expr],
        is_query: bool,
    ) -> Result<(StatementKind, Vec<AnalyzedExpr>), GlossaError> {
        // Look for patterns: binding, print, property access

        // Find verb position and type
        let mut verb_idx = None;
        let mut is_binding = false;
        let mut is_print = false;

        for (i, term) in terms.iter().enumerate() {
            if let Expr::Word(w) = term {
                let normalized = &w.normalized;
                if crate::morphology::lexicon::is_binding_verb(normalized) {
                    verb_idx = Some(i);
                    is_binding = true;
                    break;
                }
                if crate::morphology::lexicon::is_print_verb(normalized) {
                    verb_idx = Some(i);
                    is_print = true;
                    break;
                }
            }
        }

        if is_binding {
            // Pattern: name value ἔστω
            // e.g., ξ πέντε ἔστω
            if terms.len() >= 3 {
                let name = self.extract_name(&terms[0])?;
                let value = self.analyze_single_expr(&terms[1])?;

                // Register in scope
                self.scope.define(name.clone(), value.glossa_type.clone());

                return Ok((
                    StatementKind::Binding {
                        name: name.clone(),
                        value_type: value.glossa_type.clone(),
                    },
                    vec![
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(name),
                            glossa_type: value.glossa_type.clone(),
                        },
                        value,
                    ],
                ));
            }
        }

        if is_print {
            // Pattern: value λέγε
            let mut args = Vec::new();
            for term in terms.iter().take(verb_idx.unwrap_or(terms.len())) {
                args.push(self.analyze_single_expr(term)?);
            }

            return Ok((
                StatementKind::Print,
                args,
            ));
        }

        // Default: analyze all terms
        let mut analyzed = Vec::new();
        for term in terms {
            analyzed.push(self.analyze_single_expr(term)?);
        }

        let kind = if is_query {
            StatementKind::Query
        } else {
            StatementKind::Expression
        };

        Ok((kind, analyzed))
    }

    fn analyze_single_expr(&self, expr: &Expr) -> Result<AnalyzedExpr, GlossaError> {
        match expr {
            Expr::StringLiteral(s) => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::StringLiteral(s.clone()),
                glossa_type: GlossaType::String,
            }),

            Expr::NumberLiteral(n) => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::NumberLiteral(*n),
                glossa_type: GlossaType::Number,
            }),

            Expr::BooleanLiteral(b) => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::BooleanLiteral(*b),
                glossa_type: GlossaType::Boolean,
            }),

            Expr::Word(w) => {
                // Check if it's a numeral word
                if let Some(value) = crate::morphology::lexicon::numeral_value(&w.normalized) {
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(value),
                        glossa_type: GlossaType::Number,
                    });
                }

                // Check if it's a known variable
                if let Some(var_type) = self.scope.lookup(&w.normalized) {
                    return Ok(AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                        glossa_type: var_type.clone(),
                    });
                }

                // Unknown word - treat as variable reference
                Ok(AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(w.normalized.clone()),
                    glossa_type: GlossaType::Unknown,
                })
            }

            Expr::Phrase(terms) => {
                // Nested phrase - analyze recursively
                if terms.len() == 1 {
                    return self.analyze_single_expr(&terms[0]);
                }

                // For now, return first term's type
                let first = self.analyze_single_expr(&terms[0])?;
                Ok(first)
            }

            _ => Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable("_".to_string()),
                glossa_type: GlossaType::Unknown,
            }),
        }
    }

    fn extract_name(&self, expr: &Expr) -> Result<String, GlossaError> {
        match expr {
            Expr::Word(w) => Ok(w.normalized.clone()),
            _ => Err(GlossaError::semantic("Expected a word for variable name")),
        }
    }
}

impl Default for SemanticAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::build_ast;

    #[test]
    fn test_analyze_hello() {
        let ast = build_ast("«χαῖρε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert_eq!(analyzed.statements.len(), 1);
        assert!(matches!(analyzed.statements[0].kind, StatementKind::Print));
    }

    #[test]
    fn test_analyze_binding() {
        let ast = build_ast("ξ πέντε ἔστω.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert!(matches!(
            &analyzed.statements[0].kind,
            StatementKind::Binding { name, .. } if name == "ξ"
        ));

        // Check that ξ is now in scope
        assert!(analyzed.scope.lookup("ξ").is_some());
    }

    #[test]
    fn test_analyze_variable_use() {
        let ast = build_ast("ξ πέντε ἔστω. ξ λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        assert_eq!(analyzed.statements.len(), 2);
        // Second statement should reference ξ with known type
        assert!(matches!(analyzed.statements[1].kind, StatementKind::Print));
    }

    #[test]
    fn test_analyze_string_literal() {
        let ast = build_ast("«χαῖρε κόσμε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let first_expr = &analyzed.statements[0].expressions[0];
        assert_eq!(first_expr.glossa_type, GlossaType::String);
    }

    #[test]
    fn test_analyze_number_literal() {
        let ast = build_ast("42 λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();

        let first_expr = &analyzed.statements[0].expressions[0];
        assert_eq!(first_expr.glossa_type, GlossaType::Number);
    }
}
