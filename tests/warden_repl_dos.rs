#![allow(missing_docs)]
use std::io::{BufReader, Read};

#[test]
fn test_repl_dos_memory_exhaustion() {
    // Generate an infinite stream of spaces (no newlines) capped at 100MB to prevent CI timeout.
    let malicious_stream = std::io::repeat(b' ').take(100_000_000);
    let mut reader = BufReader::new(malicious_stream);

    let mut output = Vec::new();

    // Pass the actual massive stream. Without mitigation, this would try to load 100MB into a single string.
    let result = glossa::tools::repl::run_repl_inner(&mut reader, &mut output);

    // The REPL should complete processing this stream in chunks instead of hanging or OOMing on a single read.
    assert!(result.is_ok());

    let output_str = String::from_utf8_lossy(&output);
    assert!(!output_str.is_empty(), "Should have produced output");
}
