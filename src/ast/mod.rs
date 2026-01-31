//! Abstract Syntax Tree for ΓΛΩΣΣΑ
//!
//! The AST captures the semantic structure of a GLOSSA program,
//! preserving morphological information from Greek words.

mod nodes;

pub use nodes::*;

use crate::grammar::{Rule, parse};
use pest::iterators::Pair;

/// Build an AST from source code
pub fn build_ast(source: &str) -> Result<Program, AstError> {
    let pairs = parse(source).map_err(|e| AstError::ParseError(e.to_string()))?;

    let mut statements = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::statement {
                    statements.push(build_statement(inner)?);
                }
            }
        }
    }

    Ok(Program { statements })
}

fn build_statement(pair: Pair<'_, Rule>) -> Result<Statement, AstError> {
    let mut clauses = Vec::new();
    let mut is_query = false;
    let mut is_propagate = false;
    let mut type_def = None;
    let mut trait_def = None;
    let mut trait_impl = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::type_definition => {
                type_def = Some(build_type_definition(inner)?);
            }
            Rule::trait_definition => {
                trait_def = Some(build_trait_definition(inner)?);
            }
            Rule::trait_impl => {
                trait_impl = Some(build_trait_impl(inner)?);
            }
            Rule::clause_list => {
                for clause_pair in inner.into_inner() {
                    if clause_pair.as_rule() == Rule::clause {
                        clauses.push(build_clause(clause_pair)?);
                    }
                }
            }
            Rule::statement_end => {
                for end_inner in inner.into_inner() {
                    match end_inner.as_rule() {
                        Rule::query => is_query = true,
                        Rule::propagate => is_propagate = true,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    if let Some(def) = type_def {
        Ok(Statement::TypeDefinition(def))
    } else if let Some(def) = trait_def {
        Ok(Statement::TraitDefinition(def))
    } else if let Some(impl_def) = trait_impl {
        Ok(Statement::TraitImpl(impl_def))
    } else {
        Ok(Statement::Regular {
            clauses,
            is_query,
            is_propagate,
        })
    }
}

fn build_type_definition(pair: Pair<'_, Rule>) -> Result<TypeDef, AstError> {
    let mut type_name = None;
    let mut fields = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                // This is the type name (between εἶδος and ὁρίζειν)
                type_name = Some(Word {
                    original: inner.as_str().into(),
                    normalized: crate::grammar::normalize_greek(inner.as_str()),
                });
            }
            Rule::field_list => {
                for field_pair in inner.into_inner() {
                    if field_pair.as_rule() == Rule::field_declaration {
                        fields.push(build_field_declaration(field_pair)?);
                    }
                }
            }
            _ => {}
        }
    }

    if type_name.is_none() {
        return Err(AstError::UnexpectedRule(
            "Type definition needs a name".to_string(),
        ));
    }

    Ok(TypeDef {
        name: type_name.unwrap(),
        fields,
    })
}

fn build_field_declaration(pair: Pair<'_, Rule>) -> Result<FieldDecl, AstError> {
    let mut words = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::greek_word {
            words.push(Word {
                original: inner.as_str().into(),
                normalized: crate::grammar::normalize_greek(inner.as_str()),
            });
        }
    }

    // fieldname typename_genitive
    if words.len() != 2 {
        return Err(AstError::UnexpectedRule(format!(
            "Field declaration needs exactly 2 words, got {}",
            words.len()
        )));
    }

    Ok(FieldDecl {
        name: words[0].clone(),
        type_name: words[1].clone(),
    })
}

fn build_trait_definition(pair: Pair<'_, Rule>) -> Result<TraitDef, AstError> {
    let mut trait_name = None;
    let mut methods = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                // This is the trait name (between χαρακτήρ and ὁρίζειν)
                trait_name = Some(Word {
                    original: inner.as_str().into(),
                    normalized: crate::grammar::normalize_greek(inner.as_str()),
                });
            }
            Rule::trait_method_list => {
                for method_pair in inner.into_inner() {
                    if method_pair.as_rule() == Rule::trait_method {
                        methods.push(build_trait_method(method_pair)?);
                    }
                }
            }
            _ => {}
        }
    }

    if trait_name.is_none() {
        return Err(AstError::UnexpectedRule(
            "Trait definition needs a name".to_string(),
        ));
    }

    Ok(TraitDef {
        name: trait_name.unwrap(),
        methods,
    })
}

