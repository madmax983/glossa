#![allow(missing_docs)]
#![cfg(feature = "nova")]

use glossa::tools::auditor::run_auditor;

#[test]
fn test_auditor_handles_non_existent_file() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_auditor(std::path::Path::new("does_not_exist.glossa"));
    assert!(result.is_err());
    Ok(())
}

#[test]
fn test_auditor_handles_directory() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let result = run_auditor(dir.path());
    assert!(result.is_err());
    Ok(())
}
