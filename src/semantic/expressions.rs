//! Expression analysis and helpers

use super::{AnalyzedExpr, AnalyzedExprKind, Assembler, GlossaType, Literal, Scope};
use crate::ast::{Expr, Statement};
use crate::errors::GlossaError;
use crate::morphology::{self, DisambiguationContext, analyze_article, resolve_best};
use crate::text::normalize_greek;

/// Analyze an argument expression (could be literal, variable, or nested call)
///
/// This function recursively analyzes an expression AST node and converts it into
/// a typed, semantic `AnalyzedExpr`. It handles name resolution, type inference,
/// and nested structure unpacking.
///
pub fn analyze_argument_expr(expr: &Expr, scope: &Scope) -> Result<AnalyzedExpr, GlossaError> {
    analyze_argument_expr_recursive(expr, scope, 0)
}

pub const MAX_RECURSION_DEPTH: usize = 50;

fn analyze_argument_expr_recursive(
    expr: &Expr,
    scope: &Scope,
    depth: usize,
) -> Result<AnalyzedExpr, GlossaError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(GlossaError::semantic(
            "Recursion limit exceeded in expression analysis",
        ));
    }

    match expr {
        Expr::Word(w) => analyze_word(w, scope),

        Expr::NumberLiteral(_) | Expr::StringLiteral(_) | Expr::BooleanLiteral(_) => {
            analyze_literal(expr)
        }

        Expr::ArrayLiteral(elements) => analyze_array(elements, scope, depth),

        Expr::IndexAccess { array, index } => analyze_index_access(array, index, scope, depth),

        Expr::Phrase(terms) => analyze_phrase(terms, scope, depth),

        Expr::Block(statements) => analyze_block(statements, scope, depth),

        Expr::BinOp { left, op, right } => analyze_binop(left, op, right, scope, depth),

        Expr::UnaryOp { op, operand } => analyze_unaryop(op, operand, scope, depth),

        Expr::PropertyAccess { owner, property } => {
            analyze_property_access(owner, property, scope, depth)
        }

        _ => Err(GlossaError::semantic(
            "Unsupported argument expression type",
        )),
    }
}

fn analyze_word(w: &crate::ast::Word, scope: &Scope) -> Result<AnalyzedExpr, GlossaError> {
    let normalized = normalize_greek(&w.original);

    // Check if it's a numeral
    if let Some(val) = crate::morphology::lexicon::numeral_value(&normalized) {
        return Ok(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(val),
            glossa_type: GlossaType::Number,
        });
    }

    // Check if it's a variable
    if let Some(var_type) = scope.lookup(&normalized) {
        return Ok(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(normalized),
            glossa_type: var_type.clone(),
        });
    }

    // Unknown variable
    Err(GlossaError::semantic(format!(
        "Undefined variable: {}",
        normalized
    )))
}

fn analyze_literal(expr: &Expr) -> Result<AnalyzedExpr, GlossaError> {
    match expr {
        Expr::NumberLiteral(n) => Ok(AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(*n),
            glossa_type: GlossaType::Number,
        }),
        Expr::StringLiteral(s) => Ok(AnalyzedExpr {
            expr: AnalyzedExprKind::StringLiteral(s.clone()),
            glossa_type: GlossaType::String,
        }),
        Expr::BooleanLiteral(b) => Ok(AnalyzedExpr {
            expr: AnalyzedExprKind::BooleanLiteral(*b),
            glossa_type: GlossaType::Boolean,
        }),
        _ => Err(GlossaError::semantic("Not a literal expression")),
    }
}

fn analyze_array(
    elements: &[Expr],
    scope: &Scope,
    depth: usize,
) -> Result<AnalyzedExpr, GlossaError> {
    let mut analyzed_elements = Vec::with_capacity(elements.len());
    for el in elements {
        analyzed_elements.push(analyze_argument_expr_recursive(el, scope, depth + 1)?);
    }

    let element_type = analyzed_elements
        .first()
        .map(|e| e.glossa_type.clone())
        .unwrap_or(GlossaType::Unknown);

    Ok(AnalyzedExpr {
        expr: AnalyzedExprKind::ArrayLiteral(analyzed_elements),
        glossa_type: GlossaType::List(Box::new(element_type)),
    })
}

