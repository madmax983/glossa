import re

with open('src/semantic/assembly/mod.rs', 'r') as f:
    content = f.read()

content = re.sub(r'is_match_arm: [^,]+,', '', content)

with open('src/semantic/assembly/mod.rs', 'w') as f:
    f.write(content)
