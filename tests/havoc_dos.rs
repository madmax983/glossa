use std::path::Path;
use std::thread;
use std::time::Duration;
use std::sync::mpsc;

#[test]
fn test_dos_dev_zero() {
    // This test attempts to read from /dev/zero, which is an infinite stream.
    // The current implementation of load_source uses fs::read_to_string, which reads until EOF.
    // This should cause the thread to hang (or OOM eventually).
    // We use a timeout to detect the hang.

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let path = Path::new("/dev/zero");
        // We expect run_file to fail either due to file size check (if fixed) or some other error.
        // If it hangs, it won't return.
        let result = glossa::tools::runner::run_file(path);
        // Send the result back
        // If run_file hangs, this line is never reached.
        let _ = tx.send(result);
    });

    // Wait for 2 seconds. If no result, we assume it hung.
    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(res) => {
            // It returned! Check if it errored (expected).
            // Note: If the fix is NOT implemented, it might return OOM error if memory is small enough,
            // or it might just hang.
            // If it returns Ok, that's definitely wrong (it compiled infinite zeros??).
            assert!(res.is_err(), "Should return error for /dev/zero");
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            // It timed out! This means it hung reading the file.
            // This confirms the vulnerability.
            // We panic to fail the test, as per "Red Phase".
            panic!("Test timed out! run_file hung on /dev/zero, confirming DoS vulnerability.");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("Thread disconnected unexpectedly");
        }
    }
}
