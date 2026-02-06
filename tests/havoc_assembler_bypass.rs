use glossa::semantic::assembler::Assembler;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_bypass_token_limit_invariant(
        // Generate a vector of strings with length 1001..2000
        // This ensures we always try to exceed the limit
        strings in proptest::collection::vec(".*", 1001..2000)
    ) {
        let mut asm = Assembler::new();

        // Feed all strings
        // The Assembler should stop accepting input after limit is reached
        for s in strings {
            let _ = asm.feed_string(s);
        }

        let stmt = asm.finalize().expect("Finalize failed unexpectedly");

        // ASSERT INVARIANT:
        // The number of literals should not exceed the MAX_TOKENS limit (1000).
        prop_assert!(stmt.literals.len() <= 1000,
            "Invariant violated: Assembler accepted {} literals, exceeding limit of 1000",
            stmt.literals.len());
    }
}
