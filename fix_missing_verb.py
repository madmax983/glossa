import re

with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

search = """        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            if subject.lemma == "ἄλλο" || subject.lemma == "μηδέν" || subject.lemma == "οὐδέν" || subject.lemma == "τι" || crate::morphology::lexicon::numeral_value(&subject.lemma).is_some() {
                return Ok(());
            }
        }
        Err(AssemblyError::MissingVerb)"""

replace = """        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            if subject.lemma == "ἄλλο" || subject.lemma == "μηδέν" || subject.lemma == "οὐδέν" || subject.lemma == "τι" || subject.lemma == "ἕν" || crate::morphology::lexicon::numeral_value(&subject.lemma).is_some() {
                return Ok(());
            }
        }
        Err(AssemblyError::MissingVerb)"""

content = content.replace(search, replace)

with open('src/semantic/assembly/mod.rs', 'w') as f:
    f.write(content)
