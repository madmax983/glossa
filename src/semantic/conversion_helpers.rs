fn classify_iterator_pattern(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    // Check for iterator pattern with participles, comparative adjectives, or find verbs
    // Pattern 1: ξ διπλασιαζόμενα λέγε → ξ.iter().map(|x| x * 2).collect()
    // Pattern 2: ξ πέντε μείζονα λέγε → ξ.iter().filter(|x| x > 5).collect()
    // Pattern 3: ξ τριῶν μείζον εὑρέ → ξ.iter().find(|x| x > 3)
    let has_find_or_print_verb = if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);
        crate::morphology::lexicon::is_print_verb(&verb_lemma)
            || crate::morphology::lexicon::is_find_verb(&verb_lemma)
    } else {
        false
    };

    if (!asm_stmt.participles.is_empty()
        || !asm_stmt.adjectives.is_empty()
        || has_find_or_print_verb)
        && let Some(analyzed) = detect_iterator_pattern(asm_stmt, scope)?
    {
        return Ok(Some((StatementKind::Print, vec![analyzed])));
    }
    Ok(None)
}

fn classify_property_access(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    // Check for property access pattern: genitive + nominative + verb
    // Pattern: genitive_var nominative_field λέγε
    // Example: που ξ λέγε → pi.xi
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_print_verb(&verb_lemma)
            && !asm_stmt.genitives.is_empty()
            && let Some(subject) = &asm_stmt.subject
        {
            // Get owner from genitive (use lemma to get base variable name)
            let owner_lemma = &asm_stmt.genitives[0].lemma;

            // Get property from subject (nominative)
            let property = normalize_greek(&subject.original);

            // Check if owner is a struct type in scope
            if let Some(owner_type) = scope.lookup(owner_lemma)
                && matches!(owner_type, GlossaType::Struct { .. })
            {
                // Build property access
                let prop_access = AnalyzedExpr {
                    expr: AnalyzedExprKind::PropertyAccess {
                        owner: Box::new(AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(owner_lemma.clone()),
                            glossa_type: owner_type.clone(),
                        }),
                        property: property.clone(),
                    },
                    glossa_type: GlossaType::Unknown, // TODO: Look up field type
                };

                return Ok(Some((StatementKind::Print, vec![prop_access])));
            }
        }
    }
    Ok(None)
}

