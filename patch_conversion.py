import re
with open("src/semantic/conversion.rs", "r") as f:
    text = f.read()

# Replace the fallback generation from literal NumberLiteral(0) with an UndefinedName error in extract_value
def replacer(match):
    return """return Err(GlossaError::undefined("Unknown Term".to_string()))"""

text = re.sub(
    r'return Err\(GlossaError::undefined\(constituent.original.clone\(\)\)\)',
    replacer,
    text
)

with open("src/semantic/conversion.rs", "w") as f:
    f.write(text)
