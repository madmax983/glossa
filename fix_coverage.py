import re

with open('src/tools/runner.rs', 'r') as f:
    runner = f.read()

with open('src/tools/tester.rs', 'r') as f:
    tester = f.read()

with open('src/tools/mod.rs', 'r') as f:
    mod = f.read()

fallback_pattern_runner = r"""        let bin_path = std::env::var\("CARGO_BIN_EXE_glossa"\)\.unwrap_or_else\(\|\_\| \{
            if std::path::Path::new\("target/debug/glossa"\)\.exists\(\) \{
                "target/debug/glossa"\.to_string\(\)
            \} else if std::path::Path::new\("target/release/glossa"\)\.exists\(\) \{
                "target/release/glossa"\.to_string\(\)
            \} else if std::path::Path::new\("target/llvm-cov-target/debug/glossa"\)\.exists\(\) \{
                "target/llvm-cov-target/debug/glossa"\.to_string\(\)
            \} else if std::path::Path::new\("target/debug/glossa\.exe"\)\.exists\(\) \{
                "target/debug/glossa\.exe"\.to_string\(\)
            \} else if std::path::Path::new\("target/release/glossa\.exe"\)\.exists\(\) \{
                "target/release/glossa\.exe"\.to_string\(\)
            \} else \{
                // If it can't find the binary, assume 'glossa' is in the PATH or fail naturally
                "glossa"\.to_string\(\)
            \}
        \}\);"""

new_runner = '        let bin_path = crate::tools::find_glossa_binary();'

runner = re.sub(fallback_pattern_runner, new_runner, runner)
tester = re.sub(fallback_pattern_runner, new_runner, tester)

mod_addition = """

/// Centralized resolution of the glossa binary path for spawning subprocesses
#[cfg_attr(tarpaulin, coverage(off))]
pub(crate) fn find_glossa_binary() -> String {
    std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
        if std::path::Path::new("target/debug/glossa").exists() {
            "target/debug/glossa".to_string()
        } else if std::path::Path::new("target/release/glossa").exists() {
            "target/release/glossa".to_string()
        } else if std::path::Path::new("target/llvm-cov-target/debug/glossa").exists() {
            "target/llvm-cov-target/debug/glossa".to_string()
        } else if std::path::Path::new("target/debug/glossa.exe").exists() {
            "target/debug/glossa.exe".to_string()
        } else if std::path::Path::new("target/release/glossa.exe").exists() {
            "target/release/glossa.exe".to_string()
        } else {
            "glossa".to_string()
        }
    })
}
"""

if "pub(crate) fn find_glossa_binary" not in mod:
    mod = mod.replace("pub(crate) fn read_line_bounded", mod_addition + "\npub(crate) fn read_line_bounded")

with open('src/tools/runner.rs', 'w') as f:
    f.write(runner)

with open('src/tools/tester.rs', 'w') as f:
    f.write(tester)

with open('src/tools/mod.rs', 'w') as f:
    f.write(mod)
