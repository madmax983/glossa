import sys

def replace_in_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    search_str = """        let bin_path = std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
            let llvm_cov_path = "target/llvm-cov-target/debug/glossa";
            if std::path::Path::new(llvm_cov_path).exists() {
                llvm_cov_path.to_string()
            } else {
                "target/debug/glossa".to_string()
            }
        });"""

    replace_str = """        let bin_path = std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
            if let Ok(exe) = std::env::current_exe() {
                if exe.file_name().and_then(|s| s.to_str()).unwrap_or("").starts_with("glossa") {
                    return exe.display().to_string();
                }
            }
            let llvm_cov_path = "target/llvm-cov-target/debug/glossa";
            if std::path::Path::new(llvm_cov_path).exists() {
                llvm_cov_path.to_string()
            } else {
                "target/debug/glossa".to_string()
            }
        });"""

    if search_str in content:
        content = content.replace(search_str, replace_str)
        with open(filepath, 'w') as f:
            f.write(content)
        print("Success")
    else:
        print("Search string not found")

replace_in_file('src/tools/tester.rs')
