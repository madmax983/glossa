#![cfg(feature = "nova")]

use glossa::tools::weave::run_weave;
use std::fs;
use std::io::Write;
use tempfile::tempdir;

#[test]
fn test_weave_output() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test_weave.γλ");
    {
        let mut f = fs::File::create(&file_path).unwrap();
        f.write_all("«χαῖρε κόσμε» λέγε.\n".as_bytes()).unwrap();
    }

    let result = run_weave(&file_path);
    assert!(result.is_ok(), "Weave command failed");

    let output_path = dir.path().join("test_weave.md");
    assert!(output_path.exists(), "Markdown file was not created");

    let md_content = fs::read_to_string(&output_path).unwrap();

    // Verify sections exist
    assert!(md_content.contains("# ΓΛΩΣΣΑ Weave Report"));
    assert!(md_content.contains("## Original Source"));
    assert!(md_content.contains("## Semantic Assembly"));
    assert!(md_content.contains("## Generated Rust"));

    // Verify raw source code is included
    assert!(md_content.contains("«χαῖρε κόσμε» λέγε."));

    // Verify rust code is generated (due to pretty format printing it as println ! it might not match exact string, so match what is printed)
    assert!(md_content.contains("println !"));

    // Verify table structure
    assert!(md_content.contains(
        "| Line | Subject (Nom) | Verb (Action) | Object (Acc) | Indirect (Dat) | Other |"
    ));
}
