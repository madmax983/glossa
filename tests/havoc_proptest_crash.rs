use glossa::text::normalize_greek;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_normalize_greek_any_string_does_not_crash(s in ".*") {
        normalize_greek(&s);
    }
}
