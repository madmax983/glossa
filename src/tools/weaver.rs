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
use miette::{IntoDiagnostic, Result};
use std::io::Write;

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
pub fn run_weaver<W: Write>(cmd: WeaverCommands, writer: &mut W) -> Result<()> {
    match cmd {
        WeaverCommands::Struct { name, fields } => scaffold_struct(name, fields, writer),
        WeaverCommands::Function {
            name,
            params,
            returns,
        } => scaffold_function(name, params, returns, writer),
    }
}

fn scaffold_struct<W: Write>(name: String, fields: Vec<String>, writer: &mut W) -> Result<()> {
    let struct_name = normalize_input(&name);

    writeln!(writer).into_diagnostic()?;
    writeln!(writer, "   {}", "Γ Λ Ω Σ Σ Α   W E A V E R".bold().cyan()).into_diagnostic()?;
    writeln!(writer, "   {}", "Scaffolding Struct...".italic().dim()).into_diagnostic()?;
    writeln!(writer).into_diagnostic()?;

    writeln!(writer, "εἶδος {} ὁρίζειν {{", struct_name.bold()).into_diagnostic()?;

    for field in fields {
        let (field_name, field_type) = parse_field(&field);
        let type_genitive = to_genitive(&field_type);
        writeln!(writer, "    {} {}.", field_name, type_genitive).into_diagnostic()?;
    }

    writeln!(writer, "}}.").into_diagnostic()?;
    writeln!(writer).into_diagnostic()?;

    Ok(())
}

fn scaffold_function<W: Write>(
    name: String,
    params: Vec<String>,
    returns: Option<String>,
    writer: &mut W,
) -> Result<()> {
    let func_name = normalize_input(&name);

    writeln!(writer).into_diagnostic()?;
    writeln!(writer, "   {}", "Γ Λ Ω Σ Σ Α   W E A V E R".bold().cyan()).into_diagnostic()?;
    writeln!(writer, "   {}", "Scaffolding Function...".italic().dim()).into_diagnostic()?;
    writeln!(writer).into_diagnostic()?;

    // Header: name ὁρίζειν
    write!(writer, "{} ὁρίζειν", func_name.bold()).into_diagnostic()?;

    // Params: τῷ name type
    for param in params {
        let (param_name, param_type) = parse_field(&param);
        let type_genitive = to_genitive(&param_type);
        write!(writer, " τῷ {} {}", param_name, type_genitive).into_diagnostic()?;
    }

    write!(writer, "·").into_diagnostic()?;

    // Body
    writeln!(writer).into_diagnostic()?;
    writeln!(writer, "    // TODO: Implement function body").into_diagnostic()?;

    if let Some(ret) = returns {
        let ret_genitive = to_genitive(&ret);
        // Returns are usually implicit in the last expression, or explicit via `return`.
        // But for scaffolding, we just print a placeholder return value.
        writeln!(writer, "    // Returns: {}", ret_genitive).into_diagnostic()?;
        writeln!(writer, "    οὐδέν.").into_diagnostic()?;
    } else {
        writeln!(writer, "    οὐδέν.").into_diagnostic()?;
    }

    writeln!(writer).into_diagnostic()?;

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

    #[test]
    fn test_scaffold_struct() {
        let mut buffer = Vec::new();
        let cmd = WeaverCommands::Struct {
            name: "User".into(),
            fields: vec!["name:String".into(), "age:Number".into()],
        };

        run_weaver(cmd, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        // Check for content without strict formatting dependency (or handle ANSI codes)
        // We check for the presence of key components regardless of styling
        assert!(output.contains("εἶδος"));
        assert!(output.contains("User"));
        assert!(output.contains("ὁρίζειν"));
        assert!(output.contains("name ὀνόματος."));
        assert!(output.contains("age ἀριθμοῦ."));
    }

    #[test]
    fn test_scaffold_function() {
        let mut buffer = Vec::new();
        let cmd = WeaverCommands::Function {
            name: "calculate".into(),
            params: vec!["x:Number".into(), "y:Number".into()],
            returns: Some("Number".into()),
        };

        run_weaver(cmd, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("calculate"));
        assert!(output.contains("ὁρίζειν"));
        assert!(output.contains("τῷ x ἀριθμοῦ"));
        assert!(output.contains("τῷ y ἀριθμοῦ"));
        assert!(output.contains("Returns: ἀριθμοῦ"));
    }

    #[test]
    fn test_scaffold_function_no_return() {
        let mut buffer = Vec::new();
        let cmd = WeaverCommands::Function {
            name: "do_something".into(),
            params: vec![],
            returns: None,
        };

        run_weaver(cmd, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("do_something"));
        assert!(output.contains("ὁρίζειν"));
        assert!(output.contains("οὐδέν"));
        assert!(!output.contains("Returns:"));
    }
}
