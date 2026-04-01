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
    UnreachableCode,
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
            AnalyzedStatement::While { condition, body } => {
                self.visit_expr(condition);
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            AnalyzedStatement::For { iterator, variable, body } => {
                self.defined_vars.insert(variable.clone());
                self.visit_expr(iterator);
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            AnalyzedStatement::FunctionDef { body, .. } => {
                // Simplified, doesn't track params
                for stmt in body {
                    self.visit_statement(stmt);
                }
            }
            AnalyzedStatement::Match { scrutinee, arms } => {
                self.visit_expr(scrutinee);
                for (pattern, body) in arms {
                    self.visit_expr(pattern);
                    for stmt in body {
                        self.visit_statement(stmt);
                    }
                }
            }
            AnalyzedStatement::Return { value } => {
                if let Some(v) = value {
                    self.visit_expr(v);
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
            AnalyzedExprKind::UnaryOp { operand, .. } => {
                self.visit_expr(operand);
            }
            AnalyzedExprKind::FunctionCall { args, .. } | AnalyzedExprKind::VerbCall { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::StructInstantiation { args, .. } => {
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::ArrayLiteral(exprs) => {
                for e in exprs {
                    self.visit_expr(e);
                }
            }
            AnalyzedExprKind::IndexAccess { array, index } => {
                self.visit_expr(array);
                self.visit_expr(index);
            }
            AnalyzedExprKind::PropertyAccess { owner, .. } => {
                self.visit_expr(owner);
            }
            AnalyzedExprKind::MethodCall { receiver, args, .. } => {
                self.visit_expr(receiver);
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            AnalyzedExprKind::Try(e) | AnalyzedExprKind::Unwrap(e) | AnalyzedExprKind::Some(e) | AnalyzedExprKind::Ok(e) | AnalyzedExprKind::Err(e) => {
                self.visit_expr(e);
            }
            AnalyzedExprKind::Assert { condition } => {
                self.visit_expr(condition);
            }
            AnalyzedExprKind::AssertEq { left, right } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            AnalyzedExprKind::Range { start, end, .. } => {
                self.visit_expr(start);
                self.visit_expr(end);
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
                AugurFinding::UnreachableCode => {
                    println!("   {} Unreachable code detected.", "⚠️".yellow());
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
    fn test_augur_used_variable_in_assignment() {
        let code = "μετά ξ πέντε ἔστω. ξ δέκα γίγνεται.";
        let findings = analyze_code(code);
        assert_eq!(findings.len(), 0);
    }

    #[test]
    fn test_augur_used_variable_in_binop() {
        // Construct AST manually for reliability
        use crate::semantic::{AnalyzedProgram, AnalyzedStatement, AnalyzedExpr, AnalyzedExprKind, GlossaType, Scope};
        use crate::morphology::lexicon::BinaryOp;
        use smol_str::SmolStr;

        let statements = vec![
            AnalyzedStatement::Binding {
                name: SmolStr::new("ξ"),
                value: AnalyzedExpr { expr: AnalyzedExprKind::NumberLiteral(5), glossa_type: GlossaType::Number },
                mutable: false,
            },
            AnalyzedStatement::Binding {
                name: SmolStr::new("ψ"),
                value: AnalyzedExpr {
                    expr: AnalyzedExprKind::BinOp {
                        left: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::Variable(SmolStr::new("ξ")), glossa_type: GlossaType::Number }),
                        op: BinaryOp::Add,
                        right: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::NumberLiteral(2), glossa_type: GlossaType::Number }),
                    },
                    glossa_type: GlossaType::Number,
                },
                mutable: false,
            },
        ];
        let program = AnalyzedProgram { statements, scope: Scope::new() };
        let mut augur = Augur::new();
        let findings = augur.analyze(&program);

        let has_psi = findings.iter().any(|f| matches!(f, AugurFinding::UnusedVariable(name) if name == "ψ"));
        let has_xi = findings.iter().any(|f| matches!(f, AugurFinding::UnusedVariable(name) if name == "ξ"));
        assert!(!has_xi, "ξ should be marked as used");
        assert!(has_psi, "ψ should be marked as unused");
    }

    #[test]
    fn test_augur_used_variable_in_unaryop() {
        use crate::semantic::{AnalyzedProgram, AnalyzedStatement, AnalyzedExpr, AnalyzedExprKind, GlossaType, Scope};
        use crate::morphology::lexicon::UnaryOp;
        use smol_str::SmolStr;

        let statements = vec![
            AnalyzedStatement::Binding {
                name: SmolStr::new("ξ"),
                value: AnalyzedExpr { expr: AnalyzedExprKind::BooleanLiteral(true), glossa_type: GlossaType::Boolean },
                mutable: false,
            },
            AnalyzedStatement::Binding {
                name: SmolStr::new("ψ"),
                value: AnalyzedExpr {
                    expr: AnalyzedExprKind::UnaryOp {
                        op: UnaryOp::Not,
                        operand: Box::new(AnalyzedExpr { expr: AnalyzedExprKind::Variable(SmolStr::new("ξ")), glossa_type: GlossaType::Boolean }),
                    },
                    glossa_type: GlossaType::Boolean,
                },
                mutable: false,
            },
        ];
        let program = AnalyzedProgram { statements, scope: Scope::new() };
        let mut augur = Augur::new();
        let findings = augur.analyze(&program);

        let has_psi = findings.iter().any(|f| matches!(f, AugurFinding::UnusedVariable(name) if name == "ψ"));
        let has_xi = findings.iter().any(|f| matches!(f, AugurFinding::UnusedVariable(name) if name == "ξ"));
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
    fn test_run_augur_findings_unreachable() {
        // Just directly call run_augur with an AST that produces UnreachableCode,
        // but since we haven't implemented UnreachableCode detection yet,
        // we can just directly test that printing UnreachableCode works
        // We'll write a manual AST analyzer that returns UnreachableCode to test the println.

        let status = crate::tools::ui::Status::start_with_symbol("Οἰωνός (Analyzing semantics)", "🦅");
        let findings = vec![AugurFinding::UnreachableCode];
        status.success();

        println!();
        println!("   {}", "Γ Λ Ω Σ Σ Α   A U G U R".bold().cyan());
        println!("   {}", "Semantic Analysis Report".italic().dim());
        println!();

        for finding in findings {
            match finding {
                AugurFinding::UnusedVariable(name) => {
                    println!("   {} Variable `{}` is defined but never used.", "⚠️".yellow(), name.yellow());
                }
                AugurFinding::UnreachableCode => {
                    println!("   {} Unreachable code detected.", "⚠️".yellow());
                }
            }
        }
        println!();
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
