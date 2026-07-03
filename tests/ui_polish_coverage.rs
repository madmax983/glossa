use glossa::tools::{alchemist, haruspex, labyrinth, mosaic, papyrus, scholar};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_ui_polish_coverage_runs() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.gls");
    fs::write(&file_path, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.").unwrap();

    // Call the tools on the test file to ensure both TTY and non-TTY paths can run
    // (though in CI, only one will be hit per OS unless we mock it).

    let _ = alchemist::run_alchemist(&file_path);
    let _ = haruspex::run_haruspex(&file_path);
    let _ = labyrinth::run_labyrinth(&file_path);
    let _ = mosaic::run_mosaic(&file_path);
    let _ = papyrus::run_papyrus(&file_path);
    let _ = scholar::run_scholar(&file_path);
}
