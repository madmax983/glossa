use glossa::parser::parse;
use arbitrary::{Arbitrary, Unstructured};

pub fn fuzz_parse(data: &[u8]) {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = parse(text);
    }
}
