import re

with open('src/semantic/conversion.rs', 'r') as f:
    content = f.read()

replacement = """    let mut args =
        build_expressions_from_literals_and_ops(&asm_stmt.literals, &asm_stmt.operators)?;

    if let Some(ref subj) = asm_stmt.subject {
        let var_type = scope.lookup(&subj.lemma).cloned().unwrap_or(GlossaType::Unknown);
        if !scope.is_defined(&subj.lemma) && !scope.is_function(&subj.lemma) && subj.lemma.chars().count() > 1 {
            return Err(GlossaError::undefined(subj.lemma.as_str()));
        }
        args.insert(
            0,
            AnalyzedExpr {
                expr: AnalyzedExprKind::Variable(subj.lemma.clone()),
                glossa_type: var_type,
            },
        );
    }

    if let Some(ref obj) = asm_stmt.object {
        let var_type = scope.lookup(&obj.lemma).cloned().unwrap_or(GlossaType::Unknown);
        if !scope.is_defined(&obj.lemma) && !scope.is_function(&obj.lemma) && obj.lemma.chars().count() > 1 {
            return Err(GlossaError::undefined(obj.lemma.as_str()));
        }
        args.push(AnalyzedExpr {
            expr: AnalyzedExprKind::Variable(obj.lemma.clone()),
            glossa_type: var_type,
        });
    }"""

content = re.sub(
    r'    let mut args =\n        build_expressions_from_literals_and_ops\(&asm_stmt\.literals, &asm_stmt\.operators\)\?;\n\n    if let Some\(ref subj\) = asm_stmt\.subject \{\n.*?args\.push\(AnalyzedExpr \{\n.*?expr: AnalyzedExprKind::Variable\(obj\.lemma\.clone\(\)\),\n.*?glossa_type: var_type\.clone\(\),\n.*?\n.*?\}',
    replacement,
    content,
    flags=re.DOTALL
)

with open('src/semantic/conversion.rs', 'w') as f:
    f.write(content)
