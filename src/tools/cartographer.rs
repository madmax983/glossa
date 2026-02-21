//! The Cartographer Tool ("Cartographer")
//!
//! This module implements the "Cartographer" functionality, which visualizes the
//! architecture of a ΓΛΩΣΣΑ program as a Mermaid.js class diagram.
//!
//! # Purpose
//!
//! "Architectural Transparency" is a core value of the project.
//! As programs grow, the relationships between Types (Structs) and Traits (Interfaces)
//! can become complex. The Cartographer renders these relationships visible.
//!
//! # The Map
//!
//! The output is a standard Mermaid Class Diagram that shows:
//! * **Classes**: Structs with their fields and types.
//! * **Interfaces**: Traits with their methods.
//! * **Relationships**:
//!   * `-->` (Association): When a Struct field holds another Struct.
//!   * `<|..` (Implementation): When a Struct implements a Trait.
//!
//! # Example Output
//!
//! ```mermaid
//! classDiagram
//!     class User {
//!         +name: String
//!         +age: Number
//!     }
//!     class Printable {
//!         <<interface>>
//!         +print()
//!     }
//!     Printable <|.. User : implements
//! ```

use crate::semantic::{AnalyzedProgram, AnalyzedStatement, GlossaType};
use std::collections::HashSet;

/// Generate a Mermaid class diagram from an analyzed program
pub fn generate_map(program: &AnalyzedProgram) -> String {
    let mut map = String::from("classDiagram\n");
    let mut dependencies = HashSet::new();

    // 1. Iterate over types (structs)
    // We sort by name to ensure deterministic output
    let mut types: Vec<_> = program.scope.types().collect();
    types.sort_by(|a, b| a.0.cmp(b.0));

    for (_key, ty) in types {
        if let GlossaType::Struct { name, fields, .. } = ty {
            map.push_str(&format!("    class {} {{\n", name));
            for (field_name, field_type) in fields {
                // Format type for display
                map.push_str(&format!("        +{}: {}\n", field_name, field_type));

                // Extract dependencies (associations)
                for dep in extract_dependencies(field_type) {
                    if dep != *name { // Avoid self-reference arrows if desired (or keep them?)
                        // We only want arrows to other defined structs
                        dependencies.insert((name.to_string(), dep));
                    }
                }
            }
            map.push_str("    }\n");
        }
    }

    // 2. Iterate over traits
    let mut traits: Vec<_> = program.scope.traits().collect();
    traits.sort_by(|a, b| a.0.cmp(b.0));

    for (_key, trait_def) in traits {
        let name = &trait_def.name;
        map.push_str(&format!("    class {} {{\n", name));
        map.push_str("        <<interface>>\n");
        for method in &trait_def.methods {
             let params_str: Vec<String> = method.params.iter().map(|(n, t)| format!("{}: {}", n, t)).collect();
             let ret_str = method.return_type.as_ref().map(|t| format!(": {}", t)).unwrap_or_default();
             map.push_str(&format!("        +{}({}){}\n", method.name, params_str.join(", "), ret_str));
        }
        map.push_str("    }\n");
    }

    // 3. Add dependencies (struct field usage)
    // Only include dependencies if the target is actually a defined type/trait in the diagram
    // to avoid arrows to "Unknown" or external things if not modeled.
    let defined_types: HashSet<String> = program.scope.types().filter_map(|(_, ty)| {
        if let GlossaType::Struct { name, .. } = ty {
            Some(name.to_string())
        } else {
            None
        }
    }).collect();

    let mut sorted_deps: Vec<_> = dependencies.into_iter().collect();
    sorted_deps.sort();

    for (source, target) in sorted_deps {
        if defined_types.contains(&target) {
            map.push_str(&format!("    {} --> {}\n", source, target));
        }
    }

    // 4. Add trait implementations
    // Iterate statements to find implementations
    for stmt in &program.statements {
        if let AnalyzedStatement::TraitImplementation { trait_name, type_name, .. } = stmt {
             map.push_str(&format!("    {} <|.. {} : implements\n", trait_name, type_name));
        }
    }

    map
}

/// Helper to recursively extract struct names from a type
fn extract_dependencies(ty: &GlossaType) -> Vec<String> {
    match ty {
        GlossaType::Struct { name, .. } => vec![name.to_string()],
        GlossaType::List(inner) | GlossaType::Set(inner) | GlossaType::Option(inner) => extract_dependencies(inner),
        GlossaType::Map(k, v) | GlossaType::Result(k, v) => {
            let mut deps = extract_dependencies(k);
            deps.extend(extract_dependencies(v));
            deps
        },
        GlossaType::Function { params, returns } => {
            let mut deps = vec![];
            for p in params {
                deps.extend(extract_dependencies(p));
            }
            deps.extend(extract_dependencies(returns));
            deps
        },
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    #[test]
    fn test_cartographer_basic_struct() {
        let source = "
            εἶδος Χρήστης ὁρίζειν {
                ὄνομα ὀνόματος.
                ἡλικία ἀριθμοῦ.
            }.
        ";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();
        let map = generate_map(&program);

        // Names are normalized in scope
        assert!(map.contains("class χρηστης") || map.contains("class Χρήστης"));
        assert!(map.contains("+ονομα: Ὄνομα") || map.contains("+ὄνομα: Ὄνομα"));
        assert!(map.contains("+ηλικια: Ἀριθμός") || map.contains("+ἡλικία: Ἀριθμός"));
    }

    #[test]
    fn test_cartographer_relationships() {
        let source = "
            εἶδος Διεύθυνσις ὁρίζειν {
                πόλις ὀνόματος.
            }.

            εἶδος Χρήστης ὁρίζειν {
                διεύθυνσις Διεύθυνσις.
            }.
        ";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();
        let map = generate_map(&program);

        assert!(map.contains("class χρηστης") || map.contains("class Χρήστης"));
        assert!(map.contains("class διευθυνσις") || map.contains("class Διεύθυνσις"));
        // Check for relationship (normalized)
        assert!(map.contains("χρηστης --> διευθυνσις") || map.contains("Χρήστης --> Διεύθυνσις"));
    }

    #[test]
    fn test_cartographer_trait_impl() {
        let source = "
            χαρακτήρ Εκτυπώσιμος ὁρίζειν {
                δεῖ τυπωσις τῷ self.
            }.

            εἶδος Χ ὁρίζειν { χ ἀριθμοῦ. }.

            εἶδος Χ τῷ Εκτυπώσιμος ἐμπίπτειν {
                τυπωσις τῷ self· «x» λέγε.
            }.
        ";
        let ast = parse(source).unwrap();
        let program = analyze_program(&ast).unwrap();
        let map = generate_map(&program);

        assert!(map.contains("class εκτυπωσιμος") || map.contains("class Εκτυπώσιμος"));
        assert!(map.contains("<<interface>>"));
        assert!(map.contains("εκτυπωσιμος <|.. χ : implements") || map.contains("Εκτυπώσιμος <|.. Χ : implements"));
    }
}
