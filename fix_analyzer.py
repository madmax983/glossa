import re

with open("src/semantic/analyzer.rs", "r") as f:
    content = f.read()

content = content.replace("return Err(GlossaError::AssemblyError(e));", "return Err(e);")

with open("src/semantic/analyzer.rs", "w") as f:
    f.write(content)