fn classify_struct_instantiation(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    // Check for struct instantiation pattern (assembler-based, for Greek type names)
    // Pattern: subject νέον type_name args... ἔστω
    // Example: π νέον σημεῖον πέντε ἔστω
    // Note: Latin identifier type names are handled before the assembler
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        if crate::morphology::lexicon::is_binding_verb(&verb_lemma)
            && !asm_stmt.adjectives.is_empty()
            && let (Some(subject), Some(object)) = (&asm_stmt.subject, &asm_stmt.object)
        {
            // Check if adjective is νέον (new)
            let adj_lemma = normalize_greek(&asm_stmt.adjectives[0].lemma);
            if adj_lemma == "νεος" {
                // Get variable name from subject
                let var_name = normalize_greek(&subject.original);

                // Get type name from object
                let type_name = normalize_greek(&object.original);

                // Check if type exists in scope
                if let Some(struct_type) = scope.lookup_type(&type_name).cloned() {
                    // Extract field names from struct type
                    let field_names: Vec<String> =
                        if let GlossaType::Struct { fields, .. } = &struct_type {
                            fields.iter().map(|(name, _)| name.clone()).collect()
                        } else {
                            vec![]
                        };

                    // Get constructor arguments from literals
                    let args: Vec<AnalyzedExpr> = asm_stmt
                        .literals
                        .iter()
                        .map(literal_to_analyzed_expr)
                        .collect();

                    // Build struct instantiation
                    let struct_inst = AnalyzedExpr {
                        expr: AnalyzedExprKind::StructInstantiation {
                            type_name: type_name.clone(),
                            fields: field_names,
                            args,
                        },
                        glossa_type: struct_type.clone(),
                    };

                    // Register variable in scope
                    scope.define(var_name.clone(), struct_type.clone());

                    return Ok(Some((
                        StatementKind::Binding {
                            name: var_name.clone(),
                            value_type: struct_type.clone(),
                            mutable: false,
                        },
                        vec![
                            AnalyzedExpr {
                                expr: AnalyzedExprKind::Variable(var_name),
                                glossa_type: struct_type.clone(),
                            },
                            struct_inst,
                        ],
                    )));
                }

                // Check for built-in collection types (HashSet, HashMap)
                // Pattern: ξ νέον σύνολον ἔστω → let xi = HashSet::new()
                // Pattern: ξ νέον χάρτης ἔστω → let xi = HashMap::new()
                let collection_type = match type_name.as_str() {
                    "συνολον" => {
                        Some(("HashSet", GlossaType::Set(Box::new(GlossaType::Unknown))))
                    }
                    "χαρτης" => Some((
                        "HashMap",
                        GlossaType::Map(
                            Box::new(GlossaType::Unknown),
                            Box::new(GlossaType::Unknown),
                        ),
                    )),
                    _ => None,
                };

                if let Some((rust_type, glossa_type)) = collection_type {
                    let collection_new = AnalyzedExpr {
                        expr: AnalyzedExprKind::CollectionNew {
                            collection_type: rust_type.to_string(),
                        },
                        glossa_type: glossa_type.clone(),
                    };

                    // Register variable in scope (mutable for collection operations)
                    // Collections are implicitly mutable for methods like push/pop/insert
                    scope.define_mut(var_name.clone(), glossa_type.clone());

                    return Ok(Some((
                        StatementKind::Binding {
                            name: var_name.clone(),
                            value_type: glossa_type.clone(),
                            mutable: true, // Collections are implicitly mutable
                        },
                        vec![
                            AnalyzedExpr {
                                expr: AnalyzedExprKind::Variable(var_name),
                                glossa_type: glossa_type.clone(),
                            },
                            collection_new,
                        ],
                    )));
                }
            }
        }
    }
    Ok(None)
}

fn classify_function_call(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    // Check for function call pattern
    // Pattern: subject function_name args... ἔστω
    // Where function_name is not a built-in verb but a user-defined function
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        // Check if verb is a binding verb and if there's an object or genitive that could be a function call
        if crate::morphology::lexicon::is_binding_verb(&verb_lemma) {
            // Check if object is a user-defined function
            // Pattern: subject = function(args)
            // The function name might be in the object slot or genitives

            // Try nominatives first (function names are nominative)
            let mut func_name = None;
            for nominative in &asm_stmt.nominatives {
                if scope.is_function(&nominative.lemma) {
                    func_name = Some(nominative.lemma.clone());
                    break;
                }
            }

            // Try object slot if not found in nominatives
            if func_name.is_none()
                && let Some(ref object) = asm_stmt.object
                && scope.is_function(&object.lemma)
            {
                func_name = Some(object.lemma.clone());
            }

            // Try genitives if still not found
            if func_name.is_none() {
                for genitive in &asm_stmt.genitives {
                    if scope.is_function(&genitive.lemma) {
                        func_name = Some(genitive.lemma.clone());
                        break;
                    }
                }
            }

            // If we found a function name, build the call
            if let Some(func) = func_name
                && let Some(ref subject) = asm_stmt.subject
            {
                // Build function call arguments from literals and blocks
                let mut args: Vec<AnalyzedExpr> = asm_stmt
                    .literals
                    .iter()
                    .map(literal_to_analyzed_expr)
                    .collect();

                // Add nested function calls from nested phrases (parenthesized expressions)
                for nested_terms in &asm_stmt.nested_phrases {
                    let phrase_expr = Expr::Phrase(nested_terms.clone());
                    let analyzed = analyze_argument_expr(&phrase_expr, scope)?;
                    args.push(analyzed);
                }

                // Get return type from function signature
                let return_type = scope
                    .lookup_function(&func)
                    .and_then(|sig| sig.return_type.clone())
                    .unwrap_or(GlossaType::Unknown);

                let func_call = AnalyzedExpr {
                    expr: AnalyzedExprKind::FunctionCall {
                        func: func.clone(),
                        args,
                    },
                    glossa_type: return_type.clone(),
                };

                // Register subject as variable (use original form, not lemma)
                let var_name = normalize_greek(&subject.original);
                scope.define(var_name.clone(), return_type.clone());

                return Ok(Some((
                    StatementKind::Binding {
                        name: var_name.clone(),
                        value_type: return_type.clone(),
                        mutable: false,
                    },
                    vec![
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(var_name),
                            glossa_type: return_type.clone(),
                        },
                        func_call,
                    ],
                )));
            }
        }
    }
    Ok(None)
}

