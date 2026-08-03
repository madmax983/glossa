//! The Anatomist (ὁ Ἀνατόμος) - Memory Layout Analyzer
//!
//! This module implements the "Anatomist" tool, which analyzes the semantic AST
//! to compute and visualize the memory layout (sizes in bytes) of user-defined
//! Structs (`εἶδος`).

use crate::semantic::GlossaType;
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use miette::Result;
use std::path::Path;

fn size_of_type(ty: &GlossaType) -> usize {
    match ty {
        GlossaType::Number => 8,     // i64
        GlossaType::String => 24,    // String (ptr, cap, len)
        GlossaType::Boolean => 1,    // bool
        GlossaType::List(_) => 24,   // Vec (ptr, cap, len)
        GlossaType::Set(_) => 48,    // HashSet
        GlossaType::Map(_, _) => 48, // HashMap
        GlossaType::Option(inner) => {
            let inner_size = size_of_type(inner);
            if inner_size == 0 {
                1
            } else {
                let padding = if !inner_size.is_multiple_of(8) { 8 - (inner_size % 8) } else { 0 };
                (inner_size + padding).max(8) * 2 // Simplified optional overhead for word alignment
            }
        }
        GlossaType::Result(ok, err) => {
            let ok_size = size_of_type(ok);
            let err_size = size_of_type(err);
            ok_size.max(err_size) + 8 // Simplified enum overhead
        }
        GlossaType::Struct { fields, .. } => {
            // Very simplified struct layout (sum of fields + some padding)
            let mut total = 0;
            for (_, field_type) in fields {
                let size = size_of_type(field_type);
                // Assume 8-byte alignment for simplicity in this prototype
                let padding = if !size.is_multiple_of(8) { 8 - (size % 8) } else { 0 };
                total += size + padding;
            }
            if total == 0 { 0 } else { total.max(8) }
        }
        GlossaType::Function { .. } => 0, // ZST in Rust, or function pointer (8), going with 0 for pure functions
        GlossaType::Unit => 0,
        GlossaType::Unknown => 0,
    }
}

pub fn run_anatomist(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Ἀνατόμος (Analyzing Memory)", "🔬");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            return Err(e);
        }
    };

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            return Err(e);
        }
    };

    status.success();

    println!();
    println!("   Γ Λ Ω Σ Σ Α   A N A T O M I S T");
    println!("   Memory Layout for {}", input.display());
    println!();

    let mut found_structs = false;

    println!(
        "{:<20} | {:<20} | {:<30} | Size (Bytes)",
        "Struct (Εἶδος)", "Field (Πεδίο)", "Type (Τύπος)"
    );
    println!("{:-<20}-+-{:-<20}-+-{:-<30}-+-{:-<12}", "", "", "", "");

    for (name, type_def) in program.scope.types() {
        if let GlossaType::Struct { fields, .. } = type_def {
            found_structs = true;
            let total_size = size_of_type(type_def);

            println!("{:<20} | {:<20} | {:<30} | {}", name, "", "", total_size);

            for (field_name, field_type) in fields {
                let field_size = size_of_type(field_type);
                let type_str = format!("{:?}", field_type);
                println!(
                    "{:<20} | {:<20} | {:<30} | {}",
                    "", field_name, type_str, field_size
                );
            }
        }
    }

    if !found_structs {
        println!("   No user-defined structs (εἴδη) found in this file.");
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
    fn test_anatomist_basic() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("test.γλ");
        fs::write(
            &input_path,
            "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. ἡλικία ἀριθμοῦ. }.",
        )
        .unwrap();

        let result = run_anatomist(&input_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_anatomist_error() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("error.γλ");
        fs::write(&input_path, "invalid syntax").unwrap();
        let result = run_anatomist(&input_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_anatomist_no_file() {
        let result = run_anatomist(Path::new("not_found.γλ"));
        assert!(result.is_err());
    }

    #[test]
    fn test_size_of_type() {
        assert_eq!(size_of_type(&GlossaType::Number), 8);
        assert_eq!(size_of_type(&GlossaType::String), 24);
        assert_eq!(size_of_type(&GlossaType::Boolean), 1);
        assert_eq!(
            size_of_type(&GlossaType::List(Box::new(GlossaType::Number))),
            24
        );
        assert_eq!(
            size_of_type(&GlossaType::Set(Box::new(GlossaType::Number))),
            48
        );
        assert_eq!(
            size_of_type(&GlossaType::Map(
                Box::new(GlossaType::String),
                Box::new(GlossaType::Number)
            )),
            48
        );
        assert_eq!(
            size_of_type(&GlossaType::Option(Box::new(GlossaType::Number))),
            16
        );
        assert_eq!(
            size_of_type(&GlossaType::Result(
                Box::new(GlossaType::Number),
                Box::new(GlossaType::String)
            )),
            32
        );

        let struct_type = GlossaType::Struct {
            name: "test".into(),
            gender: crate::morphology::Gender::Masculine,
            fields: vec![
                ("field1".into(), GlossaType::Number),
                ("field2".into(), GlossaType::Boolean),
            ],
        };
        assert_eq!(size_of_type(&struct_type), 16); // 8 + 1 + padding(7) = 16

        assert_eq!(size_of_type(&GlossaType::Unit), 0);
        assert_eq!(size_of_type(&GlossaType::Unknown), 0);
    }
}
