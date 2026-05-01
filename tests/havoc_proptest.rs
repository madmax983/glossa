use glossa::text::normalize_greek;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_normalize_greek_does_not_crash(s in "\\PC*") {
        normalize_greek(&s);
    }
}
