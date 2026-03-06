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

        Some(Commands::Mentor) => {
            #[cfg(feature = "nova")]
            {
                glossa::tools::mentor::run_mentor()?;
            }
            #[cfg(not(feature = "nova"))]
            {
                miette::bail!(
                    "The 'mentor' command requires the experimental 'nova' feature.\nPlease run again with: cargo run --features nova -- mentor"
                );
            }
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

        Some(Commands::Mosaic { input }) => {
            #[cfg(feature = "nova")]
            {
                glossa::tools::mosaic::run_mosaic(&input)?;
            }
            #[cfg(not(feature = "nova"))]
            {
                let _ = input; // Ignore unused variable warning
                miette::bail!(
                    "The 'mosaic' command requires the experimental 'nova' feature.\nPlease run again with: cargo run --features nova -- mosaic <file>"
                );
            }
        }

        Some(Commands::Map { input }) => {
            #[cfg(feature = "nova")]
            {
                glossa::tools::cartographer::run_map(&input)?;
            }
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'map' command requires the experimental 'nova' feature.\nPlease run again with: cargo run --features nova -- map <file>"
                );
            }
        }

        Some(Commands::Weave { input }) => {
            #[cfg(feature = "nova")]
            {
                glossa::tools::weave::run_weave(&input)?;
            }
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'weave' command requires the experimental 'nova' feature.\nPlease run again with: cargo run --features nova -- weave <file>"
                );
            }
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}
