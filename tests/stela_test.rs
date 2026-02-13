use glossa::tools::stela::generate_docs;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_stela_generation() {
    // 1. Create a sample Glossa file
    let source = r#"
// This is the user type
// It represents a hero
εἶδος Ήρωας ὁρίζειν {
    ὄνομα ὀνόματος.
}.

// This is a test
// It checks if the hero is brave
δοκιμή «bravery test».
    αληθες δεῖ.
τέλος.

// A regular statement with docs
// Say hello
«χαῖρε» λέγε.

«ignored» λέγε.
"#;

    // We need a file with .γλ extension for realism, but stela doesn't enforce it strictly
    let input_file = NamedTempFile::new().unwrap();
    fs::write(input_file.path(), source).unwrap();

    let output_file = NamedTempFile::new().unwrap();
    let output_path = output_file.path();

    // 2. Run Stela
    generate_docs(input_file.path(), Some(output_path)).expect("Stela generation failed");

    // 3. Check output
    let html = fs::read_to_string(output_path).unwrap();

    // Debug print
    println!("{}", html);

    // Assertions
    assert!(html.contains("Stela of"), "Title missing");
    assert!(html.contains("This is the user type"), "Type doc missing");
    assert!(
        html.contains("It represents a hero"),
        "Type doc missing (line 2)"
    );
    assert!(html.contains("Type (Εἶδος)"), "Type kind missing");
    assert!(html.contains("Ήρωας"), "Type name missing"); // Normalized or original? The code uses original text from pest pair

    assert!(html.contains("This is a test"), "Test doc missing");
    assert!(html.contains("Test (Δοκιμή)"), "Test kind missing");
    assert!(html.contains("bravery test"), "Test name missing");

    assert!(
        html.contains("A regular statement with docs"),
        "Statement doc missing"
    );
    assert!(html.contains("Statement"), "Statement kind missing");
    assert!(html.contains("«χαῖρε» λέγε"), "Statement signature missing");

    // The undocumented statement should NOT be in the output (unless I changed logic)
    // Actually, analyze_statement logic:
    // _ => { if docs.is_empty() { return None; } ... }
    // So "ignored" should NOT be present as a documented item block.
    // However, the signature might be present if it was accidentally parsed.
    // But since it has no docs, it returns None.
    // So "Undocumented statement" comment is NOT there, and "«ignored» λέγε." code is NOT there.

    assert!(
        !html.contains("«ignored»"),
        "Undocumented statement shouldn't be documented"
    );
}
