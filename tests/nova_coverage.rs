#![allow(missing_docs)]
#![cfg(feature = "nova")]

use glossa::tools::tester::run_tests;

use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use tempfile::Builder;

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

#[test]
fn test_run_papyrus_success() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }. ξ 5 ἔστω.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::papyrus::run_papyrus(temp_file.path());
    assert!(result.is_ok(), "Papyrus failed: {:?}", result.err());
}

#[test]
fn test_run_papyrus_file_not_found() {
    let path = PathBuf::from("non_existent_file.gl");
    let result = glossa::tools::papyrus::run_papyrus(&path);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Ἀρχεῖον οὐχ εὑρέθη")
    );
}

#[test]
fn test_run_papyrus_file_too_large() {
    let dir = Builder::new().prefix("papyrus_large").tempdir().unwrap();
    let input_path = dir.path().join("too_large.γλ");

    let max_size = 1024 * 1024;
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&input_path).unwrap();
        let data = vec![0u8; max_size + 1];
        f.write_all(&data).unwrap();
    }

    let result = glossa::tools::papyrus::run_papyrus(&input_path);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Ἀρχεῖον λίαν μέγα")
    );
}

#[test]
fn test_run_papyrus_syntax_error() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "invalid syntax").expect("Failed to write");

    let result = glossa::tools::papyrus::run_papyrus(temp_file.path());
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Parse error"));
}

#[test]
fn test_run_papyrus_semantic_error() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "ψ πέντε γίγνεται.").expect("Failed to write");

    let result = glossa::tools::papyrus::run_papyrus(temp_file.path());
    assert!(result.is_err());
    let err_str = result.unwrap_err().to_string();
    assert!(
        err_str.contains("Semantic error")
            || err_str.contains("Analysis error")
            || err_str.contains("Σφάλμα")
    );
}

#[test]
fn test_run_tests_no_tests_found() {
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "ξ 1 ἔστω.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = run_tests(temp_file.path());
    assert!(result.is_ok(), "Test runner failed: {:?}", result.err());
}

#[test]
fn test_run_tests_success() {
    // Create a temporary Glossa file
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    // Write a passing test
    // Test: Let x be 5. Assert x equals 5.
    let source = "
    δοκιμή «test_simple».
       ξ πέντε ἔστω.
       ξ πέντε ἰσοῦται.
    τέλος.
    ";

    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    // Run the tester
    let result = run_tests(temp_file.path());

    // Should succeed
    assert!(result.is_ok(), "Test runner failed: {:?}", result.err());
}

#[test]
fn test_run_tests_failure() {
    // Create a temporary Glossa file
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    // Write a failing test
    // Test: Let x be 4. Assert x equals 5.
    let source = "
    δοκιμή «test_fail».
       ξ τέσσαρα ἔστω.
       ξ πέντε ἰσοῦται.
    τέλος.
    ";

    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    // Run the tester
    let result = run_tests(temp_file.path());

    // Should fail
    assert!(result.is_err(), "Test runner should have failed");
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Tests failed"));
}

#[test]
fn test_run_tests_file_not_found() {
    let path = PathBuf::from("non_existent_file.gl");
    let result = run_tests(&path);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Ἀρχεῖον οὐχ εὑρέθη")
    );
}

#[test]
fn test_run_tests_syntax_error() {
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "invalid syntax").expect("Failed to write");

    let result = run_tests(temp_file.path());
    assert!(result.is_err());
    // Error could be from parser or analyzer, but it should fail
}

#[test]
fn test_run_scholar_success() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }.

    χαρακτήρ Εὐγενής ὁρίζειν { δεῖ show τῷ self. }.

    χαιρετισμός ὁρίζειν· «χαῖρε» λέγε.

    «γεια» λέγε.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::scholar::run_scholar(temp_file.path());
    assert!(result.is_ok(), "Scholar failed: {:?}", result.err());

    let output_path = temp_file.path().with_extension("doc.md");
    assert!(output_path.exists());

    let mut f = std::fs::File::open(&output_path).unwrap();
    let mut md = String::new();
    std::io::Read::take(&mut f, 1024 * 1024 + 1)
        .read_to_string(&mut md)
        .unwrap();

    assert!(md.contains("# API Documentation"));
    assert!(md.contains("## Types (Εἴδη)"));
    assert!(md.contains("### `χρηστης`"));
    assert!(md.contains("## Traits (Χαρακτῆρες)"));
    assert!(md.contains("### `ευγενης`"));
    assert!(md.contains("## Functions (Ἔργα)"));
    assert!(md.contains("### `χαιρετισμος() -> Οὐδέν`"));
}

#[test]
fn test_run_scholar_file_not_found() {
    let path = PathBuf::from("non_existent_file.gl");
    let result = glossa::tools::scholar::run_scholar(&path);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Ἀρχεῖον οὐχ εὑρέθη")
    );
}

