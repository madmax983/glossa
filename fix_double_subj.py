import re

with open("src/semantic/assembly/mod.rs", "r") as f:
    code = f.read()

search_block = """            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_print_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_find_verb(&verb.lemma)
            {
                return Err(AssemblyError::DoubleSubject);
            }"""

replace_block = """            if !self.state.nominatives.is_empty()
                && self.state.operators.is_empty()
                && !crate::morphology::lexicon::is_binding_verb(&verb.lemma)
                && !crate::morphology::lexicon::is_find_verb(&verb.lemma)
            {
                return Err(AssemblyError::DoubleSubject);
            }"""

code = code.replace(search_block, replace_block)

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(code)
