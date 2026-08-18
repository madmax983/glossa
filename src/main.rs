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

#[cfg(feature = "nova")]
fn execute_nova_command(command: Commands) -> Result<()> {
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
        _ => unreachable!("Only nova commands should be passed to execute_nova_command"),
    }
}

#[cfg(not(feature = "nova"))]
fn execute_nova_command(command: Commands) -> Result<()> {
    // Unpack inputs conditionally if they exist in the command enum variants
    // To avoid unused variable warnings when compiling without "nova" feature
    let command_name = match command {
        Commands::Mentor => "mentor",
        Commands::Mosaic { input } => {
            let _ = input;
            "mosaic"
        }
        Commands::Map { input } => {
            let _ = input;
            "map"
        }
        Commands::Labyrinth { input } => {
            let _ = input;
            "labyrinth"
        }
        Commands::Weave { input } => {
            let _ = input;
            "weave"
        }
        Commands::Alchemist { input } => {
            let _ = input;
            "alchemist"
        }
        Commands::Papyrus { input } => {
            let _ = input;
            "papyrus"
        }
        Commands::Haruspex { input } => {
            let _ = input;
            "haruspex"
        }
        Commands::Audit { input } => {
            let _ = input;
            "audit"
        }
        Commands::Catalog => "catalog",
        Commands::Gnomon { input } => {
            let _ = input;
            "gnomon"
        }
        Commands::Scholar { input } => {
            let _ = input;
            "scholar"
        }
        _ => unreachable!("Only nova commands should be passed to execute_nova_command"),
    };

    miette::bail!(
        "The '{}' command is experimental. Recompile glossa with '--features nova' to enable it.",
        command_name
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
        // All Nova commands
        cmd @ (Commands::Mentor
        | Commands::Mosaic { .. }
        | Commands::Map { .. }
        | Commands::Labyrinth { .. }
        | Commands::Weave { .. }
        | Commands::Alchemist { .. }
        | Commands::Papyrus { .. }
        | Commands::Haruspex { .. }
        | Commands::Audit { .. }
        | Commands::Catalog
        | Commands::Gnomon { .. }
        | Commands::Scholar { .. }) => execute_nova_command(cmd),
    }
}

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

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "nova"))]
    use super::*;
    #[cfg(not(feature = "nova"))]
    use std::path::PathBuf;

    #[test]
    #[cfg(not(feature = "nova"))]
    fn test_execute_nova_command_without_nova_feature() {
        let commands = vec![
            Commands::Mentor,
            Commands::Mosaic {
                input: PathBuf::from("test.γλ"),
            },
            Commands::Map {
                input: PathBuf::from("test.γλ"),
            },
            Commands::Catalog,
        ];

        for cmd in commands {
            let result = execute_nova_command(cmd);
            assert!(result.is_err());
            let err_msg = result.unwrap_err().to_string();
            assert!(err_msg.contains("is experimental"));
            assert!(err_msg.contains("Recompile glossa with '--features nova'"));
        }
    }

    #[test]
    #[cfg(not(feature = "nova"))]
    fn test_execute_command_routes_nova_commands() {
        let cmd = Commands::Mentor;
        let result = execute_command(cmd);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("is experimental"));
    }
}
