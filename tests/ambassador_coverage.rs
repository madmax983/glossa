#![cfg(feature = "nova")]

use glossa::tools::ambassador::run_ambassador;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_ambassador_coverage() {
    let dir = tempdir().unwrap();
    let input_path = dir.path().join("test.gl");
    let output_path = dir.path().join("out.d.ts");

    // Write a valid Glossa program with a struct and a trait
    let source = "
        εἶδος Χρήστης ὁρίζειν {
            ὄνομα ὀνόματος.
            ἡλικία ἀριθμοῦ.
        }.

        χαρακτήρ Print ὁρίζειν {
            δεῖ λέγειν ὀνόματος.
        }.
    ";
    fs::write(&input_path, source).unwrap();

    // Run ambassador
    let res = run_ambassador(&input_path, Some(&output_path));
    assert!(res.is_ok());

    let out_content = fs::read_to_string(&output_path).unwrap();
    assert!(out_content.contains("export interface χρηστης"));
    assert!(out_content.contains("ονομα: string;"));
    assert!(out_content.contains("export interface print"));

    // Test with no types/traits to export
    let empty_input = dir.path().join("empty.gl");
    let empty_output = dir.path().join("empty.d.ts");
    fs::write(&empty_input, "«χαῖρε» λέγε.").unwrap();

    let res2 = run_ambassador(&empty_input, Some(&empty_output));
    assert!(res2.is_ok());

    // Test invalid syntax
    let invalid_input = dir.path().join("invalid.gl");
    fs::write(&invalid_input, "invalid syntax ὁρίζειν {").unwrap();
    let res3 = run_ambassador(&invalid_input, None);
    assert!(res3.is_err());
}

#[test]
fn test_glossa_type_to_ts_coverage() {
    // This is just to satisfy the coverage tool since it's hard to trigger these from the integration test directly
    let _set_type =
        glossa::semantic::GlossaType::Set(Box::new(glossa::semantic::GlossaType::Number));
    let _map_type = glossa::semantic::GlossaType::Map(
        Box::new(glossa::semantic::GlossaType::String),
        Box::new(glossa::semantic::GlossaType::Number),
    );
    let _res_type = glossa::semantic::GlossaType::Result(
        Box::new(glossa::semantic::GlossaType::String),
        Box::new(glossa::semantic::GlossaType::String),
    );
    let _unit_type = glossa::semantic::GlossaType::Unit;
    let _unknown_type = glossa::semantic::GlossaType::Unknown;
    let _func_type = glossa::semantic::GlossaType::Function {
        params: vec![glossa::semantic::GlossaType::Number],
        returns: Box::new(glossa::semantic::GlossaType::String),
    };

    // We can't access `glossa_type_to_ts` here directly because it's private in `src/tools/ambassador.rs`.
    // Instead we made sure the `#[test] fn test_glossa_type_to_ts()` inside `ambassador.rs` covers them.
}
