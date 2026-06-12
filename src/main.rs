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

macro_rules! run_nova_cmd {
    ($cmd:path, $name:expr) => {
        {
            #[cfg(feature = "nova")]
            $cmd()?;

            #[cfg(not(feature = "nova"))]
            miette::bail!(
                "The '{}' command is experimental. Recompile glossa with '--features nova' to enable it.",
                $name
            );
        }
    };
    ($cmd:path, $input:expr, $name:expr) => {
        {
            #[cfg(feature = "nova")]
            $cmd($input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = $input;
                miette::bail!(
                    "The '{}' command is experimental. Recompile glossa with '--features nova' to enable it.",
                    $name
                );
            }
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
        Some(Commands::Run { input }) => run_file(&input)?,
        Some(Commands::Build { input, output }) => build_file(&input, output.as_deref())?,
        Some(Commands::Check { input }) => check_file(&input)?,
        Some(Commands::Report { input }) => report_file(&input)?,
        Some(Commands::Highlight { input }) => highlight_file(&input)?,
        Some(Commands::Bard { input }) => bard_file(&input)?,
        Some(Commands::Lookup { word }) => lookup_word(&word)?,
        Some(Commands::Test { input }) => glossa::tools::tester::run_tests(&input)?,

        Some(Commands::Mentor) => run_nova_cmd!(glossa::tools::mentor::run_mentor, "mentor"),
        Some(Commands::Catalog) => run_nova_cmd!(glossa::tools::catalog::run_catalog, "catalog"),

        Some(Commands::Mosaic { input }) => {
            run_nova_cmd!(glossa::tools::mosaic::run_mosaic, &input, "mosaic")
        }
        Some(Commands::Map { input }) => {
            run_nova_cmd!(glossa::tools::cartographer::run_map, &input, "map")
        }
        Some(Commands::Labyrinth { input }) => {
            run_nova_cmd!(glossa::tools::labyrinth::run_labyrinth, &input, "labyrinth")
        }
        Some(Commands::Weave { input }) => {
            run_nova_cmd!(glossa::tools::weave::run_weave, &input, "weave")
        }
        Some(Commands::Alchemist { input }) => {
            run_nova_cmd!(glossa::tools::alchemist::run_alchemist, &input, "alchemist")
        }
        Some(Commands::Papyrus { input }) => {
            run_nova_cmd!(glossa::tools::papyrus::run_papyrus, &input, "papyrus")
        }
        Some(Commands::Haruspex { input }) => {
            run_nova_cmd!(glossa::tools::haruspex::run_haruspex, &input, "haruspex")
        }
        Some(Commands::Audit { input }) => {
            run_nova_cmd!(glossa::tools::auditor::run_auditor, &input, "audit")
        }
        Some(Commands::Gnomon { input }) => {
            run_nova_cmd!(glossa::tools::gnomon::run_gnomon, &input, "gnomon")
        }
        Some(Commands::Scholar { input }) => {
            run_nova_cmd!(glossa::tools::scholar::run_scholar, &input, "scholar")
        }

        Some(Commands::Repl) | None => run_repl()?,
    }

    Ok(())
}
