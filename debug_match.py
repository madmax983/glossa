import re

with open("src/semantic/assembly/mod.rs", "r") as f:
    code = f.read()

search_block = """        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            if subject.lemma == "ανθρωπος" {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }"""

replace_block = """        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            // Debugging
            if subject.lemma != "αλλος" && subject.lemma != "αλλο" && subject.lemma != "μηδεν" && subject.lemma != "εν" && subject.lemma != "ενα" {
                println!("MATCH ARM HIT WITH NO VERB! Lemma: {}", subject.lemma);
            }
            if subject.lemma == "ανθρωπος" {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }"""

code = code.replace(search_block, replace_block)

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(code)
