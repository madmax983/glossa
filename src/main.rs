#![deny(unsafe_code)]
//! ΓΛΩΣΣΑ Compiler CLI
//!
//! A compiler for ΓΛΩΣΣΑ - where Ancient Greek morphology encodes programming semantics.

use clap::Parser;
use miette::Result;

use glossa::tools::cli::{Cli, Commands};
use glossa::tools::dictionary::lookup_word;
use glossa::tools::repl::run_repl;
use glossa::tools::runner::{
    bard_file, build_file, check_file, highlight_file, report_file, run_file,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If a file is provided without a subcommand, run it
    if let Some(file) = cli.file {
        return run_file(&file);
    }

    if let Some(command) = cli.command {
        execute_command(command)?;
    } else {
        run_repl()?;
    }

    Ok(())
}

fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::Run { input } => {
            run_file(&input)?;
        }

        Commands::Mentor => {
            #[cfg(feature = "nova")]
            glossa::tools::mentor::run_mentor()?;

            #[cfg(not(feature = "nova"))]
            miette::bail!(
                "The 'mentor' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        Commands::Build { input, output } => {
            build_file(&input, output.as_deref())?;
        }

        Commands::Check { input } => {
            check_file(&input)?;
        }

        Commands::Report { input } => {
            report_file(&input)?;
        }

        Commands::Highlight { input } => {
            highlight_file(&input)?;
        }

        Commands::Bard { input } => {
            bard_file(&input)?;
        }

        Commands::Lookup { word } => {
            lookup_word(&word)?;
        }

        Commands::Test { input } => {
            glossa::tools::tester::run_tests(&input)?;
        }

        Commands::Mosaic { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::mosaic::run_mosaic(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'mosaic' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Map { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::cartographer::run_map(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'map' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Labyrinth { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::labyrinth::run_labyrinth(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'labyrinth' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Weave { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::weave::run_weave(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'weave' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Alchemist { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::alchemist::run_alchemist(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'alchemist' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Papyrus { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::papyrus::run_papyrus(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'papyrus' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Haruspex { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::haruspex::run_haruspex(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'haruspex' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Audit { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::auditor::run_auditor(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'audit' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Catalog => {
            #[cfg(feature = "nova")]
            glossa::tools::catalog::run_catalog()?;

            #[cfg(not(feature = "nova"))]
            miette::bail!(
                "The 'catalog' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        Commands::Gnomon { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::gnomon::run_gnomon(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'gnomon' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Scholar { input } => {
            #[cfg(feature = "nova")]
            glossa::tools::scholar::run_scholar(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'scholar' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Commands::Repl => {
            run_repl()?;
        }
    }
    Ok(())
}