#[test]
fn test_run_scholar_syntax_error() {
    let mut temp_file = Builder::new()
        .suffix(".gl")
        .tempfile()
        .expect("Failed to create temp file");

    write!(temp_file, "invalid syntax").expect("Failed to write");

    let result = glossa::tools::scholar::run_scholar(temp_file.path());
    assert!(result.is_err());
}

// removed test_run_tests_rustc_error because of environment variable pollution causing intermittent failures in parallel execution and it being redundant to runner tests

#[test]
fn test_run_diplomat_success() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "
    εἶδος Χρήστης ὁρίζειν {
        ὄνομα ὀνόματος.
        ἡλικία ἀριθμοῦ.
    }.
    ";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::diplomat::run_diplomat(temp_file.path());
    assert!(result.is_ok(), "Diplomat failed: {:?}", result.err());

    let output_path = temp_file.path().with_extension("d.ts");
    assert!(output_path.exists());

    let mut f = std::fs::File::open(&output_path).unwrap();
    let mut ts = String::new();
    std::io::Read::take(&mut f, 1024 * 1024 + 1)
        .read_to_string(&mut ts)
        .unwrap();

    assert!(ts.contains("export interface χρηστης {"));
    assert!(ts.contains("ονομα: string;"));
    assert!(ts.contains("ηλικια: number;"));
}

#[test]
fn test_run_diplomat_file_not_found() {
    let input_path = std::path::Path::new("nonexistent.gl");
    let result = glossa::tools::diplomat::run_diplomat(input_path);
    assert!(result.is_err());
}

#[test]
fn test_run_diplomat_analysis_error() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "εἶδος Χρήστης"; // Invalid syntax
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::diplomat::run_diplomat(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_run_diplomat_function_def() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "
    προσθεσις ὁρίζειν τῷ ξ ἀριθμοῦ τῷ ψ ἀριθμοῦ· δός ξ ψ ἄθροισμα.
    ";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::diplomat::run_diplomat(temp_file.path());
    assert!(result.is_ok(), "Diplomat failed: {:?}", result.err());

    let output_path = temp_file.path().with_extension("d.ts");
    assert!(output_path.exists());

    let mut f = std::fs::File::open(&output_path).unwrap();
    let mut ts = String::new();
    std::io::Read::take(&mut f, 1024 * 1024 + 1)
        .read_to_string(&mut ts)
        .unwrap();

    assert!(ts.contains("export declare function προσθεσις(ξ: number, ψ: number): number;"));
}

#[test]
fn test_run_diplomat_unsupported_statement() {
    let mut temp_file = Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "«χαῖρε κόσμε» λέγε.";
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let result = glossa::tools::diplomat::run_diplomat(temp_file.path());
    assert!(result.is_ok(), "Diplomat failed: {:?}", result.err());
}

#[test]
fn test_run_diplomat_write_error() {
    let mut temp_file = tempfile::Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "
    εἶδος Χρήστης ὁρίζειν {
        ὄνομα ὀνόματος.
    }.
    ";
    use std::io::Write;
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    let output_path = temp_file.path().with_extension("d.ts");

    // Create a directory where the file should be to cause a write error
    std::fs::create_dir(&output_path).unwrap();

    let result = glossa::tools::diplomat::run_diplomat(temp_file.path());
    assert!(result.is_err());

    // Clean up
    std::fs::remove_dir(&output_path).unwrap();
}

#[test]
fn test_run_diplomat_load_source_error() {
    let mut temp_file = tempfile::Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "
    εἶδος Χρήστης ὁρίζειν {
        ὄνομα ὀνόματος.
    }.
    ";
    use std::io::Write;
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    // Remove read permissions from the file to cause load_source to fail
    let mut perms = std::fs::metadata(temp_file.path()).unwrap().permissions();
    perms.set_readonly(true); // actually need to make it unreadable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o000);
        std::fs::set_permissions(temp_file.path(), perms).unwrap();
    }

    let result = glossa::tools::diplomat::run_diplomat(temp_file.path());
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Permission denied") || err_msg.contains("could not read"));

    // Restore permissions so it can be deleted
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(temp_file.path()).unwrap().permissions();
        perms.set_mode(0o644);
        std::fs::set_permissions(temp_file.path(), perms).unwrap();
    }
}

#[test]
fn test_run_diplomat_load_source_io_error() {
    let mut temp_file = tempfile::Builder::new()
        .suffix(".γλ")
        .tempfile()
        .expect("Failed to create temp file");

    let source = "
    εἶδος Χρήστης ὁρίζειν {
        ὄνομα ὀνόματος.
    }.
    ";
    use std::io::Write;
    write!(temp_file, "{}", source).expect("Failed to write to temp file");

    // create a directory with the same name as the file
    let dir_path = temp_file.path().with_extension("dir");
    std::fs::create_dir(&dir_path).unwrap();

    let result = glossa::tools::diplomat::run_diplomat(&dir_path);
    assert!(result.is_err());

    std::fs::remove_dir(&dir_path).unwrap();
}
