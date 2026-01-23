//! Semantic analysis for ΓΛΩΣΣΑ
//!
//! This module handles:
//! - Slot-based sentence assembly (Greek-native word-order independence)
//! - Name resolution and scope tracking
//! - Gender/number/case agreement checking
//! - Type inference from morphology
//!
//! ## The Assembler Approach
//!
//! Unlike traditional parsers that rely on word position, ΓΛΩΣΣΑ uses a
//! slot-based assembler that routes words to grammatical slots based on
//! their case endings - just like Ancient Greek actually works.
//!
//! ```text
//! Nominative → Subject slot
//! Accusative → Object slot
//! Dative     → Indirect object slot
//! Genitive   → Possession/attachment
//! ```
//!
//! This means SOV, VSO, and OVS all produce the same result!

mod resolver;
mod agreement;
mod types;
pub mod assembler;
pub mod disambiguation;

pub use resolver::*;
pub use agreement::*;
pub use types::*;
pub use assembler::{Assembler, AssembledStatement, AssemblyError, Constituent, VerbConstituent, Literal};
pub use disambiguation::{DisambiguationContext, disambiguate, resolve_best, analyze_article};

use crate::ast::{Program, Statement, Expr};
use crate::errors::GlossaError;
use crate::morphology;
use crate::grammar::normalize_greek;

/// Perform semantic analysis on a program using the slot-based assembler
/// This is the primary entry point that provides word-order independence
pub fn analyze_program(program: &Program) -> Result<AnalyzedProgram, GlossaError> {
    // Use the assembler-based approach for true word-order independence
    let assembled = analyze_with_assembler(program)?;

    // Convert assembled statements to analyzed statements
    let mut scope = Scope::new();
    let mut analyzed_statements = Vec::new();

    for asm_stmt in assembled {
        let analyzed = convert_assembled_to_analyzed(&asm_stmt, &mut scope)?;
        analyzed_statements.push(analyzed);
    }

    Ok(AnalyzedProgram {
        statements: analyzed_statements,
        scope,
    })
}

/// Legacy analysis method that doesn't use the assembler
/// Kept for comparison and fallback purposes
pub fn analyze_program_legacy(program: &Program) -> Result<AnalyzedProgram, GlossaError> {
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(program)
}

/// Perform semantic analysis using the slot-based assembler
/// This is the Greek-native approach that ignores word order
fn analyze_with_assembler(program: &Program) -> Result<Vec<AssembledStatement>, GlossaError> {
    let mut results = Vec::new();
    let mut asm = Assembler::new();

    for stmt in &program.statements {
        asm.reset();
        asm.set_query(stmt.is_query);

        // Disambiguation context accumulator - articles set context for following words
        let mut current_context = DisambiguationContext::new();

        // Feed each expression/term to the assembler with disambiguation
        for expr in &stmt.expressions {
            feed_expr_to_assembler_with_context(&mut asm, expr, &mut current_context)?;
        }

        // Finalize the statement
        match asm.finalize() {
            Ok(assembled) => results.push(assembled),
            Err(e) => return Err(GlossaError::semantic(&e.to_string())),
        }
    }

    Ok(results)
}

/// Feed an expression into the assembler with disambiguation context
fn feed_expr_to_assembler_with_context(
    asm: &mut Assembler,
    expr: &Expr,
    context: &mut DisambiguationContext,
) -> Result<(), GlossaError> {
    match expr {
        Expr::StringLiteral(s) => {
            asm.feed_string(s.clone());
        }
        Expr::NumberLiteral(n) => {
            asm.feed_number(*n);
        }
        Expr::BooleanLiteral(b) => {
            asm.feed_boolean(*b);
        }
        Expr::Word(w) => {
            // Check if this is an article using ORIGINAL form (preserves diacritics)
            // This distinguishes ἡ (article) from ἤ (or) - they differ only in breathing/accent
            if let Some(article_context) = analyze_article(&w.original) {
                *context = article_context;
                // Articles themselves don't go to assembler slots
                return Ok(());
            }

            // Get all possible analyses for the word
            let analyses = morphology::analyze_all(&w.normalized);

            // Use disambiguation context to pick the best analysis
            let best_analysis = resolve_best(analyses, context);

            // Feed the disambiguated analysis to assembler
            if let Err(e) = asm.feed(&best_analysis, &w.original) {
                return Err(GlossaError::semantic(&e.to_string()));
            }

            // Clear context after use (it was consumed by the following noun)
            *context = DisambiguationContext::new();
        }
        Expr::Phrase(terms) => {
            // Feed each term in the phrase, passing context through
            for term in terms {
                feed_expr_to_assembler_with_context(asm, term, context)?;
            }
        }
        Expr::PropertyAccess { owner, property } => {
            // Owner is genitive, property is what it attaches to
            feed_expr_to_assembler_with_context(asm, owner, context)?;
            feed_expr_to_assembler_with_context(asm, property, context)?;
        }
        Expr::Call { verb, arguments } => {
            // Feed the verb - verbs can set context for subjects
            let analyses = morphology::analyze_all(&verb.normalized);
            let best_verb = resolve_best(analyses, context);

            // Set context from verb for potential subject agreement
            *context = DisambiguationContext::from_verb(&best_verb);

            if let Err(e) = asm.feed(&best_verb, &verb.original) {
                return Err(GlossaError::semantic(&e.to_string()));
            }

            // Feed arguments
            for arg in arguments {
                feed_expr_to_assembler_with_context(asm, arg, context)?;
            }
        }
        Expr::Binding { name, value } => {
            // Feed the name and value (binding verbs handled by assembler)
            let analyses = morphology::analyze_all(&name.normalized);
            let best_name = resolve_best(analyses, context);

            if let Err(e) = asm.feed(&best_name, &name.original) {
                return Err(GlossaError::semantic(&e.to_string()));
            }
            feed_expr_to_assembler_with_context(asm, value, context)?;
        }
        Expr::BinOp { left, op: _, right } => {
            // TODO: Implement binary operation handling
            feed_expr_to_assembler_with_context(asm, left, context)?;
            feed_expr_to_assembler_with_context(asm, right, context)?;
        }
        Expr::UnaryOp { op: _, operand } => {
            // TODO: Implement unary operation handling
            feed_expr_to_assembler_with_context(asm, operand, context)?;
        }
    }
    Ok(())
}