fn build_trait_method(pair: Pair<'_, Rule>) -> Result<TraitMethodDecl, AstError> {
    let mut words = Vec::new();
    let mut body_statements = Vec::new();
    let mut has_body = false;
    let mut is_default = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::dei_keyword => {
                is_default = false;
            }
            Rule::ede_keyword => {
                is_default = true;
            }
            Rule::greek_word => {
                words.push(Word {
                    original: inner.as_str().into(),
                    normalized: crate::grammar::normalize_greek(inner.as_str()),
                });
            }
            Rule::statement => {
                has_body = true;
                body_statements.push(build_statement(inner)?);
            }
            _ => {}
        }
    }

    // words[0] = method name
    // words[1..] = parameters (τῷ self, τῷ other, etc.)
    if words.is_empty() {
        return Err(AstError::UnexpectedRule(
            "Trait method needs at least a name".to_string(),
        ));
    }

    let method_name = words[0].clone();

    // Parse parameters (skip method name)
    let mut params = Vec::new();
    let mut i = 1;
    while i < words.len() {
        // Look for τῷ (dative marker) followed by parameter name
        if words[i].normalized == "τω" || words[i].normalized == "tw" {
            if i + 1 < words.len() {
                // Parameter without type annotation (just name)
                params.push(FieldDecl {
                    name: words[i + 1].clone(),
                    type_name: Word::new("_"), // Placeholder, will be inferred
                });
                i += 2;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    Ok(TraitMethodDecl {
        name: method_name,
        params,
        is_default,
        body: if has_body {
            Some(body_statements)
        } else {
            None
        },
    })
}

fn build_trait_impl(pair: Pair<'_, Rule>) -> Result<TraitImplDef, AstError> {
    let mut type_name = None;
    let mut trait_name = None;
    let mut methods = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                // First greek_word is the type name, second is the trait name
                if type_name.is_none() {
                    type_name = Some(Word {
                        original: inner.as_str().into(),
                        normalized: crate::grammar::normalize_greek(inner.as_str()),
                    });
                } else if trait_name.is_none() {
                    trait_name = Some(Word {
                        original: inner.as_str().into(),
                        normalized: crate::grammar::normalize_greek(inner.as_str()),
                    });
                }
            }
            Rule::impl_method_list => {
                for method_pair in inner.into_inner() {
                    if method_pair.as_rule() == Rule::impl_method {
                        methods.push(build_impl_method(method_pair)?);
                    }
                }
            }
            _ => {}
        }
    }

    if type_name.is_none() || trait_name.is_none() {
        return Err(AstError::UnexpectedRule(
            "Trait impl needs type and trait names".to_string(),
        ));
    }

    Ok(TraitImplDef {
        type_name: type_name.unwrap(),
        trait_name: trait_name.unwrap(),
        methods,
    })
}

fn build_impl_method(pair: Pair<'_, Rule>) -> Result<ImplMethodDef, AstError> {
    let mut words = Vec::new();
    let mut body = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                words.push(Word {
                    original: inner.as_str().into(),
                    normalized: crate::grammar::normalize_greek(inner.as_str()),
                });
            }
            Rule::statement => {
                // Each impl_method has exactly one statement (use block for multiple)
                body = Some(build_statement(inner)?);
            }
            _ => {}
        }
    }

    // words[0] = method name
    // words[1..] = parameters (τῷ self, τῷ other, etc.)
    if words.is_empty() {
        return Err(AstError::UnexpectedRule(
            "Impl method needs at least a name".to_string(),
        ));
    }

    let method_name = words[0].clone();

    // Parse parameters (skip method name)
    let mut params = Vec::new();
    let mut i = 1;
    while i < words.len() {
        // Look for τῷ (dative marker) followed by parameter name
        if words[i].normalized == "τω" || words[i].normalized == "tw" {
            if i + 1 < words.len() {
                // Parameter without type annotation (just name)
                params.push(FieldDecl {
                    name: words[i + 1].clone(),
                    type_name: Word::new("_"), // Placeholder, will be inferred
                });
                i += 2;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    Ok(ImplMethodDef {
        name: method_name,
        params,
        body: if let Some(stmt) = body {
            vec![stmt]
        } else {
            vec![]
        },
    })
}

fn build_clause(pair: Pair<'_, Rule>) -> Result<Clause, AstError> {
    let mut expressions = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::expression {
            expressions.push(build_expression(inner)?);
        }
    }

    Ok(Clause { expressions })
}

fn build_expression(pair: Pair<'_, Rule>) -> Result<Expr, AstError> {
    let mut terms = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::term {
            terms.push(build_term(inner)?);
        }
    }

    // If there's only one term, return it directly
    if terms.len() == 1 {
        Ok(terms.into_iter().next().unwrap())
    } else {
        // Multiple terms form a phrase (e.g., "χαῖρε κόσμε λέγε")
        Ok(Expr::Phrase(terms))
    }
}

