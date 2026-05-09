use std::process::{Command, Stdio};
use std::io::Write;

fn main() {
    let mut child = Command::new("./target/debug/glossa")
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed");

    drop(child.stdout.take()); // Close stdout immediately!

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(b"").unwrap();
    }
    let status = child.wait().unwrap();
    println!("Status: {:?}", status);
}