/// Convert an AssembledStatement to an AnalyzedStatement
/// This bridges the slot-based assembler output to the HIR lowering input
fn convert_assembled_to_analyzed(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<AnalyzedStatement, GlossaError> {
    // Determine statement kind based on assembled content
    let (kind, expressions) = classify_assembled_statement(asm_stmt, scope)?;

    Ok(AnalyzedStatement { kind, expressions })
}

/// Classify an assembled statement and extract analyzed expressions
fn classify_assembled_statement(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<(StatementKind, Vec<AnalyzedExpr>), GlossaError> {
    // Check for binding pattern: has subject + literals + binding verb
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);
        if crate::morphology::lexicon::is_binding_verb(&verb_lemma) {
            // Binding: subject is the variable name, literals are the value
            if let Some(ref subject) = asm_stmt.subject {
                let name = subject.lemma.clone();

                // Get value from literals or object
                let (value_expr, value_type) = extract_value(asm_stmt);

                // Register binding in scope
                scope.define(name.clone(), value_type.clone());

                return Ok((
                    StatementKind::Binding {
                        name: name.clone(),
                        value_type: value_type.clone(),
                    },
                    vec![
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(name),
                            glossa_type: value_type.clone(),
                        },
                        value_expr,
                    ],
                ));
            }
        }

        // Check for print pattern
        if crate::morphology::lexicon::is_print_verb(&verb_lemma) {
            // If we have operators, combine subject/variables with literals using operators
            if !asm_stmt.operators.is_empty() {
                // Get left operand (subject variable)
                let left = if let Some(ref subj) = asm_stmt.subject {
                    if let Some(var_type) = scope.lookup(&subj.lemma) {
                        Some(AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                            glossa_type: var_type.clone(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Get right operand (first literal)
                let right = asm_stmt.literals.first().map(literal_to_analyzed_expr);

                // If we have both operands and an operator, build a binary expression
                if let (Some(left_expr), Some(right_expr)) = (left, right) {
                    let op = asm_stmt.operators[0];
                    let bin_expr = build_binary_expr(left_expr, op, right_expr);
                    return Ok((StatementKind::Print, vec![bin_expr]));
                }
            }

            // Build binary expressions from literals and operators if available
            // This handles cases like: true || false
            let mut args = build_expressions_from_literals_and_ops(
                &asm_stmt.literals,
                &asm_stmt.operators,
            );

            // Also include subject/object if present (variable references)
            if let Some(ref subj) = asm_stmt.subject {
                if let Some(var_type) = scope.lookup(&subj.lemma) {
                    args.insert(0, AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                        glossa_type: var_type.clone(),
                    });
                }
            }

            if let Some(ref obj) = asm_stmt.object {
                if let Some(var_type) = scope.lookup(&obj.lemma) {
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                        glossa_type: var_type.clone(),
                    });
                }
            }

            return Ok((StatementKind::Print, args));
        }
    }

    // Query pattern
    if asm_stmt.is_query {
        let mut exprs = Vec::new();
        for lit in &asm_stmt.literals {
            exprs.push(literal_to_analyzed_expr(lit));
        }
        if let Some(ref subj) = asm_stmt.subject {
            let var_type = scope.lookup(&subj.lemma).cloned().unwrap_or(GlossaType::Unknown);
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: var_type,
            });
        }
        return Ok((StatementKind::Query, exprs));
    }

    // Default: expression statement
    let exprs = build_expressions_from_literals_and_ops(
        &asm_stmt.literals,
        &asm_stmt.operators,
    );

    Ok((StatementKind::Expression, exprs))
}

