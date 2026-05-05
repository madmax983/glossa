import os

mod_path = "src/errors/mod.rs"

with open(mod_path, "r") as f:
    mod_code = f.read()

# Fix module loading and imports
mod_code = mod_code.replace("pub(crate) mod assembly;", "")

# Fix duplicate imports at the bottom
lines = mod_code.split('\n')
new_lines = []
for line in lines:
    if line.startswith("//!") and "Assembly Error Types" in line:
        continue
    if line == "//!":
        continue
    if line.startswith("//!") and "Defines errors that can occur" in line:
        continue
    if line == "use miette::Diagnostic;" and new_lines.count("use miette::{Diagnostic, SourceSpan};") > 0:
        continue
    if line == "use thiserror::Error;" and new_lines.count("use thiserror::Error;") > 0:
        continue
    new_lines.append(line)

mod_code = "\n".join(new_lines)
with open(mod_path, "w") as f:
    f.write(mod_code)

print("Fixed errors")
