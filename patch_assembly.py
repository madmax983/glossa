import re

with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

content = re.sub(
    r'        if ctx\.is_match_arm\n            && self\.state\.object\.is_none\(\)\n            && self\.state\.nominatives\.is_empty\(\)\n            && self\.state\.adjectives\.is_empty\(\)\n            && let Some\(subject\) = self\.state\.subject\.as_ref\(\)\n        \{\n            if subject\.lemma == "ανθρωπος" \{\n                return Err\(AssemblyError::MissingVerb\);\n            \}\n            return Ok\(\(\)\);\n        \}',
    '',
    content,
    flags=re.DOTALL
)

replacement = """            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && self.state.adjectives.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_find_verb(&verb.lemma)"""

content = re.sub(
    r'            if !self\.state\.nominatives\.is_empty\(\)\n                && self\.state\.operators\.is_empty\(\)\n                && !crate::morphology::lexicon::is_binding_verb\(&verb\.lemma\)\n                && !crate::morphology::lexicon::is_print_verb\(&verb\.lemma\)\n                && !crate::morphology::lexicon::is_find_verb\(&verb\.lemma\)',
    replacement,
    content,
    flags=re.DOTALL
)

with open('src/semantic/assembly/mod.rs', 'w') as f:
    f.write(content)