fn analyze_index_access(
    array: &Expr,
    index: &Expr,
    scope: &Scope,
    depth: usize,
) -> Result<AnalyzedExpr, GlossaError> {
    let array_analyzed = analyze_argument_expr_recursive(array, scope, depth + 1)?;
    let index_analyzed = analyze_argument_expr_recursive(index, scope, depth + 1)?;

    Ok(AnalyzedExpr {
        expr: AnalyzedExprKind::IndexAccess {
            array: Box::new(array_analyzed),
            index: Box::new(index_analyzed),
        },
        glossa_type: GlossaType::Unknown,
    })
}

fn analyze_property_access(
    owner: &Expr,
    property: &Expr,
    scope: &Scope,
    depth: usize,
) -> Result<AnalyzedExpr, GlossaError> {
    // Treat property access as variable lookup or method call preparation
    // This is simplified; usually property access is handled by the assembler context
    // But if we encounter it directly as an expression, we try to resolve it.
    // For now, if it's a simple property access, we might need more context or just return it as PropertyAccess
    let owner_analyzed = analyze_argument_expr_recursive(owner, scope, depth + 1)?;
    if let Expr::Word(prop_word) = property {
        Ok(AnalyzedExpr {
            expr: AnalyzedExprKind::PropertyAccess {
                owner: Box::new(owner_analyzed),
                property: normalize_greek(&prop_word.original),
            },
            glossa_type: GlossaType::Unknown,
        })
    } else {
        Err(GlossaError::semantic("Property must be a word"))
    }
}

fn analyze_phrase(
    terms: &[Expr],
    scope: &Scope,
    depth: usize,
) -> Result<AnalyzedExpr, GlossaError> {
    // A phrase could be a function call: function_name arg1 arg2 ...
    if terms.is_empty() {
        return Err(GlossaError::semantic("Empty phrase in argument"));
    }

    // Check if first term is a function name
    if let Expr::Word(w) = &terms[0] {
        let func_name = normalize_greek(&w.original);

        if scope.is_function(&func_name) {
            // It's a function call - recursively analyze arguments
            let mut args = Vec::new();
            for arg_expr in &terms[1..] {
                args.push(analyze_argument_expr_recursive(arg_expr, scope, depth + 1)?);
            }

            let return_type = scope
                .lookup_function(&func_name)
                .and_then(|sig| sig.return_type.clone())
                .unwrap_or(GlossaType::Unknown);

            return Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::FunctionCall {
                    func: func_name,
                    args,
                },
                glossa_type: return_type,
            });
        }
    }

    // Not a function call - could be a complex expression
    // For now, just analyze the first term
    analyze_argument_expr_recursive(&terms[0], scope, depth + 1)
}

fn analyze_block(
    statements: &[Statement],
    scope: &Scope,
    depth: usize,
) -> Result<AnalyzedExpr, GlossaError> {
    // Parenthesized expression - analyze as nested expression
    // Extract the expression from the block
    if let Some(stmt) = statements.first()
        && let Some(clause) = stmt.clauses().first()
        && let Some(expr) = clause.expressions.first()
    {
        return analyze_argument_expr_recursive(expr, scope, depth + 1);
    }
    Err(GlossaError::semantic("Empty or invalid block expression"))
}

