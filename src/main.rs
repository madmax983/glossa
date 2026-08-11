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
        Some(Commands::Run { input }) => {
            run_file(&input)?;
        }

        Some(Commands::Mentor) => {
            #[cfg(feature = "nova")]
            glossa::tools::mentor::run_mentor()?;

            #[cfg(not(feature = "nova"))]
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

        Some(Commands::Mosaic { input }) => {
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

        Some(Commands::Labyrinth { input }) => {
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

        Some(Commands::Weave { input }) => {
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

        Some(Commands::Alchemist { input }) => {
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

        Some(Commands::Papyrus { input }) => {
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

        Some(Commands::Haruspex { input }) => {
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

        Some(Commands::Catalog) => {
            #[cfg(feature = "nova")]
            glossa::tools::catalog::run_catalog()?;

            #[cfg(not(feature = "nova"))]
            miette::bail!(
                "The 'catalog' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }

        Some(Commands::Gnomon { input }) => {
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

        Some(Commands::Scholar { input }) => {
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
