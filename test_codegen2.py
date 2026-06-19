import re

with open("src/codegen.rs", "r") as f:
    text = f.read()

# maybe there are other recursive codegen functions?
# let's grep for generate_expr calling itself or others
