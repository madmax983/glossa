#![no_main]

use libfuzzer_sys::fuzz_target;
use glossa::parser::parse;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = parse(text);
    }
});
