#![allow(missing_docs)]

use std::io::{BufReader, Read};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

struct InfiniteA;

impl Read for InfiniteA {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for b in buf.iter_mut() {
            *b = b'A'; // No newline
        }
        Ok(buf.len())
    }
}

#[test]
fn test_repl_dos_unbounded_readline() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut input = BufReader::new(InfiniteA);
        let mut output = Vec::new();

        // This will hang and consume memory until OOM if not capped
        let _ = glossa::tools::repl::run_repl_inner(&mut input, &mut output);
        let _ = tx.send(());
    });

    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(_) => {
            // Finished!
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!(
                "Test timed out! run_repl_inner hung on infinite line, confirming DoS vulnerability."
            );
        }
        Err(_) => {
            panic!("Thread disconnected unexpectedly");
        }
    }
}
