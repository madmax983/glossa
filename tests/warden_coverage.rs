use glossa::codegen::generate_rust;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

fn compile(source: &str) {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    let _ = generate_rust(&analyzed);
}

fn compile_error(source: &str, expected_error: &str) {
    let ast = parse(source).unwrap();
    match analyze_program(&ast) {
        Ok(_) => panic!("Expected error: {}", expected_error),
        Err(e) => {
            let msg = e.to_string();
            assert!(
                msg.contains(expected_error),
                "Expected '{}', got '{}'",
                expected_error,
                msg
            );
        }
    }
}

#[test]
fn test_coverage_nested_phrase_ambiguous_error() {
    // Should error because multiple expressions are found
    // Code path: exprs.len() > 1 -> Err("Ambiguous nested phrase...")
    let source = "ξ (1 2) ἔστω.";
    compile_error(source, "Ambiguous nested phrase");
}

#[test]
fn test_coverage_nested_phrase_binding_error() {
    // Nested phrase containing a statement (binding) -> Error
    // Code path: kind is Binding -> Err("Nested phrase must evaluate to an expression")
    let source = "ξ (ψ 1 ἔστω) ἔστω.";
    compile_error(source, "Nested phrase must evaluate to an expression");
}

#[test]
fn test_coverage_nested_phrase_empty_error() {
    // Nested phrase evaluating to nothing (e.g. just article)
    // Code path: exprs.is_empty() -> Err("Expression evaluated to nothing")
    // "ὁ" is an article, ignored by assembler.
    // We use "ὁ ὁ" to ensure it parses as a Phrase (multiple terms) rather than a single Word,
    // which forces it to go through the nested phrase path.
    let source = "ξ (ὁ ὁ) ἔστω.";
    compile_error(source, "Expression evaluated to nothing");
}

#[test]
fn test_coverage_nested_phrase_multiple_error() {
    // Multiple nested phrases in binding -> Error
    // Code path: nested_phrases.len() > 1 -> Err("Multiple nested phrases...")
    // "ξ (1 2) (3 4) ἔστω."
    // (1 2) -> nested phrase [1, 2]
    // (3 4) -> nested phrase [3, 4]
    // This is parsed as multiple terms, fed to assembler as separate nested phrases.
    // Single terms like (1) are flattened by parser to just 1, so we need multiple terms.
    let source = "ξ (1 2) (3 4) ἔστω.";
    compile_error(
        source,
        "Multiple nested phrases in value position are ambiguous",
    );
}

#[test]
fn test_coverage_filter_patterns() {
    // 1. Suffix "ου" (e.g. θ -> θου)
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        θ 10 ἔστω.
        // Filter: collection + genitive(ου) + comparative_adj + print
        ξ θου μείζονα λέγε.
    ",
    );

    // 2. Suffix "ης" (e.g. αγάπη -> αγάπης)
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        αγάπη 10 ἔστω.
        // Filter: collection + genitive(ης) + comparative_adj + print
        ξ αγάπης μείζονα λέγε.
    ",
    );

    // 3. Suffix "ων" (e.g. μέτρον -> μέτρων)
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        μέτρον 10 ἔστω.
        // Filter: collection + genitive(ων) + comparative_adj + print
        ξ μέτρων μείζονα λέγε.
    ",
    );

    // 4. No suffix (else branch) - e.g. indeclinable name like Δαβιδ
    // "Αδαμ" failed because morphology might treat it weirdly (Capital Alpha issue?)
    // Trying "Δαβιδ" (David) which is definitely a proper noun and indeclinable in many contexts
    // Or just "β" which is a variable name.
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        β 10 ἔστω.
        // Filter: collection + genitive(no suffix) + comparative_adj + print
        ξ β μείζονα λέγε.
    ",
    );
}

#[test]
fn test_coverage_any_all_patterns() {
    // 1. Suffix "ου"
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        θ 10 ἔστω.
        // Any: collection + any + genitive(ου) + operator(μείζον) + print
        ξ τι θου μείζον λέγε.
    ",
    );

    // 2. Suffix "ης"
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        αγάπη 10 ἔστω.
        // Any: collection + any + genitive(ης) + operator(μείζον) + print
        ξ τι αγάπης μείζον λέγε.
    ",
    );

    // 3. Suffix "ων"
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        μέτρον 10 ἔστω.
        // Any: collection + any + genitive(ων) + operator(μείζον) + print
        ξ τι μέτρων μείζον λέγε.
    ",
    );

    // 4. No suffix (else branch)
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        β 10 ἔστω.
        // Any: collection + any + genitive(no suffix) + operator(μείζον) + print
        ξ τι β μείζον λέγε.
    ",
    );
}

#[test]
fn test_coverage_find_patterns() {
    // 1. Suffix "ου"
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        θ 10 ἔστω.
        // Find: collection + genitive(ου) + operator(μείζον) + find
        ξ θου μείζον εὑρέ.
    ",
    );

    // 2. Suffix "ης"
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        αγάπη 10 ἔστω.
        // Find: collection + genitive(ης) + operator(μείζον) + find
        ξ αγάπης μείζον εὑρέ.
    ",
    );

    // 3. Suffix "ων"
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        μέτρον 10 ἔστω.
        // Find: collection + genitive(ων) + operator(μείζον) + find
        ξ μέτρων μείζον εὑρέ.
    ",
    );

    // 4. No suffix (else branch)
    compile(
        "
        ξ [1, 2, 3] ἔστω.
        β 10 ἔστω.
        // Find: collection + genitive(no suffix) + operator(μείζον) + find
        ξ β μείζον εὑρέ.
    ",
    );
}
