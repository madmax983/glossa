#![allow(missing_docs)]
use glossa::codegen::transliterate;
use glossa::parser::parse;
use glossa::text::normalize_greek;
use glossa::tools::highlight::highlight;
use glossa::tools::runner::analyze_source;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_fuzz_endpoints(s in "\\PC*") {
        let _ = analyze_source(&s);
        let _ = transliterate(&s);
        let _ = parse(&s);
        let _ = highlight(&s);
        let _ = normalize_greek(&s);
    }
}
