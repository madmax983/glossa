#![cfg(feature = "nova")]

use clap::Parser;
use glossa::tools::cli::{Cli, Commands};
use std::path::PathBuf;

#[test]
fn test_mosaic_cli_parsing() {
    // Simulate command line arguments: "glossa mosaic test.gl"
    let args = vec!["glossa", "mosaic", "test.gl"];
    let cli = Cli::try_parse_from(args).expect("Failed to parse args");

    if let Some(Commands::Mosaic { input }) = cli.command {
        assert_eq!(input, PathBuf::from("test.gl"));
    } else {
        panic!("Expected Mosaic command");
    }
}