fn classify_comparison(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);
        if crate::morphology::lexicon::is_binding_verb(&verb_lemma) {
            // Check if this is actually a comparison with subjunctive (εἰ condition)
            // Pattern: subject operator literal subjunctive-verb
            if !asm_stmt.operators.is_empty()
                && !asm_stmt.literals.is_empty()
                && verb.mood == Some(crate::morphology::Mood::Subjunctive)
            {
                // This is a comparison expression, not a binding
                // Build: subject op literal
                if let Some(ref subject) = asm_stmt.subject {
                    // Get left operand (subject variable)
                    let left = if let Some(var_type) = scope.lookup(&subject.lemma) {
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
                            glossa_type: var_type.clone(),
                        }
                    } else {
                        // Variable not in scope, treat as boolean literal false
                        AnalyzedExpr {
                            expr: AnalyzedExprKind::BooleanLiteral(false),
                            glossa_type: GlossaType::Boolean,
                        }
                    };

                    // Get right operand (first literal)
                    let right = literal_to_analyzed_expr(&asm_stmt.literals[0]);

                    // Build binary expression
                    let op = asm_stmt.operators[0];
                    let comparison = build_binary_expr(left, op, right);

                    return Ok(Some((StatementKind::Expression, vec![comparison])));
                }
            }
        }
    }
    Ok(None)
}

fn classify_binding(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);
        if crate::morphology::lexicon::is_binding_verb(&verb_lemma) {
            // Check if there's a participle that's actually a user-defined variable name
            // Heuristic: if there's a participle with an unknown verb stem, it's probably
            // a false positive (e.g., "τοπικον" parsed as participle but really a variable name)
            let has_false_participle = !asm_stmt.participles.is_empty()
                && morphology::lexicon::lookup(&asm_stmt.participles[0].verb_lemma).is_none();

            // Binding: subject is the variable name, literals are the value
            // BUT: check for ambiguous case where subject/object might be swapped
            // Heuristic: if subject is in scope and object is not, they're probably swapped
            let (var_name, actual_asm) = if has_false_participle {
                // Use the first participle as the variable name and remove it from participles list
                let first_participle = &asm_stmt.participles[0];
                let mut fixed_asm = asm_stmt.clone();
                fixed_asm.participles = asm_stmt.participles[1..].to_vec();
                (normalize_greek(&first_participle.original), fixed_asm)
            } else if let (Some(subject), Some(object)) = (&asm_stmt.subject, &asm_stmt.object) {
                let subject_name = normalize_greek(&subject.original);
                let object_name = normalize_greek(&object.original);

                if scope.is_defined(&subject.lemma) && !scope.is_defined(&object.lemma) {
                    // Subject is in scope, object is not → they're swapped
                    // Create a new asm_stmt with swapped subject/object
                    let mut swapped = asm_stmt.clone();
                    swapped.subject = Some(object.clone());
                    swapped.object = Some(subject.clone());
                    (object_name, swapped)
                } else {
                    (subject_name, asm_stmt.clone())
                }
            } else if let Some(subject) = &asm_stmt.subject {
                (normalize_greek(&subject.original), asm_stmt.clone())
            } else if !asm_stmt.participles.is_empty() {
                // Special case: first word was incorrectly identified as a participle
                // This happens with user-defined names like "τοπικον" that have participle-like endings
                // Use the first participle's original form as the variable name
                let first_participle = &asm_stmt.participles[0];
                let mut fixed_asm = asm_stmt.clone();
                // Remove the first participle and don't add it as subject (it's not a valid constituent)
                fixed_asm.participles = asm_stmt.participles[1..].to_vec();
                (normalize_greek(&first_participle.original), fixed_asm)
            } else {
                return Err(GlossaError::semantic("Binding without subject"));
            };

            // Get value from literals or object
            // Delegate Option/Result constructor detection to extract_value
            let (value_expr, value_type) = extract_value(&actual_asm);

            // Wrap in Try if this is a propagation statement (ends with `;`)
            let final_value_expr = if asm_stmt.is_propagate {
                AnalyzedExpr {
                    glossa_type: value_type.clone(),
                    expr: AnalyzedExprKind::Try(Box::new(value_expr)),
                }
            } else {
                value_expr
            };

            // Register binding in scope (mutable if μετά marker present)
            let is_mutable = asm_stmt.has_mutable_marker;
            if is_mutable {
                scope.define_mut(var_name.clone(), value_type.clone());
            } else {
                scope.define(var_name.clone(), value_type.clone());
            }

            return Ok(Some((
                StatementKind::Binding {
                    name: var_name.clone(),
                    value_type: value_type.clone(),
                    mutable: is_mutable,
                },
                vec![
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(var_name),
                        glossa_type: value_type.clone(),
                    },
                    final_value_expr,
                ],
            )));
        }
    }
    Ok(None)
}

