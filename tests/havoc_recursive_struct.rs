use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_recursive_struct_rejection() {
    // 👺 HAVOC: Define a recursive struct without indirection.
    // In Rust, this would be: struct Node { next: Node } -> Infinite size.
    // Glossa should catch this during semantic analysis and return a nice error.
    // If it doesn't, it generates invalid Rust code and rustc explodes with a confusing error.

    // We define Node twice. First empty, so it exists in scope.
    // Then we redefine it with a field of its own type.
    // This tricks the resolver to find "Node" (the old one).
    let source = "
    εἶδος Node ὁρίζειν { }.
    εἶδος Node ὁρίζειν { next Node. }.
    ";

    let ast = parse(source).unwrap();
    let result = analyze_program(&ast);

    // We expect this to FAIL with a semantic error about recursion.
    // If it succeeds (Ok), then we failed to catch it.
    assert!(
        result.is_err(),
        "Recursive struct definition was accepted by semantic analysis! It should be rejected. Result: {:?}",
        result
    );

    // Ideally check the error message too
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Recursive type") || msg.contains("infinite size"), "Error message should mention recursion, got: {}", msg);
}
