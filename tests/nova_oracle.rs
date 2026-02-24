#![cfg(feature = "nova")]

use glossa::tools::oracle::run_oracle_inner;

fn run_test(source: &str) -> String {
    let mut buffer = Vec::new();
    run_oracle_inner(source, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[test]
fn test_oracle_clean() {
    let source = "
        εἶδος Χ ὁρίζειν { χ ἀριθμοῦ. }.
        ξ πέντε ἔστω.
        ξ λέγε.
    ";
    let output = run_test(source);
    assert!(output.contains("The Oracle is silent"));
}

#[test]
fn test_oracle_hubris() {
    // Deep nesting
    let source = "
        εἰ ἀληθές, {
            εἰ ἀληθές, {
                εἰ ἀληθές, {
                    εἰ ἀληθές, {
                        «Hubris» λέγε.
                    }.
                }.
            }.
        }.
    ";
    let output = run_test(source);
    assert!(output.contains("Hubris"));
    assert!(output.contains("Deep nesting detected"));
}

#[test]
fn test_oracle_laziness() {
    // Empty if block
    let source = "
        εἰ ἀληθές, { }.
    ";
    let output = run_test(source);
    assert!(output.contains("Laziness"));
    assert!(output.contains("Empty 'If' block"));
}

#[test]
fn test_oracle_laziness_while() {
    // Empty while loop
    let source = "
        ἕως ἀληθές, { }.
    ";
    let output = run_test(source);
    assert!(output.contains("Laziness"));
    assert!(output.contains("Empty 'While' loop"));
}

#[test]
fn test_oracle_barbarism() {
    // Latin type name
    let source = "
        εἶδος LatinStruct ὁρίζειν { }.
    ";
    let output = run_test(source);
    assert!(output.contains("Barbarism"));
    assert!(output.contains("latinstruct"));
}

#[test]
fn test_oracle_narcissus() {
    // Unused variable
    let source = "
        ξ πέντε ἔστω.
        «Hello» λέγε.
    ";
    let output = run_test(source);
    assert!(output.contains("Narcissus"));
    assert!(output.contains("Variable 'ξ' is declared but never used"));
}

#[test]
fn test_oracle_no_narcissus_when_used() {
    // Used variable
    let source = "
        ξ πέντε ἔστω.
        ξ λέγε.
    ";
    let output = run_test(source);
    assert!(!output.contains("Narcissus"));
}

#[test]
fn test_oracle_coverage_kitchen_sink() {
    // A comprehensive test to exercise recursive AST traversal
    let source = "
    εἶδος Κόσμος ὁρίζειν {
        δύναμις ἀριθμοῦ.
    }.

    χαρακτήρ Θεϊκός ὁρίζειν {
        δεῖ θαῦμα τῷ self.
    }.

    εἶδος DeusEx ὁρίζειν {
        δύναμις ἀριθμοῦ.
    }.

    εἶδος DeusEx τῷ Θεϊκός ἐμπίπτειν {
        θαῦμα τῷ self· «Miracle» λέγε.
    }.

    λειτουργία f() -> ἀριθμοῦ:
        1 1 ἄθροισμα.
    .

    δοκιμή «complex_coverage».
        ξ πέντε ἔστω.
        ψ δέκα ἔστω.

        // AssertEq
        ξ ψ ἀνισοῦται ἀληθές ἰσοῦται.

        // Assert
        ξ πέντε ἰσοῦται βεβαίωσον.

        // BinOp recursion
        ξ 1 ἄθροισμα 2 διαφορά ψ ἰσοῦται βεβαίωσον.

        // UnaryOp
        ἀληθές ὄχι.

        // Function Call
        f().

        // Lambda
        λ: |x| x.

        // Range & For loop
        ἀπὸ 1 μέχρι 10, ι λέγε.

        // While loop
        ἕως ψ 0 μεῖζον ᾖ, {
             ψ ψ 1 διαφορά γίγνεται. // Assignment
        }.

        // Match
        κατὰ ξ·
           1 ᾖ, «One» λέγε·
           ἄλλο ᾖ, «Other» λέγε.

        // Array & Index
        λίστα [1, 2, 3] ἔστω.
        λίστα[0].

        // Struct Instantiation
        κόσμος νέον Κόσμος 42 ἔστω.

        // Property Access
        κόσμος.δύναμις.

        // Method Call (needs a type with method, or generic verb call)
        «hello».len().

        // Trait Method Call (simulated via verb for now as syntax is tricky)
        // Would be like: deus.θαῦμα() if traits were fully supported in parser this way

        // If Else
        εἰ ξ 5 ἰσοῦται ᾖ, {
           «Five» λέγε.
        } εἰ δὲ μή, {
           «Not Five» λέγε.
        }.

        // Option/Result/Unwrap
        κάτι Some(5) ἔστω.
        κάτι!.

        // Return (in function context usually, but valid statement)
        ἐπίστρεφε 0.
    τέλος.
    ";

    // We just want to ensure it runs without crashing and collects usages correctly.
    // If usages aren't collected, 'ξ' and others might be flagged as Narcissus.
    // However, some vars like 'λίστα' are used, so they shouldn't trigger it.

    let output = run_test(source);

    // Check that we didn't crash
    assert!(output.contains("The Oracle"));

    // 'ξ' is used in match and assertions, so it should NOT be unused.
    assert!(!output.contains("Variable 'ξ' is declared but never used"));

    // 'ψ' is used in while loop
    assert!(!output.contains("Variable 'ψ' is declared but never used"));

    // 'κόσμος' is used in property access
    assert!(!output.contains("Variable 'κόσμος' is declared but never used"));

    // 'λίστα' is used in index access
    assert!(!output.contains("Variable 'λίστα' is declared but never used"));
}
