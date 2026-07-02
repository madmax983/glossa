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

macro_rules! nova_command {
    ($name:expr, $module:path) => {
        #[cfg(feature = "nova")]
        $module()?;

        #[cfg(not(feature = "nova"))]
        miette::bail!(
            "The '{}' command is experimental. Recompile glossa with '--features nova' to enable it.",
            $name
        );
    };
    ($name:expr, $module:path, $input:expr) => {
        #[cfg(feature = "nova")]
        $module(&$input)?;

        #[cfg(not(feature = "nova"))]
        {
            let _ = $input;
            miette::bail!(
                "The '{}' command is experimental. Recompile glossa with '--features nova' to enable it.",
                $name
            );
        }
    };
}

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
            nova_command!("mentor", glossa::tools::mentor::run_mentor);
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

        Some(Commands::Mosaic { input }) => {
            nova_command!("mosaic", glossa::tools::mosaic::run_mosaic, input);
        }

        Some(Commands::Map { input }) => {
            nova_command!("map", glossa::tools::cartographer::run_map, input);
        }

        Some(Commands::Labyrinth { input }) => {
            nova_command!("labyrinth", glossa::tools::labyrinth::run_labyrinth, input);
        }

        Some(Commands::Weave { input }) => {
            nova_command!("weave", glossa::tools::weave::run_weave, input);
        }

        Some(Commands::Alchemist { input }) => {
            nova_command!("alchemist", glossa::tools::alchemist::run_alchemist, input);
        }

        Some(Commands::Papyrus { input }) => {
            nova_command!("papyrus", glossa::tools::papyrus::run_papyrus, input);
        }

        Some(Commands::Haruspex { input }) => {
            nova_command!("haruspex", glossa::tools::haruspex::run_haruspex, input);
        }

        Some(Commands::Audit { input }) => {
            nova_command!("audit", glossa::tools::auditor::run_auditor, input);
        }

        Some(Commands::Catalog) => {
            nova_command!("catalog", glossa::tools::catalog::run_catalog);
        }

        Some(Commands::Gnomon { input }) => {
            nova_command!("gnomon", glossa::tools::gnomon::run_gnomon, input);
        }

        Some(Commands::Scholar { input }) => {
            nova_command!("scholar", glossa::tools::scholar::run_scholar, input);
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}
