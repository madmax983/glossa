import sys
import re

def main():
    for filepath in sys.argv[1:]:
        with open(filepath, 'r') as f:
            lines = f.readlines()

        current_fn = None
        start_line = 0
        brace_count = 0
        in_fn = False

        # very naive parsing
        for i, line in enumerate(lines):
            # check for function start
            match = re.match(r'^\s*(?:pub\s+|pub\(crate\)\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(', line)
            if match and not in_fn:
                in_fn = True
                current_fn = match.group(1)
                start_line = i + 1
                brace_count = line.count('{') - line.count('}')
            elif in_fn:
                brace_count += line.count('{') - line.count('}')
                if brace_count <= 0:
                    in_fn = False
                    length = i + 1 - start_line + 1
                    if length > 50:
                        print(f"{filepath}:{start_line} fn {current_fn} - {length} lines")

if __name__ == "__main__":
    main()
