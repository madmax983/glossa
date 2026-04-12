import re

with open('src/semantic/assembly.rs', 'r') as f:
    content = f.read()

content = content.replace("use glossa::semantic::assembly::AssembledStatement;", "use glossa::semantic::AssembledStatement;")
content = content.replace("use glossa::semantic::assembly::Constituent;", "use glossa::semantic::Constituent;")

with open('src/semantic/assembly.rs', 'w') as f:
    f.write(content)

with open('src/semantic/mod.rs', 'r') as f:
    content = f.read()

content = content.replace("pub(crate) mod assembly;", "pub mod assembly;")

with open('src/semantic/mod.rs', 'w') as f:
    f.write(content)
