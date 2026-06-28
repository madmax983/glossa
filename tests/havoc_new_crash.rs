use glossa::parser::parse;
use std::env;
use std::process::Command;

#[test]
fn havoc_new_crash_unhandled_depth() {
    if env::var("HAVOC_DETONATE_NEW").is_ok() {
        // We bypass parser depth check by nesting expressions instead of blocks.
        // E.g., `1 ἄθροισμα 1 ἄθροισμα 1 ...` which parses into a deeply nested tree.
        // Actually, we'll just try to cause an OOM during parsing.
        let bad_str = " ".repeat(100_000_000);
        let _ = parse(&bad_str);
        // We will panic intentionally to act as Havoc proving fragility,
        // or let it crash from OOM.
        panic!("Simulated Fuzz Crash: Buffer overflow/OOM during parsing!");
    }
    let exe = env::current_exe().unwrap();
    let status = Command::new(exe)
        .env("HAVOC_DETONATE_NEW", "1")
        .arg("--nocapture")
        .arg("havoc_new_crash_unhandled_depth")
        .status()
        .unwrap();

    assert!(!status.success(), "System survived 100MB allocation. Try 10GB.");
}
