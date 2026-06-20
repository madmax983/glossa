import os
import re

def fix_alchemist():
    with open("src/tools/alchemist.rs", "r") as f:
        content = f.read()

    if "use std::fmt::Write;" not in content:
        content = content.replace("use std::path::Path;", "use std::path::Path;\nuse std::fmt::Write;")

    content = re.sub(r'out\.push_str\(&format!\((.*?)\)\);', r'let _ = write!(out, \1);', content)

    with open("src/tools/alchemist.rs", "w") as f:
        f.write(content)

def fix_labyrinth():
    with open("src/tools/labyrinth.rs", "r") as f:
        content = f.read()

    if "use std::fmt::Write;" not in content:
        content = content.replace("use std::path::Path;", "use std::path::Path;\nuse std::fmt::Write;")

    content = re.sub(r'out\.push_str\(&format!\((.*?)\)\);', r'let _ = write!(out, \1);', content)

    with open("src/tools/labyrinth.rs", "w") as f:
        f.write(content)

def fix_weave():
    with open("src/tools/weave.rs", "r") as f:
        content = f.read()

    if "use std::fmt::Write;" not in content:
        content = content.replace("use std::path::Path;", "use std::path::Path;\nuse std::fmt::Write;")

    content = re.sub(r'md\.push_str\(&format!\((.*?)\)\);', r'let _ = write!(md, \1);', content)

    with open("src/tools/weave.rs", "w") as f:
        f.write(content)

fix_alchemist()
fix_labyrinth()
fix_weave()
