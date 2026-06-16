import re

with open('src/tools/narrator.rs', 'r') as f:
    content = f.read()

content = content.replace("use std::fmt::Write;\n", "")

with open('src/tools/narrator.rs', 'w') as f:
    f.write(content)
