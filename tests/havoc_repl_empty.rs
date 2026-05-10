#![allow(missing_docs)]
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

/// 👺 Havoc: Broken Pipe Panic in REPL
///
/// If a user runs the REPL and pipes the output to a program that closes
/// its stdin early (e.g., `head -n 0`), the REPL's `println!` will trigger
/// a Rust standard panic ("failed printing to stdout: Broken pipe").
#[test]
fn havoc_repl_empty_panic_wrapper() {
    let exe = env!("CARGO_BIN_EXE_glossa");

    let mut child = Command::new(exe)
        .arg("repl")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn glossa repl");

    // Close stdout immediately to cause a broken pipe for the child
    drop(child.stdout.take());

    // Send an input to the REPL to trigger it to print something and hit the broken pipe
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all("1 λέγε.\n".as_bytes());
    }

    let status = child.wait().expect("Failed to wait for child process");

    // The REPL should panic (which means exit code != 0) due to broken pipe.
    assert!(
        !status.success(),
        "Subprocess should have crashed due to broken pipe panic!"
    );
}
