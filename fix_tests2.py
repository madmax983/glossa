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
        errors = re.findall(r"error\[E0277\]: can't compare `&lexicon::BinaryOp` with `lexicon::BinaryOp`.*?--> ([^:]+):(\d+):", err, re.DOTALL | re.MULTILINE)
        errors2 = re.findall(r"error\[E0308\]: mismatched types.*?--> ([^:]+):(\d+):", err, re.DOTALL | re.MULTILINE)
        errors.extend(errors2)

        if not errors:
            break

        err_set = set(errors)
        if err_set == prev_errors:
            print("Stuck!")
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

            if 'assert_eq!(op, ' in l:
                l = l.replace('assert_eq!(op, ', 'assert_eq!(*op, ')
            elif '*inner, GlossaType::Unknown' in l:
                l = l.replace('*inner, GlossaType::Unknown', '**inner, GlossaType::Unknown')

            lines[line_idx] = l

        for file_path, lines in modifications.items():
            with open(file_path, 'w') as f:
                f.writelines(lines)

run()
