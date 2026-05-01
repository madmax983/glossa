use proptest::prelude::*;

proptest! {
    #[test]
    fn test_lexicon_lookup_does_not_crash(s in "\\PC*") {
        glossa::morphology::lookup(&s);
    }
}
