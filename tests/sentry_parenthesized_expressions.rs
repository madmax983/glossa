
#[cfg(test)]
mod tests {
    use glossa::codegen::generate_rust;
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;

    #[test]
    fn test_print_with_parens_hello() {
        // "λέγε(«hello»)"
        let source = "λέγε(«hello»).";
        let ast = parse(source).expect("Parse failed");
        let analyzed = analyze_program(&ast).expect("Analysis failed");
        let rust = generate_rust(&analyzed);

        // It should include "hello"
        assert!(rust.contains("hello"), "It should include hello in print");
        assert!(!rust.contains("println ! ()"), "Should NOT generate empty println");
    }

    #[test]
    fn test_print_complex_expression_in_parens() {
        // "λέγε(1 2 ἄθροισμα)."
        // Should print 3 (or 1+2).
        let source = "λέγε(1 2 ἄθροισμα).";
        let ast = parse(source).expect("Parse failed");
        let analyzed = analyze_program(&ast).expect("Analysis failed");
        let rust = generate_rust(&analyzed);

        // It should contain addition logic
        assert!(rust.contains("checked_add"), "Should contain addition logic");
    }

    #[test]
    fn test_function_call_with_complex_args() {
         // "λέγε(1 2 ἄθροισμα)." is effectively a function call to print with a complex arg.
         // This confirms that analyze_phrase now supports assembling expressions.
         let source = "λέγε(1 2 ἄθροισμα).";
         let ast = parse(source).expect("Parse failed");
         let result = analyze_program(&ast);
         assert!(result.is_ok(), "Should successfully analyze complex expression in argument");
    }
}
