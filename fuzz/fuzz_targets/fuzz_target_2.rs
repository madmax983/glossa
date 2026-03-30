#![no_main]

use libfuzzer_sys::fuzz_target;
use glossa::morphology::conjugation::analyze_verb;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = analyze_verb(text);
    }
});
