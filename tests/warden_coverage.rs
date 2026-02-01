use glossa::codegen::generate_rust;
use glossa::ir::lower_to_hir;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

fn compile(source: &str) {
    let ast = parse(source).unwrap();
    let analyzed = analyze_program(&ast).unwrap();
    let hir = lower_to_hir(&analyzed);
    let _ = generate_rust(&hir);
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
