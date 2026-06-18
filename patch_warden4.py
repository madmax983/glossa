import re

with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

replacement = """        if (!self.state.literals.is_empty()
            || !self.state.index_accesses.is_empty()
            || !self.state.property_accesses.is_empty())
            && self.state.subject.is_none()
            && self.state.object.is_none()
        {
            return Ok(());
        }

        // Allow single character variables (like ξ, χ, ν) to act as standalone expressions
        // without throwing MissingVerb, to support their use as implicit returns in blocks.
        if let Some(subj) = &self.state.subject {
            if subj.lemma.chars().count() == 1 {
                return Ok(());
            }
        }

        Err(AssemblyError::MissingVerb)"""

content = re.sub(
    r'        if \(!self\.state\.literals\.is_empty\(\)\n            \|\| !self\.state\.index_accesses\.is_empty\(\)\n            \|\| !self\.state\.property_accesses\.is_empty\(\)\)\n            && self\.state\.subject\.is_none\(\)\n            && self\.state\.object\.is_none\(\)\n        \{\n            return Ok\(\(\)\);\n        \}\n\n        Err\(AssemblyError::MissingVerb\)',
    replacement,
    content,
    flags=re.DOTALL
)

with open('src/semantic/assembly/mod.rs', 'w') as f:
    f.write(content)
