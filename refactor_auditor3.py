import re

with open('src/tools/auditor.rs', 'r') as f:
    content = f.read()

start_str = "struct AuditorVisitor {"
end_str = "        }\n    }\n}\n"

start_idx = content.find(start_str)
end_idx = content.find(end_str, start_idx) + len(end_str)
print("Found start:", start_idx != -1)
print("Found end:", end_idx != -1)
