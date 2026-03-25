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
            // coverage:ignore-start
            #[cfg(not(feature = "nova"))]
            {
                miette::bail!(
                    "The 'mentor' command requires the 'nova' experimental feature. Recompile with: cargo build --features nova"
                );
            }
            // coverage:ignore-end
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
            // coverage:ignore-start
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'mosaic' command requires the 'nova' experimental feature. Recompile with: cargo build --features nova"
                );
            }
            // coverage:ignore-end
        }

        Some(Commands::Map { input }) => {
            #[cfg(feature = "nova")]
            {
                glossa::tools::cartographer::run_map(&input)?;
            }
            // coverage:ignore-start
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'map' command requires the 'nova' experimental feature. Recompile with: cargo build --features nova"
                );
            }
            // coverage:ignore-end
        }

        Some(Commands::Weave { input }) => {
            #[cfg(feature = "nova")]
            {
                glossa::tools::weave::run_weave(&input)?;
            }
            // coverage:ignore-start
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'weave' command requires the 'nova' experimental feature. Recompile with: cargo build --features nova"
                );
            }
            // coverage:ignore-end
        }

        Some(Commands::Alchemist { input }) => {
            #[cfg(feature = "nova")]
            {
                glossa::tools::alchemist::run_alchemist(&input)?;
            }
            // coverage:ignore-start
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'alchemist' command requires the 'nova' experimental feature. Recompile with: cargo build --features nova"
                );
            }
            // coverage:ignore-end
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}
