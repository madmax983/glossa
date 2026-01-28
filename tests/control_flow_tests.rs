/// Phase 2: Control Flow Tests
/// TDD RED phase - these tests define the expected behavior
///
/// Control flow constructs:
/// - Conditionals: εἰ/ἐάν (if), εἰ δὲ μή (else)
/// - Loops: ἕως (while), διά/ἀπό...μέχρι (for)
/// - Pattern matching: κατά (match)
/// - Loop control: παῦε (break), συνέχιζε (continue)
use glossa::ast::build_ast;
use glossa::codegen::generate_rust;
use glossa::ir::lower_to_hir;
use glossa::semantic::analyze_program;

fn compile_to_rust(source: &str) -> String {
    let ast = build_ast(source).expect("AST build failed");
    let analyzed = analyze_program(&ast).expect("Analysis failed");
    let hir = lower_to_hir(&analyzed);
    generate_rust(&hir)
}

// =============================================================================
// Subjunctive mood (needed for conditionals)
// =============================================================================

#[test]
fn test_subjunctive_verb_form() {
    // ᾖ is subjunctive of εἰμί (to be)
    // Should be recognized as subjunctive mood
    let analysis = glossa::morphology::analyze("ᾖ");
    assert_eq!(
        analysis.mood,
        Some(glossa::morphology::Mood::Subjunctive),
        "ᾖ should be recognized as subjunctive"
    );
}

// =============================================================================
// Simple Conditionals
// =============================================================================

#[test]
fn test_simple_if() {
    // εἰ ξ πέντε μεῖζον ᾖ, «ναί» λέγε.
    // "if x greater than five be, say 'yes'"
    let source = "ξ δέκα ἔστω. εἰ ξ πέντε μεῖζον ᾖ, «ναί» λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("if"), "Expected if statement");
    assert!(output.contains(">"), "Expected > comparison");
}

#[test]
fn test_if_else() {
    // εἰ ξ πέντε μεῖζον ᾖ, «ναί» λέγε · εἰ δὲ μή, «οὔ» λέγε.
    // "if x greater than five be, say 'yes'; if but not, say 'no'"
    let source = "ξ τρία ἔστω. εἰ ξ πέντε μεῖζον ᾖ, «ναί» λέγε · εἰ δὲ μή, «οὔ» λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("if"), "Expected if statement");
    assert!(output.contains("else"), "Expected else clause");
}

#[test]
fn test_if_elif_else() {
    // Chain of conditions
    let source =
        "ξ πέντε ἔστω. εἰ ξ μηδὲν ᾖ, «μηδέν» λέγε · εἰ ξ ἓν ᾖ, «ἕν» λέγε · εἰ δὲ μή, «ἄλλο» λέγε.";
    let output = compile_to_rust(source);

    eprintln!("Generated output:\n{}", output);
    assert!(output.contains("if"), "Expected if statement");
    assert!(
        output.contains("else if") || output.contains("} else {"),
        "Expected else-if chain or else"
    );
}

#[test]
fn test_ean_conditional() {
    // ἐάν is alternate conditional particle (with subjunctive)
    let source = "ξ δέκα ἔστω. ἐάν ξ πέντε μεῖζον ᾖ, «ναί» λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("if"), "Expected if statement from ἐάν");
}

// =============================================================================
// While Loops
// =============================================================================

#[test]
fn test_comparison_with_genitive() {
    // ξ μηδενὸς μεῖζον - "x greater than zero"
    let source = "ξ πέντε ἔστω. ξ μηδενὸς μεῖζον.";
    let output = compile_to_rust(source);

    // Should compile without error
    assert!(output.contains("let"), "Expected binding");
}

#[test]
fn test_while_loop() {
    // ἕως ξ μηδενὸς μεῖζον ᾖ, ξ λέγε.
    // "while x greater than zero be, say x"
    let source = "ξ πέντε ἔστω. ἕως ξ μηδενὸς μεῖζον ᾖ, ξ λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("while"), "Expected while loop");
}

// =============================================================================
// For Loops (Range iteration)
// =============================================================================

#[test]
fn test_for_range_exclusive() {
    // ἀπὸ μηδενὸς μέχρι πέντε, ι λέγε.
    // "from zero until five, say i" (exclusive upper bound)
    let source = "ἀπὸ μηδενὸς μέχρι πέντε, ι λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("for"), "Expected for loop");
    assert!(output.contains(".."), "Expected range operator");
}

