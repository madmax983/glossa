import re

with open('src/semantic/assembly.rs', 'r') as f:
    content = f.read()

# Replace feed empty line with spaces
content = re.sub(r'/// ```\n\n\s*#\[allow', r'/// ```\n    #[allow', content)

# Replace feed_with_normalized empty lines with spaces
content = re.sub(r'/// ```\n\n\s*pub fn feed_with_normalized', r'/// ```\n    pub fn feed_with_normalized', content)

# And fix any extra empty doc comment lines
content = re.sub(r'///\n\s*///\n', r'///\n', content)

with open('src/semantic/assembly.rs', 'w') as f:
    f.write(content)
