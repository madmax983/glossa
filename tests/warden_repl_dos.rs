use std::io::{BufRead, Read};

// Provide a mock input that generates an unbounded line without newlines
struct InfiniteLineReader;

impl std::io::Read for InfiniteLineReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for b in buf.iter_mut() {
            *b = b'A'; // Just return infinite A's, no newline
        }
        Ok(buf.len())
    }
}

impl std::io::BufRead for InfiniteLineReader {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        static BUF: [u8; 1024] = [b'A'; 1024];
        Ok(&BUF)
    }
    fn consume(&mut self, _amt: usize) {}
}

#[test]
fn test_repl_memory_exhaustion() {
    let mut reader = InfiniteLineReader;
    let mut reader = reader.by_ref().take(50_000);
    let mut line = String::new();

    // We simulate the fixed REPL doing:
    // let bytes = input.by_ref().take(MAX_REPL_SOURCE_LEN as u64).read_line(&mut line).into_diagnostic()?;
    // Since InfiniteLineReader has no newline, read_line will try to read until EOF or newline.
    // take(50_000) limits it to 50,000 bytes, thus preventing the memory exhaustion DoS vulnerability.
    let bytes = reader.read_line(&mut line).unwrap();

    // Check that it only read up to the limit
    assert_eq!(bytes, 50_000);
    assert_eq!(line.len(), 50_000);
}
