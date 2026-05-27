#![no_main]
use libfuzzer_sys::fuzz_target;

struct FuzzReader<'a> {
    data: &'a [u8],
    pos: usize,
    repeats: usize,
    max_repeats: usize,
}

impl<'a> FuzzReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            repeats: 0,
            max_repeats: 500_000,
        }
    }
}

impl<'a> std::io::Read for FuzzReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.data.is_empty() {
            return Ok(0);
        }
        if self.repeats >= self.max_repeats {
            return Ok(0);
        }

        let mut bytes_written = 0;
        while bytes_written < buf.len() && self.repeats < self.max_repeats {
            let available = self.data.len() - self.pos;
            let space = buf.len() - bytes_written;
            let to_write = std::cmp::min(available, space);

            buf[bytes_written..bytes_written + to_write].copy_from_slice(&self.data[self.pos..self.pos + to_write]);
            bytes_written += to_write;
            self.pos += to_write;

            if self.pos >= self.data.len() {
                self.pos = 0;
                self.repeats += 1;
            }
        }
        Ok(bytes_written)
    }
}

impl<'a> std::io::BufRead for FuzzReader<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.data.is_empty() || self.repeats >= self.max_repeats {
            return Ok(&[]);
        }
        Ok(&self.data[self.pos..])
    }

    fn consume(&mut self, amt: usize) {
        let mut remaining = amt;
        while remaining > 0 {
            let available = self.data.len() - self.pos;
            if remaining < available {
                self.pos += remaining;
                remaining = 0;
            } else {
                remaining -= available;
                self.pos = 0;
                self.repeats += 1;
            }
        }
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    // We wrap the fuzzer data in FuzzReader, which will repeat it max_repeats times.
    // If the fuzzer generates data *without* a newline, `run_repl_inner`'s `read_line`
    // will attempt to buffer the entire repeated payload into a single String, causing
    // memory exhaustion (OOM). We use 500,000 repeats which turns a tiny fuzzer payload
    // into a large string that quickly exhausts typical limits and demonstrates the DoS natively.
    let mut input = FuzzReader::new(data);
    let mut output = std::io::sink();
    let _ = glossa::tools::repl::run_repl_inner(&mut input, &mut output);
});
