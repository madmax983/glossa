import re
with open('src/semantic/control_flow.rs', 'r') as f:
    content = f.read()

replacement = """    // Parse the modified clause as a statement
    let stmt = Statement::Regular {
        clauses: vec![modified_clause],
        is_query: false,
        is_propagate: false,
    };

    let analyzed_res = assemble_statement(&stmt);
    let analyzed = match analyzed_res {
        Ok(a) => a,
        Err(crate::errors::AssemblyError::MissingVerb) => {
            // Fallback for single word expressions (like `ξ`) which throw MissingVerb during assembly
            if let Statement::Regular { clauses, .. } = &stmt {
                if clauses.len() == 1 && clauses[0].expressions.len() == 1 {
                    let e = &clauses[0].expressions[0];
                    let word_opt = if let Expr::Word(w) = e {
                        Some(w)
                    } else if let Expr::Phrase(terms) = e {
                        if terms.len() == 1 {
                            if let Expr::Word(w) = &terms[0] {
                                Some(w)
                            } else { None }
                        } else { None }
                    } else { None };

                    if let Some(w) = word_opt {
                        let var_type = scope.lookup(&w.normalized).cloned().unwrap_or(crate::semantic::GlossaType::Unknown);
                        return Ok(AnalyzedExpr {
                            expr: crate::semantic::AnalyzedExprKind::Variable(w.normalized.clone()),
                            glossa_type: var_type,
                        });
                    }
                }
            }
            return Err(GlossaError::AssemblyError(crate::errors::AssemblyError::MissingVerb));
        }
        Err(e) => return Err(GlossaError::AssemblyError(e)),
    };

    // We bypass the top-level MissingVerb check by extracting the expression manually"""

content = re.sub(
    r'    // Parse the modified clause as a statement\n    let stmt = Statement::Regular \{\n        clauses: vec!\[modified_clause\],\n        is_query: false,\n        is_propagate: false,\n    \};\n    let analyzed = assemble_statement\(&stmt\)\?;\n\n    // We bypass the top-level MissingVerb check by extracting the expression manually',
    replacement,
    content,
    flags=re.DOTALL
)

with open('src/semantic/control_flow.rs', 'w') as f:
    f.write(content)
