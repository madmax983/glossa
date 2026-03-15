use std::env;
use std::fs;
use std::process::Command;

#[test]
fn havoc_trigger_pest_stack_overflow_with_unaccented_keyword() {
    // Write a malicious Glossa source file that will trigger the overflow using unaccented keywords
    let temp_dir = env::temp_dir();
    let source_file = temp_dir.join("havoc_repro_overflow.γλ");

    // Since `check_recursion_depth` manually scans for "δοκιμή" and "τέλος" using the exact
    // byte patterns of their accented versions (\xCE\xB4... and \xCF\x84...), it misses
    // unaccented variants like "δοκιμη" and "τελος" which are allowed by pest:
    // δοκιμή_keyword = @{ ("δοκιμή" | "δοκιμη") ~ !GREEK_CHAR }
    // τέλος_keyword = @{ ("τέλος" | "τελος") ~ !GREEK_CHAR }
    //
    // This allows an attacker to bypass the manual recursion depth check and trigger
    // a stack overflow in pest's recursive descent parser.
    let payload = "δοκιμη «test».".repeat(10000) + &" τελος.".repeat(10000);

    fs::write(&source_file, payload).expect("Failed to write malicious source file");

    // We invoke the glossa CLI binary (which is built alongside tests) to parse the malicious script.
    // We use the `cargo run` command to ensure the local binary is used.
    let output = Command::new(env!("CARGO"))
        .arg("run")
        .arg("--")
        .arg("check")
        .arg(&source_file)
        .output()
        .expect("Failed to execute `cargo run`");

    // The process should crash with a SIGABRT or similar signal due to a stack overflow.
    // It should NOT exit cleanly with a "Parse error: Recursion limit exceeded" error message.
    let exit_code = output.status.code();

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    // If it exited cleanly with a custom error, the vulnerability is fixed.
    // If it crashed, `status.code()` is usually `None` on Unix (terminated by signal), or a specific crash code.
    // In Rust, stack overflow usually aborts the process (SIGABRT, signal 6).
    // On Windows, a stack overflow results in exit code 0xc00000fd.
    let is_stack_overflow = stderr.contains("stack overflow")
        || stderr.contains("SIGABRT")
        || output.status.code().is_none()
        || exit_code == Some(0xc00000fd_u32 as i32);

    assert!(
        exit_code.is_none() || exit_code != Some(0) || is_stack_overflow,
        "Expected the process to crash with a stack overflow, but it exited normally or caught the error!\nExit code: {:?}\nStderr: {}\nStdout: {}",
        exit_code,
        stderr,
        stdout
    );

    // Specifically verify it's a stack overflow and not a normal handled error
    assert!(
        is_stack_overflow,
        "Vulnerability failed: Process did not encounter a stack overflow!\nExit code: {:?}\nStderr: {}\nStdout: {}",
        exit_code, stderr, stdout
    );

    // Cleanup
    let _ = fs::remove_file(source_file);
}