fn analyze_binop(
    left: &Expr,
    op: &crate::ast::BinOperator,
    right: &Expr,
    scope: &Scope,
    depth: usize,
) -> Result<AnalyzedExpr, GlossaError> {
    let left_analyzed = analyze_argument_expr_recursive(left, scope, depth + 1)?;
    let right_analyzed = analyze_argument_expr_recursive(right, scope, depth + 1)?;
    // Map AST op to semantic op
    let sem_op = match op {
        crate::ast::BinOperator::Add => crate::morphology::lexicon::BinaryOp::Add,
        crate::ast::BinOperator::Sub => crate::morphology::lexicon::BinaryOp::Sub,
        crate::ast::BinOperator::Mul => crate::morphology::lexicon::BinaryOp::Mul,
        crate::ast::BinOperator::Div => crate::morphology::lexicon::BinaryOp::Div,
        crate::ast::BinOperator::Mod => crate::morphology::lexicon::BinaryOp::Mod,
        crate::ast::BinOperator::Eq => crate::morphology::lexicon::BinaryOp::Eq,
        crate::ast::BinOperator::Ne => crate::morphology::lexicon::BinaryOp::Ne,
        crate::ast::BinOperator::Lt => crate::morphology::lexicon::BinaryOp::Lt,
        crate::ast::BinOperator::Le => crate::morphology::lexicon::BinaryOp::Le,
        crate::ast::BinOperator::Gt => crate::morphology::lexicon::BinaryOp::Gt,
        crate::ast::BinOperator::Ge => crate::morphology::lexicon::BinaryOp::Ge,
        crate::ast::BinOperator::And => crate::morphology::lexicon::BinaryOp::And,
        crate::ast::BinOperator::Or => crate::morphology::lexicon::BinaryOp::Or,
    };

    Ok(build_binary_expr(left_analyzed, sem_op, right_analyzed))
}

fn analyze_unaryop(
    op: &crate::ast::UnaryOperator,
    operand: &Expr,
    scope: &Scope,
    depth: usize,
) -> Result<AnalyzedExpr, GlossaError> {
    match op {
        crate::ast::UnaryOperator::Unwrap => {
            let inner = analyze_argument_expr_recursive(operand, scope, depth + 1)?;
            Ok(AnalyzedExpr {
                expr: AnalyzedExprKind::Unwrap(Box::new(inner)),
                glossa_type: GlossaType::Unknown,
            })
        }
        // TODO: Handle Neg and Not
        _ => Err(GlossaError::semantic(
            "Unsupported unary operator in expression",
        )),
    }
}

/// Get the first word from a statement for pattern detection
pub fn get_first_word(stmt: &Statement) -> Result<String, GlossaError> {
    if let Some(first_clause) = stmt.clauses().first()
        && let Some(first_expr) = first_clause.expressions.first()
    {
        if let Expr::Phrase(terms) = first_expr {
            if let Some(first_term) = terms.first()
                && let Expr::Word(word) = first_term
            {
                return Ok(word.original.to_string());
            }
        } else if let Expr::Word(word) = first_expr {
            return Ok(word.original.to_string());
        }
    }
    Err(GlossaError::semantic("Empty statement"))
}

/// Check if a statement contains the function definition verb (ὁρίζειν)
pub fn contains_function_definition_verb(stmt: &Statement) -> bool {
    for clause in stmt.clauses() {
        for expr in &clause.expressions {
            if contains_verb_in_expr(expr, "οριζειν") {
                return true;
            }
        }
    }
    false
}

/// Helper to check if an expression contains a specific verb
pub fn contains_verb_in_expr(expr: &Expr, verb: &str) -> bool {
    match expr {
        Expr::Word(word) => normalize_greek(&word.original) == verb,
        Expr::Phrase(terms) => terms.iter().any(|t| contains_verb_in_expr(t, verb)),
        _ => false,
    }
}

/// Feed an expression into the assembler with disambiguation context
pub fn feed_expr_to_assembler_with_context(
    asm: &mut Assembler,
    expr: &Expr,
    context: &mut DisambiguationContext,
) -> Result<(), GlossaError> {
    feed_expr_recursive(asm, expr, context, 0)
}

