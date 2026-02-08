//! The Bard (ὁ Ῥαψῳδός) - Semantic Syntax Highlighter
//!
//! This module implements a semantic syntax highlighter that colors the source code
//! based on the grammatical role of each word (Subject, Object, Verb, etc.).
//!
//! # Philosophy
//!
//! Unlike traditional syntax highlighters that use regexes, The Bard uses the
//! compiler's own morphological analysis to understand the code.
//!
//! * **Nominative (Subject)**: Blue (The agent/foundation)
//! * **Accusative (Object)**: Red (The target of action)
//! * **Dative (Indirect)**: Yellow (The recipient)
//! * **Genitive (Possession)**: Magenta (Ownership)
//! * **Verb (Action)**: Green (Go!)
//! * **Adjective**: Cyan (Modification)
//! * **Literals**: Italic/White
//!
//! # Usage
//!
//! ```rust
//! use glossa::experimental::bard::highlight;
//!
//! let source = "ὁ ἄνθρωπος τὸν λόγον λέγει.";
//! let highlighted = highlight(source).unwrap();
//! println!("{}", highlighted);
//! ```

use crossterm::style::Stylize;
use std::fmt::Write;

use crate::ast::{
    BinOperator, Clause, Expr, Program, Statement, TestDecl, TraitDef, TraitImplDef, TypeDef,
    UnaryOperator, Word,
};
use crate::errors::GlossaError;
use crate::morphology::{
    Case, DisambiguationContext, PartOfSpeech, analyze_article, analyze_participle, resolve_best,
};
use crate::parser::parse;

/// Highlight the source code with semantic colors
pub fn highlight(source: &str) -> Result<String, GlossaError> {
    let program = parse(source)?;
    let mut highlighter = Highlighter::new();
    highlighter.highlight_program(&program)?;
    Ok(highlighter.output)
}

struct Highlighter {
    output: String,
    context: DisambiguationContext,
}

impl Highlighter {
    fn new() -> Self {
        Self {
            output: String::new(),
            context: DisambiguationContext::new(),
        }
    }

    fn highlight_program(&mut self, program: &Program) -> Result<(), GlossaError> {
        for (i, stmt) in program.statements.iter().enumerate() {
            if i > 0 {
                self.output.push('\n');
            }
            self.highlight_statement(stmt)?;
        }
        Ok(())
    }

