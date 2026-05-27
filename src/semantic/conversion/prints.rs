use crate::errors::GlossaError;
use crate::semantic::assembly::AssembledStatement;
use crate::semantic::expressions::analyze_argument_expr;
use crate::semantic::model::*;
use crate::semantic::resolver::Scope;
use crate::semantic::types::GlossaType;

use super::*;

pub(crate) fn try_print_binary_op(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Option<Vec<AnalyzedExpr>> {
    if !asm_stmt.operators.is_empty() {
        let left = if let Some(ref subj) = asm_stmt.subject {
            scope.lookup(&subj.lemma).map(|var_type| AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: var_type.clone(),
            })
        } else {
            None
        };

        let right = asm_stmt.literals.first().map(literal_to_analyzed_expr);

        if let (Some(left_expr), Some(right_expr)) = (left, right) {
            let op = asm_stmt.operators[0];
            let bin_expr = build_binary_expr(left_expr, op, right_expr);
            return Some(vec![bin_expr]);
        }
    }
    None
}

pub(crate) fn try_print_property_access(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Option<Vec<AnalyzedExpr>> {
    if !asm_stmt.property_accesses.is_empty() {
        let mut args = Vec::with_capacity(asm_stmt.property_accesses.len());
        for (owner, method) in &asm_stmt.property_accesses {
            let receiver = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(owner.clone().into()),
                glossa_type: scope.lookup(owner).cloned().unwrap_or(GlossaType::Unknown),
            };
            let method_args = if let Some((ref meth, ref delim)) = asm_stmt.string_method {
                if meth == method {
                    vec![AnalyzedExpr {
                        expr: AnalyzedExprKind::StringLiteral(delim.clone()),
                        glossa_type: GlossaType::String,
                    }]
                } else {
                    vec![]
                }
            } else {
                vec![]
            };
            let return_type = match method.as_str() {
                "len" => GlossaType::Number,
                "split" => GlossaType::List(Box::new(GlossaType::String)),
                "join" => GlossaType::String,
                _ => GlossaType::Unknown,
            };
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: method.clone().into(),
                    args: method_args,
                },
                glossa_type: return_type,
            });
        }
        return Some(args);
    }
    None
}

pub(crate) fn try_print_index_access(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<Vec<AnalyzedExpr>>, GlossaError> {
    if !asm_stmt.index_accesses.is_empty() {
        let mut args = Vec::with_capacity(asm_stmt.index_accesses.len());
        for (array_expr, index_expr) in &asm_stmt.index_accesses {
            let array_analyzed = analyze_argument_expr(array_expr, scope)?;
            let index_analyzed = analyze_argument_expr(index_expr, scope)?;
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::IndexAccess {
                    array: Box::new(array_analyzed),
                    index: Box::new(index_analyzed),
                },
                glossa_type: GlossaType::Unknown,
            });
        }
        return Ok(Some(args));
    }
    Ok(None)
}

pub(crate) fn try_print_unwrap(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<Vec<AnalyzedExpr>>, GlossaError> {
    if !asm_stmt.unwraps.is_empty() {
        let mut args = Vec::with_capacity(asm_stmt.unwraps.len());
        for unwrap_expr in &asm_stmt.unwraps {
            let inner_analyzed = analyze_argument_expr(unwrap_expr, scope)?;
            args.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Unwrap(Box::new(inner_analyzed)),
                glossa_type: GlossaType::Unknown,
            });
        }
        return Ok(Some(args));
    }
    Ok(None)
}

pub(crate) fn try_print_default(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Vec<AnalyzedExpr>, GlossaError> {
    let mut args =
        build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators)?;

    if let Some(ref subj) = asm_stmt.subject
        && let Some(var_type) = scope.lookup(&subj.lemma)
    {
        args.insert(
            0,
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: var_type.clone(),
            },
        );
    }

    if let Some(ref obj) = asm_stmt.object
        && let Some(var_type) = scope.lookup(&obj.lemma)
    {
        args.push(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
            glossa_type: var_type.clone(),
        });
    }

    Ok(args)
}

/// Helper: Detect print statement
pub(crate) fn classify_print(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = &verb.lemma;

        if crate::morphology::lexicon::is_print_verb(verb_lemma) {
            if let Some(args) = try_print_binary_op(asm_stmt, scope) {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            if let Some(args) = try_print_property_access(asm_stmt, scope) {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            if let Some(args) = try_print_index_access(asm_stmt, scope)? {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            if let Some(args) = try_print_unwrap(asm_stmt, scope)? {
                return Ok(Some(AnalyzedStatement::Print(args)));
            }

            let args = try_print_default(asm_stmt, scope)?;
            return Ok(Some(AnalyzedStatement::Print(args)));
        }
    }
    Ok(None)
}

/// Helper: Detect query statement
pub(crate) fn classify_query(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if !asm_stmt.is_query {
        return Ok(None);
    }

    if let Some(analyzed) = classify_containment_query(asm_stmt, scope)? {
        return Ok(Some(analyzed));
    }

    // Regular query
    let mut exprs = Vec::with_capacity(asm_stmt.literals.len() + 1);
    for lit in &asm_stmt.literals {
        exprs.push(literal_to_analyzed_expr(lit));
    }
    if let Some(ref subj) = asm_stmt.subject {
        let var_type = scope
            .lookup(&subj.lemma)
            .cloned()
            .unwrap_or(GlossaType::Unknown);
        exprs.push(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
            glossa_type: var_type,
        });
    }
    Ok(Some(AnalyzedStatement::Query(exprs)))
}

/// Helper: Detect containment queries
pub(crate) fn classify_containment_query(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
    if !asm_stmt.has_containment_preposition {
        return Ok(None);
    }

    let Some(ref subj) = asm_stmt.subject else {
        return Ok(None);
    };

    let subj_name = &subj.normalized;
    let subj_type = scope
        .lookup(subj_name)
        .cloned()
        .unwrap_or(GlossaType::Unknown);

    let collection = AnalyzedExpr {
        expr: AnalyzedExprKind::Variable(subj_name.clone()),
        glossa_type: subj_type.clone(),
    };

    let element = if let Some(lit) = asm_stmt.literals.first() {
        literal_to_analyzed_expr(lit)
    } else {
        AnalyzedExpr {
            expr: AnalyzedExprKind::NumberLiteral(0),
            glossa_type: GlossaType::Number,
        }
    };

    let is_map = matches!(subj_type, GlossaType::Map(_, _));
    let method = if is_map { "contains_key" } else { "contains" };

    // Handle referencing argument if not a string literal
    let arg_expr = if matches!(element.expr, AnalyzedExprKind::StringLiteral(_)) {
        element
    } else {
        AnalyzedExpr {
            expr: AnalyzedExprKind::UnaryOp {
                op: crate::morphology::lexicon::UnaryOp::Ref,
                operand: Box::new(element),
            },
            glossa_type: GlossaType::Unknown,
        }
    };

    let contains_expr = AnalyzedExpr {
        expr: AnalyzedExprKind::MethodCall {
            receiver: Box::new(collection),
            method: method.into(),
            args: vec![arg_expr],
        },
        glossa_type: GlossaType::Boolean,
    };

    Ok(Some(AnalyzedStatement::Query(vec![contains_expr])))
}
