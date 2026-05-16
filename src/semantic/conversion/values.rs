use crate::ast::Expr;
use crate::errors::GlossaError;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::expressions::analyze_argument_expr;
use crate::semantic::model::{AnalyzedExpr, AnalyzedExprKind};
use crate::semantic::resolver::Scope;
use crate::semantic::types::GlossaType;

pub(crate) use crate::semantic::conversion::statements::*;

/// Extract value from assembled statement
///
/// This function looks at the fields of the [`AssembledStatement`] and tries
/// to extract a single meaningful value from it. It prioritizes different kinds
/// of expressions in the following order:
///
/// 1. **Unwraps**: `expr!`
/// 2. **Enum Variants**: `Some(val)`, `Ok(val)`, `None` (on subject or nominatives)
/// 3. **Genitive Methods**: `owner.method`
/// 4. **Property Access**: `user.name`
/// 5. **Index Access**: `arr[0]`
/// 6. **Array Literals**: `[1, 2]`
/// 7. **Binary Operations**: `1 + 2`
/// 8. **Object Enum Variants**: `Some(val)`, `Ok(val)`, `None` (on object) - *Prioritized over literals*
/// 9. **Literals**: `42`, `"hello"`
/// 10. **Variables (Object)**: `x`
///
/// Consolidates scattering values (numbers, strings, blocks) into a single logical expression.
///
/// In GLOSSA, depending on the sentence phrasing, the "value" of an assignment might be located
/// in the subject slot, an explicit number literal slot, a string slot, or nested inside a phrase.
/// This function acts as a semantic vacuum, pulling out the first valid expression value it can find
/// in the statement regardless of where the `Assembler` categorized it grammatically.
///
/// # Returns
///
/// * `Ok((AnalyzedExpr, GlossaType))` containing the resolved expression and its inferred type.
/// * `Err(GlossaError)` if no valid value expression can be identified.
///
/// # Examples
///
/// ```rust,ignore
/// // Example cannot be run as a doctest because this module is pub(crate)
/// use glossa::semantic::assembly::AssembledStatement;
/// use glossa::semantic::conversion::extract_value;
/// use glossa::semantic::resolver::Scope;
/// use glossa::semantic::types::GlossaType;
/// use glossa::semantic::AnalyzedExprKind;
///
/// let scope = Scope::new();
/// let mut asm = AssembledStatement::new();
///
/// // Simulate a statement that contains a number literal: 42
/// asm.numbers.push(42);
///
/// let (expr, ty) = extract_value(&asm, &scope).unwrap();
///
/// assert_eq!(ty, GlossaType::Number);
/// match expr.expr {
///     AnalyzedExprKind::NumberLiteral(n) => assert_eq!(n, 42),
///     _ => panic!("Expected NumberLiteral"),
/// }
/// ```
pub fn extract_value(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<(AnalyzedExpr, GlossaType), GlossaError> {
    if !asm_stmt.nested_phrases.is_empty() {
        // Handle nested phrases (parenthesized expressions) which act as values
        // Usually there is only one for a value expression
        if let Some(terms) = asm_stmt.nested_phrases.first() {
            let phrase_expr = Expr::Phrase(terms.clone());
            // Analyze with recursion depth check reset (as it's a new analysis root)
            let analyzed = analyze_argument_expr(&phrase_expr, scope)?;
            let ty = analyzed.glossa_type.clone();
            return Ok((analyzed, ty));
        }
    }

    if !asm_stmt.blocks.is_empty() {
        // Handle blocks (braced expressions) which act as values
        if let Some(stmts) = asm_stmt.blocks.first() {
            let block_expr = Expr::Block(stmts.clone());
            // Analyze with recursion depth check reset (as it's a new analysis root)
            // Note: analyze_argument_expr will call analyze_block, which now enforces single-statement logic
            let analyzed = analyze_argument_expr(&block_expr, scope)?;
            let ty = analyzed.glossa_type.clone();
            return Ok((analyzed, ty));
        }
    }

    if let Some(res) = extract_unwrap(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_enum_from_subject(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_genitive_method(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_enum_from_nominatives(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_property_access(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_index_access(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_array(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_binary_op(asm_stmt, scope)? {
        return Ok(res);
    }
    // Fix: Check object for enum variants BEFORE literals to avoid shadowing Some(literal) by literal
    if let Some(res) = extract_enum_from_object(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_literal(asm_stmt, scope)? {
        return Ok(res);
    }
    if let Some(res) = extract_object_fallback(asm_stmt, scope)? {
        return Ok(res);
    }

    // Default
    Ok((
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        },
        GlossaType::Number,
    ))
}
