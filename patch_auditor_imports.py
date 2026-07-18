import re

with open("src/tools/auditor.rs", "r") as f:
    code = f.read()

search = """
#[cfg(test)]
mod tests {
    use super::*;
    use super::{visit_expr, visit_statement};
    use std::io::Write;
"""

replace = """
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
"""

code = code.replace(search, replace)

with open("src/tools/auditor.rs", "w") as f:
    f.write(code)
