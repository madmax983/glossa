import re

with open('src/tools/mod.rs', 'r') as f:
    content = f.read()

# Make all `pub mod` into `pub(crate) mod` to encapsulate the tools.
# EXCEPT for `pub mod highlight;` because it is specifically mentioned in .jules/atlas.md as a required public API.
def replace_mod(m):
    name = m.group(1)
    if name == 'highlight':
        return f'pub mod {name};'
    return f'pub(crate) mod {name};'

new_content = re.sub(r'pub mod ([a-zA-Z0-9_]+);', replace_mod, content)

with open('src/tools/mod.rs', 'w') as f:
    f.write(new_content)
