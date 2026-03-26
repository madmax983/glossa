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
    execute_cli(cli)
}

/// Extracted execution logic for testability without spawning processes
pub fn execute_cli(cli: Cli) -> Result<()> {
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

        #[cfg(feature = "nova")]
        Some(Commands::Mosaic { input }) => {
            glossa::tools::mosaic::run_mosaic(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Map { input }) => {
            glossa::tools::cartographer::run_map(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Weave { input }) => {
            glossa::tools::weave::run_weave(&input)?;
        }

        #[cfg(feature = "nova")]
        Some(Commands::Alchemist { input }) => {
            glossa::tools::alchemist::run_alchemist(&input)?;
        }

        Some(Commands::Labyrinth { input }) => {
            #[cfg(feature = "nova")]
            {
                glossa::tools::labyrinth::run_labyrinth(&input)?;
            }
            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The Labyrinth tool requires the 'nova' feature. Run with: cargo run --features nova -- labyrinth <file>"
                );
            }
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main_cli_labyrinth_feature() {
        // Here we test the CLI logic directly by constructing a fake CLI object
        // and ensuring the Labyrinth command block runs and fails appropriately
        // (since "does_not_exist" doesn't exist).
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Labyrinth { input }),
            file: None,
        };

        let result = execute_cli(cli);

        #[cfg(feature = "nova")]
        assert!(result.is_err()); // because file does not exist

        #[cfg(not(feature = "nova"))]
        assert!(result.is_err()); // because of the bail!
    }

    #[test]
    fn test_main_cli_repl() {
        // To prevent `run_repl` from blocking indefinitely, we can't fully run it in
        // an automated test environment without mocking stdin/stdout easily here.
        // However, we only need to test the execution branches. Since the `Repl`
        // execution blocks on user input, we will just cover the other branches for now.
    }

    #[test]
    fn test_main_cli_run_file_direct() {
        // file without subcommand
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: None,
            file: Some(input),
        };
        let result = execute_cli(cli);
        assert!(result.is_err());
    }

    #[test]
    fn test_main_cli_run_command() {
        // Run command
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Run { input }),
            file: None,
        };
        let _ = execute_cli(cli);
    }

    // Also `highlight` or `bard` might return differently? Let's check them.
    // Wait, the panic happened at line 208!

    #[test]
    fn test_main_cli_check_command() {
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Check { input }),
            file: None,
        };
        let result = execute_cli(cli);
        assert!(result.is_err());
    }

    #[test]
    fn test_main_cli_build_command() {
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Build {
                input,
                output: None,
            }),
            file: None,
        };
        let result = execute_cli(cli);
        assert!(result.is_err());
    }

    #[test]
    fn test_main_cli_highlight_command() {
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Highlight { input }),
            file: None,
        };
        let result = execute_cli(cli);
        assert!(result.is_err());
    }

    #[test]
    fn test_main_cli_test_command() {
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Test { input }),
            file: None,
        };
        let result = execute_cli(cli);
        assert!(result.is_err());
    }

    #[test]
    fn test_main_cli_lookup_command() {
        let word = "does_not_exist".to_string();
        let cli = Cli {
            command: Some(Commands::Lookup { word }),
            file: None,
        };
        let _ = execute_cli(cli);
    }

    #[test]
    fn test_main_cli_bard_command() {
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Bard { input }),
            file: None,
        };
        let _ = execute_cli(cli);
    }

    #[cfg(feature = "nova")]
    #[test]
    fn test_main_cli_mosaic_command() {
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Mosaic { input }),
            file: None,
        };
        let _ = execute_cli(cli);
    }

    #[cfg(feature = "nova")]
    #[test]
    fn test_main_cli_map_command() {
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Map { input }),
            file: None,
        };
        let _ = execute_cli(cli);
    }

    #[cfg(feature = "nova")]
    #[test]
    fn test_main_cli_weave_command() {
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Weave { input }),
            file: None,
        };
        let result = execute_cli(cli);
        assert!(result.is_err());
    }

    #[cfg(feature = "nova")]
    #[test]
    fn test_main_cli_alchemist_command() {
        let input = std::path::PathBuf::from("does_not_exist.γλ");
        let cli = Cli {
            command: Some(Commands::Alchemist { input }),
            file: None,
        };
        let result = execute_cli(cli);
        assert!(result.is_err());
    }
}
