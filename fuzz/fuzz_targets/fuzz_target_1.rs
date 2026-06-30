#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = glossa::text::normalize_greek(s);
        let _ = glossa::morphology::analyze_verb(s);
        let _ = glossa::morphology::analyze_participle(s);
        if let Ok(ast) = glossa::parser::parse(s) {
            let _ = glossa::semantic::analyze_program(&ast);
            let _ = glossa::tools::highlight::highlight(s);
        }
    }
});
