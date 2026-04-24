import re

with open("src/semantic/conversion.rs", "r") as f:
    content = f.read()

content = re.sub(
    r'''    \} else if let Some\(ref obj\) = asm_stmt\.object \{\n        if !scope\.is_defined\(&obj\.lemma\)\n            && crate::morphology::lexicon::numeral_value\(&obj\.lemma\)\.is_none\(\)\n            && obj\.lemma != "αληθες"\n            && obj\.lemma != "ψευδος"\n            && obj\.lemma != "self"\n            && obj\.lemma != "selfου"\n            && asm_stmt\.genitives\.is_empty\(\)\n            && !scope\.is_function\(&obj\.lemma\)\n            && scope\.lookup_type\(&obj\.lemma\)\.is_none\(\)\n            && !scope\.is_defined\("self"\)\n        \{\n            return Err\(GlossaError::undefined\(&\*obj\.original\)\);\n        \}\n    \}''',
    r'''    } else {
        #[allow(clippy::collapsible_if)]
        if let Some(ref obj) = asm_stmt.object {
            if !scope.is_defined(&obj.lemma)
                && crate::morphology::lexicon::numeral_value(&obj.lemma).is_none()
                && obj.lemma != "αληθες"
                && obj.lemma != "ψευδος"
                && obj.lemma != "self"
                && obj.lemma != "selfου"
                && asm_stmt.genitives.is_empty()
                && !scope.is_function(&obj.lemma)
                && scope.lookup_type(&obj.lemma).is_none()
                && !scope.is_defined("self")
            {
                return Err(GlossaError::undefined(&*obj.original));
            }
        }
    }''',
    content
)

with open("src/semantic/conversion.rs", "w") as f:
    f.write(content)