fn classify_assignment(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);
        // Check for assignment: ξ δέκα γίγνεται (middle voice)
        if crate::morphology::lexicon::is_assignment_verb(&verb_lemma) {
            // Get the variable name from the subject
            let var_name = if let Some(ref subject) = asm_stmt.subject {
                normalize_greek(&subject.original)
            } else {
                return Err(GlossaError::semantic("Assignment without subject"));
            };

            // Check if variable is defined and mutable
            let binding = scope.lookup_binding(&var_name);
            match binding {
                None => {
                    return Err(GlossaError::semantic(format!(
                        "Τὸ «{}» οὐχ ὡρίσθη — πρῶτον ὅρισον αὐτό",
                        var_name
                    )));
                }
                Some(b) if !b.mutable => {
                    return Err(GlossaError::semantic(crate::errors::immutable_assignment(
                        &var_name,
                    )));
                }
                Some(b) => {
                    // An assignment must have a value
                    let has_value = !asm_stmt.literals.is_empty()
                        || asm_stmt.object.is_some()
                        || !asm_stmt.arrays.is_empty()
                        || !asm_stmt.unwraps.is_empty()
                        || !asm_stmt.index_accesses.is_empty()
                        || !asm_stmt.property_accesses.is_empty()
                        || !asm_stmt.nested_phrases.is_empty();

                    if !has_value {
                        return Err(GlossaError::semantic(format!(
                            "Τῇ πράξει «{} γίγνεται» δεῖ τιμῆς",
                            var_name
                        )));
                    }

                    let value_type = b.glossa_type.clone();
                    let (value_expr, _) = extract_value(asm_stmt);
                    scope.mark_used(&var_name);
                    return Ok(Some((
                        StatementKind::Assignment {
                            name: var_name.clone(),
                            value_type: value_type.clone(),
                        },
                        vec![
                            AnalyzedExpr {
                                expr: AnalyzedExprKind::Variable(var_name),
                                glossa_type: value_type.clone(),
                            },
                            value_expr,
                        ],
                    )));
                }
            }
        }
    }
    Ok(None)
}

