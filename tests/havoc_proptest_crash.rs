use std::env;
use std::process::Command;

#[test]
fn havoc_crash_proof() {
    if env::var("HAVOC_DETONATE_PROPTEST").is_ok() {
        // Just intentionally stack overflow
        #[allow(unconditional_recursion)]
        fn overflow() {
            overflow();
        }
        overflow();
        std::process::exit(0);
    }

    let exe = env::current_exe().expect("Failed to get current executable");

    let status = Command::new(exe)
        .env("HAVOC_DETONATE_PROPTEST", "1")
        .arg("--nocapture")
        .arg("havoc_crash_proof")
        .status()
        .expect("Failed to spawn subprocess");

    // The test SUCCEEDS if the subprocess CRASHED
    assert!(!status.success(), "Subprocess should have crashed!");
}
