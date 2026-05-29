#[cfg(feature = "nova")]
use glossa::tools::mentor::run_mentor_inner;
use glossa::tools::repl::run_repl_inner;
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex};
use std::thread;

struct EndlessStream {
    count: Arc<Mutex<usize>>,
}

impl Read for EndlessStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for b in buf.iter_mut() {
            *b = b'A'; // No newline
        }
        let mut c = self.count.lock().unwrap();
        *c += buf.len();
        Ok(buf.len())
    }
}

#[test]
fn test_repl_exhaustion_exploit() {
    let count = Arc::new(Mutex::new(0));
    let stream = EndlessStream {
        count: count.clone(),
    };

    let handle = thread::spawn(move || {
        let mut bounded = BufReader::new(stream.take(1_000_000));
        let mut output = Vec::new();
        let _ = run_repl_inner(&mut bounded, &mut output);
    });

    handle.join().unwrap();
    let final_count = *count.lock().unwrap();

    assert!(
        final_count < 100_000,
        "REPL is vulnerable to memory exhaustion. It read {} bytes without returning!",
        final_count
    );
}

#[cfg(feature = "nova")]
#[test]
fn test_mentor_exhaustion_exploit() {
    let count = Arc::new(Mutex::new(0));
    let stream = EndlessStream {
        count: count.clone(),
    };

    let handle = thread::spawn(move || {
        let mut bounded = BufReader::new(stream.take(1_000_000));
        let mut output = Vec::new();
        let _ = run_mentor_inner(&mut bounded, &mut output);
    });

    handle.join().unwrap();
    let final_count = *count.lock().unwrap();

    assert!(
        final_count < 100_000,
        "Mentor is vulnerable to memory exhaustion. It read {} bytes without returning!",
        final_count
    );
}
