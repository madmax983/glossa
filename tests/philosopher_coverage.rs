#[cfg(test)]
mod tests {
    use glossa::semantic::{AnalyzedProgram, AnalyzedStatement, AnalyzedMethod, AnalyzedExpr, AnalyzedExprKind, GlossaType, Scope};
    use smol_str::SmolStr;
    use glossa::tools::philosopher::{analyze_program_smells, CodeSmell};
    use std::fs;
    use std::process::Command;
    use tempfile::tempdir;

    fn get_smells(program: &AnalyzedProgram) -> Vec<CodeSmell> {
        let mut smells = Vec::new();
        analyze_program_smells(program, &mut smells);
        smells
    }

    fn dummy_expr() -> AnalyzedExpr {
        AnalyzedExpr { expr: AnalyzedExprKind::StringLiteral("".to_string()), glossa_type: GlossaType::Unknown }
    }

    #[test]
    fn test_deep_nesting() {
        let mut inner = AnalyzedStatement::Expression(vec![]);
        for _ in 0..4 {
            inner = AnalyzedStatement::If {
                condition: Box::new(dummy_expr()),
                then_body: vec![inner],
                else_body: Some(vec![]),
            };
        }
        let program = AnalyzedProgram {
            statements: vec![inner],
            scope: Scope::new(),
        };
        let smells = get_smells(&program);
        assert!(smells.iter().any(|s| s.maxim == "Λαβύρινθος (Labyrinth)"));
    }

    #[test]
    fn test_too_many_params_method() {
        let method = AnalyzedMethod {
            name: SmolStr::new("test"),
            params: vec![
                (SmolStr::new("a"), GlossaType::Unknown),
                (SmolStr::new("b"), GlossaType::Unknown),
                (SmolStr::new("c"), GlossaType::Unknown),
                (SmolStr::new("d"), GlossaType::Unknown),
            ],
            body: Some(vec![]),
            return_type: None,
        };
        let program = AnalyzedProgram {
            statements: vec![AnalyzedStatement::TraitImplementation {
                trait_name: SmolStr::new("T"),
                type_name: SmolStr::new("S"),
                methods: vec![method],
            }],
            scope: Scope::new(),
        };
        let smells = get_smells(&program);
        assert!(smells.iter().any(|s| s.maxim == "Μηδὲν ἄγαν (Nothing in excess)"));
    }

    #[test]
    fn test_method_too_long() {
        let method = AnalyzedMethod {
            name: SmolStr::new("test"),
            params: vec![],
            body: Some(vec![AnalyzedStatement::Break; 11]),
            return_type: None,
        };
        let program = AnalyzedProgram {
            statements: vec![AnalyzedStatement::TraitImplementation {
                trait_name: SmolStr::new("T"),
                type_name: SmolStr::new("S"),
                methods: vec![method],
            }],
            scope: Scope::new(),
        };
        let smells = get_smells(&program);
        assert!(smells.iter().any(|s| s.maxim == "Μέτρον ἄριστον (Moderation is best)"));
    }

    #[test]
    fn test_test_declaration_too_long() {
        let program = AnalyzedProgram {
            statements: vec![AnalyzedStatement::TestDeclaration {
                name: "test".to_string(),
                body: vec![AnalyzedStatement::Break; 16],
            }],
            scope: Scope::new(),
        };
        let smells = get_smells(&program);
        assert!(smells.iter().any(|s| s.maxim == "Μέτρον ἄριστον (Moderation is best)"));
    }

    #[test]
    fn test_other_statements() {
        let program = AnalyzedProgram {
            statements: vec![
                AnalyzedStatement::While {
                    condition: Box::new(dummy_expr()),
                    body: vec![AnalyzedStatement::Break],
                },
                AnalyzedStatement::For {
                    variable: SmolStr::new("x"),
                    iterator: Box::new(dummy_expr()),
                    body: vec![AnalyzedStatement::Break],
                },
                AnalyzedStatement::Match {
                    scrutinee: Box::new(dummy_expr()),
                    arms: vec![(dummy_expr(), vec![AnalyzedStatement::Break])],
                }
            ],
            scope: Scope::new(),
        };
        let smells = get_smells(&program);
        assert!(smells.is_empty());
    }

    #[test]
    fn test_philosopher_cli_integration() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test.gl");
        let source_code = "
            πολλαπαραμ ὁρίζειν τῷ a τῷ b τῷ c τῷ d·
                a λέγε.
        ";
        fs::write(&source_path, source_code).unwrap();

        let output = Command::new(env!("CARGO"))
            .arg("run")
            .arg("--features")
            .arg("nova")
            .arg("--")
            .arg("philosopher")
            .arg(&source_path)
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Μηδὲν ἄγαν (Nothing in excess)"));
    }

    #[test]
    fn test_philosopher_cli_labyrinth_and_moderation() {
        let temp_dir = tempdir().unwrap();
        let source_path = temp_dir.path().join("test2.gl");
        let source_code = "
            τολονγ ὁρίζειν τῷ α·
                α λέγε· α λέγε· α λέγε· α λέγε· α λέγε· α λέγε·
                α λέγε· α λέγε· α λέγε· α λέγε· α λέγε· α λέγε.
        ";
        fs::write(&source_path, source_code).unwrap();

        let output = Command::new(env!("CARGO"))
            .arg("run")
            .arg("--features")
            .arg("nova")
            .arg("--")
            .arg("philosopher")
            .arg(&source_path)
            .output()
            .unwrap();

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Μέτρον ἄριστον (Moderation is best)"));
    }
}
