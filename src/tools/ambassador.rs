//! The Ambassador (ὁ Πρόξενος) - C Header Generator
//!
//! This module implements the "Ambassador" tool, which inspects the type definitions
//! and functions within a ΓΛΩΣΣΑ program and translates them into a C header file.
//!
//! # Purpose
//!
//! Enables C/C++ FFI interoperability by exporting C structs and function prototypes.

use crate::semantic::{AnalyzedProgram, AnalyzedStatement, GlossaType};
use crate::tools::runner::load_source;
use crate::tools::ui::Status;
use crossterm::style::Stylize;
use miette::Result;
use std::fmt::Write;
use std::path::Path;

/// Map Glossa types to C types
fn glossa_to_c_type(g_type: &GlossaType) -> &'static str {
    match g_type {
        GlossaType::Number => "int64_t",
        GlossaType::String => "const char*",
        GlossaType::Boolean => "bool",
        _ => "void*",
    }
}

pub fn run_ambassador(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(miette::miette!("Ἀρχεῖον οὐχ εὑρέθη: {}", input.display()));
    }

    let status = Status::start_with_symbol("Πρόξενος (Generating C Header)", "📜");

    let source = match load_source(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(e);
        }
    };

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα ἀναλύσεως (Analysis Error)");
            return Err(e);
        }
    };

    let c_code = transpile_to_c_header(&program, "GLOSSA_EXPORT_H");

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   A M B A S S A D O R".bold().cyan());
    println!("   {}", "C Header Generation Result".italic().dim());
    println!();
    println!("{}", c_code.yellow());
    println!();

    Ok(())
}

pub fn transpile_to_c_header(program: &AnalyzedProgram, guard_name: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "#ifndef {}", guard_name);
    let _ = writeln!(&mut out, "#define {}", guard_name);
    let _ = writeln!(&mut out, "\n#include <stdint.h>");
    let _ = writeln!(&mut out, "#include <stdbool.h>\n");

    for stmt in &program.statements {
        match stmt {
            AnalyzedStatement::TypeDefinition { name, fields } => {
                let _ = writeln!(&mut out, "typedef struct {} {{", name);
                for (field_name, field_type) in fields {
                    let c_type = glossa_to_c_type(field_type);
                    let _ = writeln!(&mut out, "    {} {};", c_type, field_name);
                }
                let _ = writeln!(&mut out, "}} {};\n", name);
            }
            AnalyzedStatement::FunctionDef {
                name,
                params,
                return_type,
                ..
            } => {
                let ret_c_type = return_type.as_ref().map_or("void", glossa_to_c_type);
                let _ = write!(&mut out, "{} {}(", ret_c_type, name);

                if params.is_empty() {
                    let _ = write!(&mut out, "void");
                } else {
                    for (i, (param_name, param_type)) in params.iter().enumerate() {
                        if i > 0 {
                            let _ = write!(&mut out, ", ");
                        }
                        let c_type = param_type.as_ref().map_or("void*", glossa_to_c_type);
                        let _ = write!(&mut out, "{} {}", c_type, param_name);
                    }
                }
                let _ = writeln!(&mut out, ");\n");
            }
            _ => {}
        }
    }

    let _ = writeln!(&mut out, "#endif // {}", guard_name);
    out
}

#[cfg(test)]
mod tests_ambassador {
    use super::*;
    use crate::semantic::Scope;
    use smol_str::SmolStr;

    #[test]
    fn test_transpile_to_c_header() {
        let program = AnalyzedProgram {
            statements: vec![
                AnalyzedStatement::TypeDefinition {
                    name: SmolStr::new("User"),
                    fields: vec![
                        (SmolStr::new("age"), GlossaType::Number),
                        (SmolStr::new("name"), GlossaType::String),
                    ],
                },
                AnalyzedStatement::FunctionDef {
                    name: SmolStr::new("add"),
                    params: vec![
                        (SmolStr::new("a"), Some(GlossaType::Number)),
                        (SmolStr::new("b"), Some(GlossaType::Number)),
                    ],
                    body: vec![],
                    return_type: Some(GlossaType::Number),
                },
            ],
            scope: Scope::new(),
        };

        let result = transpile_to_c_header(&program, "TEST_H");

        assert!(result.contains("#ifndef TEST_H"));
        assert!(result.contains("typedef struct User {"));
        assert!(result.contains("int64_t age;"));
        assert!(result.contains("const char* name;"));
        assert!(result.contains("} User;"));
        assert!(result.contains("int64_t add(int64_t a, int64_t b);"));
        assert!(result.contains("#endif // TEST_H"));
    }
}
