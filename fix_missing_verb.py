import re

with open("src/semantic/assembly/mod.rs", "r") as f:
    code = f.read()

search_block = """        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            if crate::morphology::lexicon::numeral_value(&subject.lemma).is_none() && subject.lemma != "αλλος" && subject.lemma != "αλλο" && subject.lemma != "μηδεν" && subject.lemma != "τι" && subject.lemma != "τις" && subject.lemma != "τινα" && subject.lemma != "ουδεν" && subject.lemma != "ουδεις" {
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
            if subject.lemma == "ανθρωπος" || subject.lemma == "θεος" {
                return Err(AssemblyError::MissingVerb);
            }
            return Ok(());
        }"""

code = code.replace(search_block, replace_block)

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(code)
