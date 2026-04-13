#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = glossa::morphology::analyze(s);
        let _ = glossa::morphology::analyze_all(s);
        let _ = glossa::morphology::analyze_noun(s);
        let _ = glossa::morphology::analyze_noun_all(s);
        let _ = glossa::morphology::lookup(s);
    }
});
