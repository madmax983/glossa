#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        if let Ok(ast) = glossa::parser::parse(text) {
             let _ = glossa::semantic::analyzer::analyze_program(&ast);
        }
    }
});
