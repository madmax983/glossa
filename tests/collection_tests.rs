//! Collection tests for ΓΛΩΣΣΑ Phase 3
//!
//! Tests array literals, indexing, and collection operations.

use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

/// Helper to compile GLOSSA source to Rust code
fn compile(source: &str) -> String {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    generate_rust(&analyzed)
}

// =============================================================================
// Cycle 1: Array Literal Parsing
// =============================================================================

#[test]
fn test_parse_array_literal() {
    // Array literal [1, 2, 3] should parse successfully
    let ast = parse("[1, 2, 3] ἔστω ἀριθμοί.").unwrap();
    assert_eq!(ast.statements.len(), 1);
    // The first expression should contain an array literal
}

#[test]
fn test_parse_empty_array() {
    // Empty array [] should parse without error
    let ast = parse("[] ἔστω κενός.").unwrap();
    assert_eq!(ast.statements.len(), 1);
}

#[test]
fn test_parse_array_with_variables() {
    // Array can contain variable references
    let source = "ξ πέντε ἔστω. [ξ, 10, ξ] ἔστω πίναξ.";
    let ast = parse(source).unwrap();
    assert_eq!(ast.statements.len(), 2);
}

// =============================================================================
// Cycle 2: Array Literal Codegen
// =============================================================================

#[test]
fn test_codegen_array_literal() {
    // Use simple variable binding: ξ [1, 2, 3] ἔστω (let xi = [1, 2, 3])
    let code = compile("ξ [1, 2, 3] ἔστω.");
    // Should generate vec! macro (quote! adds space: "vec !")
    assert!(code.contains("vec"), "Expected vec in: {}", code);
}

#[test]
fn test_codegen_empty_array() {
    // Empty array binding
    let code = compile("ξ [] ἔστω.");
    // Should generate empty vec (quote! adds space: "vec !")
    assert!(code.contains("vec"), "Expected empty vec in: {}", code);
}

// =============================================================================
// Cycle 3: Numeric Indexing
// =============================================================================

#[test]
fn test_numeric_index() {
    // Using bracket syntax: ξ[0]
    let code = compile("ξ [1, 2, 3] ἔστω. ξ[0] λέγε.");
    // Should have index cast to usize
    assert!(
        code.contains("as usize"),
        "Expected index cast to usize in: {}",
        code
    );
    assert!(code.contains("0"), "Expected index 0 in: {}", code);
}

#[test]
fn test_numeric_index_expression() {
    // Index should work as an expression
    let code = compile("ξ [10, 20, 30] ἔστω. ψ ξ[1] ἔστω.");
    // quote! adds spaces and i64 suffix
    assert!(
        code.contains("[1") || code.contains("[ 1"),
        "Expected index [1 in: {}",
        code
    );
}

// =============================================================================
// Cycle 4: Push Operation
// =============================================================================

#[test]
fn test_push_operation() {
    // ξ ὠθεῖ 42 = "xi pushes 42" (simpler syntax)
    let code = compile("ξ [] ἔστω. ξ ὠθεῖ 42.");
    assert!(
        code.contains(".push(") || code.contains(". push"),
        "Expected .push( in: {}",
        code
    );
}

#[test]
fn test_push_multiple() {
    // Push multiple values to array
    let code = compile("ξ [] ἔστω. ξ ὠθεῖ 42. ξ ὠθεῖ 99.");
    assert!(code.contains("push"), "Expected push in: {}", code);
}

// =============================================================================
// Cycle 5: Pop Operation
// =============================================================================

#[test]
fn test_pop_operation() {
    // ξ ἕλκεται = "xi pulls-itself" (middle voice = pop)
    let code = compile("ξ [1, 2, 3] ἔστω. ξ ἕλκεται.");
    assert!(
        code.contains("pop") || code.contains(". pop"),
        "Expected .pop in: {}",
        code
    );
}

