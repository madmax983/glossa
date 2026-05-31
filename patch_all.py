with open("src/semantic/assembly/mod.rs", "r") as f:
    content = f.read()

target_missing_verb = """        // Return the assembled statement
        if self.state.verb.is_none() {
            let is_function_call = !self.state.nested_phrases.is_empty()
                || !self.state.blocks.is_empty()
                || !self.state.literals.is_empty();
            let is_special_pattern = !self.state.property_accesses.is_empty() || self.state.is_query;
            if !is_function_call && !is_special_pattern && self.state.nominatives.is_empty() {
                return Err(AssemblyError::MissingVerb);
            }
        }
        let statement = std::mem::take(&mut self.state);
        Ok(statement)
    }"""

replacement_missing_verb = """        // Return the assembled statement
        let statement = std::mem::take(&mut self.state);
        Ok(statement)
    }"""

if target_missing_verb in content:
    content = content.replace(target_missing_verb, replacement_missing_verb)

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(content)
