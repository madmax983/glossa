//! The Envoy (ὁ Ἄγγελος) - TypeScript definition generator
//!
//! This module implements the "Envoy" tool, which parses a ΓΛΩΣΣΑ program
//! and exports its types and functions as TypeScript definitions (`.d.ts`).
//!

use crate::parser::parse;
use crate::semantic::{GlossaType, analyze_program};
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::fs;
use std::path::Path;

fn glossa_type_to_ts(ty: &GlossaType) -> String {
    match ty {
        GlossaType::Number => "number".to_string(),
        GlossaType::String => "string".to_string(),
        GlossaType::Boolean => "boolean".to_string(),
        GlossaType::List(inner) => format!("Array<{}>", glossa_type_to_ts(inner)),
        GlossaType::Set(inner) => format!("Set<{}>", glossa_type_to_ts(inner)),
        GlossaType::Map(k, v) => {
            format!("Record<{}, {}>", glossa_type_to_ts(k), glossa_type_to_ts(v))
        }
        GlossaType::Option(inner) => format!("{} | null", glossa_type_to_ts(inner)),
        GlossaType::Result(ok, _err) => format!("{} | Error", glossa_type_to_ts(ok)),
        GlossaType::Struct { name, .. } => name.to_string(),
        GlossaType::Function { .. } => "Function".to_string(),
        GlossaType::Unit => "void".to_string(),
        GlossaType::Unknown => "any".to_string(),
    }
}

pub fn run_envoy(input: &Path) -> Result<()> {
    let source =
        fs::read_to_string(input).map_err(|e| miette::miette!("Failed to read file: {}", e))?;
    println!("✈️ {}...", "Μετάφρασις (Generating TS definitions)".bold());

    let ast = parse(&source).map_err(|e| miette::miette!("{}", e))?;
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    let mut ts = String::with_capacity(4096);

    // Types (Structs)
    let mut types = program.scope.types().peekable();
    if types.peek().is_some() {
        for (name, type_def) in types {
            if let GlossaType::Struct { fields, .. } = type_def {
                writeln!(ts, "export interface {} {{", name).unwrap();
                for (field_name, field_type) in fields {
                    writeln!(ts, "    {}: {};", field_name, glossa_type_to_ts(field_type)).unwrap();
                }
                writeln!(ts, "}}\n").unwrap();
            }
        }
    }

    // Functions
    let mut functions = program.scope.functions().peekable();
    if functions.peek().is_some() {
        for func in functions {
            write!(ts, "export declare function {}(", func.name).unwrap();
            for (i, t) in func.param_types.iter().enumerate() {
                if i > 0 {
                    write!(ts, ", ").unwrap();
                }
                write!(ts, "arg{}: {}", i, glossa_type_to_ts(t)).unwrap();
            }
            write!(ts, "): ").unwrap();
            if let Some(ret_type) = &func.return_type {
                write!(ts, "{};\n\n", glossa_type_to_ts(ret_type)).unwrap();
            } else {
                write!(ts, "void;\n\n").unwrap();
            }
        }
    }

    let output_path = input.with_extension("d.ts");
    if let Err(e) = std::fs::write(&output_path, &ts) {
        return Err(miette::miette!("Failed to write TS file: {}", e));
    }

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   E N V O Y".bold().cyan());
    println!("   {}", "TypeScript Definitions Generated".italic().dim());
    println!();
    println!(
        "   {} {}",
        "Saved to:".bold(),
        output_path.display().to_string().cyan()
    );
    println!();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_glossa_type_to_ts() {
        assert_eq!(glossa_type_to_ts(&GlossaType::Number), "number");
        assert_eq!(glossa_type_to_ts(&GlossaType::String), "string");
        assert_eq!(glossa_type_to_ts(&GlossaType::Boolean), "boolean");
        assert_eq!(glossa_type_to_ts(&GlossaType::Unit), "void");
        assert_eq!(glossa_type_to_ts(&GlossaType::Unknown), "any");
        assert_eq!(
            glossa_type_to_ts(&GlossaType::List(Box::new(GlossaType::Number))),
            "Array<number>"
        );
    }

    #[test]
    fn test_run_envoy_success() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("api.γλ");
        fs::write(&input_path, "εἶδος Χρήστης ὁρίζειν { ὄνομα ὀνόματος. }.").unwrap();

        let result = run_envoy(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("d.ts");
        assert!(output_path.exists());
        let ts_content = fs::read_to_string(&output_path).unwrap();
        assert!(ts_content.contains("export interface Χρήστης {"));
        assert!(ts_content.contains("ὄνομα: string;"));
    }

    #[test]
    fn test_run_envoy_with_functions() {
        let dir = tempdir().unwrap();
        let input_path = dir.path().join("api.γλ");
        let source = "προσθεσις ὁρίζειν τῷ ξ ἀριθμοῦ τῷ ψ ἀριθμοῦ· δός ξ ψ ἄθροισμα.";
        fs::write(&input_path, source).unwrap();

        let result = run_envoy(&input_path);
        assert!(result.is_ok());

        let output_path = input_path.with_extension("d.ts");
        assert!(output_path.exists());
        let ts_content = fs::read_to_string(&output_path).unwrap();
        assert!(
            ts_content
                .contains("export declare function προσθεσις(arg0: number, arg1: number): number;")
        );
    }
}
