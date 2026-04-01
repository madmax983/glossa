#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use glossa::parser::parse;

    #[test]
    fn test_deeply_nested_tests_dos() {
        // Generate 20000 nested test declarations to ensure stack overflow if unchecked
        // δοκιμή "1" δοκιμή "2" ... τέλος τέλος
        let mut source = String::new();
        let depth = 20000;

        // Use a loop to avoid stack overflow during string construction if we used recursion
        source.reserve(depth * 30);

        for i in 0..depth {
            source.push_str(&format!("δοκιμή \"t{}\". ", i));
        }

        // Add a body to the innermost test
        source.push_str("ξ 1 ἔστω. ");

        for _ in 0..depth {
            source.push_str("τέλος. ");
        }

        // Now this should fail with RecursionLimitExceeded
        let result = parse(&source);

        assert!(
            result.is_err(),
            "Expected parsing to fail due to recursion limit"
        );

        let err = result.unwrap_err();
        let msg = err.to_string();

        assert!(
            msg.contains("Recursion limit exceeded"),
            "Unexpected error message: {}",
            msg
        );
    }
}
