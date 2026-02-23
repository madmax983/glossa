//! The Weaver (ὁ Ὑφάντης) - Scaffolding Tool
//!
//! This module implements the "Weaver" tool, which generates boilerplate ΓΛΩΣΣΑ code
//! from high-level descriptions (e.g., "struct User name:String").
//!
//! # Purpose
//!
//! Writing grammatically correct Ancient Greek can be challenging. The Weaver
//! automates the declension and syntax rules, allowing developers to focus on logic.

use clap::Subcommand;
use crossterm::style::Stylize;
use miette::Result;

/// Commands for the Weaver tool
#[derive(Subcommand, Debug, Clone)]
pub enum WeaverCommands {
    /// Scaffold a new struct (Type Definition)
    Struct {
        /// Name of the struct (e.g., "User" or "Χρήστης")
        name: String,
        /// Fields in format name:Type (e.g. "score:Number")
        #[arg(required = true)]
        fields: Vec<String>,
    },

    /// Scaffold a new function (Function Definition)
    Function {
        /// Name of the function (e.g., "calculate" or "υπολογίζω")
        name: String,
        /// Parameters in format name:Type (e.g. "x:Number")
        #[arg(short, long)]
        params: Vec<String>,
        /// Return type (e.g., "Number")
        #[arg(short, long)]
        returns: Option<String>,
    },
}

/// Run the Weaver tool
pub fn run_weaver(cmd: WeaverCommands) -> Result<()> {
    match cmd {
        WeaverCommands::Struct { name, fields } => scaffold_struct(name, fields),
        WeaverCommands::Function {
            name,
            params,
            returns,
        } => scaffold_function(name, params, returns),
    }
}

fn scaffold_struct(name: String, fields: Vec<String>) -> Result<()> {
    let struct_name = normalize_input(&name);

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   W E A V E R".bold().cyan());
    println!("   {}", "Scaffolding Struct...".italic().dim());
    println!();

    println!("εἶδος {} ὁρίζειν {{", struct_name.bold());

    for field in fields {
        let (field_name, field_type) = parse_field(&field);
        let type_genitive = to_genitive(&field_type);
        println!("    {} {}.", field_name, type_genitive);
    }

    println!("}}.");
    println!();

    Ok(())
}

fn scaffold_function(name: String, params: Vec<String>, returns: Option<String>) -> Result<()> {
    let func_name = normalize_input(&name);

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   W E A V E R".bold().cyan());
    println!("   {}", "Scaffolding Function...".italic().dim());
    println!();

    // Header: name ὁρίζειν
    print!("{} ὁρίζειν", func_name.bold());

    // Params: τῷ name type
    for param in params {
        let (param_name, param_type) = parse_field(&param);
        let type_genitive = to_genitive(&param_type);
        print!(" τῷ {} {}", param_name, type_genitive);
    }

    print!("·");

    // Body
    println!();
    println!("    // TODO: Implement function body");

    if let Some(ret) = returns {
        let ret_genitive = to_genitive(&ret);
        // Returns are usually implicit in the last expression, or explicit via `return`.
        // But for scaffolding, we just print a placeholder return value.
        println!("    // Returns: {}", ret_genitive);
        println!("    οὐδέν.");
    } else {
        println!("    οὐδέν.");
    }

    println!();

    Ok(())
}

/// Parse "name:Type" into ("name", "Type")
fn parse_field(input: &str) -> (String, String) {
    if let Some((name, type_part)) = input.split_once(':') {
        (name.trim().to_string(), type_part.trim().to_string())
    } else {
        // Default to Number if no type specified? Or just use as is?
        // Let's assume the input is just the name and default type is Unknown (or user must fill it)
        (input.trim().to_string(), "Unknown".to_string())
    }
}

/// Normalize input (capitalization etc.)
fn normalize_input(input: &str) -> String {
    // For now, just return as is.
    // In future, we could enforce capitalization for Types.
    input.to_string()
}

/// Convert a type name to its Genitive form (for definitions)
fn to_genitive(type_name: &str) -> String {
    match type_name.to_lowercase().as_str() {
        // Standard types (English -> Greek Genitive)
        "string" | "str" | "text" | "όνομα" | "ονομα" => "ὀνόματος".to_string(),
        "number" | "int" | "i64" | "num" | "αριθμός" | "αριθμος" => {
            "ἀριθμοῦ".to_string()
        }
        "list" | "vec" | "array" | "λίστη" | "λιστη" => "λίστης".to_string(),
        "bool" | "boolean" => "ἀληθοῦς".to_string(), // Best guess, though not standard supported yet

        // Default: If it ends in 's' (English plural?), keep as is?
        // If it's a Greek word, we might want to decline it, but we lack the gender context here.
        // So we return it as is, assuming the user knows the Genitive or will fix it.
        _ => type_name.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_genitive() {
        assert_eq!(to_genitive("String"), "ὀνόματος");
        assert_eq!(to_genitive("Number"), "ἀριθμοῦ");
        assert_eq!(to_genitive("User"), "User");
        assert_eq!(to_genitive("Παίκτης"), "Παίκτης");
    }

    #[test]
    fn test_parse_field() {
        assert_eq!(
            parse_field("score:Number"),
            ("score".to_string(), "Number".to_string())
        );
        assert_eq!(
            parse_field("name"),
            ("name".to_string(), "Unknown".to_string())
        );
    }
}
