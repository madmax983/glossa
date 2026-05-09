use std::process::{Command, Stdio};

#[test]
fn havoc_repl_empty_panic_wrapper() {
    let bin_path = std::env::var("CARGO_BIN_EXE_glossa").unwrap_or_else(|_| {
        let mut path = std::env::current_exe().unwrap();
        path.pop();
        if path.ends_with("deps") {
            path.pop();
        }
        path.push("glossa");
        path.to_str().unwrap().to_string()
    });

    let mut child = Command::new(bin_path)
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start REPL");

    // Detonate: Close stdout immediately to cause a broken pipe crash
    drop(child.stdout.take());

    // We write to stdin to trigger a read_line, which will print to stdout and crash
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(b" \n");
    }

    let status = child.wait().unwrap();

    // The test SUCCEEDS if the REPL CRASHED
    assert!(!status.success(), "Havoc expected the REPL to crash due to broken pipe!");
}
