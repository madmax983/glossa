use std::io::{BufRead, Read, Result as IoResult};

struct EndlessGarbage {
    bytes_read: usize,
    limit: usize,
    chunk: Vec<u8>,
}

impl EndlessGarbage {
    fn new(limit: usize) -> Self {
        Self {
            bytes_read: 0,
            limit,
            // 1MB chunks to speed up read_line's internal vector extensions
            chunk: vec![b'A'; 1024 * 1024],
        }
    }
}

impl Read for EndlessGarbage {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let remaining = self.limit - self.bytes_read;
        if remaining == 0 {
            return Ok(0);
        }
        let amt = std::cmp::min(buf.len(), remaining);
        buf[..amt].copy_from_slice(&self.chunk[..amt]);
        self.bytes_read += amt;
        Ok(amt)
    }
}

impl BufRead for EndlessGarbage {
    fn fill_buf(&mut self) -> IoResult<&[u8]> {
        let remaining = self.limit - self.bytes_read;
        if remaining == 0 {
            return Ok(&[]);
        }
        let amt = std::cmp::min(self.chunk.len(), remaining);
        Ok(&self.chunk[..amt])
    }

    fn consume(&mut self, amt: usize) {
        self.bytes_read += amt;
    }
}

#[test]
fn test_repl_memory_exhaustion() {
    // 20 billion limit ensures we either successfully unbounded read to string till failure (Havoc success)
    let limit = 20_000_000_000;

    // Memory exhaust trigger
    let mut bad_input = EndlessGarbage::new(limit);

    // Dev null writer
    let mut dev_null = std::io::sink();

    // The test is to prove memory exhaustion so `read_line` expands its buffer infinitely until limit.
    // If we OOM before reading all 20 billion bytes, we've demonstrated the vulnerability.
    let _ = glossa::tools::repl::run_repl_inner(&mut bad_input, &mut dev_null);
}
