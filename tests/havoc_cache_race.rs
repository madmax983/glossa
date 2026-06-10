#![allow(missing_docs)]

/// 👺 Havoc: TOCTOU Cache Race Condition
///
/// If multiple threads attempt to run `run_file` on identical sources
/// at the same time, `Cache::is_valid` does not use any file locking.
/// As a result, threads will concurrently write and invoke `rustc` on the
/// exact same `.glossa/cache/{hash}.rs` file, leading to internal compiler
/// errors or corrupted output files.
#[test]
fn havoc_cache_race() {
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::process::Command;

    // We cannot change the working directory globally or pollute the global file system,
    // so we spawn subprocesses using the glossa executable itself instead.
    let exe = std::env::current_exe().expect("Failed to get test exe path");
    let glossa_exe = exe.parent().unwrap().parent().unwrap().join("glossa");
    if !glossa_exe.exists() {
        // Skip if glossa hasn't been built
        return;
    }

    let source_code = "«Hello» λέγε.";
    let temp_dir = tempfile::tempdir().unwrap();
    let source_path = temp_dir.path().join("main.gl");
    std::fs::write(&source_path, source_code).unwrap();

    // Set HOME to temp dir so .glossa cache goes there, avoiding pollution
    let num_threads = 50;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = vec![];

    for _ in 0..num_threads {
        let b = Arc::clone(&barrier);
        let p = source_path.clone();
        let exe_clone = glossa_exe.clone();
        let home_dir = temp_dir.path().to_path_buf();
        handles.push(thread::spawn(move || {
            b.wait();
            Command::new(exe_clone)
                .arg(&p)
                .env("HOME", &home_dir)
                .output()
                .unwrap()
        }));
    }

    let mut failure_count = 0;
    for handle in handles {
        let output = handle.join().unwrap();
        if !output.status.success() {
            println!("Crash Trace (App Error): {:?}", String::from_utf8_lossy(&output.stderr));
            failure_count += 1;
        }
    }

    // Since this is a Havoc test designed to *prove* the vulnerability, we want
    // the test itself to return "ok" (pass) only if the breakage is successfully reproduced.
    // If we assert NO failures, cargo test marks it as a failure.
    // By convention in this repository (e.g., `havoc_codegen_stack_overflow.rs`), we
    // expect the bad thing to happen, so `failure_count > 0` should be asserted!
    assert!(
        failure_count > 0,
        "System survived cache race condition. Expected concurrency breakage!"
    );
}