fn feed_expr_recursive(
    asm: &mut Assembler,
    expr: &Expr,
    context: &mut DisambiguationContext,
    depth: usize,
) -> Result<(), GlossaError> {
    if depth > MAX_RECURSION_DEPTH {
        return Err(GlossaError::semantic(
            "Recursion limit exceeded in expression analysis",
        ));
    }

    match expr {
        Expr::StringLiteral(s) => {
            asm.feed_string(s.clone())?;
        }
        Expr::NumberLiteral(n) => {
            asm.feed_number(*n)?;
        }
        Expr::BooleanLiteral(b) => {
            asm.feed_boolean(*b)?;
        }
        Expr::Word(w) => {
            // Check if this is an article using ORIGINAL form (preserves diacritics)
            // This distinguishes ἡ (article) from ἤ (or) - they differ only in breathing/accent
            let article_check = analyze_article(&w.original);
            if let Some(article_context) = article_check {
                *context = article_context;
                // Articles themselves don't go to assembler slots
                return Ok(());
            }

            // Check if this word is a participle (for lambda construction)
            // BUT: skip participle check if the word is in the lexicon as something else
            // This prevents comparative adjectives like μείζον from being misidentified as participles
            let in_lexicon = morphology::lexicon::lookup(&w.normalized).is_some();
            let is_numeral = morphology::lexicon::numeral_value(&w.normalized).is_some();

            if !in_lexicon && !is_numeral {
                let participle_check = morphology::analyze_participle(&w.normalized);
                if let Some(participle_analysis) = participle_check {
                    asm.feed_participle(&participle_analysis, &w.original)?;
                    return Ok(());
                }
            }

            // Get all possible analyses for the word
            let analyses = morphology::analyze_all(&w.normalized);

            // Use disambiguation context to pick the best analysis
            let best_analysis = resolve_best(analyses, context);

            // Feed the disambiguated analysis to assembler
            if let Err(e) = asm.feed(&best_analysis, &w.original) {
                return Err(GlossaError::semantic(e.to_string()));
            }

            // Clear context after use (it was consumed by the following noun)
            *context = DisambiguationContext::new();
        }
        Expr::Phrase(terms) => {
            // Feed each term in the phrase, passing context through
            // But detect nested phrases (parenthesized expressions) and store them separately
            for term in terms {
                if matches!(term, Expr::Phrase(_)) {
                    // This is a nested phrase (parenthesized expression)
                    // Store it for later analysis instead of flattening
                    if let Expr::Phrase(nested_terms) = term {
                        asm.feed_nested_phrase(nested_terms.clone())?;
                    }
                } else {
                    feed_expr_recursive(asm, term, context, depth + 1)?;
                }
            }
        }
        Expr::PropertyAccess { owner, property } => {
            // Owner is genitive, property is what it attaches to
            feed_expr_recursive(asm, owner, context, depth + 1)?;
            feed_expr_recursive(asm, property, context, depth + 1)?;
        }
        Expr::Call { verb, arguments } => {
            // Feed the verb - verbs can set context for subjects
            let analyses = morphology::analyze_all(&verb.normalized);
            let best_verb = resolve_best(analyses, context);

            // Set context from verb for potential subject agreement
            *context = DisambiguationContext::from_verb(&best_verb);

            if let Err(e) = asm.feed(&best_verb, &verb.original) {
                return Err(GlossaError::semantic(e.to_string()));
            }

            // Feed arguments
            for arg in arguments {
                feed_expr_recursive(asm, arg, context, depth + 1)?;
            }
        }
        Expr::Binding { name, value } => {
            // Feed the name and value (binding verbs handled by assembler)
            let analyses = morphology::analyze_all(&name.normalized);
            let best_name = resolve_best(analyses, context);

            if let Err(e) = asm.feed(&best_name, &name.original) {
                return Err(GlossaError::semantic(e.to_string()));
            }
            feed_expr_recursive(asm, value, context, depth + 1)?;
        }
        Expr::BinOp { left, op: _, right } => {
            // TODO: Implement binary operation handling
            feed_expr_recursive(asm, left, context, depth + 1)?;
            feed_expr_recursive(asm, right, context, depth + 1)?;
        }
        Expr::UnaryOp { op, operand } => {
            // Handle unwrap operator specially - it's a postfix operator that doesn't need word-order handling
            if matches!(op, crate::ast::UnaryOperator::Unwrap) {
                // Store the unwrap expression for special handling
                asm.feed_unwrap(operand.as_ref().clone())?;
            } else {
                // TODO: Implement other unary operations (Not, Neg)
                feed_expr_recursive(asm, operand, context, depth + 1)?;
            }
        }
        Expr::Block(statements) => {
            // Parenthesized expressions are stored as blocks for later analysis
            // Don't feed their contents to the main assembler - they'll be analyzed separately
            asm.feed_block(statements.clone())?;
        }
        Expr::ArrayLiteral(elements) => {
            // Feed array literal to assembler
            asm.feed_array(elements.clone())?;
        }
        Expr::IndexAccess { array, index } => {
            // Feed index access to assembler
            asm.feed_index_access(array.as_ref().clone(), index.as_ref().clone())?;
        }
    }
    Ok(())
}

