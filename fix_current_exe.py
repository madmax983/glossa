import re

with open('src/tools/tester.rs', 'r') as f:
    tester = f.read()

with open('src/tools/runner.rs', 'r') as f:
    runner = f.read()

bad = """            } else {
                std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
            }"""

# A better fallback if glossa is missing: return an error, or just return "glossa" assuming it's in PATH, or try the windows targets
good = """            } else if std::path::Path::new("target/debug/glossa.exe").exists() {
                "target/debug/glossa.exe".to_string()
            } else if std::path::Path::new("target/release/glossa.exe").exists() {
                "target/release/glossa.exe".to_string()
            } else {
                // If it can't find the binary, assume 'glossa' is in the PATH or fail naturally
                "glossa".to_string()
            }"""

tester = tester.replace(bad, good)
runner = runner.replace(bad, good)

with open('src/tools/tester.rs', 'w') as f:
    f.write(tester)

with open('src/tools/runner.rs', 'w') as f:
    f.write(runner)