fn classify_collection_op(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);

        // Check for pop pattern: subject ἕλκεται (middle voice)
        if crate::morphology::lexicon::is_pop_verb(&verb_lemma)
            && let Some(ref subject) = asm_stmt.subject
        {
            // Get the receiver (array variable)
            let receiver = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
                glossa_type: scope
                    .lookup(&subject.lemma)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown),
            };

            // Return method call expression with no arguments
            let method_call = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: "pop".to_string(),
                    args: vec![],
                },
                glossa_type: GlossaType::Unknown, // Option<T>
            };

            return Ok(Some((StatementKind::Expression, vec![method_call])));
        }

        // Check for push pattern: subject ὠθεῖ value
        if crate::morphology::lexicon::is_push_verb(&verb_lemma)
            && let Some(ref subject) = asm_stmt.subject
        {
            // Get the receiver (array variable)
            let receiver = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subject.lemma.clone()),
                glossa_type: scope
                    .lookup(&subject.lemma)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown),
            };

            // Get the argument to push (from literals or object)
            let arg = if let Some(lit) = asm_stmt.literals.first() {
                literal_to_analyzed_expr(lit)
            } else if let Some(ref obj) = asm_stmt.object {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                    glossa_type: scope
                        .lookup(&obj.lemma)
                        .cloned()
                        .unwrap_or(GlossaType::Unknown),
                }
            } else {
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(0),
                    glossa_type: GlossaType::Number,
                }
            };

            // Return method call expression
            let method_call = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: "push".to_string(),
                    args: vec![arg],
                },
                glossa_type: GlossaType::Unit,
            };

            return Ok(Some((StatementKind::Expression, vec![method_call])));
        }

        // Check for insert pattern: subject value(s) τίθησι
        // For HashSet: set element τίθησι → set.insert(element)
        // For HashMap: map key value τίθησι → map.insert(key, value)
        if crate::morphology::lexicon::is_insert_verb(&verb_lemma)
            && let Some(ref subject) = asm_stmt.subject
        {
            // Use original form for the subject variable name
            let subj_name = normalize_greek(&subject.original);
            let subj_type = scope
                .lookup(&subj_name)
                .cloned()
                .unwrap_or(GlossaType::Unknown);

            // Get the receiver (collection variable)
            let receiver = AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj_name.clone()),
                glossa_type: subj_type.clone(),
            };

            // Check if it's a Map or Set to determine argument count
            let is_map = matches!(subj_type, GlossaType::Map(_, _));

            // Build arguments for insert
            let args = if is_map && asm_stmt.literals.len() >= 2 {
                // HashMap: insert(key, value)
                vec![
                    literal_to_analyzed_expr(&asm_stmt.literals[0]),
                    literal_to_analyzed_expr(&asm_stmt.literals[1]),
                ]
            } else if let Some(lit) = asm_stmt.literals.first() {
                // HashSet: insert(element)
                vec![literal_to_analyzed_expr(lit)]
            } else if let Some(ref obj) = asm_stmt.object {
                vec![AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                    glossa_type: scope
                        .lookup(&obj.lemma)
                        .cloned()
                        .unwrap_or(GlossaType::Unknown),
                }]
            } else {
                vec![]
            };

            // Return method call expression
            // HashSet::insert returns bool, HashMap::insert returns Option<V>
            let return_type = if is_map {
                GlossaType::Option(Box::new(GlossaType::Unknown))
            } else {
                GlossaType::Boolean
            };
            let method_call = AnalyzedExpr {
                expr: AnalyzedExprKind::MethodCall {
                    receiver: Box::new(receiver),
                    method: "insert".to_string(),
                    args,
                },
                glossa_type: return_type,
            };

            return Ok(Some((StatementKind::Expression, vec![method_call])));
        }
    }
    Ok(None)
}

