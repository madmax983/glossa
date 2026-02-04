//! Mappings from Domain Models to Rust code
//!
//! This module handles the translation of Semantic and Morphological types
//! into their Rust string equivalents. It isolates the "Leak" where domain
//! objects previously knew about their Rust representation.

use crate::morphology::Case;
use crate::morphology::lexicon::{BinaryOp, UnaryOp};
use crate::semantic::{GlossaType, Ownership};
use std::fmt::Write;

/// Map a GlossaType to its Rust type annotation
pub fn map_type(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Number => "i64".to_string(),
        GlossaType::String => "String".to_string(),
        GlossaType::Boolean => "bool".to_string(),
        GlossaType::List(inner) => format!("Vec<{}>", map_type(inner)),
        GlossaType::Set(inner) => format!("HashSet<{}>", map_type(inner)),
        GlossaType::Map(key, value) => {
            format!("HashMap<{}, {}>", map_type(key), map_type(value))
        }
        GlossaType::Option(inner) => format!("Option<{}>", map_type(inner)),
        GlossaType::Result(ok, err) => format!("Result<{}, {}>", map_type(ok), map_type(err)),
        GlossaType::Unit => "()".to_string(),
        GlossaType::Struct { name, .. } => capitalize(&sanitize_name(name)),
        GlossaType::Function { .. } => "fn".to_string(), // Placeholder/TODO
        GlossaType::Unknown => "_".to_string(),
    }
}

/// Map Ownership semantic to Rust reference prefix
pub fn map_ownership(ownership: Ownership) -> &'static str {
    match ownership {
        Ownership::Move => "",
        Ownership::Borrow => "&",
        Ownership::BorrowMut => "&mut ",
        Ownership::Copy => "",
    }
}

/// Map Case to Rust ownership prefix (Legacy/Direct mapping)
pub fn map_case_ownership(case: Case) -> &'static str {
    match case {
        Case::Nominative => "",
        Case::Genitive => "&",
        Case::Dative => "&mut",
        Case::Accusative => "",
        Case::Vocative => "",
    }
}

/// Map BinaryOp to Rust operator string
pub fn map_bin_op(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::Eq => "==",
        BinaryOp::Ne => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::Le => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::Ge => ">=",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
    }
}

/// Map UnaryOp to Rust operator string
pub fn map_unary_op(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Not => "!",
        UnaryOp::Neg => "-",
    }
}

/// Capitalize the first letter of a string (for Rust type/trait names)
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Sanitize a Greek name for use as a Rust identifier
pub fn sanitize_name(name: &str) -> String {
    // Map single Greek letters to their names
    if name.len() <= 2 {
        // Could be a single Greek letter (2 bytes for UTF-8)
        match name {
            "α" => return "alpha".to_string(),
            "β" => return "beta".to_string(),
            "γ" => return "gamma".to_string(),
            "δ" => return "delta".to_string(),
            "ε" => return "epsilon".to_string(),
            "ζ" => return "zeta".to_string(),
            "η" => return "eta".to_string(),
            "θ" => return "theta".to_string(),
            "ι" => return "iota".to_string(),
            "κ" => return "kappa".to_string(),
            "λ" => return "lambda".to_string(),
            "μ" => return "mu".to_string(),
            "ν" => return "nu".to_string(),
            "ξ" => return "xi".to_string(),
            "ο" => return "omicron".to_string(),
            "π" => return "pi".to_string(),
            "ρ" => return "rho".to_string(),
            "σ" | "ς" => return "sigma".to_string(),
            "τ" => return "tau".to_string(),
            "υ" => return "upsilon".to_string(),
            "φ" => return "phi".to_string(),
            "χ" => return "chi".to_string(),
            "ψ" => return "psi".to_string(),
            "ω" => return "omega".to_string(),
            _ => {}
        }
    }

    // Transliterate the full name
    transliterate(name)
}

/// Transliterate Greek to Latin characters
pub fn transliterate(greek: &str) -> String {
    let mut result = String::new();

    for c in greek.chars() {
        let trans = match c {
            'α' => "a",
            'β' => "b",
            'γ' => "g",
            'δ' => "d",
            'ε' => "e",
            'ζ' => "z",
            'η' => "e",
            'θ' => "th",
            'ι' => "i",
            'κ' => "k",
            'λ' => "l",
            'μ' => "m",
            'ν' => "n",
            'ξ' => "x",
            'ο' => "o",
            'π' => "p",
            'ρ' => "r",
            'σ' | 'ς' => "s",
            'τ' => "t",
            'υ' => "u",
            'φ' => "ph",
            'χ' => "ch",
            'ψ' => "ps",
            'ω' => "o",
            _ => {
                // Keep only ASCII alphanumeric characters and underscore
                if c.is_ascii_alphanumeric() || c == '_' {
                    result.push(c);
                } else {
                    // Replace invalid characters with unique hex code to prevent collisions
                    // e.g. ϟ -> _u3df_
                    write!(result, "_u{:x}_", c as u32).unwrap();
                }
                continue;
            }
        };
        result.push_str(trans);
    }

    // Ensure it starts with a letter or underscore (valid Rust identifier)
    if result
        .chars()
        .next()
        .map(|c| c.is_numeric())
        .unwrap_or(true)
    {
        format!("_{}", result)
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_type() {
        assert_eq!(map_type(&GlossaType::Number), "i64".to_string());
        assert_eq!(map_type(&GlossaType::String), "String".to_string());
        assert_eq!(map_type(&GlossaType::Boolean), "bool".to_string());
    }

    #[test]
    fn test_map_ownership() {
        assert_eq!(map_ownership(Ownership::Borrow), "&");
        assert_eq!(map_ownership(Ownership::BorrowMut), "&mut ");
        assert_eq!(map_ownership(Ownership::Move), "");
    }

    #[test]
    fn test_map_case_ownership() {
        assert_eq!(map_case_ownership(Case::Genitive), "&");
        assert_eq!(map_case_ownership(Case::Dative), "&mut");
        assert_eq!(map_case_ownership(Case::Accusative), "");
    }

    #[test]
    fn test_map_bin_op() {
        assert_eq!(map_bin_op(BinaryOp::Add), "+");
        assert_eq!(map_bin_op(BinaryOp::Gt), ">");
        assert_eq!(map_bin_op(BinaryOp::And), "&&");
    }

    #[test]
    fn test_map_unary_op() {
        assert_eq!(map_unary_op(UnaryOp::Not), "!");
    }

    #[test]
    fn test_sanitize_greek_letter() {
        assert_eq!(sanitize_name("ξ"), "xi");
        assert_eq!(sanitize_name("α"), "alpha");
        assert_eq!(sanitize_name("ω"), "omega");
    }

    #[test]
    fn test_transliterate() {
        assert_eq!(transliterate("χρηστος"), "chrestos");
        assert_eq!(transliterate("λογος"), "logos");
        assert_eq!(transliterate("φιλοσοφια"), "philosophia");
    }

    #[test]
    fn test_transliterate_unique() {
        let koppa = "ϟ";
        let stigma = "ϛ";
        let t_koppa = transliterate(koppa);
        let t_stigma = transliterate(stigma);
        assert_ne!(t_koppa, t_stigma);
        assert!(t_koppa.contains("_u3df_"));
        assert!(t_stigma.contains("_u3db_"));
    }
}
