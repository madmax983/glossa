import re

with open("src/semantic/conversion/mod.rs", "r") as f:
    lines = f.readlines()

new_lines = []
for line in lines:
    if line.startswith("#[cfg(test)]"):
        new_lines.append(line)
        new_lines.append("use crate::semantic::{Constituent, Literal, GlossaType, AnalyzedExprKind};\n")
    else:
        new_lines.append(line)

with open("src/semantic/conversion/mod.rs", "w") as f:
    f.writelines(new_lines)
