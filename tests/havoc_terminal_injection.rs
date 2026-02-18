use glossa::tools::highlight::highlight;

#[test]
fn test_terminal_injection_vulnerability() {
    // 👺 HAVOC: The input string contains ANSI escape code for Red (31m)
    // If the highlighter blindly writes this, it will color the terminal output
    // outside of its control, proving injection.
    let evil_input = "«\x1b[31mEVIL» λέγε.";

    let result = highlight(evil_input).unwrap();

    // The result should contain the escaped version (e.g. \u{1b}[31m)
    // NOT the raw escape code.
    // If it contains the raw escape code, it means we have terminal injection.

    // We expect this test to FAIL if the vulnerability exists.
    // The vulnerability is that raw escape codes are passed through.
    // So we assert that the result DOES NOT contain the raw escape code.

    // Note: highlight() adds its own escape codes. We want to make sure OUR payload is neutralized.
    // The payload is "\x1b[31m".
    // If sanitized, it should look like "\\x1b[31m" or similar visible text.

    let raw_payload = "\x1b[31m";

    assert!(
        !result.contains(raw_payload),
        "Terminal Injection Vulnerability: Input string literal contained raw ANSI escape codes in output!\nOutput: {:?}",
        result
    );
}
