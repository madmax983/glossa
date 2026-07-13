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

    let Some(command) = cli.command else {
        return run_repl();
    };

    execute_command(command)
}

#[cfg(not(feature = "nova"))]
fn nova_disabled_error(cmd_name: &str) -> Result<()> {
    miette::bail!(
        "The '{cmd_name}' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

fn execute_command(command: Commands) -> Result<()> {
    match command {
        Commands::Run { input } => run_file(&input),
        Commands::Build { input, output } => build_file(&input, output.as_deref()),
        Commands::Check { input } => check_file(&input),
        Commands::Report { input } => report_file(&input),
        Commands::Highlight { input } => highlight_file(&input),
        Commands::Bard { input } => bard_file(&input),
        Commands::Lookup { word } => lookup_word(&word),
        Commands::Test { input } => glossa::tools::tester::run_tests(&input),
        Commands::Repl => run_repl(),

        Commands::Mentor => {
            #[cfg(feature = "nova")]
            return glossa::tools::mentor::run_mentor();
            #[cfg(not(feature = "nova"))]
            return nova_disabled_error("mentor");
        }
        Commands::Mosaic { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::mosaic::run_mosaic(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("mosaic");
            }
        }
        Commands::Map { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::cartographer::run_map(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("map");
            }
        }
        Commands::Labyrinth { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::labyrinth::run_labyrinth(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("labyrinth");
            }
        }
        Commands::Weave { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::weave::run_weave(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("weave");
            }
        }
        Commands::Alchemist { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::alchemist::run_alchemist(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("alchemist");
            }
        }
        Commands::Papyrus { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::papyrus::run_papyrus(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("papyrus");
            }
        }
        Commands::Haruspex { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::haruspex::run_haruspex(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("haruspex");
            }
        }
        Commands::Audit { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::auditor::run_auditor(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("audit");
            }
        }
        Commands::Catalog => {
            #[cfg(feature = "nova")]
            return glossa::tools::catalog::run_catalog();
            #[cfg(not(feature = "nova"))]
            return nova_disabled_error("catalog");
        }
        Commands::Gnomon { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::gnomon::run_gnomon(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("gnomon");
            }
        }
        Commands::Scholar { input } => {
            #[cfg(feature = "nova")]
            return glossa::tools::scholar::run_scholar(&input);
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                return nova_disabled_error("scholar");
            }
        }
    }
}