/// Extract value from assembled statement (literals or constituents)
fn extract_value(asm_stmt: &AssembledStatement) -> (AnalyzedExpr, GlossaType) {
    // If we have operators, build a binary expression from literals
    if !asm_stmt.operators.is_empty() && asm_stmt.literals.len() >= 2 {
        let exprs = build_expressions_from_literals_and_ops(
            &asm_stmt.literals,
            &asm_stmt.operators,
        );
        if let Some(expr) = exprs.into_iter().next() {
            let ty = expr.glossa_type.clone();
            return (expr, ty);
        }
    }

    // Prefer literals (single value, no operators)
    if let Some(lit) = asm_stmt.literals.first() {
        return (literal_to_analyzed_expr(lit), literal_to_type(lit));
    }

    // Otherwise use object
    if let Some(ref obj) = asm_stmt.object {
        // Check if it's a numeral word
        if let Some(value) = crate::morphology::lexicon::numeral_value(&normalize_greek(&obj.lemma)) {
            return (
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(value),
                    glossa_type: GlossaType::Number,
                },
                GlossaType::Number,
            );
        }

        return (
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            },
            GlossaType::Unknown,
        );
    }

    // Default
    (
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        },
        GlossaType::Number,
    )
}

/// Convert a Literal to an AnalyzedExpr
fn literal_to_analyzed_expr(lit: &Literal) -> AnalyzedExpr {
    match lit {
        Literal::String(s) => AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral(s.clone()),
            glossa_type: GlossaType::String,
        },
        Literal::Number(n) => AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(*n),
            glossa_type: GlossaType::Number,
        },
        Literal::Boolean(b) => AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(*b),
            glossa_type: GlossaType::Boolean,
        },
    }
}

/// Get the type of a Literal
fn literal_to_type(lit: &Literal) -> GlossaType {
    match lit {
        Literal::String(_) => GlossaType::String,
        Literal::Number(_) => GlossaType::Number,
        Literal::Boolean(_) => GlossaType::Boolean,
    }
}

/// Build a binary expression from two analyzed expressions and an operator
fn build_binary_expr(
    left: AnalyzedExpr,
    op: crate::morphology::lexicon::BinaryOp,
    right: AnalyzedExpr,
) -> AnalyzedExpr {
    let result_type = infer_binop_type(&left.glossa_type, &op, &right.glossa_type);
    AnalyzedExpr {
        expr: AnalyzedExprKind::BinOp {
            left: Box::new(left),
            op,
            right: Box::new(right),
        },
        glossa_type: result_type,
    }
}

/// Build expressions from literals and operators
/// If there are operators, builds a binary expression tree
/// Otherwise, returns the literals as-is
fn build_expressions_from_literals_and_ops(
    literals: &[Literal],
    operators: &[crate::morphology::lexicon::BinaryOp],
) -> Vec<AnalyzedExpr> {
    // If no operators, just return literals as separate expressions
    if operators.is_empty() {
        return literals.iter().map(literal_to_analyzed_expr).collect();
    }

    // If we have operators, build a binary expression
    // Pattern: lit0 op0 lit1 op1 lit2 ... -> ((lit0 op0 lit1) op1 lit2) ...
    if literals.len() < 2 || operators.is_empty() {
        return literals.iter().map(literal_to_analyzed_expr).collect();
    }

    // Build left-associative tree
    let mut result = literal_to_analyzed_expr(&literals[0]);

    for (i, op) in operators.iter().enumerate() {
        if i + 1 < literals.len() {
            let right = literal_to_analyzed_expr(&literals[i + 1]);
            let result_type = infer_binop_type(&result.glossa_type, op, &right.glossa_type);
            result = AnalyzedExpr {
                expr: AnalyzedExprKind::BinOp {
                    left: Box::new(result),
                    op: *op,
                    right: Box::new(right),
                },
                glossa_type: result_type,
            };
        }
    }

    vec![result]
}

/// Infer the result type of a binary operation
fn infer_binop_type(
    _left: &GlossaType,
    op: &crate::morphology::lexicon::BinaryOp,
    _right: &GlossaType,
) -> GlossaType {
    use crate::morphology::lexicon::BinaryOp;

    match op {
        // Arithmetic operations on numbers return numbers
        BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
            GlossaType::Number
        }
        // Comparison operations return booleans
        BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
            GlossaType::Boolean
        }
        // Boolean operations return booleans
        BinaryOp::And | BinaryOp::Or => {
            GlossaType::Boolean
        }
    }
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
    /// Binary operation (arithmetic, comparison, boolean)
    BinOp {
        left: Box<AnalyzedExpr>,
        op: crate::morphology::lexicon::BinaryOp,
        right: Box<AnalyzedExpr>,
    },
    /// Unary operation (negation)
    UnaryOp {
        op: crate::morphology::lexicon::UnaryOp,
        operand: Box<AnalyzedExpr>,
    },
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
