#[cfg(test)]
mod tests {
    use glossa::codegen::generate_rust;
    use glossa::parser::parse;
    use glossa::semantic::analyze_program;

    fn compile(source: &str) -> String {
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        generate_rust(&analyzed)
    }

    #[test]
    fn test_genitive_function_call() {
        // Test calling a function where the function name is in Genitive case
        // "let x be log's (5)" -> "ξ λόγου 5 ἔστω" where λόγος is function
        // λόγος -> genitive λόγου

        let source = "
        λογος ὁρίζειν τῷ α· δός α.
        ξ λογου 5 ἔστω.
        ";

        let code = compile(source);
        println!("Genitive function call output:\n{}", code);
        // Should generate function call to logos
        assert!(code.contains("logos"));
        assert!(code.contains("(5"));
    }

    #[test]
    fn test_participle_looking_variable_binding() {
        // Test binding a variable that looks like a participle (ends in -ων)
        // "Ιασων" (Jason) ends in -ων, could be mistaken for participle
        // "Ιασων 5 ἔστω."

        let source = "Ιασων 5 ἔστω.";
        let code = compile(source);
        println!("Participle binding output:\n{}", code);

        // Should generate binding for Iason
        assert!(code.contains("iaswn") || code.contains("Iaswn"));
        assert!(code.contains("5"));
    }

    #[test]
    fn test_property_access_print_coverage() {
        // Explicit test for "user.name print" pattern to hit classify_property_access_print
        // We define variable 'π' (pi) which has known genitive 'που'
        let source = r#"
            εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.
            π νέον Χρήστης «Σωκράτης» ἔστω.
            που ὄνομα λέγε.
        "#;
        // που (genitive of π) ὄνομα (subject) λέγε (print verb)
        let code = compile(source);
        println!("Property access output:\n{}", code);

        assert!(code.contains("println"));
        // ὄνομα -> onoma
        // π -> p (transliterated)
        // Access: p.onoma
        assert!(code.contains("p.onoma") || code.contains("p . onoma"));
    }
}
