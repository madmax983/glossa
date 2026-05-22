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

    match cli.command {
        Some(Commands::Run { input }) => {
            run_file(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Mentor) => {
            glossa::tools::mentor::run_mentor()?;
        }

        #[cfg(not(feature = "nova"))]
        Some(Commands::Mentor) => {
            miette::bail!(
                "The 'mentor' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        Some(Commands::Build { input, output }) => {
            build_file(&input, output.as_deref())?;
        }

        Some(Commands::Check { input }) => {
            check_file(&input)?;
        }

        Some(Commands::Report { input }) => {
            report_file(&input)?;
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

        #[cfg(not(feature = "nova"))]
        Some(Commands::Mosaic { .. }) => {
            miette::bail!(
                "The 'mosaic' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        Some(Commands::Map { input }) => {
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

        #[cfg(feature = "nova")]
        Some(Commands::Labyrinth { input }) => {
            glossa::tools::labyrinth::run_labyrinth(&input)?;
        }

        #[cfg(not(feature = "nova"))]
        Some(Commands::Labyrinth { .. }) => {
            miette::bail!(
                "The 'labyrinth' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        #[cfg(feature = "nova")]
        Some(Commands::Weave { input }) => {
            glossa::tools::weave::run_weave(&input)?;
        }

        #[cfg(not(feature = "nova"))]
        Some(Commands::Weave { .. }) => {
            miette::bail!(
                "The 'weave' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        #[cfg(feature = "nova")]
        Some(Commands::Alchemist { input }) => {
            glossa::tools::alchemist::run_alchemist(&input)?;
        }

        #[cfg(not(feature = "nova"))]
        Some(Commands::Alchemist { .. }) => {
            miette::bail!(
                "The 'alchemist' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        #[cfg(feature = "nova")]
        Some(Commands::Papyrus { input }) => {
            glossa::tools::papyrus::run_papyrus(&input)?;
        }

        #[cfg(not(feature = "nova"))]
        Some(Commands::Papyrus { .. }) => {
            miette::bail!(
                "The 'papyrus' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        #[cfg(feature = "nova")]
        Some(Commands::Haruspex { input }) => {
            glossa::tools::haruspex::run_haruspex(&input)?;
        }

        #[cfg(not(feature = "nova"))]
        Some(Commands::Haruspex { .. }) => {
            miette::bail!(
                "The 'haruspex' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        Some(Commands::Audit { input }) => {
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

        #[cfg(feature = "nova")]
        Some(Commands::Catalog) => {
            glossa::tools::catalog::run_catalog()?;
        }

        #[cfg(not(feature = "nova"))]
        Some(Commands::Catalog) => {
            miette::bail!(
                "The 'catalog' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        #[cfg(feature = "nova")]
        Some(Commands::Gnomon { input }) => {
            glossa::tools::gnomon::run_gnomon(&input)?;
        }

        #[cfg(not(feature = "nova"))]
        Some(Commands::Gnomon { .. }) => {
            miette::bail!(
                "The 'gnomon' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        #[cfg(feature = "nova")]
        Some(Commands::Scholar { input }) => {
            glossa::tools::scholar::run_scholar(&input)?;
        }

        #[cfg(not(feature = "nova"))]
        Some(Commands::Scholar { .. }) => {
            miette::bail!(
                "The 'scholar' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}
