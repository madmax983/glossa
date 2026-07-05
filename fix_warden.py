import sys

def replace_in_file(filepath, search, replace):
    with open(filepath) as f:
        content = f.read()
    content = content.replace(search, replace)
    with open(filepath, 'w') as f:
        f.write(content)

replace_in_file("tests/warden_hashdos.rs", "key.chars().all(|c| c.is_ascii_hexdigit())", "key.chars().all(|c: char| c.is_ascii_hexdigit())")
