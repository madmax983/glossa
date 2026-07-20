import os
import re

def process_file(filepath):
    long_fns = []
    with open(filepath, 'r') as f:
        lines = f.readlines()

    in_fn = False
    fn_start = 0
    fn_name = ""
    brace_count = 0

    for i, line in enumerate(lines):
        if not in_fn:
            match = re.search(r'\bfn\s+([a-zA-Z0-9_]+)', line)
            if match:
                in_fn = True
                fn_start = i
                fn_name = match.group(1)
                brace_count = line.count('{') - line.count('}')
        else:
            brace_count += line.count('{') - line.count('}')
            if brace_count == 0:
                in_fn = False
                length = i - fn_start + 1
                if length > 50:
                    long_fns.append((fn_name, length, filepath, fn_start + 1, i + 1))
    return long_fns

all_long = []
for root, _, files in os.walk('src'):
    for file in files:
        if file.endswith('.rs'):
            filepath = os.path.join(root, file)
            all_long.extend(process_file(filepath))

all_long.sort(key=lambda x: x[1], reverse=True)
for fn in all_long[:30]:
    print(f"{fn[1]:>5} lines: {fn[0]} in {fn[2]}:{fn[3]}-{fn[4]}")
