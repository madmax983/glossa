#[test]
fn havoc_codegen_index_panic() {
    let code = "ξ 1, 2, 3 ἔστω.\nψ 0 1 πλὴν ἔστω.\nξ[ψ] λέγε.\n";

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.glossa");
    std::fs::write(&file_path, code).unwrap();

    let run = std::process::Command::new(env!("CARGO_BIN_EXE_glossa"))
        .arg("run")
        .arg(&file_path)
        .output()
        .unwrap();

    assert!(!run.stderr.is_empty());
    let stderr = String::from_utf8_lossy(&run.stderr);
    assert!(stderr.contains("INTERNAL COMPILER ERROR (Codegen Failed)"));
    assert!(stderr.contains("function or associated item not found in `usize`"));
}
