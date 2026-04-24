import re

with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

# 1. Double subject fix: Ensure we don't trip when there's an unused find verb, and correctly exempt the functional tokens and numerals
search2 = """        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.state.subject, &self.state.verb) {
            self.check_agreement(subject, verb)?;
            // If we have a verb, a subject, and extra nominatives, but it's not a function definition or binary operation
            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_print_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_find_verb(&verb.lemma)
            {
                // Only throw DoubleSubject if the extra nominative is not a number or pronoun used functionally
                let mut invalid = true;
                for nom in &self.state.nominatives {
                    if nom.lemma == "ἄλλο" || nom.lemma == "μηδέν" || nom.lemma == "οὐδέν" || nom.lemma == "τι" || nom.lemma == "ἕν" || crate::morphology::lexicon::numeral_value(&nom.lemma).is_some() {
                        invalid = false;
                        break;
                    }
                }
                if invalid && (subject.lemma == "ἄλλο" || subject.lemma == "μηδέν" || subject.lemma == "οὐδέν" || subject.lemma == "τι" || subject.lemma == "ἕν" || crate::morphology::lexicon::numeral_value(&subject.lemma).is_some()) {
                    invalid = false;
                }
                if invalid {
                    return Err(AssemblyError::DoubleSubject);
                }
            }
        }"""

replace2 = """        // Check subject-verb agreement if both present
        if let (Some(subject), Some(verb)) = (&self.state.subject, &self.state.verb) {
            self.check_agreement(subject, verb)?;
            // If we have a verb, a subject, and extra nominatives, but it's not a function definition or binary operation
            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_print_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_find_verb(&verb.lemma)
            {
                // Only throw DoubleSubject if the extra nominative is not a number or pronoun used functionally
                let mut invalid = true;
                for nom in &self.state.nominatives {
                    if nom.lemma == "ἄλλο" || nom.lemma == "μηδέν" || nom.lemma == "οὐδέν" || nom.lemma == "τι" || nom.lemma == "ἕν" || crate::morphology::lexicon::numeral_value(&nom.lemma).is_some() {
                        invalid = false;
                        break;
                    }
                }
                if invalid && (subject.lemma == "ἄλλο" || subject.lemma == "μηδέν" || subject.lemma == "οὐδέν" || subject.lemma == "τι" || subject.lemma == "ἕν" || crate::morphology::lexicon::numeral_value(&subject.lemma).is_some()) {
                    invalid = false;
                }
                if invalid {
                    return Err(AssemblyError::DoubleSubject);
                }
            }
        }"""

content = content.replace(search2, replace2)

with open('src/semantic/assembly/mod.rs', 'w') as f:
    f.write(content)
