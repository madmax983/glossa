#![cfg(feature = "nova")]
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
fn test_mentor_dos_unbounded_readline() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let mut input = BufReader::new(InfiniteA);
        let mut output = Vec::new();

        // This will hang and consume memory until OOM if not capped
        let _ = glossa::tools::mentor::run_mentor_inner(&mut input, &mut output);
        let _ = tx.send(());
    });

    match rx.recv_timeout(Duration::from_secs(2)) {
        Ok(_) => {
            // Finished!
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            panic!(
                "Test timed out! run_mentor_inner hung on infinite line, confirming DoS vulnerability."
            );
        }
        Err(_) => {
            panic!("Thread disconnected unexpectedly");
        }
    }
}

#[test]
fn test_mentor_dos_second_prompt() {
    let mut data = String::from("ξ 10 ἔστω.\n");
    data.push_str(&"A".repeat(50_005));

    let mut input = std::io::Cursor::new(data);
    let mut output = Vec::new();

    let _ = glossa::tools::mentor::run_mentor_inner(&mut input, &mut output);
}
