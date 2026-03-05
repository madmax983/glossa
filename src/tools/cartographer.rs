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

use crate::parser::parse;
use crate::semantic::{AnalyzedProgram, AnalyzedStatement, GlossaType, analyze_program};
use crate::tools::ui::Status;
use comfy_table::{Attribute, Cell, Color, Table, presets};
use crossterm::style::Stylize;
use miette::Result;
use std::collections::HashSet;
use std::path::Path;

/// Run the Cartographer tool on a file
///
/// Reads the source file, parses it, and prints the architectural map to stdout.
pub fn run_map(input: &Path) -> Result<()> {
    let status = Status::start_with_symbol("Χαρτογράφησις (Mapping)", "🗺️");

    let source = crate::tools::runner::load_source(input)?;
    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    let map = generate_map(&program);

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   M A P".bold().cyan());
    println!("   {}", "Architectural Blueprint".italic().dim());
    println!();

    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);

    if map.trim() == "classDiagram" {
        table.set_header(vec![
            Cell::new("Status")
                .add_attribute(Attribute::Bold)
                .fg(Color::Yellow),
        ]);
        table.add_row(vec![
            Cell::new("No architectural structures (Structs) found.")
                .fg(Color::DarkGrey)
                .add_attribute(Attribute::Italic),
        ]);
        println!("{table}");
        println!();
    } else {
        table.set_header(vec![
            Cell::new("Mermaid.js Diagram")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
        ]);

        // Wrap in markdown code block for easy copying
        let formatted_map = format!("```mermaid\n{}\n```", map.trim());

        table.add_row(vec![Cell::new(formatted_map)]);

        println!("{table}");
        println!();
        println!("   {}", "📋 Usage Instructions:".bold().underlined());
        println!("   1. Copy the code block above.");
        println!(
            "   2. Paste it into {}",
            "https://mermaid.live".cyan().underlined()
        );
        println!();
    }

    Ok(())
}

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
                    if dep != *name {
                        // Avoid self-reference arrows if desired (or keep them?)
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
            let params_str: Vec<String> = method
                .params
                .iter()
                .map(|(n, t)| format!("{}: {}", n, t))
                .collect();
            let ret_str = method
                .return_type
                .as_ref()
                .map(|t| format!(": {}", t))
                .unwrap_or_default();
            map.push_str(&format!(
                "        +{}({}){}\n",
                method.name,
                params_str.join(", "),
                ret_str
            ));
        }
        map.push_str("    }\n");
    }

    // 3. Add dependencies (struct field usage)
    // Only include dependencies if the target is actually a defined type/trait in the diagram
    // to avoid arrows to "Unknown" or external things if not modeled.
    let defined_types: HashSet<String> = program
        .scope
        .types()
        .filter_map(|(_, ty)| {
            if let GlossaType::Struct { name, .. } = ty {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect();

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
        if let AnalyzedStatement::TraitImplementation {
            trait_name,
            type_name,
            ..
        } = stmt
        {
            map.push_str(&format!(
                "    {} <|.. {} : implements\n",
                trait_name, type_name
            ));
        }
    }

    map
}

/// Helper to recursively extract struct names from a type
fn extract_dependencies(ty: &GlossaType) -> Vec<String> {
    match ty {
        GlossaType::Struct { name, .. } => vec![name.to_string()],
        GlossaType::List(inner) | GlossaType::Set(inner) | GlossaType::Option(inner) => {
            extract_dependencies(inner)
        }
        GlossaType::Map(k, v) | GlossaType::Result(k, v) => {
            let mut deps = extract_dependencies(k);
            deps.extend(extract_dependencies(v));
            deps
        }
        GlossaType::Function { params, returns } => {
            let mut deps = vec![];
            for p in params {
                deps.extend(extract_dependencies(p));
            }
            deps.extend(extract_dependencies(returns));
            deps
        }
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
        assert!(
            map.contains("εκτυπωσιμος <|.. χ : implements")
                || map.contains("Εκτυπώσιμος <|.. Χ : implements")
        );
    }

    #[test]
    fn test_cartographer_complex_dependencies() {
        // Manually construct types to test extract_dependencies logic for List, Map, etc.
        // since current parser doesn't support generic syntax yet.
        use crate::semantic::Scope;

        let mut scope = Scope::new();

        // Define "Inner" struct
        let inner_type = GlossaType::Struct {
            name: "Inner".into(),
            gender: crate::morphology::Gender::Neuter,
            fields: vec![],
        };
        scope.define_type("Inner", inner_type.clone());

        // Define "Container" struct with complex fields
        let container_type = GlossaType::Struct {
            name: "Container".into(),
            gender: crate::morphology::Gender::Neuter,
            fields: vec![
                ("l".into(), GlossaType::List(Box::new(inner_type.clone()))),
                (
                    "m".into(),
                    GlossaType::Map(Box::new(inner_type.clone()), Box::new(inner_type.clone())),
                ),
                ("o".into(), GlossaType::Option(Box::new(inner_type.clone()))),
                (
                    "r".into(),
                    GlossaType::Result(Box::new(inner_type.clone()), Box::new(inner_type.clone())),
                ),
                (
                    "f".into(),
                    GlossaType::Function {
                        params: vec![inner_type.clone()],
                        returns: Box::new(inner_type.clone()),
                    },
                ),
            ],
        };
        scope.define_type("Container", container_type);

        let program = AnalyzedProgram {
            statements: vec![],
            scope,
        };

        let map = generate_map(&program);

        // Verify Container exists
        assert!(map.contains("class Container"));
        // Verify Inner exists
        assert!(map.contains("class Inner"));

        // Verify dependency arrows
        // Container -> Inner should appear exactly once due to HashSet
        assert!(map.contains("Container --> Inner"));
        let arrow_count = map.matches("-->").count();
        assert_eq!(arrow_count, 1);
    }

    #[test]
    fn test_cartographer_filtering() {
        use crate::semantic::Scope;

        // Manually construct a case where a struct refers to a type NOT in the scope
        // This simulates a filtered dependency (or filtering of self-reference)
        let mut scope = Scope::new();

        // Define "Node" struct with self-reference
        // Note: In real analysis, the type object itself is recursive or placeholder.
        // We use a manual construction here.
        let node_type = GlossaType::Struct {
            name: "Node".into(),
            gender: crate::morphology::Gender::Neuter,
            fields: vec![],
        };

        // Define fields referencing "Node" (self) and "Other" (undefined in scope)
        let node_with_fields = GlossaType::Struct {
            name: "Node".into(),
            gender: crate::morphology::Gender::Neuter,
            fields: vec![
                ("self_ref".into(), node_type.clone()),
                (
                    "other_ref".into(),
                    GlossaType::Struct {
                        name: "Other".into(), // "Other" is not added to scope!
                        gender: crate::morphology::Gender::Neuter,
                        fields: vec![],
                    },
                ),
            ],
        };

        scope.define_type("Node", node_with_fields);

        let program = AnalyzedProgram {
            statements: vec![],
            scope,
        };

        let map = generate_map(&program);

        assert!(map.contains("class Node"));

        // Should NOT contain arrow to self
        assert!(!map.contains("Node --> Node"));

        // Should NOT contain arrow to Other (because Other is not in scope)
        assert!(!map.contains("Node --> Other"));
    }

    #[test]
    fn test_cartographer_complex_methods() {
        use crate::semantic::{AnalyzedMethod, Scope, TraitDef};

        let mut scope = Scope::new();

        // Manually define a trait with complex method signature
        let method = AnalyzedMethod {
            name: "complex_method".into(),
            params: vec![
                ("a".into(), GlossaType::Number),
                ("b".into(), GlossaType::String),
            ],
            body: None,
            return_type: Some(GlossaType::Boolean),
        };

        let trait_def = TraitDef {
            name: "ComplexTrait".into(),
            methods: vec![method],
        };

        scope.define_trait("ComplexTrait", trait_def);

        let program = AnalyzedProgram {
            statements: vec![],
            scope,
        };

        let map = generate_map(&program);

        assert!(map.contains("class ComplexTrait"));
        assert!(map.contains("complex_method"));
        assert!(map.contains("a: Ἀριθμός"));
        assert!(map.contains("b: Ὄνομα"));
        assert!(map.contains(": Ἀληθές/Ψεῦδος"));
    }

    #[test]
    fn test_run_map_success() {
        // Create a temporary file with a simple struct
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("test_map.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("εἶδος Τ ὁρίζειν { }.\n".as_bytes()).unwrap();
        }

        // Run the command
        let result = run_map(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_map_file_not_found() {
        use std::path::PathBuf;
        let path = PathBuf::from("non_existent_file.γλ");
        let result = run_map(&path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("οὐχ εὑρέθη"));
    }

    #[test]
    fn test_run_map_empty_diagram() {
        let dir = tempfile::tempdir().unwrap();
        let input_path = dir.path().join("test_map_empty.γλ");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&input_path).unwrap();
            f.write_all("ξ 1 ἔστω.\n".as_bytes()).unwrap();
        }

        // Run the command to ensure the empty logic is hit without panicking
        let result = run_map(&input_path);
        assert!(result.is_ok());
    }
}
