//! The Weaver (ὁ Ὑφάντης) - Schema Generator
//!
//! This module converts simple English struct schemas into ΓΛΩΣΣΑ `εἶδος` definitions.
//! It maps common programming types to their corresponding Ancient Greek genitives.

use miette::{Diagnostic, Report};
use thiserror::Error;

/// Errors that can occur during weaving
#[derive(Debug, Error, Diagnostic)]
pub enum WeaveError {
    #[error("Invalid schema format")]
    #[diagnostic(help("Expected format: struct Name {{ field: Type, ... }}"))]
    InvalidFormat,
}

/// Parse a simple English struct schema and generate ΓΛΩΣΣΑ code.
///
/// Example input: `struct User { name: String, age: Number }`
pub fn weave_schema(input: &str) -> Result<String, Report> {
    let input = input.trim();

    // Basic validation: "struct Name { ... }"
    if !input.starts_with("struct ") || !input.ends_with('}') {
        return Err(WeaveError::InvalidFormat.into());
    }

    let without_struct = input.trim_start_matches("struct ").trim();

    // Find the opening brace
    let brace_idx = without_struct.find('{').ok_or(WeaveError::InvalidFormat)?;

    let struct_name = without_struct[..brace_idx].trim();
    if struct_name.is_empty() {
        return Err(WeaveError::InvalidFormat.into());
    }

    // Extract fields
    let fields_str = &without_struct[brace_idx + 1..without_struct.len() - 1];

    let mut fields = Vec::new();
    for field_decl in fields_str.split(',') {
        let field_decl = field_decl.trim();
        if field_decl.is_empty() {
            continue;
        }

        let mut parts = field_decl.split(':');
        let field_name = parts.next().ok_or(WeaveError::InvalidFormat)?.trim();
        let field_type = parts.next().ok_or(WeaveError::InvalidFormat)?.trim();

        fields.push((field_name, field_type));
    }

    // Generate output
    let mut out = String::new();
    out.push_str(&format!("εἶδος {} ὁρίζειν {{\n", struct_name));

    for (name, ty) in fields {
        let glossa_type = match ty {
            "String" => "ὀνόματος",
            "Number" => "ἀριθμα", // From memory instruction
            "Bool" => "ἀληθοῦς",
            "List" => "λίστης",
            _ => "ἀγνώστου", // Fallback for unknown
        };
        out.push_str(&format!("    {} {}.\n", name, glossa_type));
    }

    out.push_str("}.\n");

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weave_schema() {
        let input = "struct User { name: String, age: Number }";
        let expected = "εἶδος User ὁρίζειν {\n    name ὀνόματος.\n    age ἀριθμα.\n}.\n";

        let output = weave_schema(input).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_weave_invalid_format() {
        let input = "User { name: String }";
        assert!(weave_schema(input).is_err());
    }

    #[test]
    fn test_weave_all_types() {
        let input = "struct Data { s: String, n: Number, b: Bool, l: List }";
        let expected = "εἶδος Data ὁρίζειν {\n    s ὀνόματος.\n    n ἀριθμα.\n    b ἀληθοῦς.\n    l λίστης.\n}.\n";
        let output = weave_schema(input).unwrap();
        assert_eq!(output, expected);
    }
}
