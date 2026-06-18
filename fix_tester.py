import re

with open('src/tools/tester.rs', 'r') as f:
    content = f.read()

bad_fallback = """        // Spawn a child process so we don't mutate the global PATH/env.
        let bin_path = std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
            if std::path::Path::new("target/debug/glossa").exists() {
                "target/debug/glossa".to_string()
            } else if std::path::Path::new("target/release/glossa").exists() {
                "target/release/glossa".to_string()
            } else if std::path::Path::new("target/llvm-cov-target/debug/glossa").exists() {
                "target/llvm-cov-target/debug/glossa".to_string()
            } else {
                std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            }
        });"""

good_fallback = """        // Spawn a child process so we don't mutate the global PATH/env.
        let bin_path = std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
            if std::path::Path::new("target/debug/glossa").exists() {
                "target/debug/glossa".to_string()
            } else if std::path::Path::new("target/release/glossa").exists() {
                "target/release/glossa".to_string()
            } else if std::path::Path::new("target/llvm-cov-target/debug/glossa").exists() {
                "target/llvm-cov-target/debug/glossa".to_string()
            } else {
                "cargo".to_string() // Fallback to invoking via cargo run if glossa binary isn't directly found
            }
        });"""

content = content.replace(bad_fallback, good_fallback)

# Now fix the command logic. If bin_path is "cargo", we need to run `cargo run -- test ...`
# Let's adjust this. Actually, the original implementation was probably different. Let's look at the original code.
