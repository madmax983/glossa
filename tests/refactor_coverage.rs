#[cfg(test)]
mod tests {
    use glossa::ast::{Expr, Statement, Word};
    use glossa::codegen::{generate_rust, to_rust_type};
    use glossa::errors::{GlossaError, help};
    use glossa::parser::parse_greek_numeral;
    use glossa::semantic::{AnalyzedProgram, GlossaType, Scope};
    use smol_str::SmolStr;

    // --- Errors Coverage ---

    #[test]
    fn test_error_variants_display() {
        let parse_err = GlossaError::parse("parse msg");
        assert!(parse_err.to_string().contains("Σφάλμα συντάξεως"));
        assert_eq!(parse_err.category_greek(), "Σύνταξις");

        let semantic_err = GlossaError::semantic("semantic msg");
        assert!(semantic_err.to_string().contains("Σφάλμα σημασίας"));
        assert_eq!(semantic_err.category_greek(), "Σημασία");

        let undefined_err = GlossaError::undefined("var");
        assert!(undefined_err.to_string().contains("Ἄγνωστον ὄνομα"));
        assert_eq!(undefined_err.category_greek(), "Ὄνομα");

        let agreement_err = GlossaError::agreement("agreement msg");
        assert!(agreement_err.to_string().contains("Σφάλμα συμφωνίας"));
        assert_eq!(agreement_err.category_greek(), "Συμφωνία");

        let codegen_err = GlossaError::codegen("codegen msg");
        assert!(codegen_err.to_string().contains("Σφάλμα κώδικος"));
        assert_eq!(codegen_err.category_greek(), "Κῶδιξ");

        let limit_err = GlossaError::LimitExceeded {
            resource: "res".to_string(),
            max: 10,
        };
        assert!(limit_err.to_string().contains("Ὑπέρβασις ὀρίου"));
        assert_eq!(limit_err.category_greek(), "Όριον");
    }

    #[test]
    fn test_help_constants() {
        assert!(!help::BINDING.is_empty());
        assert!(!help::PRINT.is_empty());
        assert!(!help::CASES.is_empty());
    }

    // --- AST Coverage (Derives) ---

    #[test]
    fn test_ast_derives() {
        let word = Word::new("test");
        let expr = Expr::Word(word.clone());
        let stmt = Statement::Regular {
            clauses: vec![],
            is_query: false,
            is_propagate: false,
        };

        // Exercise Clone and Debug
        let _ = word.clone();
        let _ = format!("{:?}", word);
        let _ = expr.clone();
        let _ = format!("{:?}", expr);
        let _ = stmt.clone();
        let _ = format!("{:?}", stmt);
    }

    // --- Codegen Coverage ---

    #[test]
    fn test_codegen_types() {
        // Function type coverage
        let fn_type = GlossaType::Function {
            params: vec![GlossaType::Number],
            returns: Box::new(GlossaType::Unit),
        };
        assert_eq!(to_rust_type(&fn_type), "fn"); // Basic check for now

        // Struct type
        let struct_type = GlossaType::Struct {
            name: SmolStr::from("User"),
            gender: glossa::morphology::Gender::Masculine,
            fields: vec![],
        };
        assert!(to_rust_type(&struct_type).contains("User"));
    }

    #[test]
    fn test_codegen_empty_program() {
        let program = AnalyzedProgram {
            statements: vec![],
            scope: Scope::new(),
        };
        let code = generate_rust(&program);
        assert!(code.contains("fn main"));
    }

    // --- Parser Coverage ---

    #[test]
    fn test_numerals_error() {
        assert!(parse_greek_numeral("abc").is_err());
        assert!(parse_greek_numeral("").is_err());
    }
}
