#![deny(unsafe_code)]
//! ΓΛΩΣΣΑ Compiler CLI
//!
//! A compiler for ΓΛΩΣΣΑ - where Ancient Greek morphology encodes programming semantics.

use clap::Parser;
use miette::Result;

use glossa::tools::cli::{Cli, Commands};
use glossa::tools::repl::run_repl;
use glossa::tools::runner::run_file;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If a file is provided without a subcommand, run it
    if let Some(file) = cli.file {
        return run_file(&file);
    }

    execute_command(cli.command)
}

fn execute_command(command: Option<Commands>) -> Result<()> {
    match command {
        Some(Commands::Run { input }) => glossa::tools::runner::run_file(&input),
        Some(Commands::Build { input, output }) => {
            glossa::tools::runner::build_file(&input, output.as_deref())
        }
        Some(Commands::Check { input }) => glossa::tools::runner::check_file(&input),
        Some(Commands::Report { input }) => glossa::tools::runner::report_file(&input),
        Some(Commands::Highlight { input }) => glossa::tools::runner::highlight_file(&input),
        Some(Commands::Bard { input }) => glossa::tools::runner::bard_file(&input),
        Some(Commands::Lookup { word }) => glossa::tools::dictionary::lookup_word(&word),
        Some(Commands::Test { input }) => glossa::tools::tester::run_tests(&input),
        Some(Commands::Repl) | None => run_repl(),
        Some(cmd @ Commands::Mentor)
        | Some(cmd @ Commands::Mosaic { .. })
        | Some(cmd @ Commands::Map { .. })
        | Some(cmd @ Commands::Labyrinth { .. })
        | Some(cmd @ Commands::Weave { .. })
        | Some(cmd @ Commands::Alchemist { .. })
        | Some(cmd @ Commands::Papyrus { .. })
        | Some(cmd @ Commands::Haruspex { .. })
        | Some(cmd @ Commands::Audit { .. })
        | Some(cmd @ Commands::Catalog)
        | Some(cmd @ Commands::Gnomon { .. })
        | Some(cmd @ Commands::Scholar { .. }) => execute_nova_command(cmd),
    }
}

fn execute_nova_command(command: Commands) -> Result<()> {
    #[cfg(feature = "nova")]
    match command {
        Commands::Mentor => glossa::tools::mentor::run_mentor(),
        Commands::Mosaic { input } => glossa::tools::mosaic::run_mosaic(&input),
        Commands::Map { input } => glossa::tools::cartographer::run_map(&input),
        Commands::Labyrinth { input } => glossa::tools::labyrinth::run_labyrinth(&input),
        Commands::Weave { input } => glossa::tools::weave::run_weave(&input),
        Commands::Alchemist { input } => glossa::tools::alchemist::run_alchemist(&input),
        Commands::Papyrus { input } => glossa::tools::papyrus::run_papyrus(&input),
        Commands::Haruspex { input } => glossa::tools::haruspex::run_haruspex(&input),
        Commands::Audit { input } => glossa::tools::auditor::run_auditor(&input),
        Commands::Catalog => glossa::tools::catalog::run_catalog(),
        Commands::Gnomon { input } => glossa::tools::gnomon::run_gnomon(&input),
        Commands::Scholar { input } => glossa::tools::scholar::run_scholar(&input),
        _ => unreachable!(), // The outer match guarantees we only pass Nova commands here.
    }

    #[cfg(not(feature = "nova"))]
    {
        let cmd_name = match command {
            Commands::Mentor => "mentor",
            Commands::Mosaic { .. } => "mosaic",
            Commands::Map { .. } => "map",
            Commands::Labyrinth { .. } => "labyrinth",
            Commands::Weave { .. } => "weave",
            Commands::Alchemist { .. } => "alchemist",
            Commands::Papyrus { .. } => "papyrus",
            Commands::Haruspex { .. } => "haruspex",
            Commands::Audit { .. } => "audit",
            Commands::Catalog => "catalog",
            Commands::Gnomon { .. } => "gnomon",
            Commands::Scholar { .. } => "scholar",
            _ => unreachable!(), // The outer match guarantees we only pass Nova commands here.
        };
        miette::bail!(
            "The '{}' command is experimental. Recompile glossa with '--features nova' to enable it.",
            cmd_name
        );
    }
}
