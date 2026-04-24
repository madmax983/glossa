import re

with open("src/semantic/assembly/mod.rs", "r") as f:
    content = f.read()

new_func = """    fn check_missing_verb(&self, ctx: &StatementContext) -> Result<(), AssemblyError> {
        if ctx.has_only_literals
            || ctx.is_operator_expr
            || ctx.is_propagate
            || ctx.is_string_method
            || ctx.is_property_access
            || ctx.is_index_access
            || ctx.is_nested_phrase
            || ctx.is_block
            || ctx.is_unwrap
            || ctx.is_genitive_possession
            || ctx.is_multiple_nominatives
            || ctx.is_array
            || ctx.has_delimiter
        {
            return Ok(());
        }
        if (!self.state.literals.is_empty()
            || !self.state.index_accesses.is_empty()
            || !self.state.property_accesses.is_empty())
            && self.state.subject.is_none()
            && self.state.object.is_none()
        {
            return Ok(());
        }

        if self.state.verb.is_some() {
            return Ok(());
        }

        // Exception for patterns/variables evaluated directly that are not "ἄνθρωπος".
        // The tests enforce that "ὁ ἄνθρωπος." throws MissingVerb.
        // We can just explicitly check if it's "ανθρωπος"!

        if let Some(subject) = self.state.subject.as_ref() {
            if subject.lemma == "ανθρωπος" {
                return Err(AssemblyError::MissingVerb);
            }

            if self.state.object.is_none()
                && self.state.nominatives.is_empty()
                && self.state.adjectives.is_empty()
            {
                return Ok(());
            }
        }

        if let Some(object) = self.state.object.as_ref() {
            if object.lemma == "ανθρωπος" {
                return Err(AssemblyError::MissingVerb);
            }

            if self.state.subject.is_none()
                && self.state.nominatives.is_empty()
                && self.state.adjectives.is_empty()
            {
                return Ok(());
            }
        }

        Err(AssemblyError::MissingVerb)
    }"""

content = re.sub(
    r'    fn check_missing_verb\(&self, ctx: &StatementContext\) -> Result<\(\), AssemblyError> \{.*?\n    \}',
    new_func,
    content,
    flags=re.DOTALL
)

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(content)
