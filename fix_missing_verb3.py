content = open('src/semantic/assembly/mod.rs', 'r').read()

old_code = """        if ctx.has_only_literals
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
            if self.state.subject.is_some() && self.state.object.is_none() && self.state.literals.is_empty() && self.state.operators.is_empty() && !self.state.is_query {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }"""

new_code = """        if ctx.has_only_literals
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
            if self.state.subject.is_some() && self.state.object.is_none() && self.state.literals.is_empty() && self.state.operators.is_empty() && !self.state.is_query && !ctx.is_string_method && !ctx.is_genitive_possession {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }"""

if old_code in content:
    with open('src/semantic/assembly/mod.rs', 'w') as f:
        f.write(content.replace(old_code, new_code))
    print("Replaced!")
else:
    print("Not found")
