import re

with open("src/tools/scholar.rs", "r") as f:
    content = f.read()

# Define the new helper functions and the new run_scholar
new_functions = """
fn write_types_docs(md: &mut String, program: &crate::semantic::AnalyzedProgram) {
    let mut types = program.scope.types().peekable();
    if types.peek().is_some() {
        writeln!(md, "## Types (Εἴδη)\\n").unwrap();
        for (name, type_def) in types {
            writeln!(md, "### `{}`\\n", name).unwrap();
            if let crate::semantic::GlossaType::Struct { fields, .. } = type_def {
                if !fields.is_empty() {
                    writeln!(md, "| Field | Type |\\n|-------|------|").unwrap();
                    for (field_name, field_type) in fields {
                        writeln!(md, "| `{}` | `{}` |", field_name, field_type).unwrap();
                    }
                    md.push('\\n');
                } else {
                    writeln!(md, "*No fields defined.*\\n").unwrap();
                }
            }
        }
    }
}

fn write_traits_docs(md: &mut String, program: &crate::semantic::AnalyzedProgram) {
    let mut traits = program.scope.traits().peekable();
    if traits.peek().is_some() {
        writeln!(md, "## Traits (Χαρακτῆρες)\\n").unwrap();
        for (name, trait_def) in traits {
            writeln!(md, "### `{}`\\n", name).unwrap();
            if !trait_def.methods.is_empty() {
                for method in &trait_def.methods {
                    writeln!(md, "* `{}`", method.name).unwrap();
                }
                md.push('\\n');
            } else {
                writeln!(md, "*No methods defined.*\\n").unwrap();
            }
        }
    }
}

fn write_functions_docs(md: &mut String, program: &crate::semantic::AnalyzedProgram) {
    let mut functions = program.scope.functions().peekable();
    if functions.peek().is_some() {
        writeln!(md, "## Functions (Ἔργα)\\n").unwrap();
        for func in functions {
            // ⚡ Bolt Optimization: Use `write!` to build strings dynamically without intermediate `Vec` collections.
            write!(md, "### `{}(", func.name).unwrap();
            for (i, t) in func.param_types.iter().enumerate() {
                if i > 0 {
                    write!(md, ", ").unwrap();
                }
                write!(md, "{}", t).unwrap();
            }
            write!(md, ") -> ").unwrap();
            if let Some(ret_type) = &func.return_type {
                write!(md, "{}", ret_type).unwrap();
            } else {
                write!(md, "Οὐδέν").unwrap();
            }
            writeln!(md, "`\\n").unwrap();
        }
    }
}

pub fn run_scholar(input: &Path) -> Result<()> {
    let source = load_source(input)?;
    let status = Status::start_with_symbol("Συγγραφή (Generating Docs)", "📜");

    let program = match crate::tools::runner::analyze_source(&source) {
        Ok(p) => p,
        Err(e) => {
            status.error("Σφάλμα (Error)");
            return Err(e);
        }
    };

    let mut md = String::with_capacity(4096);
    let filename = input.file_name().unwrap_or_default().to_string_lossy();

    writeln!(md, "# API Documentation: `{}`\\n", filename).unwrap();

    write_types_docs(&mut md, &program);
    write_traits_docs(&mut md, &program);
    write_functions_docs(&mut md, &program);

    let output_path = input.with_extension("doc.md");
    if let Err(e) = std::fs::write(&output_path, &md) {
        status.error("Σφάλμα ἀρχείου (File Error)");
        return Err(miette::miette!("Failed to write documentation file: {}", e));
    }

    status.success();

    println!();
    println!("   {}", "Γ Λ Ω Σ Σ Α   S C H O L A R".bold().cyan());
    println!("   {}", "API Documentation Generated".italic().dim());
    println!();
    println!(
        "   {} {}",
        "Saved to:".bold(),
        output_path.display().to_string().cyan()
    );
    println!();

    Ok(())
}
"""

start_idx = content.find("pub fn run_scholar(input: &Path) -> Result<()> {")
end_idx = content.find("#[cfg(test)]", start_idx)

if start_idx != -1 and end_idx != -1:
    new_content = content[:start_idx] + new_functions + content[end_idx:]
    with open("src/tools/scholar.rs", "w") as f:
        f.write(new_content)
    print("Refactored src/tools/scholar.rs successfully.")
else:
    print("Failed to find replacement boundaries.")
