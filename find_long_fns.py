import os
import sys
import re

def main():
    root = "src"
    results = []
    for dirpath, dirnames, filenames in os.walk(root):
        for filename in filenames:
            if not filename.endswith(".rs"):
                continue
            path = os.path.join(dirpath, filename)
            with open(path, "r", encoding="utf-8") as f:
                lines = f.readlines()

            in_fn = False
            fn_name = ""
            fn_start = 0
            brace_count = 0

            for i, line in enumerate(lines):
                # Basic heuristic
                if not in_fn:
                    m = re.match(r'^\s*(?:pub(?:\([^)]+\))?\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(', line)
                    if m:
                        in_fn = True
                        fn_name = m.group(1)
                        fn_start = i
                        brace_count = line.count("{") - line.count("}")
                else:
                    brace_count += line.count("{") - line.count("}")
                    if brace_count <= 0:
                        fn_len = i - fn_start + 1
                        if fn_len > 50:
                            results.append((fn_len, fn_name, path, fn_start + 1))
                        in_fn = False

    results.sort(key=lambda x: x[0], reverse=True)
    for res in results[:20]:
        print(f"{res[0]} lines: {res[1]} in {res[2]}:{res[3]}")

if __name__ == "__main__":
    main()
