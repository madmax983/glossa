#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str;
use glossa::parser::parse;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = str::from_utf8(data) {
        if s.len() > 1024 { return; } // prevent slow fuzzing
        let _ = parse(s);
    }
});
