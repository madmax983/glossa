#![cfg(feature = "nova")]

use glossa::tools::simulator::run_simulator;

#[test]
fn test_run_simulator_empty_output() {
    let dir = tempfile::tempdir().unwrap();
    let input_path = dir.path().join("empty_output.γλ");
    // Just evaluate an expression statement, which shouldn't print anything
    std::fs::write(&input_path, "1.").unwrap();

    let result = run_simulator(&input_path);
    assert!(result.is_ok());
}
