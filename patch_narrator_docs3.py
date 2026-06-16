import re

with open('src/tools/narrator.rs', 'r') as f:
    content = f.read()

# Make sure use std::fmt::Write; is present
if "use std::fmt::Write;" not in content:
    content = "use std::fmt::Write;\n" + content

with open('src/tools/narrator.rs', 'w') as f:
    f.write(content)
