use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[test]
fn test_scholar_dos_dev_zero() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let path = Path::new("/dev/zero");
        let result = glossa::tools::scholar::run_scholar(path);
        let _ = tx.send(result);
    });

    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(res) => {
            assert!(res.is_err(), "Should return error for /dev/zero");
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!("Test timed out! run_scholar hung on /dev/zero, confirming DoS vulnerability.");
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            panic!("Thread disconnected unexpectedly");
        }
    }
}
