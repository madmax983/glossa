#[test]
fn test_run_weave_success() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "«χαῖρε κόσμε» λέγε.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::weave::run_weave(temp_file.path());
    assert!(result.is_ok(), "Weave failed: {:?}", result.err());

    let output_path = temp_file.path().with_extension("md");
    assert!(output_path.exists());

    let mut f = std::fs::File::open(&output_path).unwrap();
    let mut md = String::new();
    std::io::Read::take(&mut f, 1024 * 1024 + 1)
        .read_to_string(&mut md)
        .unwrap();

    assert!(md.contains("# Rosetta Stone"));
    assert!(md.contains("```glossa"));
    assert!(md.contains("«χαῖρε κόσμε» λέγε."));
    assert!(md.contains("## 🧩 Semantic Assembly (Mosaic)"));
    assert!(md.contains("## 🦀 Generated Rust Code"));
    assert!(md.contains("```rust"));
    assert!(md.contains("println"));
}
