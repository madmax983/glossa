#![allow(missing_docs)]

use glossa::parser::parse;
use glossa::semantic::analyze_program;
use glossa::tools::*;
use std::path::Path;

#[test]
fn test_tools_facade_coverage() {
    let _ = Cache::new();

    let source = "ξ 5 ἔστω.";
    let ast = parse(source).unwrap();
    let _program = analyze_program(&ast).unwrap();

    let _: fn(&str) -> miette::Result<()> = lookup_word;
    let _: fn(&str) -> Result<String, glossa::errors::GlossaError> = highlight;
    let _: fn(&glossa::semantic::AnalyzedProgram) -> String = tell_tale;
    let _: fn() -> miette::Result<()> = run_repl;

    let _: fn(&str) -> miette::Result<glossa::semantic::AnalyzedProgram> = analyze_source;
    let _: fn(&Path) -> miette::Result<()> = bard_file;
    let _: fn(&Path, Option<&Path>) -> miette::Result<()> = build_file;
    let _: fn(&Path) -> miette::Result<()> = check_file;
    let _: fn(&Path) -> miette::Result<()> = highlight_file;
    let _: fn(&Path) -> miette::Result<()> = report_file;
    let _: fn(&Path) -> miette::Result<()> = run_file;

    let _: fn(&Path) -> miette::Result<()> = run_tests;

    // Call Status to satisfy it
    let mut status = Status::start("Test");
    status.update("Update");
    status.success();
}

#[cfg(feature = "nova")]
#[test]
fn test_nova_tools_facade_coverage() {
    let _: fn(&std::path::Path) -> miette::Result<()> = run_alchemist;
    let _: fn(&glossa::semantic::AnalyzedProgram) -> String = transpile_to_python;

    let _: fn(&std::path::Path) -> miette::Result<()> = run_auditor;

    let _: fn(&glossa::semantic::AnalyzedProgram) -> String = generate_map;
    let _: fn(&std::path::Path) -> miette::Result<()> = run_map;

    let _: fn(&std::path::Path) -> miette::Result<()> = run_labyrinth;
    let _: fn(&glossa::semantic::AnalyzedProgram) -> String = generate_cfg;

    let _: fn() -> miette::Result<()> = run_mentor;

    let _: fn(&std::path::Path) -> miette::Result<()> = run_mosaic;
    let _: fn(&str, &mut std::io::Sink) -> miette::Result<()> = run_mosaic_inner::<std::io::Sink>;

    let _: fn(&std::path::Path) -> miette::Result<()> = run_papyrus;

    let _: fn(&std::path::Path) -> miette::Result<()> = run_weave;
}