fn classify_print(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    if let Some(ref verb) = asm_stmt.verb {
        let verb_lemma = normalize_greek(&verb.lemma);
        // Check for print pattern
        if crate::morphology::lexicon::is_print_verb(&verb_lemma) {
            // If we have operators, combine subject/variables with literals using operators
            if !asm_stmt.operators.is_empty() {
                // Get left operand (subject variable)
                let left = if let Some(ref subj) = asm_stmt.subject {
                    scope.lookup(&subj.lemma).map(|var_type| AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                        glossa_type: var_type.clone(),
                    })
                } else {
                    None
                };

                // Get right operand (first literal)
                let right = asm_stmt.literals.first().map(literal_to_analyzed_expr);

                // If we have both operands and an operator, build a binary expression
                if let (Some(left_expr), Some(right_expr)) = (left, right) {
                    let op = asm_stmt.operators[0];
                    let bin_expr = build_binary_expr(left_expr, op, right_expr);
                    return Ok(Some((StatementKind::Print, vec![bin_expr])));
                }
            }

            // Check if we have property accesses to print (e.g., ξ μῆκος λέγε)
            if !asm_stmt.property_accesses.is_empty() {
                let mut args = Vec::new();
                for (owner, method) in &asm_stmt.property_accesses {
                    let receiver = AnalyzedExpr {
                        expr: AnalyzedExprKind::Variable(owner.clone()),
                        glossa_type: scope.lookup(owner).cloned().unwrap_or(GlossaType::Unknown),
                    };
                    // Check if this is a split/join method with a delimiter
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
                    // Determine return type based on method
                    let return_type = match method.as_str() {
                        "len" => GlossaType::Number,
                        "split" => GlossaType::List(Box::new(GlossaType::String)), // Iterator of &str
                        "join" => GlossaType::String,
                        _ => GlossaType::Unknown,
                    };
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::MethodCall {
                            receiver: Box::new(receiver),
                            method: method.clone(),
                            args: method_args,
                        },
                        glossa_type: return_type,
                    });
                }
                return Ok(Some((StatementKind::Print, args)));
            }

            // Check if we have index accesses to print
            if !asm_stmt.index_accesses.is_empty() {
                let mut args = Vec::new();
                for (array_expr, index_expr) in &asm_stmt.index_accesses {
                    let array_analyzed = convert_expr_to_analyzed(array_expr);
                    let index_analyzed = convert_expr_to_analyzed(index_expr);
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::IndexAccess {
                            array: Box::new(array_analyzed),
                            index: Box::new(index_analyzed),
                        },
                        glossa_type: GlossaType::Unknown,
                    });
                }
                return Ok(Some((StatementKind::Print, args)));
            }

            // Check if we have unwrap expressions to print
            if !asm_stmt.unwraps.is_empty() {
                let mut args = Vec::new();
                for unwrap_expr in &asm_stmt.unwraps {
                    let inner_analyzed = convert_expr_to_analyzed(unwrap_expr);
                    args.push(AnalyzedExpr {
                        expr: AnalyzedExprKind::Unwrap(Box::new(inner_analyzed)),
                        glossa_type: GlossaType::Unknown, // Type will be inferred
                    });
                }
                return Ok(Some((StatementKind::Print, args)));
            }

            // Build binary expressions from literals and operators if available
            // This handles cases like: true || false
            let mut args =
                build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators);

            // Also include subject/object if present (variable references)
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

            return Ok(Some((StatementKind::Print, args)));
        }
    }
    Ok(None)
}

fn classify_query(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<(StatementKind, Vec<AnalyzedExpr>)>, GlossaError> {
    // Query pattern - check for containment pattern first
    if asm_stmt.is_query {
        // Containment pattern: element ἐν collection? → collection.contains(&element)
        // For HashMap: key ἐν map? → map.contains_key(&key)
        if asm_stmt.has_containment_preposition {
            // The element is in literals, the collection is the subject
            if let Some(ref subj) = asm_stmt.subject {
                let subj_name = normalize_greek(&subj.original);
                let subj_type = scope
                    .lookup(&subj_name)
                    .cloned()
                    .unwrap_or(GlossaType::Unknown);

                let collection = AnalyzedExpr {
                    expr: AnalyzedExprKind::Variable(subj_name.clone()),
                    glossa_type: subj_type.clone(),
                };

                // Get the element from literals
                let element = if let Some(lit) = asm_stmt.literals.first() {
                    literal_to_analyzed_expr(lit)
                } else {
                    AnalyzedExpr {
                        expr: AnalyzedExprKind::NumberLiteral(0),
                        glossa_type: GlossaType::Number,
                    }
                };

                // Check if it's a Map or Set
                let is_map = matches!(subj_type, GlossaType::Map(_, _));

                let contains_expr = AnalyzedExpr {
                    expr: AnalyzedExprKind::CollectionContains {
                        collection: Box::new(collection),
                        element: Box::new(element),
                        is_map,
                    },
                    glossa_type: GlossaType::Boolean,
                };

                return Ok(Some((StatementKind::Query, vec![contains_expr])));
            }
        }

        // Regular query pattern
        let mut exprs = Vec::new();
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
        return Ok(Some((StatementKind::Query, exprs)));
    }
    Ok(None)
}
