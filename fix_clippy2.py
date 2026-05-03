content = open('src/semantic/conversion.rs', 'r').read()

old_code = """    // Fallback: If no literals/ops, check Subject/Object
    if exprs.is_empty() {
        if let Some(ref subj) = asm_stmt.subject {
            if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() {
                if subj.lemma != "self" && subj.lemma != "selfου" && subj.lemma != "selfους" {
                    // Only enforce undefined if it's not a trait/struct context variable
                    if scope.types().next().is_none() {
                        return Err(GlossaError::undefined(subj.lemma.as_str()));
                    }
                }
            }
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        } else if let Some(ref obj) = asm_stmt.object {
            if !scope.is_defined(&obj.lemma) && crate::morphology::lexicon::numeral_value(&obj.lemma).is_none() {
                if obj.lemma != "self" && obj.lemma != "selfου" && obj.lemma != "selfους" {
                    // Only enforce undefined if it's not a trait/struct context variable
                    if scope.types().next().is_none() {
                        return Err(GlossaError::undefined(obj.lemma.as_str()));
                    }
                }
            }
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        }
    }"""

new_code = """    // Fallback: If no literals/ops, check Subject/Object
    if exprs.is_empty() {
        if let Some(ref subj) = asm_stmt.subject {
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        } else if let Some(ref obj) = asm_stmt.object {
            exprs.push(AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
                glossa_type: GlossaType::Unknown,
            });
        }
    }"""

if old_code in content:
    with open('src/semantic/conversion.rs', 'w') as f:
        f.write(content.replace(old_code, new_code))
    print("Replaced!")
else:
    print("Not found")

old_code2 = """            if args.is_empty() {
                if let Some(ref subj) = asm_stmt.subject {
                    if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() {
                        if scope.types().next().is_none() {
                            return Err(GlossaError::undefined(subj.lemma.as_str()));
                        }
                    }
                }
            }"""

new_code2 = """            if args.is_empty() && scope.types().next().is_none() {
                if let Some(ref subj) = asm_stmt.subject {
                    if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() {
                        return Err(GlossaError::undefined(subj.lemma.as_str()));
                    }
                }
            }"""

content = open('src/semantic/conversion.rs', 'r').read()
if old_code2 in content:
    with open('src/semantic/conversion.rs', 'w') as f:
        f.write(content.replace(old_code2, new_code2))
    print("Replaced 2!")
else:
    print("Not found 2")
