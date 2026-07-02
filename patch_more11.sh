#!/bin/bash
cat << 'PY_EOF' > patch_tools8.py
import re
with open('src/tools/mod.rs', 'r') as f:
    content = f.read()

exports = """
#[cfg(feature = "nova")]
pub use mentor::Lesson;
#[cfg(feature = "nova")]
pub use labyrinth::generate_cfg;
"""
if "pub use mentor::Lesson;" not in content:
    content += exports

with open('src/tools/mod.rs', 'w') as f:
    f.write(content)
PY_EOF
python3 patch_tools8.py
