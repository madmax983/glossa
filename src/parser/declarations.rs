use crate::ast::*;
use crate::parser::build_statement;
use crate::parser::common::ParseError;
use crate::parser::grammar::Rule;
use pest::iterators::Pair;

pub(crate) fn build_type_definition(pair: Pair<'_, Rule>) -> Result<TypeDef, ParseError> {
    let mut type_name = None;
    let mut fields = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                // This is the type name (between εἶδος and ὁρίζειν)
                type_name = Some(Word {
                    original: inner.as_str().into(),
                    normalized: crate::text::normalize_greek(inner.as_str()),
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

    let Some(name) = type_name else {
        return Err(ParseError::UnexpectedRule(
            "Type definition needs a name".to_string(),
        ));
    };

    Ok(TypeDef {
        name,
        fields,
    })
}

fn build_field_declaration(pair: Pair<'_, Rule>) -> Result<FieldDecl, ParseError> {
    let mut words = Vec::new();

    for inner in pair.into_inner() {
        if inner.as_rule() == Rule::greek_word {
            words.push(Word {
                original: inner.as_str().into(),
                normalized: crate::text::normalize_greek(inner.as_str()),
            });
        }
    }

    // fieldname typename_genitive
    if words.len() != 2 {
        return Err(ParseError::UnexpectedRule(format!(
            "Field declaration needs exactly 2 words, got {}",
            words.len()
        )));
    }

    Ok(FieldDecl {
        name: words[0].clone(),
        type_name: words[1].clone(),
    })
}

pub(crate) fn build_trait_definition(pair: Pair<'_, Rule>) -> Result<TraitDef, ParseError> {
    let mut trait_name = None;
    let mut methods = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                // This is the trait name (between χαρακτήρ and ὁρίζειν)
                trait_name = Some(Word {
                    original: inner.as_str().into(),
                    normalized: crate::text::normalize_greek(inner.as_str()),
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

    let Some(name) = trait_name else {
        return Err(ParseError::UnexpectedRule(
            "Trait definition needs a name".to_string(),
        ));
    };

    Ok(TraitDef {
        name,
        methods,
    })
}

fn parse_method_parameters(words: &[Word]) -> Vec<FieldDecl> {
    let mut params = Vec::new();
    let mut iter = words.iter();
    while let Some(word) = iter.next() {
        // Look for τῷ (dative marker) followed by parameter name
        if word.normalized != "τω" && word.normalized != "tw" {
            continue;
        }

        if let Some(name) = iter.next() {
            // Parameter without type annotation (just name)
            params.push(FieldDecl {
                name: name.clone(),
                type_name: Word::new("_"), // Placeholder, will be inferred
            });
        }
    }
    params
}

fn build_trait_method(pair: Pair<'_, Rule>) -> Result<TraitMethodDecl, ParseError> {
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
                    normalized: crate::text::normalize_greek(inner.as_str()),
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
        return Err(ParseError::UnexpectedRule(
            "Trait method needs at least a name".to_string(),
        ));
    }

    let method_name = words[0].clone();

    // Parse parameters (skip method name)
    let params = parse_method_parameters(&words[1..]);

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

pub(crate) fn build_trait_impl(pair: Pair<'_, Rule>) -> Result<TraitImplDef, ParseError> {
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
                        normalized: crate::text::normalize_greek(inner.as_str()),
                    });
                } else if trait_name.is_none() {
                    trait_name = Some(Word {
                        original: inner.as_str().into(),
                        normalized: crate::text::normalize_greek(inner.as_str()),
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

    let (Some(type_n), Some(trait_n)) = (type_name, trait_name) else {
        return Err(ParseError::UnexpectedRule(
            "Trait impl needs type and trait names".to_string(),
        ));
    };

    Ok(TraitImplDef {
        type_name: type_n,
        trait_name: trait_n,
        methods,
    })
}

pub(crate) fn build_test_declaration(pair: Pair<'_, Rule>) -> Result<TestDecl, ParseError> {
    let mut test_name = None;
    let mut body = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string_literal => {
                // Extract the content from «name»
                for content_pair in inner.into_inner() {
                    if content_pair.as_rule() == Rule::string_content {
                        test_name = Some(content_pair.as_str().to_string());
                    }
                }
            }
            Rule::test_body => {
                // test_body contains statements
                for stmt_pair in inner.into_inner() {
                    if stmt_pair.as_rule() == Rule::statement {
                        body.push(build_statement(stmt_pair)?);
                    }
                }
            }
            Rule::statement => {
                // Fallback for direct statements (shouldn't happen with test_body)
                body.push(build_statement(inner)?);
            }
            Rule::statement_end => {
                // Skip these - they're punctuation
            }
            _ => {
                // Skip keywords and other tokens
            }
        }
    }

    let Some(name) = test_name else {
        return Err(ParseError::UnexpectedRule(
            "Test declaration needs a name".to_string(),
        ));
    };

    Ok(TestDecl {
        name,
        body,
    })
}

fn build_impl_method(pair: Pair<'_, Rule>) -> Result<ImplMethodDef, ParseError> {
    let mut words = Vec::new();
    let mut body = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::greek_word => {
                words.push(Word {
                    original: inner.as_str().into(),
                    normalized: crate::text::normalize_greek(inner.as_str()),
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
        return Err(ParseError::UnexpectedRule(
            "Impl method needs at least a name".to_string(),
        ));
    }

    let method_name = words[0].clone();

    // Parse parameters (skip method name)
    let params = parse_method_parameters(&words[1..]);

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
