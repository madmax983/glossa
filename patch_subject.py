import re

with open("src/semantic/conversion.rs", "r") as f:
    code = f.read()

# Instead of not throwing an error...
# Oh! Is it panicking because of the swap?
# Let me look at the memory AGAIN:
# "In src/semantic/conversion.rs, when resolve_binding_target swaps Subject and Object due to one being undefined, ensure the bound variable is completely removed from the AST (fixed.object = None or fixed.subject = None) so that subsequent value extraction doesn't attempt to process the undefined name and panic."

code = code.replace(
"""        if scope.is_defined(&subject.lemma) && !scope.is_defined(&object.lemma) {
            let mut swapped = asm_stmt.clone();
            swapped.subject = Some(object.clone());
            swapped.object = Some(subject.clone());
            return Ok((object_name.to_string(), std::borrow::Cow::Owned(swapped)));
        } else {
            return Ok((
                subject_name.to_string(),
                std::borrow::Cow::Borrowed(asm_stmt),
            ));
        }""",
"""        if scope.is_defined(&subject.lemma) && !scope.is_defined(&object.lemma) {
            let mut swapped = asm_stmt.clone();
            swapped.subject = None;
            swapped.object = Some(subject.clone());
            return Ok((object_name.to_string(), std::borrow::Cow::Owned(swapped)));
        } else {
            let mut fixed = asm_stmt.clone();
            fixed.subject = None;
            return Ok((
                subject_name.to_string(),
                std::borrow::Cow::Owned(fixed),
            ));
        }"""
)

code = code.replace(
"""    // Default case: Bind to Subject
    if let Some(subject) = &asm_stmt.subject {
        return Ok((
            subject.normalized.to_string(),
            std::borrow::Cow::Borrowed(asm_stmt),
        ));
    }""",
"""    // Default case: Bind to Subject
    if let Some(subject) = &asm_stmt.subject {
        let mut fixed = asm_stmt.clone();
        fixed.subject = None;
        return Ok((
            subject.normalized.to_string(),
            std::borrow::Cow::Owned(fixed),
        ));
    }"""
)


with open("src/semantic/conversion.rs", "w") as f:
    f.write(code)
