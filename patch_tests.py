import os
import glob

def replace_in_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    new_content = content
    new_content = new_content.replace("glossa::tools::alchemist::{run_alchemist, transpile_to_python}", "glossa::tools::{run_alchemist, transpile_to_python}")
    new_content = new_content.replace("glossa::tools::alchemist::", "glossa::tools::")
    new_content = new_content.replace("glossa::tools::interpreter::Interpreter", "glossa::tools::Interpreter")
    new_content = new_content.replace("glossa::tools::mosaic::run_mosaic_inner", "glossa::tools::run_mosaic_inner")
    new_content = new_content.replace("glossa::tools::narrator::tell_tale", "glossa::tools::tell_tale")
    new_content = new_content.replace("glossa::tools::runner::run_file", "glossa::tools::run_file")
    new_content = new_content.replace("glossa::tools::tester::run_tests", "glossa::tools::run_tests")
    new_content = new_content.replace("glossa::tools::papyrus::run_papyrus", "glossa::tools::run_papyrus")
    new_content = new_content.replace("glossa::tools::weave::run_weave", "glossa::tools::run_weave")

    if content != new_content:
        with open(filepath, 'w') as f:
            f.write(new_content)

for filepath in glob.glob('tests/*.rs'):
    replace_in_file(filepath)
