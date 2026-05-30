use std::io::{Read};
use glossa::tools::repl::run_repl_inner;

struct ExploitReader {
    bytes_read: usize,
    limit: usize,
}

impl Read for ExploitReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.bytes_read > self.limit {
            panic!("Havoc 🧨: Buffer overflow! The system did not bound the reader. Read {} bytes without stopping.", self.bytes_read);
        }
        for b in buf.iter_mut() {
            *b = b' ';
        }
        self.bytes_read += buf.len();
        Ok(buf.len())
    }
}

#[test]
#[should_panic(expected = "Havoc 🧨")]
fn test_repl_dos_memory_exhaustion() {
    let exploit = ExploitReader { bytes_read: 0, limit: 100_000 };
    let mut reader = std::io::BufReader::new(exploit);
    let mut output = Vec::new();
    let _ = run_repl_inner(&mut reader, &mut output);
}
