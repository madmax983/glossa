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
        assert_eq!(map_type(&GlossaType::Unit), "()".to_string());
        assert_eq!(map_type(&GlossaType::Unknown), "_".to_string());
        assert_eq!(map_type(&GlossaType::Function { params: vec![], returns: Box::new(GlossaType::Unit) }), "fn".to_string());

        let list_type = GlossaType::List(Box::new(GlossaType::Number));
        assert_eq!(map_type(&list_type), "Vec<i64>");

        let set_type = GlossaType::Set(Box::new(GlossaType::String));
        assert_eq!(map_type(&set_type), "HashSet<String>");

        let map_type_obj = GlossaType::Map(Box::new(GlossaType::String), Box::new(GlossaType::Number));
        assert_eq!(map_type(&map_type_obj), "HashMap<String, i64>");

        let struct_type = GlossaType::Struct {
            name: "χρηστης".into(),
            gender: crate::morphology::Gender::Masculine,
            fields: vec![]
        };
        assert_eq!(map_type(&struct_type), "Chrestes"); // sanitized and capitalized

        let option_type = GlossaType::Option(Box::new(GlossaType::Number));
        assert_eq!(map_type(&option_type), "Option<i64>");

        let result_type = GlossaType::Result(Box::new(GlossaType::Number), Box::new(GlossaType::String));
        assert_eq!(map_type(&result_type), "Result<i64, String>");
    }

    #[test]
    fn test_map_ownership() {
        assert_eq!(map_ownership(Ownership::Borrow), "&");
        assert_eq!(map_ownership(Ownership::BorrowMut), "&mut ");
        assert_eq!(map_ownership(Ownership::Move), "");
        assert_eq!(map_ownership(Ownership::Copy), "");
    }

    #[test]
    fn test_map_case_ownership() {
        assert_eq!(map_case_ownership(Case::Nominative), "");
        assert_eq!(map_case_ownership(Case::Genitive), "&");
        assert_eq!(map_case_ownership(Case::Dative), "&mut");
        assert_eq!(map_case_ownership(Case::Accusative), "");
        assert_eq!(map_case_ownership(Case::Vocative), "");
    }

    #[test]
    fn test_map_bin_op() {
        assert_eq!(map_bin_op(BinaryOp::Add), "+");
        assert_eq!(map_bin_op(BinaryOp::Sub), "-");
        assert_eq!(map_bin_op(BinaryOp::Mul), "*");
        assert_eq!(map_bin_op(BinaryOp::Div), "/");
        assert_eq!(map_bin_op(BinaryOp::Mod), "%");
        assert_eq!(map_bin_op(BinaryOp::Eq), "==");
        assert_eq!(map_bin_op(BinaryOp::Ne), "!=");
        assert_eq!(map_bin_op(BinaryOp::Lt), "<");
        assert_eq!(map_bin_op(BinaryOp::Le), "<=");
        assert_eq!(map_bin_op(BinaryOp::Gt), ">");
        assert_eq!(map_bin_op(BinaryOp::Ge), ">=");
        assert_eq!(map_bin_op(BinaryOp::And), "&&");
        assert_eq!(map_bin_op(BinaryOp::Or), "||");
    }

    #[test]
    fn test_map_unary_op() {
        assert_eq!(map_unary_op(UnaryOp::Not), "!");
        assert_eq!(map_unary_op(UnaryOp::Neg), "-");
    }

    #[test]
    fn test_sanitize_greek_letter() {
        assert_eq!(sanitize_name("α"), "alpha");
        assert_eq!(sanitize_name("β"), "beta");
        assert_eq!(sanitize_name("γ"), "gamma");
        assert_eq!(sanitize_name("δ"), "delta");
        assert_eq!(sanitize_name("ε"), "epsilon");
        assert_eq!(sanitize_name("ζ"), "zeta");
        assert_eq!(sanitize_name("η"), "eta");
        assert_eq!(sanitize_name("θ"), "theta");
        assert_eq!(sanitize_name("ι"), "iota");
        assert_eq!(sanitize_name("κ"), "kappa");
        assert_eq!(sanitize_name("λ"), "lambda");
        assert_eq!(sanitize_name("μ"), "mu");
        assert_eq!(sanitize_name("ν"), "nu");
        assert_eq!(sanitize_name("ξ"), "xi");
        assert_eq!(sanitize_name("ο"), "omicron");
        assert_eq!(sanitize_name("π"), "pi");
        assert_eq!(sanitize_name("ρ"), "rho");
        assert_eq!(sanitize_name("σ"), "sigma");
        assert_eq!(sanitize_name("ς"), "sigma");
        assert_eq!(sanitize_name("τ"), "tau");
        assert_eq!(sanitize_name("υ"), "upsilon");
        assert_eq!(sanitize_name("φ"), "phi");
        assert_eq!(sanitize_name("χ"), "chi");
        assert_eq!(sanitize_name("ψ"), "psi");
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

    #[test]
    fn test_transliterate_numbers() {
        // Rust identifiers cannot start with a number
        assert_eq!(transliterate("123"), "_123");
        assert_eq!(transliterate("1α"), "_1a");
    }
}
