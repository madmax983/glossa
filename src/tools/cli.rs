//! Command-line interface definition

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "glossa")]
#[command(about = "ΓΛΩΣΣΑ - Ancient Greek morphology as programming semantics")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Run a .γλ file directly (default action)
    #[arg(value_name = "FILE")]
    pub file: Option<PathBuf>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Commands {
    /// Run a .γλ file (default)
    Run {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Compile a .γλ file to Rust source
    Build {
        /// Input file (.γλ)
        input: PathBuf,

        /// Output file (.rs)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Check a .γλ file without running
    Check {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Highlight a .γλ file with semantic colors
    Highlight {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Start the interactive REPL
    Repl,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_default_run() {
        let args = vec!["glossa", "file.gl"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.file, Some(PathBuf::from("file.gl")));
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_subcommand_run() {
        let args = vec!["glossa", "run", "file.gl"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Run { input }) => assert_eq!(input, PathBuf::from("file.gl")),
            _ => panic!("Expected Run subcommand"),
        }
    }

    #[test]
    fn test_cli_subcommand_build() {
        let args = vec!["glossa", "build", "file.gl"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Build { input, output }) => {
                assert_eq!(input, PathBuf::from("file.gl"));
                assert_eq!(output, None);
            }
            _ => panic!("Expected Build subcommand"),
        }
    }

    #[test]
    fn test_cli_subcommand_build_output() {
        let args = vec!["glossa", "build", "file.gl", "-o", "out.rs"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Build { input, output }) => {
                assert_eq!(input, PathBuf::from("file.gl"));
                assert_eq!(output, Some(PathBuf::from("out.rs")));
            }
            _ => panic!("Expected Build subcommand"),
        }
    }

    #[test]
    fn test_cli_subcommand_check() {
        let args = vec!["glossa", "check", "file.gl"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Check { input }) => assert_eq!(input, PathBuf::from("file.gl")),
            _ => panic!("Expected Check subcommand"),
        }
    }

    #[test]
    fn test_cli_subcommand_highlight() {
        let args = vec!["glossa", "highlight", "file.gl"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Highlight { input }) => assert_eq!(input, PathBuf::from("file.gl")),
            _ => panic!("Expected Highlight subcommand"),
        }
    }

    #[test]
    fn test_cli_subcommand_repl() {
        let args = vec!["glossa", "repl"];
        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::Repl) => {}
            _ => panic!("Expected Repl subcommand"),
        }
    }

    #[test]
    fn test_cli_no_args() {
        let args = vec!["glossa"];
        let cli = Cli::try_parse_from(args).unwrap();
        assert!(cli.file.is_none());
        assert!(cli.command.is_none()); // Implicitly runs REPL in main.rs logic, but struct is None
    }
}
