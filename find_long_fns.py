import sys
import os
import re

def find_functions(file_path):
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    lines = content.split('\n')

    in_function = False
    brace_count = 0
    fn_name = ""
    fn_start = 0

    results = []

    for i, line in enumerate(lines):
        # Very simple heuristic: looking for "fn "
        if not in_function:
            match = re.search(r'\bfn\s+([a-zA-Z0-9_]+)\s*\(', line)
            if match:
                fn_name = match.group(1)
                fn_start = i
                in_function = True
                brace_count = line.count('{') - line.count('}')
        else:
            brace_count += line.count('{')
            brace_count -= line.count('}')

            if brace_count == 0:
                length = i - fn_start + 1
                if length > 50:
                    results.append((length, fn_name, file_path, fn_start + 1))
                in_function = False

    return results

all_results = []
for root, _, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            file_path = os.path.join(root, file)
            all_results.extend(find_functions(file_path))

all_results.sort(reverse=True, key=lambda x: x[0])

for r in all_results[:30]:
    print(f"{r[0]} lines: {r[1]} in {r[2]}:{r[3]}")
