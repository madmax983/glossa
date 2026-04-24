import re

with open("src/semantic/assembly/mod.rs", "r") as f:
    content = f.read()

# Make check_missing_verb coverage friendly!
# There are many clauses like:
#                if self.state.object.is_none() && self.state.nominatives.is_empty() && self.state.adjectives.is_empty() {
#                    return Err(AssemblyError::MissingVerb);
#                }
#            } else {
#                 if self.state.object.is_none() && self.state.nominatives.is_empty() && self.state.adjectives.is_empty() {
#                    return Ok(());
#                }
#            }

content = re.sub(
    r'''                // Is there anything else indicating a complex expression\?\n                if self\.state\.object\.is_none\(\) && self\.state\.nominatives\.is_empty\(\) && self\.state\.adjectives\.is_empty\(\) \{\n                    return Err\(AssemblyError::MissingVerb\);\n                \}\n            \} else \{\n                 if self\.state\.object\.is_none\(\) && self\.state\.nominatives\.is_empty\(\) && self\.state\.adjectives\.is_empty\(\) \{\n                    return Ok\(\(\)\);\n                \}\n            \}''',
    r'''                // Is there anything else indicating a complex expression?
                if self.state.object.is_none() && self.state.nominatives.is_empty() && self.state.adjectives.is_empty() {
                    return Err(AssemblyError::MissingVerb);
                }
            } else if self.state.object.is_none() && self.state.nominatives.is_empty() && self.state.adjectives.is_empty() {
                return Ok(());
            }''',
    content
)

content = re.sub(
    r'''                if self\.state\.subject\.is_none\(\) && self\.state\.nominatives\.is_empty\(\) && self\.state\.adjectives\.is_empty\(\) \{\n                    return Err\(AssemblyError::MissingVerb\);\n                \}\n            \} else \{\n                 if self\.state\.subject\.is_none\(\) && self\.state\.nominatives\.is_empty\(\) && self\.state\.adjectives\.is_empty\(\) \{\n                    return Ok\(\(\)\);\n                \}\n            \}''',
    r'''                if self.state.subject.is_none() && self.state.nominatives.is_empty() && self.state.adjectives.is_empty() {
                    return Err(AssemblyError::MissingVerb);
                }
            } else if self.state.subject.is_none() && self.state.nominatives.is_empty() && self.state.adjectives.is_empty() {
                return Ok(());
            }''',
    content
)

with open("src/semantic/assembly/mod.rs", "w") as f:
    f.write(content)
