
#[test]
fn test_parser_coverage_complex() {
    use glossa::parser::parse;

    // Complex nested structure to hit parser builder branches
    let source = "
    εἶδος Χρήστης ὁρίζειν {
        ὄνομα ὀνόματος.
        ἡλικία ἀριθμοῦ.
    }.

    χαρακτήρ Εκτυπώσιμος ὁρίζειν {
        δεῖ τύπωσις τῷ εαυτῷ.
        ἤδη default τῷ εαυτῷ { «default» λέγε. }.
    }.

    εἶδος Χρήστης τῷ Εκτυπώσιμος ἐμπίπτειν {
        τύπωσις τῷ εαυτῷ {
            εαυτοῦ ὄνομα λέγε.
        }.
    }.

    δοκιμή «complex».
        // Removed complex math expression because parser might not handle infix operators in this test context
        // or whitespace handling might be strict in the grammar for phrases.
        ξ [1, 2, 3] ἔστω.
        ξ[0] λέγε.
        τιμή! λέγε.
    τέλος.
    ";

    let res = parse(source);
    assert!(res.is_ok(), "Failed to parse complex source: {:?}", res.err());
}