#[test]
fn test_for_range_inclusive() {
    // ἀπὸ μηδενὸς ἕως πέντε, ι λέγε.
    // "from zero to five, say i" (inclusive upper bound)
    let source = "ἀπὸ μηδενὸς ἕως πέντε, ι λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("for"), "Expected for loop");
    assert!(output.contains("..="), "Expected inclusive range operator");
}

#[test]
fn test_for_iteration() {
    // διὰ στοιχείων, στοιχεῖον λέγε.
    // "through elements, say element"
    let source = "διὰ στοιχείων, στοιχεῖον λέγε.";
    let output = compile_to_rust(source);

    eprintln!("Generated output:\n{}", output);
    assert!(output.contains("for"), "Expected for loop");
    assert!(output.contains("in"), "Expected 'in' keyword");
}

// =============================================================================
// Pattern Matching
// =============================================================================

#[test]
fn test_match_basic() {
    // κατὰ ξ· μηδὲν ᾖ, «μηδέν»· ἓν ᾖ, «ἕν»· ἄλλο ᾖ, «ἄλλο».
    // "according to x: if zero, 'zero'; if one, 'one'; if other, 'other'"
    let source =
        "ξ πέντε ἔστω. κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἓν ᾖ, «ἕν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("match"), "Expected match expression");
}

#[test]
fn test_match_wildcard() {
    // ἄλλο represents wildcard/default case
    let source = "ξ πέντε ἔστω. κατὰ ξ· μηδὲν ᾖ, «μηδέν» λέγε· ἄλλο ᾖ, «ἄλλο» λέγε.";
    let output = compile_to_rust(source);

    assert!(output.contains("_"), "Expected wildcard pattern");
}

// =============================================================================
// Loop Control
// =============================================================================

#[test]
fn test_break() {
    // παῦε means "stop" - translates to break
    let source = "ἀπὸ μηδενὸς μέχρι δέκα, εἰ ι πέντε μεῖζον ᾖ, παῦε.";
    let output = compile_to_rust(source);

    eprintln!("Generated output:\n{}", output);
    assert!(output.contains("break"), "Expected break statement");
}

#[test]
fn test_continue() {
    // συνέχιζε means "continue" - translates to continue
    let source = "ἀπὸ μηδενὸς μέχρι δέκα, εἰ ι τρία ἔλαττον ᾖ, συνέχιζε.";
    let output = compile_to_rust(source);

    assert!(output.contains("continue"), "Expected continue statement");
}

// =============================================================================
// Control Flow Particles in Lexicon
// =============================================================================

#[test]
fn test_lexicon_conditional_particles() {
    use glossa::morphology::lexicon;

    // These should be recognized as control flow particles
    assert!(
        lexicon::is_conditional_particle("ει"),
        "εἰ should be conditional"
    );
    assert!(
        lexicon::is_conditional_particle("εαν"),
        "ἐάν should be conditional"
    );
    assert!(
        lexicon::is_conditional_particle("ην"),
        "ἤν should be conditional"
    );
}

#[test]
fn test_lexicon_else_particle() {
    use glossa::morphology::lexicon;

    // "εἰ δὲ μή" is the else pattern
    assert!(
        lexicon::is_else_pattern("ει δε μη"),
        "εἰ δὲ μή should be else"
    );
}

#[test]
fn test_lexicon_loop_particles() {
    use glossa::morphology::lexicon;

    assert!(
        lexicon::is_loop_particle("εως"),
        "ἕως should be loop particle (while)"
    );
    assert!(
        lexicon::is_loop_particle("δια"),
        "διά should be loop particle (for)"
    );
    assert!(
        lexicon::is_range_particle("απο"),
        "ἀπό should be range start"
    );
    assert!(
        lexicon::is_range_particle("μεχρι"),
        "μέχρι should be range end (exclusive)"
    );
}

#[test]
fn test_lexicon_loop_control() {
    use glossa::morphology::lexicon;

    assert!(lexicon::is_break_verb("παυε"), "παῦε should be break");
    assert!(
        lexicon::is_continue_verb("συνεχιζε"),
        "συνέχιζε should be continue"
    );
}

#[test]
fn test_lexicon_match_particle() {
    use glossa::morphology::lexicon;

    assert!(
        lexicon::is_match_particle("κατα"),
        "κατά should be match particle"
    );
}
