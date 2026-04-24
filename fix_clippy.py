import re

with open('src/semantic/conversion.rs', 'r') as f:
    content = f.read()

search = """    // Check for undefined subject before defaulting
    if let Some(ref subj) = asm_stmt.subject {
        if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() {
            return Err(GlossaError::undefined(subj.lemma.as_str()));
        }
    }"""

replace = """    // Check for undefined subject before defaulting
    #[allow(clippy::collapsible_if)]
    if let Some(ref subj) = asm_stmt.subject {
        if !scope.is_defined(&subj.lemma) && crate::morphology::lexicon::numeral_value(&subj.lemma).is_none() {
            return Err(GlossaError::undefined(subj.lemma.as_str()));
        }
    }"""

content = content.replace(search, replace)

with open('src/semantic/conversion.rs', 'w') as f:
    f.write(content)
