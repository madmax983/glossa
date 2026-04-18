import re

with open('src/semantic/assembly.rs', 'r') as f:
    content = f.read()

# Just wipe out the bad docs and rewrite them without ANY newlines around them
content = re.sub(
    r'(\s*)/// Feeds an AST element into the assembler\.(.*?)(#\[allow\(dead_code\)\])?\s*pub fn feed\(',
    r'\1/// Feeds an AST element into the assembler.\n\1///\n\1/// It parses a string into morphologic traits and saves it to the ongoing statement structure. It exists as the primary interface to collect terms.\n\1///\n\1/// # Examples\n\1///\n\1/// ```rust,ignore\n\1/// asm.feed(&analysis, "λόγος").unwrap();\n\1/// ```\n\1#[allow(dead_code)]\n\1pub fn feed(',
    content,
    flags=re.DOTALL
)

content = re.sub(
    r'(\s*)/// Feeds an element into the assembler using its normalized form directly\.(.*?)pub fn feed_with_normalized\(',
    r'\1/// Feeds an element into the assembler using its normalized form directly.\n\1///\n\1/// This is a zero-allocation path when the normalized form is already known (e.g. from AST).\n\1/// It bypasses the costly `normalize_greek` call which may allocate strings.\n\1///\n\1/// # Examples\n\1///\n\1/// ```rust,ignore\n\1/// asm.feed_with_normalized(&analysis, "λογος", "λόγος").unwrap();\n\1/// ```\n\1pub fn feed_with_normalized(',
    content,
    flags=re.DOTALL
)

with open('src/semantic/assembly.rs', 'w') as f:
    f.write(content)
