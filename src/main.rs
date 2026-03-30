//! ΓΛΩΣΣΑ Compiler CLI
//!
//! A compiler for ΓΛΩΣΣΑ - where Ancient Greek morphology encodes programming semantics.

use clap::Parser;
use miette::Result;

use glossa::tools::lookup_word;
use glossa::tools::run_repl;
use glossa::tools::{Cli, Commands};
use glossa::tools::{bard_file, build_file, check_file, highlight_file, run_file};

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
            glossa::tools::run_mentor()?;
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
            glossa::tools::run_tests(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Mosaic { input }) => {
            glossa::tools::run_mosaic(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Map { input }) => {
            glossa::tools::run_map(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Weave { input }) => {
            glossa::tools::run_weave(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Alchemist { input }) => {
            glossa::tools::run_alchemist(&input)?;
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}
