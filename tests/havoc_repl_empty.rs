use std::process::{Command, Stdio};
use std::io::Write;

// test wrapper for REPL crash
#[test]
fn havoc_repl_empty_panic_wrapper() {
    let mut child = Command::new("cargo")
        .args(["run", "--bin", "glossa", "--", "repl"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn glossa");

    let mut stdin = child.stdin.take().expect("Failed to open stdin");
    stdin.write_all(b"  // empty comments \n.exit\n").expect("Failed to write to stdin");
    drop(stdin);

    let output = child.wait_with_output().expect("Failed to wait on child");
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr_str.contains("panicked"), "Panic detected!");
}
