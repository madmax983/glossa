use glossa::text::normalize_greek;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_normalize_greek_no_panic(s in "\\PC*") {
        let _ = normalize_greek(&s);
    }
}

use glossa::tools::highlight;

proptest! {
    #[test]
    fn test_highlight_no_panic(s in "\\PC*") {
        let _ = highlight(&s);
    }
}

use glossa::semantic::detect_collection_type;

proptest! {
    #[test]
    fn test_detect_collection_type_no_panic(s in "\\PC*") {
        let _ = detect_collection_type(&s);
    }
}

use glossa::morphology::{analyze, analyze_all};

proptest! {
    #[test]
    fn test_morphology_analyze_no_panic(s in "\\PC*") {
        let _ = analyze(&s);
    }
    #[test]
    fn test_morphology_analyze_all_no_panic(s in "\\PC*") {
        let _ = analyze_all(&s);
    }
}

use glossa::codegen::transliterate;

proptest! {
    #[test]
    fn test_transliterate_no_panic(s in "\\PC*") {
        let _ = transliterate(&s);
    }
}

// Fuzz the parser with arbitrary bytes to see if anything panics
// Actually, proptest strings are valid UTF-8, which is good enough to fuzz parser.

use glossa::morphology::Declension;
use glossa::morphology::conjugation::analyze_verb;
use glossa::morphology::declension::{analyze_noun, get_stem};
use glossa::morphology::disambiguation::analyze_article;
use glossa::morphology::participle::analyze_participle;

proptest! {
    #[test]
    fn test_analyze_verb_no_panic(s in "\\PC*") {
        let _ = analyze_verb(&s);
    }

    #[test]
    fn test_analyze_noun_no_panic(s in "\\PC*") {
        let _ = analyze_noun(&s);
    }

    #[test]
    fn test_analyze_article_no_panic(s in "\\PC*") {
        let _ = analyze_article(&s);
    }

    #[test]
    fn test_analyze_participle_no_panic(s in "\\PC*") {
        let _ = analyze_participle(&s);
    }

    #[test]
    fn test_get_stem_no_panic(s in "\\PC*") {
        let _ = get_stem(&s, Declension::Second);
    }
}

use glossa::semantic::assembly::Assembler;

proptest! {
    #[test]
    fn test_assembler_feed_string_no_panic(s in "\\PC*") {
        let mut asm = Assembler::new();
        let _ = asm.feed_string(s.to_string());
    }
}
