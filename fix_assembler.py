import re

with open("src/semantic/assembly/mod.rs", "r") as f:
    content = f.read()

# restore DoubleSubject check for adjectives.
new_func = """        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.state.subject, &self.state.verb) {
            self.check_agreement(subject, verb)?;
            // If we have a verb, a subject, and extra nominatives, but it's not a function definition or binary operation
            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_print_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_find_verb(&verb.lemma)
            {
                return Err(AssemblyError::DoubleSubject);
            }
        }"""

content = re.sub(
    r'        // Check subject-verb agreement if both present\n        if let \(Some\(subject\), Some\(verb\)\) = \(&self\.state\.subject, &self\.state\.verb\) \{.*?\n        \}',
    new_func,
    content,
    flags=re.DOTALL
)

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(content)
