//! The Grammarian (ὁ Γραμματικός) - Vocabulary Study Guide Generator
//!
//! This module implements the "Grammarian" tool, which extracts all unique Greek words
//! from a ΓΛΩΣΣΑ program and generates a personalized vocabulary study guide.
//!
//! # Purpose
//!
//! Ancient Greek is hard. This tool connects the raw AST parsing phase with the
//! morphological lexicon to provide a cheat sheet for the words used in a specific file.
//! It outputs a table containing the original word, its lemma (dictionary form),
//! its part of speech, and its English/Rust equivalent.

use crate::ast::{Clause, Expr, Statement};
use crate::morphology::lexicon;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use crossterm::style::Stylize;
use miette::Result;
use rustc_hash::FxHashSet;
use std::path::Path;

/// A visitor that traverses the AST to extract all unique `Word`s.
#[derive(Default)]
pub struct GrammarianVisitor {
    /// Tracks unique normalized words
    pub unique_words: FxHashSet<String>,
}

impl GrammarianVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn visit_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Regular { clauses, .. } => {
                for clause in clauses {
                    self.visit_clause(clause);
                }
            }
            Statement::TypeDefinition(type_def) => {
                self.unique_words
                    .insert(type_def.name.normalized.to_string());
                for field in &type_def.fields {
                    self.unique_words.insert(field.name.normalized.to_string());
                    self.unique_words
                        .insert(field.type_name.normalized.to_string());
                }
            }

            Statement::TraitDefinition(trait_def) => {
                self.unique_words
                    .insert(trait_def.name.normalized.to_string());
                for method in &trait_def.methods {
                    self.unique_words.insert(method.name.normalized.to_string());
                    for p in &method.params {
                        self.unique_words.insert(p.name.normalized.to_string());
                        self.unique_words.insert(p.type_name.normalized.to_string());
                    }
                    if let Some(body) = &method.body {
                        for s in body {
                            self.visit_statement(s);
                        }
                    }
                }
            }
            Statement::TraitImpl(trait_impl) => {
                self.unique_words
                    .insert(trait_impl.trait_name.normalized.to_string());
                self.unique_words
                    .insert(trait_impl.type_name.normalized.to_string());
                for method in &trait_impl.methods {
                    self.unique_words.insert(method.name.normalized.to_string());
                    for p in &method.params {
                        self.unique_words.insert(p.name.normalized.to_string());
                        self.unique_words.insert(p.type_name.normalized.to_string());
                    }
                    for s in &method.body {
                        self.visit_statement(s);
                    }
                }
            }
            Statement::TestDeclaration(test_decl) => {
                for s in &test_decl.body {
                    self.visit_statement(s);
                }
            }
        }
    }

    fn visit_clause(&mut self, clause: &Clause) {
        for expr in &clause.expressions {
            self.visit_expr(expr);
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Word(word) => {
                self.unique_words.insert(word.normalized.to_string());
            }
            Expr::ArrayLiteral(exprs) | Expr::Phrase(exprs) => {
                for e in exprs {
                    self.visit_expr(e);
                }
            }
            Expr::IndexAccess { array, index } => {
                self.visit_expr(array);
                self.visit_expr(index);
            }
            Expr::PropertyAccess { owner, property } => {
                self.visit_expr(owner);
                self.visit_expr(property);
            }
            Expr::Call { verb, arguments } => {
                self.unique_words.insert(verb.normalized.to_string());
                for arg in arguments {
                    self.visit_expr(arg);
                }
            }
            Expr::Binding { name, value } => {
                self.unique_words.insert(name.normalized.to_string());
                self.visit_expr(value);
            }
            Expr::BinOp { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            Expr::UnaryOp { operand, .. } => {
                self.visit_expr(operand);
            }
            Expr::Block(stmts) => {
                for s in stmts {
                    self.visit_statement(s);
                }
            }

            Expr::StringLiteral(_) | Expr::NumberLiteral(_) | Expr::BooleanLiteral(_) => {}
        }
    }
}

pub fn run_grammarian(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Γραμματικός (Extracting Vocabulary)", "📚");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    let program = match crate::parser::parse(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Parse Error)");
            return Err(e.into());
        }
    };

    let mut visitor = GrammarianVisitor::new();
    for stmt in &program.statements {
        visitor.visit_statement(stmt);
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   G R A M M A R I A N".cyan().bold());
    println!(
        "   {}",
        format!("Vocabulary Study Guide for {}", input.display())
            .italic()
            .dim()
    );
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);

    table.set_header(vec![
        Cell::new("Word (Normalized)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Cyan),
        Cell::new("Lemma (Dictionary Form)")
            .add_attribute(Attribute::Bold)
            .fg(Color::Yellow),
        Cell::new("Part of Speech")
            .add_attribute(Attribute::Bold)
            .fg(Color::Magenta),
        Cell::new("Meaning / Equivalent")
            .add_attribute(Attribute::Bold)
            .fg(Color::Green),
    ]);

    let mut found_words: Vec<String> = visitor.unique_words.into_iter().collect();
    found_words.sort();

    let mut found_count = 0;

    for word in found_words {
        if let Some(entry) = lexicon::lookup(&word) {
            let rust_equiv = entry.rust_equiv.unwrap_or("Unknown");
            table.add_row(vec![
                Cell::new(&word),
                Cell::new(entry.lemma),
                Cell::new(format!("{:?}", entry.pos)),
                Cell::new(rust_equiv),
            ]);
            found_count += 1;
        }
    }

    if found_count > 0 {
        println!("{table}");
    } else {
        println!("   No built-in lexicon words were found in this program.");
    }
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_grammarian_success() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("vocab.γλ");
        fs::write(&input_path, "ξ πέντε ἔστω. «χαῖρε» λέγε.").unwrap();

        let result = run_grammarian(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_grammarian_file_not_found() {
        let input_path = Path::new("does_not_exist.γλ");
        let result = run_grammarian(input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_grammarian_parse_error() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("vocab_err.γλ");
        fs::write(&input_path, "ξ πέντε ἔστω «χαῖρε» λέγε").unwrap();

        let result = run_grammarian(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_grammarian_types_and_traits() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("vocab_traits.γλ");
        let source = "
        εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.
        χαρακτήρ Ὁμιλητής ὁρίζειν { λέγειν(αὐτός). }.
        εἶδος Χρήστης τῷ Ὁμιλητής ἐμπίπτειν { λέγειν(αὐτός) { αὐτοῦ ὄνομα λέγε. } }.
        ";
        fs::write(&input_path, source).unwrap();

        let result = run_grammarian(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_grammarian_control_flow_and_blocks() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("vocab_blocks.γλ");
        let source = "
        εἰ αληθες, {
            πίναξ[0] 1 γίγνεται.
            δοκιμή «foo» τέλος.
        } εἰ δὲ μή, {
            παῦε.
        }
        διὰ α, β λέγε.
        ";
        fs::write(&input_path, source).unwrap();

        let result = run_grammarian(&input_path);
        assert!(result.is_ok());
    }
}
