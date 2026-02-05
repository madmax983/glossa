use crate::errors::GlossaError;
use crate::morphology::DisambiguationContext;
use crate::parser::parse;
use crate::semantic::expressions::feed_expr_to_assembler_with_context;
use crate::semantic::{Assembler, Literal};
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use crossterm::style::Stylize;

/// The Oracle - A tool to explain the semantic analysis of Glossa code
pub struct Oracle;

impl Oracle {
    /// Create a new Oracle
    pub fn new() -> Self {
        Oracle
    }

    /// Explain the given source code
    ///
    /// This parses and assembles the code, then generates a human-readable report
    /// explaining how the morphological assembler interpreted each sentence.
    pub fn explain(&self, source: &str) -> Result<String, GlossaError> {
        let ast = parse(source)?;
        let mut report = String::new();

        report.push_str(&format!(
            "\n{}\n",
            "🔮 Ὁ Μάντις (The Oracle) Speaks...".bold().magenta()
        ));

        for (i, stmt) in ast.statements.iter().enumerate() {
            report.push_str(&format!(
                "\n{}\n",
                format!("📜 Statement #{}", i + 1).yellow().underlined()
            ));

            // Check for non-assembler patterns first (like Type Definitions)
            if matches!(stmt, crate::ast::Statement::TypeDefinition(_)) {
                report.push_str(&format!("{}\n", "Type Definition detected.".blue()));
                continue;
            }

            // Re-assemble to inspect slots
            let mut asm = Assembler::new();
            asm.set_query(stmt.is_query());
            asm.set_propagate(stmt.is_propagate());

            let mut context = DisambiguationContext::new();

            for clause in stmt.clauses() {
                for expr in &clause.expressions {
                    feed_expr_to_assembler_with_context(&mut asm, expr, &mut context)?;
                }
            }

            let assembled = asm.finalize()?;

            // Build Table
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Role")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Cyan),
                    Cell::new("Greek Word").add_attribute(Attribute::Bold),
                    Cell::new("Morphology")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Green),
                    Cell::new("Lemma/Value").add_attribute(Attribute::Bold),
                ]);

            if let Some(subject) = &assembled.subject {
                table.add_row(vec![
                    "Subject (Agent)",
                    &subject.original,
                    "Nominative Case",
                    subject.lemma.as_ref(),
                ]);
            }

            for nom in &assembled.nominatives {
                table.add_row(vec![
                    "Nominative (Secondary)",
                    &nom.original,
                    "Nominative Case",
                    nom.lemma.as_ref(),
                ]);
            }

            if let Some(verb) = &assembled.verb {
                let tense_str = verb
                    .tense
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|| "Unknown".to_string());
                let voice_str = verb
                    .voice
                    .map(|v| format!("{:?}", v))
                    .unwrap_or_else(|| "Unknown".to_string());
                let morph = format!("{} {}", tense_str, voice_str);

                table.add_row(vec![
                    "Verb (Action)",
                    &verb.original,
                    &morph,
                    verb.lemma.as_ref(),
                ]);
            }

            if let Some(object) = &assembled.object {
                table.add_row(vec![
                    "Object (Patient)",
                    &object.original,
                    "Accusative Case",
                    object.lemma.as_ref(),
                ]);
            }

            if let Some(indirect) = &assembled.indirect {
                table.add_row(vec![
                    "Indirect Object",
                    &indirect.original,
                    "Dative Case",
                    indirect.lemma.as_ref(),
                ]);
            }

            for participle in &assembled.participles {
                let morph = format!("{:?} {:?} Participle", participle.tense, participle.voice);
                table.add_row(vec![
                    "Participle (Lambda)",
                    &participle.original,
                    &morph,
                    &participle.verb_lemma,
                ]);
            }

            for literal in &assembled.literals {
                let (val, type_name) = match literal {
                    Literal::String(s) => (format!("«{}»", s), "String"),
                    Literal::Number(n) => (n.to_string(), "Number"),
                    Literal::Boolean(b) => (b.to_string(), "Boolean"),
                };
                table.add_row(vec!["Literal Value", &val, type_name, &val]);
            }

            report.push_str(&table.to_string());
            report.push('\n');

            // Interpretation
            report.push_str(&format!("\n{}\n", "👁️ Interpretation:".bold()));
            if assembled.verb.is_some() {
                if let Some(s) = &assembled.subject {
                    if let Some(o) = &assembled.object {
                        report.push_str(&format!(
                            "The subject '{}' acts upon '{}'.",
                            s.original, o.original
                        ));
                    } else {
                        report
                            .push_str(&format!("The subject '{}' performs an action.", s.original));
                    }
                } else if let Some(o) = &assembled.object {
                    report.push_str(&format!("An action is performed on '{}'.", o.original));
                } else {
                    report.push_str("An action occurs.");
                }
            } else if !assembled.literals.is_empty() {
                report.push_str("A value is being expressed.");
            } else if !assembled.participles.is_empty() {
                report.push_str("A functional operation (lambda) is being defined.");
            } else {
                report.push_str(
                    "This statement contains no main verb (possibly a definition or fragment).",
                );
            }

            if assembled.verb.is_some() && !assembled.participles.is_empty() {
                report.push_str(" A functional operation (lambda) is also involved.");
            }

            report.push('\n');
        }

        Ok(report)
    }
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oracle_explains_basic_sentence() {
        let oracle = Oracle::new();
        // Use ἀρχήν (beginning/origin) which ends in -ην to avoid misidentification as participle (-ον)
        let source = "ὁ ἄνθρωπος τὴν ἀρχήν λέγει.";

        let explanation = oracle.explain(source).unwrap();

        println!("{}", explanation);

        assert!(explanation.contains("Subject (Agent)"));
        assert!(explanation.contains("ἄνθρωπος"));
        assert!(explanation.contains("Object (Patient)"));
        assert!(explanation.contains("ἀρχήν"));
        assert!(explanation.contains("Verb (Action)"));
        assert!(explanation.contains("λέγει"));
    }

    #[test]
    fn test_oracle_type_definition() {
        let oracle = Oracle::new();
        let source = "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.";
        let explanation = oracle.explain(source).unwrap();
        assert!(explanation.contains("Type Definition detected"));
    }

    #[test]
    fn test_oracle_literals() {
        let oracle = Oracle::new();
        let source = "«χαῖρε» 42 ἀληθές λέγε.";
        let explanation = oracle.explain(source).unwrap();

        assert!(explanation.contains("Literal Value"));
        assert!(explanation.contains("«χαῖρε»"));
        assert!(explanation.contains("42"));
        assert!(explanation.contains("true")); // boolean true prints as true
    }

    #[test]
    fn test_oracle_participle() {
        let oracle = Oracle::new();
        // Use a participle form
        let source = "διπλασιαζόμενα λέγε.";
        let explanation = oracle.explain(source).unwrap();

        assert!(explanation.contains("Participle (Lambda)"));
        assert!(explanation.contains("διπλασιαζόμενα"));
        assert!(explanation.contains("lambda"));
    }

    #[test]
    fn test_oracle_indirect_object() {
        let oracle = Oracle::new();
        // Dative case: τῷ ἀνθρώπῳ (to the man)
        let source = "τῷ ἀνθρώπῳ δίδωμι.";
        let explanation = oracle.explain(source).unwrap();

        assert!(explanation.contains("Indirect Object"));
        assert!(explanation.contains("ἀνθρώπῳ"));
    }

    #[test]
    fn test_oracle_verbless() {
        let oracle = Oracle::new();
        // Just a literal, no verb
        let source = "42.";
        let explanation = oracle.explain(source).unwrap();

        assert!(explanation.contains("A value is being expressed"));
    }

    #[test]
    fn test_oracle_empty_or_fragment() {
        let oracle = Oracle::new();
        // Empty statement (just a period, though parser might reject, let's try a variable decl without verb if possible, or just check the fallback path)
        // Actually, "User." is valid syntax for a variable reference statement
        let source = "User.";
        let explanation = oracle.explain(source).unwrap();

        assert!(explanation.contains("This statement contains no main verb"));
    }

    #[test]
    fn test_oracle_secondary_nominative() {
        let oracle = Oracle::new();
        // "Subject Predicate is." - "User God is."
        // We need a sentence that Assembler treats as having multiple nominatives.
        // Usually, function calls or 'is' sentences do this.
        let source = "ὁ ἄνθρωπος θεός ἐστιν.";
        let explanation = oracle.explain(source).unwrap();

        assert!(explanation.contains("Subject (Agent)"));
        assert!(explanation.contains("Nominative (Secondary)"));
    }
}
