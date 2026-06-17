import sys

def replace_in_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    search_str = """            if let Ok(exe) = std::env::current_exe() {
                if exe.file_name().and_then(|s| s.to_str()).unwrap_or("").starts_with("glossa") {
                    return exe.display().to_string();
                }
            }"""

    replace_str = """            if let Ok(exe) = std::env::current_exe() {
                let is_glossa = exe.file_name().and_then(|s| s.to_str()).unwrap_or("").starts_with("glossa");
                if is_glossa {
                    return exe.display().to_string();
                }
            }"""

    if search_str in content:
        content = content.replace(search_str, replace_str)
        with open(filepath, 'w') as f:
            f.write(content)
        print("Success")
    else:
        print("Search string not found")

replace_in_file('src/tools/tester.rs')
