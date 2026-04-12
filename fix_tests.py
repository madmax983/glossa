import re
import subprocess
import os

def check():
    result = subprocess.run(['cargo', 'check', '--tests'], capture_output=True, text=True)
    return result.stderr

def run():
    prev_errors = set()
    while True:
        err = check()
        errors = re.findall(r"error\[E0509\]: cannot move out of type.*?--> ([^:]+):(\d+):", err, re.DOTALL | re.MULTILINE)

        if not errors:
            break

        err_set = set(errors)
        if err_set == prev_errors:
            print("Stuck on E0509!")
            break
        prev_errors = err_set

        modifications = {}
        for file_path, line in reversed(errors): # reverse to not mess up lines
            line_idx = int(line) - 1
            if file_path not in modifications:
                with open(file_path, 'r') as f:
                    modifications[file_path] = f.readlines()

            lines = modifications[file_path]
            l = lines[line_idx]

            if 'match ' in l and '&' not in l:
                l = l.replace('match ', 'match &')
            elif '= expr' in l and ' &' not in l:
                l = l.replace('= expr', '= &expr')
            elif '= analyzed' in l and ' &' not in l:
                l = l.replace('= analyzed', '= &analyzed')
            elif '= left' in l and ' &' not in l:
                l = l.replace('= left', '= &left')
            elif '= right' in l and ' &' not in l:
                l = l.replace('= right', '= &right')
            elif '= value' in l and ' &' not in l:
                l = l.replace('= value', '= &value')
            elif '= stmt' in l and ' &' not in l:
                l = l.replace('= stmt', '= &stmt')
            elif '= receiver' in l and ' &' not in l:
                l = l.replace('= receiver', '= &receiver')
            elif '= glossa_type' in l and ' &' not in l:
                l = l.replace('= glossa_type', '= &glossa_type')
            elif '= result' in l and ' &' not in l:
                l = l.replace('= result', '= &result')
            elif '.unwrap()' in l and '.as_ref().unwrap()' not in l:
                l = l.replace('.unwrap()', '.as_ref().unwrap()')

            lines[line_idx] = l

        for file_path, lines in modifications.items():
            with open(file_path, 'w') as f:
                f.writelines(lines)

run()