    fn highlight_statement(&mut self, stmt: &Statement) -> Result<(), GlossaError> {
        match stmt {
            Statement::Regular {
                clauses,
                is_query,
                is_propagate,
            } => {
                for (i, clause) in clauses.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.highlight_clause(clause)?;
                }

                if *is_query {
                    self.output.push('?');
                } else if *is_propagate {
                    self.output.push(';');
                } else {
                    self.output.push('.');
                }
            }
            Statement::TypeDefinition(def) => self.highlight_type_def(def)?,
            Statement::TraitDefinition(def) => self.highlight_trait_def(def)?,
            Statement::TraitImpl(def) => self.highlight_trait_impl(def)?,
            Statement::TestDeclaration(decl) => self.highlight_test_decl(decl)?,
        }
        Ok(())
    }

    fn highlight_clause(&mut self, clause: &Clause) -> Result<(), GlossaError> {
        for (i, expr) in clause.expressions.iter().enumerate() {
            if i > 0 {
                self.output.push(' ');
            }
            self.highlight_expr(expr)?;
        }
        Ok(())
    }

    fn highlight_expr(&mut self, expr: &Expr) -> Result<(), GlossaError> {
        match expr {
            Expr::StringLiteral(s) => {
                write!(self.output, "«{}»", s.as_str().italic()).unwrap();
            }
            Expr::NumberLiteral(n) => {
                write!(self.output, "{}", n.to_string().italic()).unwrap();
            }
            Expr::BooleanLiteral(b) => {
                let s = if *b { "ἀληθές" } else { "ψεῦδος" };
                write!(self.output, "{}", s.italic()).unwrap();
            }
            Expr::Word(w) => self.highlight_word(w)?,
            Expr::Phrase(terms) => {
                for (i, term) in terms.iter().enumerate() {
                    if i > 0 {
                        self.output.push(' ');
                    }
                    self.highlight_expr(term)?;
                }
            }
            Expr::PropertyAccess { owner, property } => {
                self.highlight_expr(owner)?;
                self.output.push(' ');
                self.highlight_expr(property)?;
            }
            Expr::Call { verb, arguments } => {
                // Highlight verb
                self.highlight_word(verb)?;
                // Arguments
                for arg in arguments {
                    self.output.push(' ');
                    self.highlight_expr(arg)?;
                }
            }
            Expr::Binding { name, value } => {
                self.highlight_word(name)?;
                self.output.push(' ');
                self.highlight_expr(value)?;
                self.output.push(' ');
                write!(self.output, "{}", "ἔστω".bold()).unwrap();
            }
            Expr::BinOp { left, op, right } => {
                self.highlight_expr(left)?;
                self.output.push(' ');
                self.highlight_binop(op);
                self.output.push(' ');
                self.highlight_expr(right)?;
            }
            Expr::UnaryOp { op, operand } => {
                match op {
                    UnaryOperator::Unwrap => {
                        self.highlight_expr(operand)?;
                        write!(self.output, "{}", "!".bold().red()).unwrap();
                    }
                    UnaryOperator::Not => {
                        write!(self.output, "{}", "οὐ".bold()).unwrap(); // Simplified
                        self.output.push(' ');
                        self.highlight_expr(operand)?;
                    }
                    UnaryOperator::Neg => {
                        write!(self.output, "-").unwrap();
                        self.highlight_expr(operand)?;
                    }
                }
            }
            Expr::Block(stmts) => {
                self.output.push_str("{ ");
                for (i, stmt) in stmts.iter().enumerate() {
                    if i > 0 {
                        self.output.push(' ');
                    }
                    self.highlight_statement(stmt)?;
                }
                self.output.push_str(" }");
            }
            Expr::ArrayLiteral(elements) => {
                self.output.push('[');
                for (i, el) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.highlight_expr(el)?;
                }
                self.output.push(']');
            }
            Expr::IndexAccess { array, index } => {
                self.highlight_expr(array)?;
                self.output.push('[');
                self.highlight_expr(index)?;
                self.output.push(']');
            }
        }
        Ok(())
    }

    fn highlight_word(&mut self, w: &Word) -> Result<(), GlossaError> {
        // 1. Check for article (sets context)
        if let Some(ctx) = analyze_article(&w.original) {
            self.context = ctx;
            write!(self.output, "{}", w.original).unwrap(); // Articles plain or dim? Let's leave plain
            return Ok(());
        }

        // 2. Check for participle
        let in_lexicon = crate::morphology::lexicon::lookup(&w.normalized).is_some();
        let is_numeral = crate::morphology::lexicon::numeral_value(&w.normalized).is_some();

        if !in_lexicon && !is_numeral && analyze_participle(&w.normalized).is_some() {
            write!(self.output, "{}", w.original.cyan()).unwrap(); // Participles as cyan (adjectival)
            return Ok(());
        }

        // 3. Analyze and disambiguate
        let analyses = crate::morphology::analyze_all(&w.normalized);
        let best = resolve_best(analyses, &self.context);

        // Update context if it's a verb
        if best.part_of_speech == PartOfSpeech::Verb {
            self.context = DisambiguationContext::from_verb(&best);
        } else {
            // Consume context for nouns
            self.context = DisambiguationContext::new();
        }

        // 4. Apply Color
        let styled = match best.part_of_speech {
            PartOfSpeech::Verb => w.original.green().bold(),
            PartOfSpeech::Noun | PartOfSpeech::Pronoun => match best.case {
                Some(Case::Nominative) => w.original.blue().bold(),
                Some(Case::Accusative) => w.original.red(),
                Some(Case::Dative) => w.original.yellow(),
                Some(Case::Genitive) => w.original.magenta(),
                Some(Case::Vocative) => w.original.blue().italic(), // Vocative as blue italic
                None => w.original.white(),
            },
            PartOfSpeech::Adjective => w.original.cyan(),
            PartOfSpeech::Preposition => w.original.white().bold(),
            PartOfSpeech::Conjunction => w.original.white().bold(),
            PartOfSpeech::Numeral => w.original.italic(),
            _ => w.original.white(), // Default
        };

        write!(self.output, "{}", styled).unwrap();
        Ok(())
    }

    fn highlight_binop(&mut self, op: &BinOperator) {
        let s = match op {
            BinOperator::Add => "+",
            BinOperator::Sub => "-",
            BinOperator::Mul => "*",
            BinOperator::Div => "/",
            BinOperator::Mod => "%",
            BinOperator::Eq => "==",
            BinOperator::Ne => "!=",
            BinOperator::Lt => "<",
            BinOperator::Le => "<=",
            BinOperator::Gt => ">",
            BinOperator::Ge => ">=",
            BinOperator::And => "&&",
            BinOperator::Or => "||",
        };
        write!(self.output, "{}", s.bold()).unwrap();
    }

    // --- Definitions (Simplified highlighting for now) ---

    fn highlight_type_def(&mut self, def: &TypeDef) -> Result<(), GlossaError> {
        write!(
            self.output,
            "{} {} {} {{ ... }}",
            "εἶδος".bold(),
            def.name.original.blue().bold(),
            "ὁρίζειν".bold()
        )
        .unwrap();
        Ok(())
    }

    fn highlight_trait_def(&mut self, def: &TraitDef) -> Result<(), GlossaError> {
        write!(
            self.output,
            "{} {} {} {{ ... }}",
            "χαρακτήρ".bold(),
            def.name.original.blue().bold(),
            "ὁρίζειν".bold()
        )
        .unwrap();
        Ok(())
    }

    fn highlight_trait_impl(&mut self, def: &TraitImplDef) -> Result<(), GlossaError> {
        write!(
            self.output,
            "{} {} {} {} {{ ... }}",
            "εἶδος".bold(),
            def.type_name.original.blue().bold(),
            "τῷ".white(),
            def.trait_name.original.cyan(),
            // Missing implementation keyword? Syntax is `εἶδος Type τῷ Trait ἐμπίπτειν`
        )
        .unwrap();
        Ok(())
    }

    fn highlight_test_decl(&mut self, decl: &TestDecl) -> Result<(), GlossaError> {
        writeln!(
            self.output,
            "{} «{}»",
            "δοκιμή".bold().green(),
            decl.name.as_str().italic()
        )
        .unwrap();

        for stmt in &decl.body {
            self.output.push_str("  ");
            self.highlight_statement(stmt)?;
            self.output.push('\n');
        }

        write!(self.output, "{}", "τέλος".bold()).unwrap();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_simple_sentence() {
        let source = "ὁ ἄνθρωπος τὸν λόγον λέγει.";
        let result = highlight(source);
        assert!(result.is_ok());
        let output = result.unwrap();

        // Check for ANSI codes
        // Verify that words are present and some color codes are applied
        assert!(output.contains("ἄνθρωπος"));
        assert!(output.contains("\x1b[")); // Contains escape sequence
    }

    #[test]
    fn test_highlight_literals() {
        let source = "«χαῖρε» λέγε.";
        let result = highlight(source);
        assert!(result.is_ok());
        let output = result.unwrap();

        // Italic (3) for string
        assert!(output.contains("\x1b[3mχαῖρε\x1b[0m"));
    }
}
