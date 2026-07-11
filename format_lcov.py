import sys

def parse_lcov(file_path):
    lines = {}
    current_file = None
    with open(file_path, 'r') as f:
        for line in f:
            if line.startswith('SF:'):
                current_file = line.strip().split(':')[1]
                lines[current_file] = {'uncovered': set(), 'covered': set(), 'fn_uncovered': set()}
            elif line.startswith('DA:'):
                parts = line.strip().split(':')[1].split(',')
                line_num = int(parts[0])
                hits = int(parts[1])
                if hits == 0:
                    lines[current_file]['uncovered'].add(line_num)
                else:
                    lines[current_file]['covered'].add(line_num)
            elif line.startswith('FN:'):
                pass
            elif line.startswith('FNDA:'):
                parts = line.strip().split(':')[1].split(',')
                hits = int(parts[0])
                fn_name = parts[1]
                if hits == 0:
                    lines[current_file]['fn_uncovered'].add(fn_name)
    return lines

if __name__ == '__main__':
    data = parse_lcov('lcov.info')
    for file, info in data.items():
        if 'semantic/assembly/mod.rs' in file:
            print(f"File: {file}")
            print(f"Uncovered lines: {sorted(list(info['uncovered']))}")
            print(f"Uncovered functions: {sorted(list(info['fn_uncovered']))}")
            print("-" * 20)
