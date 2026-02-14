use crate::errors::GlossaError;
use crate::semantic::patterns::detect_iterator_pattern;
use crate::semantic::{AnalyzedStatement, AssembledStatement, Scope};
use crate::text::normalize_greek;

/// Helper: Detect iterator pattern
pub fn classify_iterator_pattern(
    asm_stmt: &AssembledStatement,
    scope: &mut Scope,
) -> Result<Option<AnalyzedStatement>, GlossaError> {
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
        return Ok(Some(AnalyzedStatement::Print(vec![analyzed])));
    }

    Ok(None)
}