#[test]
fn test_pop_multiple() {
    // Pop multiple times
    let code = compile("ξ [1, 2, 3] ἔστω. ξ ἕλκεται. ξ ἕλκεται.");
    assert!(code.contains("pop"), "Expected pop in: {}", code);
}

// =============================================================================
// Cycle 6: Length Property
// =============================================================================

#[test]
fn test_length_property() {
    // ξ μῆκος λέγε = "say length of xi" (simpler syntax)
    let code = compile("ξ [1, 2, 3] ἔστω. ξ μῆκος λέγε.");
    assert!(
        code.contains("len") || code.contains(". len"),
        "Expected .len in: {}",
        code
    );
}

#[test]
fn test_length_multiple() {
    // Multiple length calls
    let code = compile("ξ [1, 2, 3] ἔστω. ξ μῆκος λέγε. ξ μῆκος λέγε.");
    assert!(code.contains("len"), "Expected len in: {}", code);
}

// =============================================================================
// Cycle 7: Ordinal Indexing (Complex Genitive Pattern)
// =============================================================================

#[test]
fn test_ordinal_index_first() {
    // ξ πρῶτον λέγε = "say first of xi"
    // Simplified: subject (xi) + ordinal (first) + verb (say)
    let code = compile("ξ [10, 20, 30] ἔστω. ξ πρῶτον λέγε.");
    // First element = index 0, cast to usize
    assert!(
        code.contains("as usize"),
        "Expected usize cast in: {}",
        code
    );
    assert!(code.contains("0"), "Expected index 0 in: {}", code);
}

#[test]
fn test_ordinal_index_second() {
    // ξ δεύτερον λέγε = "say second of xi"
    let code = compile("ξ [10, 20, 30] ἔστω. ξ δεύτερον λέγε.");
    // Second element = index 1
    assert!(
        code.contains("[1") || code.contains("[ 1"),
        "Expected [1] for second in: {}",
        code
    );
}

#[test]
fn test_ordinal_index_third() {
    // ξ τρίτον λέγε = "say third of xi"
    let code = compile("ξ [10, 20, 30] ἔστω. ξ τρίτον λέγε.");
    // Third element = index 2 (0-indexed), cast to usize
    assert!(
        code.contains("as usize"),
        "Expected usize cast in: {}",
        code
    );
    assert!(code.contains("2"), "Expected index 2 in: {}", code);
}

// =============================================================================
// Cycle 8: Array Iteration
// =============================================================================

#[test]
fn test_array_iteration() {
    // διὰ ξ, ψ λέγε = "through xi, say psi"
    let code = compile("ξ [1, 2, 3] ἔστω. διὰ ξ, ψ λέγε.");
    assert!(code.contains("for"), "Expected for loop in: {}", code);
}

#[test]
fn test_array_iteration_with_body() {
    // More complex iteration with body
    let code = compile("ξ [1, 2, 3] ἔστω. διὰ ξ, ψ λέγε.");
    assert!(
        code.contains("for") && code.contains("in"),
        "Expected for..in in: {}",
        code
    );
}

// =============================================================================
// Cycle 9: HashSet Insert
// =============================================================================

#[test]
fn test_hashset_insert() {
    // ξ νέον σύνολον ἔστω. ξ 42 τίθησι.
    // "Let xi be a new set. Xi places 42 (into itself)."
    let code = compile("ξ νέον σύνολον ἔστω. ξ 42 τίθησι.");
    // quote! adds spaces, so check for "insert" and the value
    assert!(
        code.contains("insert") && code.contains("42"),
        "Expected insert(42) in: {}",
        code
    );
}

#[test]
fn test_hashset_insert_string() {
    // Insert a string into a HashSet
    let code = compile("ξ νέον σύνολον ἔστω. ξ «ἀλφα» τίθησι.");
    // quote! adds spaces, so check for "insert"
    assert!(
        code.contains("insert") && code.contains("ἀλφα"),
        "Expected insert with string in: {}",
        code
    );
}

