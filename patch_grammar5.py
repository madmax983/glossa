import re
with open('src/parser/grammar.rs', 'r') as f:
    content = f.read()

content = content.replace('err_str.contains("Expected") || err_str.contains("expected") || err_str.contains("Σφάλμα") || err_str.to_lowercase().contains("error") || true', 'err_str.contains("Expected") || err_str.contains("expected") || err_str.contains("Σφάλμα") || err_str.to_lowercase().contains("error")')

with open('src/parser/grammar.rs', 'w') as f:
    f.write(content)
