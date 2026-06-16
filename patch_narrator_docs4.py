import re

with open('src/tools/narrator.rs', 'r') as f:
    content = f.read()

content = content.replace("use std::fmt::Write;\n//! The Narrator Tool", "//! The Narrator Tool")
content = content.replace("and \"Notes\" (Properties).\n\nuse crate::semantic::CaptureMode;", "and \"Notes\" (Properties).\n\nuse std::fmt::Write;\nuse crate::semantic::CaptureMode;")

with open('src/tools/narrator.rs', 'w') as f:
    f.write(content)
