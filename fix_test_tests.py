import re

with open('src/tools/tester.rs', 'r') as f:
    tester = f.read()

# Add an exploit test
exploit_test = """
    #[test]
    fn test_extract_failures_with_empty_output() {
        let output = "";
        let failures = extract_failures(output);
        assert!(failures.is_empty());
    }"""

new_exploit = """
    #[test]
    fn test_extract_failures_with_empty_output() {
        let output = "";
        let failures = extract_failures(output);
        assert!(failures.is_empty());
    }

    #[test]
    fn test_warden_exploit_tester_unbounded_memory() {
        // Exploit attempt: Very large output to see if it causes an OOM panic or stalls
        // In actual scenarios we just simulate parsing a huge string
        let mut massive_output = String::with_capacity(10_000_000);
        massive_output.push_str("failures:\\n\\n");
        for i in 0..10_000 {
            massive_output.push_str(&format!("---- test_{} stdout ----\\nSome failure details\\n", i));
        }
        massive_output.push_str("failures:\\n");
        for i in 0..10_000 {
            massive_output.push_str(&format!("    test_{}\\n", i));
        }
        // Instead of executing, we test the extract_failures function which is where the DoS vector would be
        let failures = extract_failures(&massive_output);
        assert_eq!(failures.len(), 10_000);
    }"""

tester = tester.replace(exploit_test, new_exploit)

with open('src/tools/tester.rs', 'w') as f:
    f.write(tester)
