import re

with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

# Fix check_missing_verb
search = """        if ctx.is_match_arm
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

replace = """        if ctx.is_match_arm
            && self.state.object.is_none()
            && self.state.nominatives.is_empty()
            && self.state.adjectives.is_empty()
            && let Some(subject) = self.state.subject.as_ref()
        {
            // If it's a match arm and it consists solely of a subject that evaluates to a pattern or a numeral, it's valid
            if subject.lemma == "ἄλλο" || subject.lemma == "μηδέν" || subject.lemma == "οὐδέν" || subject.lemma == "τι" || subject.lemma == "ἕν" || crate::morphology::lexicon::numeral_value(&subject.lemma).is_some() {
                return Ok(());
            }
        }

        // Final fallback: in match arms, sometimes there's a subject and a subjunctive verb, but it might be caught here if verb is missing.
        // Wait, if it has a verb it wouldn't reach here. If it reaches here, verb is None.
        // What if subject is "ἓν" (one) or "ἄλλο"? This is covered above.
        // What if there is no subject, but only an adjective?
        if ctx.is_match_arm && self.state.subject.is_none() && self.state.verb.is_none() {
             return Ok(()); // Allow empty patterns or pattern components to be handled elsewhere
        }

        Err(AssemblyError::MissingVerb)"""

content = content.replace(search, replace)


with open('src/semantic/assembly/mod.rs', 'w') as f:
    f.write(content)
