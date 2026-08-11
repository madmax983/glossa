#![deny(unsafe_code)]
//! ΓΛΩΣΣΑ Compiler CLI
//!
//! A compiler for ΓΛΩΣΣΑ - where Ancient Greek morphology encodes programming semantics.

use clap::Parser;
use miette::Result;

use glossa::tools::cli::{Cli, Commands};
use glossa::tools::dictionary::lookup_word;
#[cfg(not(test))]
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

    execute_command(cli.command)
}

fn execute_command(command: Option<Commands>) -> Result<()> {
    match command {
        Some(Commands::Run { input }) => run_file(&input)?,
        Some(Commands::Build { input, output }) => build_file(&input, output.as_deref())?,
        Some(Commands::Check { input }) => check_file(&input)?,
        Some(Commands::Report { input }) => report_file(&input)?,
        Some(Commands::Highlight { input }) => highlight_file(&input)?,
        Some(Commands::Bard { input }) => bard_file(&input)?,
        Some(Commands::Lookup { word }) => lookup_word(&word)?,
        Some(Commands::Test { input }) => glossa::tools::tester::run_tests(&input)?,
        Some(
            cmd @ Commands::Mentor
            | cmd @ Commands::Mosaic { .. }
            | cmd @ Commands::Map { .. }
            | cmd @ Commands::Labyrinth { .. }
            | cmd @ Commands::Weave { .. }
            | cmd @ Commands::Alchemist { .. }
            | cmd @ Commands::Papyrus { .. }
            | cmd @ Commands::Haruspex { .. }
            | cmd @ Commands::Audit { .. }
            | cmd @ Commands::Catalog
            | cmd @ Commands::Gnomon { .. }
            | cmd @ Commands::Scholar { .. },
        ) => execute_experimental_command(cmd)?,
        Some(Commands::Repl) | None => {
            #[cfg(test)]
            {
                return Ok(());
            }
            #[cfg(not(test))]
            run_repl()?;
        }
    }

    Ok(())
}

#[cfg(feature = "nova")]
fn execute_experimental_command(command: Commands) -> Result<()> {
    match command {
        Commands::Mentor => glossa::tools::mentor::run_mentor()?,
        Commands::Mosaic { input } => glossa::tools::mosaic::run_mosaic(&input)?,
        Commands::Map { input } => glossa::tools::cartographer::run_map(&input)?,
        Commands::Labyrinth { input } => glossa::tools::labyrinth::run_labyrinth(&input)?,
        Commands::Weave { input } => glossa::tools::weave::run_weave(&input)?,
        Commands::Alchemist { input } => glossa::tools::alchemist::run_alchemist(&input)?,
        Commands::Papyrus { input } => glossa::tools::papyrus::run_papyrus(&input)?,
        Commands::Haruspex { input } => glossa::tools::haruspex::run_haruspex(&input)?,
        Commands::Audit { input } => glossa::tools::auditor::run_auditor(&input)?,
        Commands::Catalog => glossa::tools::catalog::run_catalog()?,
        Commands::Gnomon { input } => glossa::tools::gnomon::run_gnomon(&input)?,
        Commands::Scholar { input } => glossa::tools::scholar::run_scholar(&input)?,
        _ => unreachable!(),
    }
    Ok(())
}

#[cfg(not(feature = "nova"))]
fn execute_experimental_command(command: Commands) -> Result<()> {
    let name = match command {
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
        _ => unreachable!(),
    };
    miette::bail!(
        "The '{}' command is experimental. Recompile glossa with '--features nova' to enable it.",
        name
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_execute_command_error() {
        let cmd = Commands::Run {
            input: PathBuf::from("does_not_exist.glossa"),
        };
        let result = execute_command(Some(cmd));
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_command_variants() {
        let dummy = PathBuf::from("dummy.glossa");

        let _ = execute_command(Some(Commands::Check {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Report {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Highlight {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Bard {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Test {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Build {
            input: dummy.clone(),
            output: None,
        }));

        let _ = execute_command(Some(Commands::Lookup {
            word: "λόγος".to_string(),
        }));
        let _ = execute_command(Some(Commands::Mentor));
        let _ = execute_command(Some(Commands::Catalog));

        let _ = execute_command(Some(Commands::Mosaic {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Map {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Labyrinth {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Weave {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Alchemist {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Papyrus {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Haruspex {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Audit {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Gnomon {
            input: dummy.clone(),
        }));
        let _ = execute_command(Some(Commands::Scholar {
            input: dummy.clone(),
        }));

        let _ = execute_command(Some(Commands::Repl));
        let _ = execute_command(None);
    }
}