fn build_term(pair: Pair<'_, Rule>) -> Result<Expr, AstError> {
    let inner = pair.into_inner().next().ok_or(AstError::EmptyTerm)?;

    match inner.as_rule() {
        Rule::block => {
            let mut statements = Vec::new();
            for stmt_pair in inner.into_inner() {
                if stmt_pair.as_rule() == Rule::statement {
                    statements.push(build_statement(stmt_pair)?);
                }
            }
            Ok(Expr::Block(statements))
        }
        Rule::array_literal => {
            let mut elements = Vec::new();
            for child in inner.into_inner() {
                if child.as_rule() == Rule::array_elements {
                    for elem in child.into_inner() {
                        if elem.as_rule() == Rule::array_element {
                            elements.push(build_array_element(elem)?);
                        }
                    }
                }
            }
            Ok(Expr::ArrayLiteral(elements))
        }
        Rule::indexed_word => {
            let mut parts = inner.into_inner();
            // First is the greek_word (array name)
            let array_word = parts.next().ok_or(AstError::EmptyTerm)?;
            let array = Expr::Word(Word {
                original: array_word.as_str().into(),
                normalized: crate::grammar::normalize_greek(array_word.as_str()),
            });
            // Second is the index_expr
            let index_pair = parts.next().ok_or(AstError::EmptyTerm)?;
            let index_inner = index_pair.into_inner().next().ok_or(AstError::EmptyTerm)?;
            let index = match index_inner.as_rule() {
                Rule::number_literal => {
                    let value: i64 = index_inner
                        .as_str()
                        .parse()
                        .map_err(|_| AstError::InvalidNumber(index_inner.as_str().to_string()))?;
                    Expr::NumberLiteral(value)
                }
                Rule::greek_word => Expr::Word(Word {
                    original: index_inner.as_str().into(),
                    normalized: crate::grammar::normalize_greek(index_inner.as_str()),
                }),
                _ => {
                    return Err(AstError::UnexpectedRule(format!(
                        "{:?}",
                        index_inner.as_rule()
                    )));
                }
            };
            Ok(Expr::IndexAccess {
                array: Box::new(array),
                index: Box::new(index),
            })
        }
        Rule::string_literal => {
            let content = inner
                .into_inner()
                .find(|p| p.as_rule() == Rule::string_content)
                .map(|p| p.as_str().to_string())
                .unwrap_or_default();
            Ok(Expr::StringLiteral(content))
        }
        Rule::number_literal => {
            let value: i64 = inner
                .as_str()
                .parse()
                .map_err(|_| AstError::InvalidNumber(inner.as_str().to_string()))?;
            Ok(Expr::NumberLiteral(value))
        }
        Rule::boolean_literal => {
            let normalized = crate::grammar::normalize_greek(inner.as_str());
            let value = normalized == "αληθες";
            Ok(Expr::BooleanLiteral(value))
        }
        Rule::greek_word => Ok(Expr::Word(Word {
            original: inner.as_str().into(),
            normalized: crate::grammar::normalize_greek(inner.as_str()),
        })),
        Rule::parenthesized_expr => {
            // Unwrap the parentheses and build the inner expression
            let expr_pair = inner.into_inner().next().ok_or(AstError::EmptyTerm)?;
            build_expression(expr_pair)
        }
        Rule::unwrap_expr => {
            // Extract the word from "word!"
            let word_pair = inner.into_inner().next().ok_or(AstError::EmptyTerm)?;
            let word = Expr::Word(Word {
                original: word_pair.as_str().into(),
                normalized: crate::grammar::normalize_greek(word_pair.as_str()),
            });
            Ok(Expr::UnaryOp {
                op: UnaryOperator::Unwrap,
                operand: Box::new(word),
            })
        }
        _ => Err(AstError::UnexpectedRule(format!("{:?}", inner.as_rule()))),
    }
}

