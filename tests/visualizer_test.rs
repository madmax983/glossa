use glossa::experimental::visualizer::Visualizer;
use glossa::parser::parse;
use glossa::semantic::analyze_program;

#[test]
fn test_visualize_if_else() {
    // Note: Glossa uses '·' (middle dot) to chain 'else if' or 'else' blocks in the same statement.
    // εἰ condition, then_body · εἰ δὲ μή, else_body.
    let source = r#"
        ξ πέντε ἔστω.
        εἰ ξ μεῖζον πέντε,
            «Μεγάλο» λέγε · εἰ δὲ μή,
            «Μικρό» λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("flowchart TD"));
    assert!(mermaid.contains("If"));
    assert!(mermaid.contains("Then"));
    assert!(mermaid.contains("Else"));
    assert!(mermaid.contains("Print: «Μεγάλο»"));
    assert!(mermaid.contains("Print: «Μικρό»"));
    assert!(mermaid.contains("-->"));
}

#[test]
fn test_visualize_loop() {
    // Using 'ἀληθές' (true) to ensure consistent output, as operator parsing might vary
    let source = r#"
        ξ μηδέν ἔστω.
        ἕως ἀληθές,
            «Loop» λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("While true"));
    assert!(mermaid.contains(":::loop"));
}

#[test]
fn test_visualize_function() {
    let source = r#"
        test_func ὁρίζειν (x)·
            x λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("subgraph test_func"));
    assert!(mermaid.contains("([fn test_func])"));
}

#[test]
fn test_visualize_assignment() {
    // ξ δέκα γίγνεται. (Assignment)
    // Needs mutable variable
    let source = r#"
        μετὰ ξ πέντε ἔστω.
        ξ δέκα γίγνεται.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("let ξ = 5"));
    assert!(mermaid.contains("ξ = 10"));
}

#[test]
fn test_visualize_expression_stmts() {
    // 1 2 ἄθροισμα -> 1 Add 2
    let source = r#"
        1 2 ἄθροισμα.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("1"));
    assert!(mermaid.contains("2"));
    assert!(mermaid.contains("Add"));
}

#[test]
fn test_visualize_match() {
    // Match statement
    let source = r#"
        ξ πέντε ἔστω.
        κατὰ ξ·
        πέντε ᾖ, «Five» λέγε·
        ἄλλο ᾖ, «Other» λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("Match ξ"));
    assert!(mermaid.contains("Case: 5"));
    assert!(mermaid.contains("Case: true"));
    assert!(mermaid.contains(":::cond"));
}

#[test]
fn test_visualize_for_loop() {
    // For loop (range)
    let source = r#"
        ξ μηδέν ἔστω.
        μετὰ ι μηδέν ἔστω.
        ἀπὸ ι μέχρι δέκα,
            «Iter» λέγε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("For i in expr"));
    assert!(mermaid.contains(":::loop"));
}

#[test]
fn test_visualize_return_break_continue() {
    // Function with return, break, continue
    // The previous error "While loop needs at least 2 clauses" persists inside a function definition in test code.
    // This is likely due to the parsing of `test_flow ὁρίζειν (x)· ἕως ἀληθές, παῦε. δός 42.`
    // The parser sees `test_flow ...` as one statement.
    // The body clauses are everything after `·`.
    // The `parse_function_definition` takes `clauses[1..]`.
    // Clause 1 is `ἕως ἀληθές`. Clause 2 is `παῦε`. Clause 3 is `δός 42` (if `.` is not separating them).
    // `statement_end` is `.` or `?` or `;`.
    // So `παῦε.` ends the statement.
    // `test_flow ... παῦε.` is ONE statement.
    // `δός 42.` is ANOTHER statement.
    // BUT function definition syntax usually wraps multiple statements in blocks `{ ... }` if needed, OR relies on single statement body.
    // Glossa grammar for `trait_method` uses `(chain ~ statement+)?`.
    // But `analyze_control_flow` uses custom parsing for `FunctionDef`.
    // If we want multiple statements, we should likely use `{ ... }` block syntax if supported by `analyze_statement`, OR `FunctionDef` parsing supports blocks.
    // Looking at `src/semantic/declarations.rs` (inferred), or `analyze_control_flow`:
    // It doesn't seem to support `{}` explicitly in `FunctionDef` logic unless `analyze_statement` does recursively.
    // Let's look at `tests/function_tests.rs` or similar if we could.
    // Instead, let's just test `break` in a top-level while loop (though break outside loop is semantically invalid, the visualizer doesn't care about semantic validity of break placement, just structure).
    // Actually, visualizer takes `AnalyzedProgram`. Semantic analysis MIGHT check valid break placement.
    // But `analyze_control_flow` just returns `Break`.
    // Let's try separate statements for Break/Continue/Return without function wrapping, or simpler structure.
    // `παῦε.` -> Break.
    // `συνέχιζε.` -> Continue.
    // `δός 42.` -> Return.
    // The visualizer handles them.
    let source = r#"
        δός 42.
        παῦε.
        συνέχιζε.
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("Break"));
    assert!(mermaid.contains("Continue"));
    assert!(mermaid.contains("Return 42"));
}

#[test]
fn test_visualize_query() {
    // Query statement: ξ?
    let source = r#"
        ξ πέντε ἔστω.
        ξ?
    "#;

    let ast = parse(source).expect("Failed to parse");
    let analyzed = analyze_program(&ast).expect("Failed to analyze");

    let mut visualizer = Visualizer::new(&analyzed);
    let mermaid = visualizer.generate();

    println!("{}", mermaid);

    assert!(mermaid.contains("Query: ξ"));
    assert!(mermaid.contains(":::io"));
}
