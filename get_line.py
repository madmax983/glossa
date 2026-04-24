import subprocess

def test():
    with open("src/semantic/assembly/mod.rs", "r") as f:
        content = f.read()

    import re
    content = re.sub(
        r'    fn check_missing_verb\(&self, ctx: &StatementContext\) -> Result<\(\), AssemblyError> \{.*?\n    \}',
        r'''    fn check_missing_verb(&self, ctx: &StatementContext) -> Result<(), AssemblyError> {
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

        eprintln!("MissingVerb: stmt={:?}", self.state);
        Err(AssemblyError::MissingVerb)
    }''',
        content,
        flags=re.DOTALL
    )
    with open("src/semantic/assembly/mod.rs", "w") as f:
        f.write(content)

    subprocess.run(["cargo", "test", "--test", "warden_nested_phrase", "test_function_definition_scope", "--", "--nocapture"])

test()
