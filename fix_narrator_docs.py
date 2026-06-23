import re

with open('src/tools/narrator.rs', 'r') as f:
    content = f.read()

if not content.startswith('//!'):
    content = '''//! The Narrator (ὁ Ἀφηγητής) - Semantic-to-English Translator
//!
//! This module implements the "Narrator" tool, affectionately known internally as the "Bard".
//! It translates the internal semantic logic of a ΓΛΩΣΣΑ program into a readable
//! English narrative ("The Scroll of Logic").
//!
//! # Purpose
//!
//! Ancient Greek is hard. This tool proves that the compiler understands the code
//! by restating it in plain English. It is an invaluable debugging tool for checking
//! if the semantic analysis phase correctly interpreted the free-word-order syntax.

''' + content

with open('src/tools/narrator.rs', 'w') as f:
    f.write(content)
