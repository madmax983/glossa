#![no_main]

use libfuzzer_sys::fuzz_target;
use glossa::morphology::disambiguation::disambiguate;
use glossa::morphology::analyze_all;
use glossa::morphology::DisambiguationContext;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let analyses = analyze_all(text);
        if !analyses.is_empty() {
             let ctx = DisambiguationContext::new();
             let _ = disambiguate(analyses, &ctx);
        }
    }
});
