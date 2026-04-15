import re

with open("src/semantic/conversion.rs", "r") as f:
    content = f.read()

# Replace extract_object_fallback
search = """fn extract_object_fallback(
    asm_stmt: &AssembledStatement,
    _scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(ref obj) = asm_stmt.object {
        let obj_lemma = &obj.lemma;

        // Check if it's a numeral word
        if let Some(value) = crate::morphology::lexicon::numeral_value(obj_lemma) {
            return Ok(Some((
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(value),
                    glossa_type: GlossaType::Number,
                },
                GlossaType::Number,
            )));
        }

        return Ok(Some(("""

replace = """fn extract_object_fallback(
    asm_stmt: &AssembledStatement,
    scope: &Scope,
) -> Result<Option<(AnalyzedExpr, GlossaType)>, GlossaError> {
    if let Some(ref obj) = asm_stmt.object {
        let obj_lemma = &obj.lemma;

        // Check if it's a numeral word
        if let Some(value) = crate::morphology::lexicon::numeral_value(obj_lemma) {
            return Ok(Some((
                AnalyzedExpr {
                    expr: AnalyzedExprKind::NumberLiteral(value),
                    glossa_type: GlossaType::Number,
                },
                GlossaType::Number,
            )));
        }

        if !scope.is_defined(obj_lemma) {
            return Err(GlossaError::undefined(obj_lemma.as_str()));
        }

        return Ok(Some(("""

content = content.replace(search, replace)

with open("src/semantic/conversion.rs", "w") as f:
    f.write(content)
