use crate::ast::Statement;
use crate::errors::GlossaError;
use crate::parser::parse;
use crate::semantic::{analyze_single_statement_with_assembler, AssembledStatement, Constituent};
use std::fmt::Write;

/// Generate a Mermaid flowchart from the semantic analysis of the source
pub fn generate_mermaid(source: &str) -> Result<String, GlossaError> {
    let program = parse(source)?;
    let mut diagram = String::from("graph TD\n");

    // Global style
    diagram.push_str("    %% Styles\n");
    diagram.push_str("    classDef nominative fill:#e1f5fe,stroke:#01579b,stroke-width:2px;\n");
    diagram.push_str("    classDef accusative fill:#f3e5f5,stroke:#4a148c,stroke-width:2px;\n");
    diagram.push_str("    classDef verb fill:#e8f5e9,stroke:#1b5e20,stroke-width:2px;\n");
    diagram.push_str("    classDef literal fill:#fff3e0,stroke:#e65100,stroke-width:2px;\n");

    for (i, stmt) in program.statements.iter().enumerate() {
        // Skip definitions and complex control flow for now to focus on the assembler
        if matches!(
            stmt,
            Statement::TypeDefinition(_)
                | Statement::TraitDefinition(_)
                | Statement::TraitImpl(_)
        ) {
            writeln!(diagram, "    stmt_{}[Definition]:::literal", i).unwrap();
            continue;
        }

        // Try to analyze with the assembler
        match analyze_single_statement_with_assembler(stmt) {
            Ok(assembled) => {
                diagram.push_str(&render_assembled(i, &assembled));
            }
            Err(_) => {
                // If assembler fails (e.g. control flow that isn't a simple sentence), just mark it
                writeln!(diagram, "    stmt_{}[Complex Statement]:::literal", i).unwrap();
            }
        }
    }

    Ok(diagram)
}

fn render_assembled(id: usize, stmt: &AssembledStatement) -> String {
    let mut s = String::new();
    let prefix = format!("stmt_{}", id);

    writeln!(s, "    subgraph sub_{}", id).unwrap();
    writeln!(s, "        direction TB").unwrap();
    writeln!(s, "        {}[Sentence {}]", prefix, id + 1).unwrap();

    // Subject (Nominative)
    if let Some(subj) = &stmt.subject {
        render_constituent(&mut s, &prefix, "Subject", subj, "nominative");
    }

    // Additional Nominatives
    for (i, nom) in stmt.nominatives.iter().enumerate() {
        render_constituent(&mut s, &prefix, &format!("Nominative_{}", i), nom, "nominative");
    }

    // Verb
    if let Some(verb) = &stmt.verb {
        let node_id = format!("{}_verb", prefix);
        writeln!(s, "        {}[Verb: {}]:::verb", node_id, verb.original).unwrap();
        writeln!(s, "        {} -->|Action| {}", prefix, node_id).unwrap();

        // Verb details
        if let Some(p) = verb.person {
             writeln!(s, "        {}_person[Person: {:?}]", node_id, p).unwrap();
             writeln!(s, "        {} --- {}_person", node_id, node_id).unwrap();
        }
    }

    // Object (Accusative)
    if let Some(obj) = &stmt.object {
        render_constituent(&mut s, &prefix, "Object", obj, "accusative");
    }

    // Indirect Object (Dative)
    if let Some(ind) = &stmt.indirect {
        render_constituent(&mut s, &prefix, "Indirect", ind, "accusative"); // Re-using style
    }

    // Literals
    for (i, lit) in stmt.literals.iter().enumerate() {
        let node_id = format!("{}_lit_{}", prefix, i);
        let content = match lit {
            crate::semantic::Literal::String(s) => format!("\"{}\"", s),
            crate::semantic::Literal::Number(n) => format!("{}", n),
            crate::semantic::Literal::Boolean(b) => format!("{}", b),
        };
        writeln!(s, "        {}[Literal: {}]:::literal", node_id, content).unwrap();
        writeln!(s, "        {} -->|Value| {}", prefix, node_id).unwrap();
    }

    writeln!(s, "    end").unwrap();
    s
}

fn render_constituent(s: &mut String, parent_id: &str, role: &str, c: &Constituent, style: &str) {
    let node_id = format!("{}_{}", parent_id, role);
    writeln!(s, "        {}[{}: {}]:::{}", node_id, role, c.original, style).unwrap();
    writeln!(s, "        {} -->|{}| {}", parent_id, role, node_id).unwrap();

    // Add grammar details
    let detail_id = format!("{}_case", node_id);
    writeln!(s, "        {}[{:?}]", detail_id, c.case).unwrap();
    writeln!(s, "        {} -.-> {}", node_id, detail_id).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_diagram() {
        let source = "ὁ ἄνθρωπος λέγει.";
        let mermaid = generate_mermaid(source).unwrap();

        assert!(mermaid.contains("graph TD"));
        assert!(mermaid.contains("Subject: ἄνθρωπος"));
        assert!(mermaid.contains("Verb: λέγει"));
    }

    #[test]
    fn test_generate_diagram_definition() {
        let source = "εἶδος Πρόσωπον ὁρίζειν { ὄνομα ἀριθμοῦ. }.";
        let mermaid = generate_mermaid(source).unwrap();

        assert!(mermaid.contains("[Definition]"));
    }

    #[test]
    fn test_generate_diagram_complex_statement() {
        // Use a statement that triggers an assembler error (Double Verb)
        let source = "λέγει λέγει.";
        let mermaid = generate_mermaid(source).unwrap();

        // Should fall back to Complex Statement
        assert!(mermaid.contains("[Complex Statement]"));
    }

    #[test]
    fn test_generate_diagram_literals() {
        let source = "«χαῖρε» 42 λέγε.";
        let mermaid = generate_mermaid(source).unwrap();

        assert!(mermaid.contains("Literal: \"χαῖρε\""));
        assert!(mermaid.contains("Literal: 42"));
    }

    #[test]
    fn test_generate_diagram_indirect_object() {
        // τῷ ἀνθρώπῳ δίδωμι (I give to the man)
        let source = "τῷ ἀνθρώπῳ δίδωμι.";
        let mermaid = generate_mermaid(source).unwrap();

        assert!(mermaid.contains("Indirect: ἀνθρώπῳ"));
    }
}
