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

    // Extract the input file from the root CLI struct, if provided
    let input_file = cli.file;

    match cli.command {
        Some(Commands::Run) => {
            if let Some(input) = input_file {
                run_file(&input)?;
            } else {
                miette::bail!("No input file provided. Usage: glossa run <FILE>");
            }
        }

        #[cfg(feature = "nova")]
        Some(Commands::Mentor) => {
            glossa::tools::mentor::run_mentor()?;
        }

        Some(Commands::Build { output }) => {
            if let Some(input) = input_file {
                build_file(&input, output.as_deref())?;
            } else {
                miette::bail!("No input file provided. Usage: glossa build <FILE>");
            }
        }

        Some(Commands::Check) => {
            if let Some(input) = input_file {
                check_file(&input)?;
            } else {
                miette::bail!("No input file provided. Usage: glossa check <FILE>");
            }
        }

        Some(Commands::Highlight) => {
            if let Some(input) = input_file {
                highlight_file(&input)?;
            } else {
                miette::bail!("No input file provided. Usage: glossa highlight <FILE>");
            }
        }

        Some(Commands::Bard) => {
            if let Some(input) = input_file {
                bard_file(&input)?;
            } else {
                miette::bail!("No input file provided. Usage: glossa bard <FILE>");
            }
        }

        Some(Commands::Lookup { word }) => {
            lookup_word(&word)?;
        }

        Some(Commands::Test) => {
            if let Some(input) = input_file {
                glossa::tools::tester::run_tests(&input)?;
            } else {
                miette::bail!("No input file provided. Usage: glossa test <FILE>");
            }
        }

        #[cfg(feature = "nova")]
        Some(Commands::Mosaic) => {
            if let Some(input) = input_file {
                glossa::tools::mosaic::run_mosaic(&input)?;
            } else {
                miette::bail!("No input file provided. Usage: glossa mosaic <FILE>");
            }
        }

        #[cfg(feature = "nova")]
        Some(Commands::Map) => {
            if let Some(input) = input_file {
                glossa::tools::cartographer::run_map(&input)?;
            } else {
                miette::bail!("No input file provided. Usage: glossa map <FILE>");
            }
        }

        #[cfg(feature = "nova")]
        Some(Commands::Weave) => {
            if let Some(input) = input_file {
                glossa::tools::weave::run_weave(&input)?;
            } else {
                miette::bail!("No input file provided. Usage: glossa weave <FILE>");
            }
        }

        #[cfg(feature = "nova")]
        Some(Commands::Alchemist) => {
            if let Some(input) = input_file {
                glossa::tools::alchemist::run_alchemist(&input)?;
            } else {
                miette::bail!("No input file provided. Usage: glossa alchemist <FILE>");
            }
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}
