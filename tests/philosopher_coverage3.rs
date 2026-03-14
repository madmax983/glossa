#[cfg(test)]
mod tests {
    use glossa::semantic::{AnalyzedProgram, AnalyzedStatement, AnalyzedExpr, AnalyzedExprKind, GlossaType, Scope};
    use glossa::tools::philosopher::{analyze_program_smells, CodeSmell};

    fn get_smells(program: &AnalyzedProgram) -> Vec<CodeSmell> {
        let mut smells = Vec::new();
        analyze_program_smells(program, &mut smells);
        smells
    }

    #[test]
    fn test_coverage_other_match_arms() {
        let program = AnalyzedProgram {
            statements: vec![
                AnalyzedStatement::If {
                    condition: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::StringLiteral("".to_string()), glossa_type: GlossaType::Unknown }),
                    then_body: vec![],
                    else_body: Some(vec![AnalyzedStatement::Break]),
                },
                AnalyzedStatement::Match {
                    scrutinee: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::StringLiteral("".to_string()), glossa_type: GlossaType::Unknown }),
                    arms: vec![
                        (AnalyzedExpr { expr: AnalyzedExprKind::StringLiteral("".to_string()), glossa_type: GlossaType::Unknown }, vec![AnalyzedStatement::Break]),
                    ],
                },
                AnalyzedStatement::While {
                    condition: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::StringLiteral("".to_string()), glossa_type: GlossaType::Unknown }),
                    body: vec![AnalyzedStatement::Break],
                },
                AnalyzedStatement::For {
                    variable: smol_str::SmolStr::new("x"),
                    iterator: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::StringLiteral("".to_string()), glossa_type: GlossaType::Unknown }),
                    body: vec![AnalyzedStatement::Break],
                },
                AnalyzedStatement::Break,
                AnalyzedStatement::Continue,
                AnalyzedStatement::Return { value: None },
            ],
            scope: Scope::new(),
        };

        get_smells(&program);
    }
}
