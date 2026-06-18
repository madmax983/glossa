import re

with open('src/tools/mod.rs', 'r') as f:
    mod = f.read()

# Remove the empty line
mod = mod.replace("/// A bounded read_line to prevent DoS via infinite stream memory exhaustion\n\n\n/// Centralized resolution", "/// Centralized resolution")

with open('src/tools/mod.rs', 'w') as f:
    f.write(mod)