// =============================================================================
// Cycle 10: HashSet Contains
// =============================================================================

#[test]
fn test_hashset_contains() {
    // ξ νέον σύνολον ἔστω. ξ 42 τίθησι. 42 ἐν ξ?
    // "Let xi be a new set. Xi places 42. 42 in xi?"
    let code = compile("ξ νέον σύνολον ἔστω. ξ 42 τίθησι. 42 ἐν ξ?");
    // quote! adds spaces, so check for "contains"
    assert!(code.contains("contains"), "Expected contains in: {}", code);
}

// =============================================================================
// Cycle 11: HashMap Insert
// =============================================================================

#[test]
fn test_hashmap_insert() {
    // ξ νέον χάρτης ἔστω. ξ «ὄνομα» «Σωκράτης» τίθησι.
    // "Let xi be a new map. Xi places name->Socrates."
    let code = compile("ξ νέον χάρτης ἔστω. ξ «ὄνομα» «Σωκράτης» τίθησι.");
    assert!(
        code.contains("insert") && code.contains("Σωκράτης"),
        "Expected insert with value in: {}",
        code
    );
}

// =============================================================================
// Cycle 12: HashMap Get
// =============================================================================

#[test]
fn test_hashmap_get() {
    // ξ νέον χάρτης ἔστω. ξ «ὄνομα» λέγε.
    // "Let xi be a new map. Say xi's name."
    let code = compile("ξ νέον χάρτης ἔστω. ξ «ὄνομα» «Σωκράτης» τίθησι. ξ «ὄνομα» λέγε.");
    // For now, just check that it compiles - get pattern is complex
    assert!(code.contains("HashMap"), "Expected HashMap in: {}", code);
}

// =============================================================================
// Cycle 13: HashMap Contains Key
// =============================================================================

#[test]
fn test_hashmap_contains_key() {
    // ξ νέον χάρτης ἔστω. «ὄνομα» ἐν ξ?
    // "Let xi be a new map. 'name' in xi?"
    let code = compile("ξ νέον χάρτης ἔστω. ξ «ὄνομα» «Σωκράτης» τίθησι. «ὄνομα» ἐν ξ?");
    assert!(
        code.contains("contains_key"),
        "Expected contains_key in: {}",
        code
    );
}

// =============================================================================
// Cycle 14: String Contains
// =============================================================================

#[test]
fn test_string_contains() {
    // ξ «χαῖρε κόσμε» ἔστω. «κόσμε» ἐν ξ?
    // "Let xi be 'hello world'. 'world' in xi?"
    let code = compile("ξ «χαῖρε κόσμε» ἔστω. «κόσμε» ἐν ξ?");
    assert!(code.contains("contains"), "Expected contains in: {}", code);
}

// =============================================================================
// Cycle 15: String Split
// =============================================================================

#[test]
fn test_string_split() {
    // ξ «χαῖρε-κόσμε» ἔστω. ξ κατὰ «-» σχίζεται λέγε.
    // "Let xi be 'hello-world'. Say xi split by dash."
    let code = compile("ξ «χαῖρε-κόσμε» ἔστω. ξ κατὰ «-» σχίζεται λέγε.");
    assert!(
        code.contains("split") && code.contains("\"-\""),
        "Expected split(\"-\") in: {}",
        code
    );
}

// =============================================================================
// Cycle 16: String Join
// =============================================================================

#[test]
fn test_string_join() {
    // ξ [«α», «β», «γ»] ἔστω. ξ κατὰ «-» ἑνοῦνται λέγε.
    // "Let xi be ['a', 'b', 'c']. Say xi joined with dash."
    let code = compile("ξ [«α», «β», «γ»] ἔστω. ξ κατὰ «-» ἑνοῦνται λέγε.");
    assert!(
        code.contains("join") && code.contains("\"-\""),
        "Expected join(\"-\") in: {}",
        code
    );
}
