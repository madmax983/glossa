with open("src/tools/alchemist.rs", "r") as f:
    content = f.read()

import re
content = re.sub(r'        let (mut |)_?program = AnalyzedProgram \{\n            statements: vec!\[\],\n            scope: crate::semantic::Scope::new\(\),\n        \};', '', content)

with open("src/tools/alchemist.rs", "w") as f:
    f.write(content)
