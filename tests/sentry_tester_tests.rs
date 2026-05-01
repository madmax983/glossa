use glossa::tools::tester::{TestResult, TestStatus, parse_test_output};

#[test]
fn test_print_test_results_coverage() {
    let output = "
running 2 tests
test test_one ... ok
test test_two ... FAILED
test test_three ... ignored

test result: FAILED. 1 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s
";
    let results = parse_test_output(output);
    assert_eq!(results.len(), 3);
}
