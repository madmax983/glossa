//! Type generation for Rust
//!
//! This module handles the conversion of Semantic types ([`GlossaType`]) to their Rust equivalents.
//!
//! It provides the bridge between the high-level semantic analysis (where types are represented
//! by Greek concepts like `ἀριθμός` or `ὄνομα`) and the low-level code generation (which outputs
//! Rust `i64` or `String`).
//!
//! # Mapping Strategy
//!
//! | ΓΛΩΣΣΑ Type | Rust Type | Notes |
//! |-------------|-----------|-------|
//! | `ἀριθμός` | `i64` | 64-bit signed integer |
//! | `ὄνομα` | `String` | Heap-allocated UTF-8 string |
//! | `ἀληθές/ψεῦδος` | `bool` | Boolean |
//! | `λίστη` | `Vec<T>` | Dynamic array |
//! | `εὑρεθείη` | `Option<T>` | Optional value |
//! | `ἀποτέλεσμα` | `Result<T, E>` | Success or Error |

use crate::codegen::utils::{capitalize, sanitize_name};
use crate::semantic::GlossaType;

/// Convert a Glossa type to its Rust equivalent string
///
/// This function recursively traverses complex types (like `Vec<Option<i64>>`)
/// and produces a string that is valid Rust syntax.
///
/// # Examples
///
/// ```
/// use glossa::codegen::types::to_rust_type;
/// use glossa::semantic::GlossaType;
///
/// // Simple types
/// assert_eq!(to_rust_type(&GlossaType::Number), "i64");
/// assert_eq!(to_rust_type(&GlossaType::String), "String");
///
/// // Complex nested types
/// let list_of_numbers = GlossaType::List(Box::new(GlossaType::Number));
/// assert_eq!(to_rust_type(&list_of_numbers), "Vec<i64>");
///
/// // Result types
/// let result_type = GlossaType::Result(
///     Box::new(GlossaType::Number),
///     Box::new(GlossaType::String)
/// );
/// assert_eq!(to_rust_type(&result_type), "Result<i64, String>");
/// ```
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
