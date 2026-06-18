import re

with open('src/tools/mod.rs', 'r') as f:
    mod = f.read()

mod = mod.replace("#[cfg_attr(tarpaulin, coverage(off))]\n", "")

with open('src/tools/mod.rs', 'w') as f:
    f.write(mod)
