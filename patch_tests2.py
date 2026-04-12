import os
import glob

def replace_in_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    new_content = content
    new_content = new_content.replace("glossa::tools::highlight::highlight", "glossa::tools::highlight")

    if content != new_content:
        with open(filepath, 'w') as f:
            f.write(new_content)

for filepath in glob.glob('tests/*.rs'):
    replace_in_file(filepath)
