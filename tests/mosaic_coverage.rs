
#[test]
fn test_mosaic_file_read_error() {
    // Test the run_mosaic wrapper function with a non-existent file
    let path = std::path::PathBuf::from("non_existent_file_for_coverage_test.gl");
    let result = glossa::tools::mosaic::run_mosaic(&path);
    assert!(result.is_err());
}
