import re

with open('src/semantic/assembly.rs', 'r') as f:
    content = f.read()

# Fix feed empty lines
content = re.sub(r'/// ```\n\n\s*#\[allow\(dead_code\)\]', r'/// ```\n    #[allow(dead_code)]', content)

# Fix feed_with_normalized empty lines
content = re.sub(r'/// ```\n\n\s*pub fn feed_with_normalized', r'/// ```\n    pub fn feed_with_normalized', content)

# Also fix any multiple /// without content inside
content = re.sub(r'///\n\s*///\n', r'///\n', content)

with open('src/semantic/assembly.rs', 'w') as f:
    f.write(content)
