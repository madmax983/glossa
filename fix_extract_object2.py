content = open('src/semantic/conversion.rs', 'r').read()

old_code = """    if !scope.is_defined(obj_lemma) {
        return Err(GlossaError::undefined(obj_lemma.as_str()));
    }"""

new_code = """    if !scope.is_defined(obj_lemma) {
        if obj_lemma != "self" && obj_lemma != "selfου" && obj_lemma != "selfους" {
            if scope.types().next().is_none() {
                return Err(GlossaError::undefined(obj_lemma.as_str()));
            }
        }
    }"""

if old_code in content:
    with open('src/semantic/conversion.rs', 'w') as f:
        f.write(content.replace(old_code, new_code))
    print("Replaced!")
else:
    print("Not found")
