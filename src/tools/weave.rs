use crate::codegen::generate_rust_file;
use crate::parser::parse;
use crate::semantic::analyze_program;
use crate::tools::mosaic::run_mosaic_inner;
use crate::tools::narrator::tell_tale;
use crate::tools::ui::Status;
use miette::{IntoDiagnostic, Result};
use std::fs;
use std::io::Write;
use std::path::Path;

/// Run the Weave tool on a file
///
/// Loads the source file, compiles it, runs visualization tools, and writes
/// a combined Markdown report.
pub fn run_weave(input: &Path, output: Option<&Path>) -> Result<()> {
    let status = Status::start_with_symbol("Ὕφανσις (Weaving)", "🧶");

    let source = fs::read_to_string(input).into_diagnostic()?;

    // Determine output path: given or input with `.md` extension
    let default_out = input.with_extension("md");
    let out_path = output.unwrap_or(&default_out);

    let mut file = fs::File::create(out_path).into_diagnostic()?;
    run_weave_inner(&source, &mut file)?;

    status.success();
    println!("   Wove Rosetta Stone to {}", out_path.display());

    Ok(())
}

/// Internal core function for Weave, useful for testing with a buffer
pub fn run_weave_inner<W: Write>(source: &str, writer: &mut W) -> Result<()> {
    writeln!(writer, "# ΓΛΩΣΣΑ (GLOSSA) Rosetta Stone\n").into_diagnostic()?;

    writeln!(writer, "## 1. Original Text\n").into_diagnostic()?;
    writeln!(writer, "```glossa\n{}\n```\n", source.trim()).into_diagnostic()?;

    // Parse once for the rest of the tools
    let ast = parse(source).map_err(|e| miette::miette!("{}", e))?;

    // Semantic Analysis
    let program = analyze_program(&ast).map_err(|e| miette::miette!("{}", e))?;

    // Mosaic Assembly
    writeln!(writer, "## 2. Morphological Assembly\n").into_diagnostic()?;
    writeln!(
        writer,
        "The compiler's `Assembler` deconstructs the free word order into grammatical slots based on case endings.\n"
    )
    .into_diagnostic()?;

    // Mosaic outputs its table which we capture in a buffer and inject into a code block
    let mut mosaic_buf = Vec::new();
    run_mosaic_inner(source, &mut mosaic_buf)?;
    let mosaic_str = String::from_utf8(mosaic_buf).into_diagnostic()?;
    writeln!(writer, "```text\n{}\n```\n", mosaic_str.trim()).into_diagnostic()?;

    // Bard Narrative
    writeln!(writer, "## 3. The Scroll of Logic\n").into_diagnostic()?;
    writeln!(
        writer,
        "The English narrative translation of the program's logic.\n"
    )
    .into_diagnostic()?;
    let tale = tell_tale(&program);
    writeln!(writer, "```text\n{}\n```\n", tale.trim()).into_diagnostic()?;

    // Codegen
    writeln!(writer, "## 4. Generated Rust Code\n").into_diagnostic()?;
    writeln!(writer, "The transpiled target code.\n").into_diagnostic()?;
    let rust_code = generate_rust_file(&program);
    writeln!(writer, "```rust\n{}\n```\n", rust_code.trim()).into_diagnostic()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weave_generation() {
        let source = "ξ πέντε ἔστω.";
        let mut buffer = Vec::new();

        run_weave_inner(source, &mut buffer).unwrap();
        let output = String::from_utf8(buffer).unwrap();

        assert!(output.contains("# ΓΛΩΣΣΑ (GLOSSA) Rosetta Stone"));
        assert!(output.contains("## 1. Original Text"));
        assert!(output.contains("## 2. Morphological Assembly"));
        assert!(output.contains("## 3. The Scroll of Logic"));
        assert!(output.contains("## 4. Generated Rust Code"));

        // Ensure specific tool outputs are present
        assert!(output.contains("ξ πέντε ἔστω."));
        assert!(output.contains("BIND"));
        assert!(output.contains("let")); // Using 'let' instead of 'let mut' since bindings aren't mutable by default.
    }
}