fn build_array_element(pair: Pair<'_, Rule>) -> Result<Expr, AstError> {
    let inner = pair.into_inner().next().ok_or(AstError::EmptyTerm)?;

    match inner.as_rule() {
        Rule::string_literal => {
            let content = inner
                .into_inner()
                .find(|p| p.as_rule() == Rule::string_content)
                .map(|p| p.as_str().to_string())
                .unwrap_or_default();
            Ok(Expr::StringLiteral(content))
        }
        Rule::number_literal => {
            let value: i64 = inner
                .as_str()
                .parse()
                .map_err(|_| AstError::InvalidNumber(inner.as_str().to_string()))?;
            Ok(Expr::NumberLiteral(value))
        }
        Rule::boolean_literal => {
            let normalized = crate::grammar::normalize_greek(inner.as_str());
            let value = normalized == "αληθες";
            Ok(Expr::BooleanLiteral(value))
        }
        Rule::greek_word => Ok(Expr::Word(Word {
            original: inner.as_str().into(),
            normalized: crate::grammar::normalize_greek(inner.as_str()),
        })),
        _ => Err(AstError::UnexpectedRule(format!("{:?}", inner.as_rule()))),
    }
}

/// Errors that can occur during AST construction
#[derive(Debug, Clone, thiserror::Error)]
pub enum AstError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Empty term in expression")]
    EmptyTerm,

    #[error("Invalid number: {0}")]
    InvalidNumber(String),

    #[error("Unexpected rule: {0}")]
    UnexpectedRule(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to get the first expression of the first clause
    fn first_expr(stmt: &Statement) -> &Expr {
        match stmt {
            Statement::Regular { clauses, .. } => &clauses[0].expressions[0],
            Statement::TypeDefinition(_) => panic!("Cannot get first_expr from TypeDefinition"),
            Statement::TraitDefinition(_) => panic!("Cannot get first_expr from TraitDefinition"),
            Statement::TraitImpl(_) => panic!("Cannot get first_expr from TraitImpl"),
        }
    }

    #[test]
    fn test_build_ast_hello() {
        let source = "«χαῖρε» λέγε.";
        let ast = build_ast(source).unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert!(!ast.statements[0].is_query());
    }

    #[test]
    fn test_build_ast_string_literal() {
        let source = "«χαῖρε κόσμε» λέγε.";
        let ast = build_ast(source).unwrap();

        let expr = first_expr(&ast.statements[0]);
        if let Expr::Phrase(terms) = expr {
            assert!(matches!(&terms[0], Expr::StringLiteral(s) if s == "χαῖρε κόσμε"));
        } else {
            panic!("Expected Phrase, got {:?}", expr);
        }
    }

    #[test]
    fn test_build_ast_variable_binding() {
        let source = "ξ πέντε ἔστω.";
        let ast = build_ast(source).unwrap();

        assert_eq!(ast.statements.len(), 1);
        let expr = first_expr(&ast.statements[0]);

        // Should have three words: ξ, πέντε, ἔστω
        if let Expr::Phrase(terms) = expr {
            assert_eq!(terms.len(), 3);
            if let Expr::Word(w) = &terms[0] {
                assert_eq!(w.normalized, "ξ");
            }
        } else {
            panic!("Expected Phrase");
        }
    }

    #[test]
    fn test_build_ast_number_literal() {
        let source = "42 λέγε.";
        let ast = build_ast(source).unwrap();

        let expr = first_expr(&ast.statements[0]);
        if let Expr::Phrase(terms) = expr {
            assert!(matches!(&terms[0], Expr::NumberLiteral(42)));
        }
    }

    #[test]
    fn test_build_ast_query() {
        let source = "ξ?";
        let ast = build_ast(source).unwrap();

        assert!(ast.statements[0].is_query());
    }

    #[test]
    fn test_build_ast_multiple_statements() {
        let source = "ξ πέντε ἔστω. ξ λέγε.";
        let ast = build_ast(source).unwrap();

        assert_eq!(ast.statements.len(), 2);
    }

    #[test]
    fn test_word_normalization() {
        let source = "χρήστου ὄνομα λέγε.";
        let ast = build_ast(source).unwrap();

        let expr = first_expr(&ast.statements[0]);
        if let Expr::Phrase(terms) = expr
            && let Expr::Word(w) = &terms[0]
        {
            assert_eq!(w.original, "χρήστου");
            assert_eq!(w.normalized, "χρηστου");
        }
    }

    #[test]
    fn test_build_ast_with_comma() {
        // Test that commas create multiple clauses
        let source = "εἰ ξ μεῖζον, «ναί» λέγε.";
        let ast = build_ast(source).unwrap();

        assert_eq!(ast.statements.len(), 1);
        assert_eq!(ast.statements[0].clauses().len(), 2); // Two clauses separated by comma
    }
}
