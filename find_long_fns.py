import re
import os

for root, _, files in os.walk("src"):
    for file in files:
        if file.endswith(".rs"):
            with open(os.path.join(root, file), 'r') as f:
                lines = f.readlines()

            in_fn = False
            fn_name = ""
            fn_start = 0
            brace_count = 0

            for i, line in enumerate(lines):
                if re.match(r'^\s*(pub\s+|pub\([^)]+\)\s+)?(async\s+)?fn\s+([a-zA-Z0-9_]+)', line):
                    in_fn = True
                    m = re.match(r'^\s*(pub\s+|pub\([^)]+\)\s+)?(async\s+)?fn\s+([a-zA-Z0-9_]+)', line)
                    fn_name = m.group(3)
                    fn_start = i

                if in_fn:
                    brace_count += line.count('{')
                    brace_count -= line.count('}')

                    if brace_count == 0 and '{' in line:
                        pass
                    elif brace_count == 0 and i > fn_start:
                        length = i - fn_start + 1
                        if length > 50:
                            print(f"{length} lines: {os.path.join(root, file)} : {fn_name}")
                        in_fn = False
