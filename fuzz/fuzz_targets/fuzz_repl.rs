#![no_main]
use libfuzzer_sys::fuzz_target;

use std::io::{BufRead, Read, BufReader};
use glossa::tools::repl::run_repl_inner;
use std::thread;

struct EndlessStream<'a> {
    data: &'a [u8],
}

impl<'a> Read for EndlessStream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.data.is_empty() {
            return Ok(0);
        }
        for b in buf.iter_mut() {
            *b = self.data[0];
        }
        Ok(buf.len())
    }
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() || data[0] == b'\n' {
        return;
    }

    // Create an endless stream that repeats the first byte of `data`
    let stream = EndlessStream { data };

    // Bounded externally for the fuzzer so we don't literally OOM the fuzzer process
    // But large enough to crash or timeout if there's no bound inside the REPL
    // Let's take 2MB. If the REPL unbounded-reads, it will consume all 2MB in one read_line.
    let mut bounded_input = BufReader::new(stream.take(2_000_000));

    let mut output = Vec::new();
    let _ = run_repl_inner(&mut bounded_input, &mut output);
});
