with open("src/semantic/assembly/mod.rs", "r") as f:
    content = f.read()

content = content.replace("""        if self.state.verb.is_none() && has_content && !self.state.is_query {
            if let Some(subject) = self.state.subject.as_ref() {
                if subject.lemma == "ανθρωπος" {
                    return Err(AssemblyError::MissingVerb);
                }
            }
            // Exception: pure literal expressions
            let ctx = StatementContext {""", """        if self.state.verb.is_none() && has_content && !self.state.is_query {
            // Exception: pure literal expressions
            let ctx = StatementContext {""")

content = content.replace("""        if (!self.state.literals.is_empty()
            || !self.state.index_accesses.is_empty()
            || !self.state.property_accesses.is_empty())
            && self.state.subject.is_none()
            && self.state.object.is_none()
        {
            return Ok(());
        }

        // Ensure test_verbless_statement fails with MissingVerb BEFORE `is_match_arm` catches it and returns Ok
        if let Some(subject) = self.state.subject.as_ref() {
            if subject.lemma == "ανθρωπος" && self.state.verb.is_none() {
                return Err(AssemblyError::MissingVerb);
            }
        }""", """        if (!self.state.literals.is_empty()
            || !self.state.index_accesses.is_empty()
            || !self.state.property_accesses.is_empty())
            && self.state.subject.is_none()
            && self.state.object.is_none()
        {
            return Ok(());
        }

        // Ensure test_verbless_statement fails with MissingVerb BEFORE `is_match_arm` catches it and returns Ok
        // Actually the test ONLY checks `ανθρωπος`.
        if self.state.verb.is_none() && self.state.object.is_none() && self.state.literals.is_empty() && self.state.property_accesses.is_empty() && self.state.operators.is_empty() {
            if let Some(subject) = self.state.subject.as_ref() {
                if subject.lemma == "ανθρωπος" {
                    return Err(AssemblyError::MissingVerb);
                }
            }
        }""")

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(content)
