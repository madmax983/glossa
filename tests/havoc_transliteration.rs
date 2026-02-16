use glossa::codegen::transliterate;
use proptest::prelude::*;

// Havoc Test: Verify transliteration uniqueness
// Logic: If two different source identifiers map to the same target identifier,
// the compiler will generate broken code (shadowing or type errors).

#[test]
fn test_known_collision_alpha() {
    // The "Golden Hammer" of collisions: α vs a
    let greek = "α";
    let latin = "a";

    // They are different strings
    assert_ne!(greek, latin);

    // They should transliterate to different strings to avoid collision.
    assert_ne!(transliterate(greek), transliterate(latin), "Collision detected: α maps to same output as a");
}

proptest! {
    #[test]
    fn test_transliteration_unique(s1 in "[a-zA-Zα-ω]+", s2 in "[a-zA-Zα-ω]+") {
        // Generate strings containing Latin and Greek letters
        if s1 != s2 {
            // We expect unique transliterations for unique inputs
            prop_assert_ne!(transliterate(&s1), transliterate(&s2));
        }
    }
}
