import re

with open("tests/warden_coverage.rs", "r") as f:
    content = f.read()

content = content.replace("let res = glossa::semantic::analyze_program", "let _res = glossa::semantic::analyze_program")
content = content.replace(");", ");")
content = content.replace('let _res = glossa::semantic::analyze_program(&glossa::parser::parse("\n        ξ [1, 2, 3] ἔστω.\n        θ 10 ἔστω.\n        // Filter: collection + genitive(ου) + comparative_adj + print\n        ξ θου μείζονα λέγε.\n    ").unwrap());', 'compile(\n        "\n        ξ [1, 2, 3] ἔστω.\n        θ 10 ἔστω.\n        // Filter: collection + genitive(ου) + comparative_adj + print\n        ξ θου μείζονα λέγε.\n    ",\n    );')

with open("tests/warden_coverage.rs", "w") as f:
    f.write(content)
