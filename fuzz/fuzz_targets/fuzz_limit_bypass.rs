#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        if s.len() > 1000 {
            return;
        }
        let ast = glossa::parser::parse(s);
        if let Ok(ast) = ast {
            let _ = glossa::semantic::analyze_program(&ast);
        }
    }
});
