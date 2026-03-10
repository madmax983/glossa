//! ΓΛΩΣΣΑ Compiler CLI
//!
//! A compiler for ΓΛΩΣΣΑ - where Ancient Greek morphology encodes programming semantics.

use clap::Parser;
use miette::Result;

use glossa::tools::cli::{Cli, Commands};
use glossa::tools::dictionary::lookup_word;
use glossa::tools::repl::run_repl;
use glossa::tools::runner::{bard_file, build_file, check_file, highlight_file, run_file};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If a file is provided without a subcommand, run it
    if let Some(file) = cli.file {
        return run_file(&file);
    }

    match cli.command {
        Some(Commands::Run { input }) => {
            run_file(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Mentor) => {
            glossa::tools::mentor::run_mentor()?;
        }

        Some(Commands::Build { input, output }) => {
            build_file(&input, output.as_deref())?;
        }

        Some(Commands::Check { input }) => {
            check_file(&input)?;
        }

        Some(Commands::Highlight { input }) => {
            highlight_file(&input)?;
        }

        Some(Commands::Bard { input }) => {
            bard_file(&input)?;
        }

        Some(Commands::Lookup { word }) => {
            lookup_word(&word)?;
        }

        Some(Commands::Test { input }) => {
            glossa::tools::tester::run_tests(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Mosaic { input }) => {
            glossa::tools::mosaic::run_mosaic(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Map { input }) => {
            glossa::tools::cartographer::run_map(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Weave { input }) => {
            glossa::tools::weave::run_weave(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Trace { input }) => {
            let source = std::fs::read_to_string(&input)
                .map_err(|e| miette::miette!("Failed to read {}: {}", input.display(), e))?;
            let ast = glossa::parser::parse(&source)
                .map_err(|e| miette::miette!("Parse error: {}", e))?;
            let program = glossa::semantic::analyze_program(&ast)
                .map_err(|e| miette::miette!("Semantic error: {}", e))?;
            glossa::tools::tracer::run_trace(&program)?;
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}
