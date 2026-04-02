use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::semantic::{AnalyzedExpr, AnalyzedExprKind, AnalyzedProgram, AnalyzedStatement};
use crossterm::style::Stylize;
use smol_str::SmolStr;
use std::collections::HashSet;
use std::path::Path;

/// Represents an analysis finding from the Augur.
#[derive(Debug, PartialEq)]
pub enum AugurFinding {
    UnusedVariable(SmolStr),
}

/// The Augur static analyzer.
pub struct Augur {
    defined_vars: HashSet<SmolStr>,
    used_vars: HashSet<SmolStr>,
}

impl Default for Augur {
    fn default() -> Self {
        Self::new()
    }
}

impl Augur {
    pub fn new() -> Self {
        Self {
            defined_vars: HashSet::new(),
            used_vars: HashSet::new(),
        }
    }

    pub fn analyze(&mut self, program: &AnalyzedProgram) -> Vec<AugurFinding> {
        self.defined_vars.clear();
        self.used_vars.clear();

        for stmt in &program.statements {
            self.visit_statement(stmt);
        }

        let mut findings = Vec::new();
        for var in &self.defined_vars {
            if !self.used_vars.contains(var) {
                findings.push(AugurFinding::UnusedVariable(var.clone()));
            }
        }
        findings
    }

    fn visit_statement(&mut self, stmt: &AnalyzedStatement) {
        match stmt {
            AnalyzedStatement::Binding { name, value, .. } => {
                self.defined_vars.insert(name.clone());
                self.visit_expr(value);
            }
            AnalyzedStatement::Assignment { name, value } => {
                self.used_vars.insert(name.clone());
                self.visit_expr(value);
            }
            AnalyzedStatement::Print(exprs) | AnalyzedStatement::Query(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            AnalyzedStatement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.visit_expr(condition);
                for stmt in then_body {
                    self.visit_statement(stmt);
                }
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.visit_statement(stmt);
                    }
                }
            }
            AnalyzedStatement::Expression(exprs) => {
                for expr in exprs {
                    self.visit_expr(expr);
                }
            }
            // Add remaining matches as needed
            _ => {}
        }
    }

    fn visit_expr(&mut self, expr: &AnalyzedExpr) {
        match &expr.expr {
            AnalyzedExprKind::Variable(name) => {
                self.used_vars.insert(name.clone());
            }
            AnalyzedExprKind::BinOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            AnalyzedExprKind::Try(e) | AnalyzedExprKind::Unwrap(e) => {
                self.visit_expr(e);
            }
            AnalyzedExprKind::UnaryOp { operand, .. } => {
                self.visit_expr(operand);
            }
            // Add remaining matches as needed
            _ => {}
        }
    }
}

pub fn run_augur(input: &Path) -> miette::Result<()> {
    let source = crate::tools::runner::load_source(input)?;

    let status = crate::tools::ui::Status::start_with_symbol("Οἰωνός (Analyzing semantics)", "🦅");

    let ast = match parse(&source) {
        Ok(a) => a,
        Err(e) => {
            status.error("Σφάλμα συντάξεως (Syntax Error)");
            return Err(miette::miette!("Parse error: {}", e));
        }
    };
    let program = match analyze_program(&ast) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα σημασίας (Semantic Error)");
            return Err(miette::miette!("Semantic error: {}", e));
        }
    };

    let mut augur = Augur::new();
    let findings = augur.analyze(&program);

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A U G U R".bold().cyan());
    println!("   {}", "Semantic Analysis Report".italic().dim());
    println!();

    if findings.is_empty() {
        println!("   {}", "✨ No warnings found.".green());
    } else {
        for finding in findings {
            match finding {
                AugurFinding::UnusedVariable(name) => {
                    println!(
                        "   {} Variable `{}` is defined but never used.",
                        "⚠️".yellow(),
                        name.yellow()
                    );
                }
            }
        }
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    fn analyze_code(code: &str) -> Vec<AugurFinding> {
        let ast = parse(code).expect("Failed to parse");
        let program = analyze_program(&ast).expect("Failed to analyze");
        let mut augur = Augur::new();
        augur.analyze(&program)
    }

    #[test]
    fn test_augur_unused_variable() {
        let code = "ξ πέντε ἔστω.";
        let findings = analyze_code(code);
        assert_eq!(findings.len(), 1);
        assert!(matches!(findings[0], AugurFinding::UnusedVariable(ref name) if name == "ξ"));
    }

    #[test]
    fn test_augur_used_variable() {
        let code = "ξ πέντε ἔστω. ξ λέγε.";
        let findings = analyze_code(code);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_augur_used_variable_in_query() {
        let code = "ξ πέντε ἔστω. ξ λέγε?";
        let findings = analyze_code(code);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_augur_used_variable_in_assignment() {
        let code = "μετά ξ πέντε ἔστω. ξ δέκα γίγνεται.";
        let findings = analyze_code(code);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_augur_used_variable_in_binop() {
        let code = "ξ πέντε ἔστω. ψ 2 ἔστω. ζ τοῦ ξ τοῦ ψ ἄθροισμα ἔστω.";
        let findings = analyze_code(code);

        let has_psi = findings
            .iter()
            .any(|f| matches!(f, AugurFinding::UnusedVariable(name) if name == "ψ"));
        let has_xi = findings
            .iter()
            .any(|f| matches!(f, AugurFinding::UnusedVariable(name) if name == "ξ"));
        let has_zeta = findings
            .iter()
            .any(|f| matches!(f, AugurFinding::UnusedVariable(name) if name == "ζ"));

        assert!(!has_xi, "ξ should be marked as used");
        assert!(!has_psi, "ψ should be marked as used");
        assert!(has_zeta, "ζ should be marked as unused");
    }

    #[test]
    fn test_augur_used_variable_in_unaryop() {
        let code = "ξ ἀληθές ἔστω. ψ ξ! ἔστω.";
        let findings = analyze_code(code);

        let has_psi = findings
            .iter()
            .any(|f| matches!(f, AugurFinding::UnusedVariable(name) if name == "ψ"));
        let has_xi = findings
            .iter()
            .any(|f| matches!(f, AugurFinding::UnusedVariable(name) if name == "ξ"));
        assert!(!has_xi, "ξ should be marked as used");
        assert!(has_psi, "ψ should be marked as unused");
    }

    #[test]
    fn test_augur_if_statement() {
        let code = "ξ ἀληθές ἔστω. ψ πέντε ἔστω. εἰ ξ ἐστι, ψ λέγε.";
        let findings = analyze_code(code);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_augur_if_else_statement() {
        let code = "ξ ἀληθές ἔστω. ψ πέντε ἔστω. εἰ ξ ἐστι, «ναι» λέγε. εἰ δὲ μή, ψ λέγε.";
        let findings = analyze_code(code);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_run_augur_success() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("augur_test.γλ");
        std::fs::write(&input_path, "ξ πέντε ἔστω. ξ λέγε.").unwrap();

        let result = run_augur(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_augur_unused() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("augur_unused.γλ");
        std::fs::write(&input_path, "ξ πέντε ἔστω.").unwrap();

        let result = run_augur(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_augur_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("augur_parse_error.γλ");
        std::fs::write(&input_path, "invalid syntax").unwrap();

        let result = run_augur(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Parse error"));
    }

    #[test]
    fn test_run_augur_semantic_error() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("augur_semantic_error.γλ");
        std::fs::write(&input_path, "ψ 10 γίγνεται.").unwrap();

        let result = run_augur(&input_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Semantic error"));
    }
}
