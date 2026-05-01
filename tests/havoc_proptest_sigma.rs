use glossa::text::normalize_greek;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_normalize_greek_sigma_does_not_crash(s in ".*Σ.*") {
        normalize_greek(&s);
    }
}