/// Convert a Literal to an AnalyzedExpr
pub fn literal_to_analyzed_expr(lit: &Literal) -> AnalyzedExpr {
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
pub fn literal_to_type(lit: &Literal) -> GlossaType {
    match lit {
        Literal::String(_) => GlossaType::String,
        Literal::Number(_) => GlossaType::Number,
        Literal::Boolean(_) => GlossaType::Boolean,
    }
}

/// Build a binary expression from two analyzed expressions and an operator
pub fn build_binary_expr(
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
pub fn build_expressions_from_literals_and_ops(
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
pub fn infer_binop_type(
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
        BinaryOp::And | BinaryOp::Or => GlossaType::Boolean,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::UnaryOperator;

    #[test]
    fn test_analyze_argument_expr_handles_unwrap() {
        let expr = Expr::UnaryOp {
            op: UnaryOperator::Unwrap,
            operand: Box::new(Expr::BooleanLiteral(true)),
        };

        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope).unwrap();

        match result.expr {
            AnalyzedExprKind::Unwrap(_) => {
                // Success - correctly identified as Unwrap
            }
            _ => panic!("Expected Unwrap, got {:?}", result),
        }
    }

    #[test]
    fn test_analyze_argument_expr_handles_array() {
        let expr = Expr::ArrayLiteral(vec![Expr::NumberLiteral(1), Expr::NumberLiteral(2)]);
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope).unwrap();

        match result.expr {
            AnalyzedExprKind::ArrayLiteral(elements) => {
                assert_eq!(elements.len(), 2);
                assert!(matches!(
                    elements[0].expr,
                    AnalyzedExprKind::NumberLiteral(1)
                ));
            }
            _ => panic!("Expected ArrayLiteral"),
        }
    }

    #[test]
    fn test_analyze_argument_expr_handles_index_access() {
        let expr = Expr::IndexAccess {
            array: Box::new(Expr::ArrayLiteral(vec![])),
            index: Box::new(Expr::NumberLiteral(0)),
        };
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope).unwrap();

        match result.expr {
            AnalyzedExprKind::IndexAccess { array: _, index } => {
                assert!(matches!(index.expr, AnalyzedExprKind::NumberLiteral(0)));
            }
            _ => panic!("Expected IndexAccess"),
        }
    }

    #[test]
    fn test_analyze_argument_expr_handles_binop() {
        let scope = Scope::new();
        let ops = vec![
            (
                crate::ast::BinOperator::Add,
                crate::morphology::lexicon::BinaryOp::Add,
            ),
            (
                crate::ast::BinOperator::Sub,
                crate::morphology::lexicon::BinaryOp::Sub,
            ),
            (
                crate::ast::BinOperator::Mul,
                crate::morphology::lexicon::BinaryOp::Mul,
            ),
            (
                crate::ast::BinOperator::Div,
                crate::morphology::lexicon::BinaryOp::Div,
            ),
            (
                crate::ast::BinOperator::Mod,
                crate::morphology::lexicon::BinaryOp::Mod,
            ),
            (
                crate::ast::BinOperator::Eq,
                crate::morphology::lexicon::BinaryOp::Eq,
            ),
            (
                crate::ast::BinOperator::Ne,
                crate::morphology::lexicon::BinaryOp::Ne,
            ),
            (
                crate::ast::BinOperator::Lt,
                crate::morphology::lexicon::BinaryOp::Lt,
            ),
            (
                crate::ast::BinOperator::Le,
                crate::morphology::lexicon::BinaryOp::Le,
            ),
            (
                crate::ast::BinOperator::Gt,
                crate::morphology::lexicon::BinaryOp::Gt,
            ),
            (
                crate::ast::BinOperator::Ge,
                crate::morphology::lexicon::BinaryOp::Ge,
            ),
            (
                crate::ast::BinOperator::And,
                crate::morphology::lexicon::BinaryOp::And,
            ),
            (
                crate::ast::BinOperator::Or,
                crate::morphology::lexicon::BinaryOp::Or,
            ),
        ];

        for (ast_op, expected_sem_op) in ops {
            let expr = Expr::BinOp {
                left: Box::new(Expr::NumberLiteral(1)),
                op: ast_op,
                right: Box::new(Expr::NumberLiteral(2)),
            };
            let result = analyze_argument_expr(&expr, &scope).unwrap();

            match result.expr {
                AnalyzedExprKind::BinOp { op, .. } => {
                    assert_eq!(
                        op, expected_sem_op,
                        "Mismatch for AST operator {:?}",
                        ast_op
                    );
                }
                _ => panic!("Expected BinOp"),
            }
        }
    }

    #[test]
    fn test_analyze_argument_expr_handles_property_access() {
        let expr = Expr::PropertyAccess {
            owner: Box::new(Expr::Word(crate::ast::Word::new("x"))),
            property: Box::new(Expr::Word(crate::ast::Word::new("y"))),
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::Unknown);
        let result = analyze_argument_expr(&expr, &scope).unwrap();

        match result.expr {
            AnalyzedExprKind::PropertyAccess { property, .. } => {
                assert_eq!(property, "y");
            }
            _ => panic!("Expected PropertyAccess"),
        }
    }

    #[test]
    fn test_analyze_argument_expr_errors_on_invalid_property() {
        let expr = Expr::PropertyAccess {
            owner: Box::new(Expr::Word(crate::ast::Word::new("x"))),
            property: Box::new(Expr::NumberLiteral(1)),
        };
        let mut scope = Scope::new();
        scope.define("x", GlossaType::Unknown);
        let result = analyze_argument_expr(&expr, &scope);

        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_errors_on_empty_phrase() {
        let expr = Expr::Phrase(vec![]);
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope);

        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_propagates_error_in_array() {
        // Array with an invalid element (empty phrase)
        let expr = Expr::ArrayLiteral(vec![Expr::NumberLiteral(1), Expr::Phrase(vec![])]);
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope);

        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_propagates_error_in_index_access() {
        // Index access with invalid array
        let expr = Expr::IndexAccess {
            array: Box::new(Expr::Phrase(vec![])),
            index: Box::new(Expr::NumberLiteral(0)),
        };
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope);

        assert!(result.is_err());

        // Index access with invalid index
        let expr2 = Expr::IndexAccess {
            array: Box::new(Expr::ArrayLiteral(vec![])),
            index: Box::new(Expr::Phrase(vec![])),
        };
        let result2 = analyze_argument_expr(&expr2, &scope);
        assert!(result2.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_propagates_error_in_binop() {
        // BinOp with invalid left
        let expr = Expr::BinOp {
            left: Box::new(Expr::Phrase(vec![])),
            op: crate::ast::BinOperator::Add,
            right: Box::new(Expr::NumberLiteral(1)),
        };
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope);
        assert!(result.is_err());

        // BinOp with invalid right
        let expr2 = Expr::BinOp {
            left: Box::new(Expr::NumberLiteral(1)),
            op: crate::ast::BinOperator::Add,
            right: Box::new(Expr::Phrase(vec![])),
        };
        let result2 = analyze_argument_expr(&expr2, &scope);
        assert!(result2.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_propagates_error_in_unary_op() {
        // UnaryOp with invalid operand
        let expr = Expr::UnaryOp {
            op: crate::ast::UnaryOperator::Unwrap,
            operand: Box::new(Expr::Phrase(vec![])),
        };
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_propagates_error_in_property_owner() {
        // Property access with invalid owner
        let expr = Expr::PropertyAccess {
            owner: Box::new(Expr::Phrase(vec![])),
            property: Box::new(Expr::Word(crate::ast::Word::new("y"))),
        };
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_handles_phrase_recursion() {
        // Phrase that is not a function call -> recursive analysis of first term
        // "((1))" -> Phrase(vec![Phrase(vec![Number(1)])])
        let inner = Expr::Phrase(vec![Expr::NumberLiteral(1)]);
        let outer = Expr::Phrase(vec![inner]);
        let scope = Scope::new();
        let result = analyze_argument_expr(&outer, &scope).unwrap();

        match result.expr {
            AnalyzedExprKind::NumberLiteral(n) => assert_eq!(n, 1),
            _ => panic!("Expected NumberLiteral"),
        }
    }

    #[test]
    fn test_analyze_argument_expr_handles_function_call() {
        // Mock a function in scope
        let mut scope = Scope::new();
        scope.define_function(
            "add",
            vec![GlossaType::Number, GlossaType::Number],
            Some(GlossaType::Number),
        );

        // "add 1 2"
        let expr = Expr::Phrase(vec![
            Expr::Word(crate::ast::Word::new("add")),
            Expr::NumberLiteral(1),
            Expr::NumberLiteral(2),
        ]);

        let result = analyze_argument_expr(&expr, &scope).unwrap();

        match result.expr {
            AnalyzedExprKind::FunctionCall { func, args } => {
                assert_eq!(func, "add");
                assert_eq!(args.len(), 2);
                assert!(matches!(args[0].expr, AnalyzedExprKind::NumberLiteral(1)));
            }
            _ => panic!("Expected FunctionCall"),
        }
    }

    #[test]
    fn test_analyze_argument_expr_propagates_error_in_function_args() {
        let mut scope = Scope::new();
        scope.define_function("add", vec![GlossaType::Number], Some(GlossaType::Number));

        // "add (error)"
        let expr = Expr::Phrase(vec![
            Expr::Word(crate::ast::Word::new("add")),
            Expr::Phrase(vec![]), // Invalid arg
        ]);

        let result = analyze_argument_expr(&expr, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_handles_block() {
        // Block containing a statement with a clause with an expression
        // { 1. }
        let stmt = crate::ast::Statement::Regular {
            clauses: vec![crate::ast::Clause {
                expressions: vec![Expr::NumberLiteral(1)],
            }],
            is_query: false,
            is_propagate: false,
        };
        let expr = Expr::Block(vec![stmt]);
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope).unwrap();

        match result.expr {
            AnalyzedExprKind::NumberLiteral(n) => assert_eq!(n, 1),
            _ => panic!("Expected NumberLiteral"),
        }
    }

    #[test]
    fn test_analyze_argument_expr_errors_on_empty_block() {
        let expr = Expr::Block(vec![]);
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_errors_on_invalid_block_structure() {
        // Block with a statement that has no clauses (should trigger error)
        let stmt = crate::ast::Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: false,
        };
        let expr = Expr::Block(vec![stmt]);
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_argument_expr_errors_on_block_with_empty_clause() {
        // Block with a statement that has a clause with no expressions
        let stmt = crate::ast::Statement::Regular {
            clauses: vec![crate::ast::Clause {
                expressions: vec![],
            }],
            is_query: false,
            is_propagate: false,
        };
        let expr = Expr::Block(vec![stmt]);
        let scope = Scope::new();
        let result = analyze_argument_expr(&expr, &scope);
        assert!(result.is_err());
    }
}
