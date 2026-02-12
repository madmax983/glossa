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
