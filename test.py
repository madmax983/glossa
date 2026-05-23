import re
with open('src/tools/gnomon.rs', 'r') as f:
    content = f.read()
print("GnomonVisitor" in content)
