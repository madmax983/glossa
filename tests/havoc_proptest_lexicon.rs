#![allow(missing_docs)]
use glossa::morphology::lookup;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_lexicon_lookup_no_panic(
        s in ".*"
    ) {
        let _ = lookup(&s);
    }
}
