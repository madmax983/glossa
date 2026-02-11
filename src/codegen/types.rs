//! Type generation for Rust
//!
//! Handles the conversion of semantic types to Rust types.

use crate::codegen::utils::{capitalize, sanitize_name};
use crate::semantic::GlossaType;

/// Convert a Glossa type to its Rust equivalent string
pub fn to_rust_type(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Number => "i64".to_string(),
        GlossaType::String => "String".to_string(),
        GlossaType::Boolean => "bool".to_string(),
        GlossaType::List(inner) => format!("Vec<{}>", to_rust_type(inner)),
        GlossaType::Set(inner) => format!("HashSet<{}>", to_rust_type(inner)),
        GlossaType::Map(key, value) => {
            format!("HashMap<{}, {}>", to_rust_type(key), to_rust_type(value))
        }
        GlossaType::Option(inner) => format!("Option<{}>", to_rust_type(inner)),
        GlossaType::Result(ok, err) => {
            format!("Result<{}, {}>", to_rust_type(ok), to_rust_type(err))
        }
        GlossaType::Unit => "()".to_string(),
        GlossaType::Struct { name, .. } => capitalize(&sanitize_name(name)),
        // TODO: Better representation for function types if they appear in type signatures
        GlossaType::Function { .. } => "fn".to_string(),
        GlossaType::Unknown => "_".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol_str::SmolStr;

    #[test]
    fn test_basic_types() {
        assert_eq!(to_rust_type(&GlossaType::Number), "i64");
        assert_eq!(to_rust_type(&GlossaType::String), "String");
        assert_eq!(to_rust_type(&GlossaType::Boolean), "bool");
        assert_eq!(to_rust_type(&GlossaType::Unit), "()");
        assert_eq!(to_rust_type(&GlossaType::Unknown), "_");
    }

    #[test]
    fn test_container_types() {
        assert_eq!(
            to_rust_type(&GlossaType::List(Box::new(GlossaType::Number))),
            "Vec<i64>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Set(Box::new(GlossaType::String))),
            "HashSet<String>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Map(
                Box::new(GlossaType::String),
                Box::new(GlossaType::Number)
            )),
            "HashMap<String, i64>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Option(Box::new(GlossaType::Number))),
            "Option<i64>"
        );
        assert_eq!(
            to_rust_type(&GlossaType::Result(
                Box::new(GlossaType::Number),
                Box::new(GlossaType::String)
            )),
            "Result<i64, String>"
        );
    }

    #[test]
    fn test_struct_type() {
        // Use normalized name as per compiler convention
        let ty = GlossaType::Struct {
            name: SmolStr::new("χρηστης"),
            gender: crate::morphology::Gender::Masculine,
            fields: vec![],
        };
        // Sanitize: χρηστης -> g__u3c7_rhsths
        // Capitalize: g__u3c7_rhsths -> G__u3c7_rhsths
        assert_eq!(to_rust_type(&ty), "G__u3c7_rhsths");
    }
}
