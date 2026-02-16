use clap::Parser;
use glossa::experimental::muse::Inspiration;
use glossa::tools::cli::{Cli, Commands};

#[test]
fn test_cli_muse_hero_parsing() {
    let args = vec!["glossa", "muse", "hero"];
    let cli = Cli::try_parse_from(args).expect("Failed to parse muse hero");

    if let Some(Commands::Muse { inspiration }) = cli.command {
        assert!(matches!(inspiration, Inspiration::Hero));
    } else {
        panic!("Expected Muse command");
    }
}

#[test]
fn test_cli_muse_myth_parsing() {
    let args = vec!["glossa", "muse", "myth"];
    let cli = Cli::try_parse_from(args).expect("Failed to parse muse myth");

    if let Some(Commands::Muse { inspiration }) = cli.command {
        assert!(matches!(inspiration, Inspiration::Myth));
    } else {
        panic!("Expected Muse command");
    }
}

#[test]
fn test_cli_muse_chorus_parsing() {
    let args = vec!["glossa", "muse", "chorus"];
    let cli = Cli::try_parse_from(args).expect("Failed to parse muse chorus");

    if let Some(Commands::Muse { inspiration }) = cli.command {
        assert!(matches!(inspiration, Inspiration::Chorus));
    } else {
        panic!("Expected Muse command");
    }
}

#[test]
fn test_cli_muse_epic_parsing() {
    let args = vec!["glossa", "muse", "epic"];
    let cli = Cli::try_parse_from(args).expect("Failed to parse muse epic");

    if let Some(Commands::Muse { inspiration }) = cli.command {
        assert!(matches!(inspiration, Inspiration::Epic));
    } else {
        panic!("Expected Muse command");
    }
}

#[test]
fn test_inspiration_debug() {
    let insp = Inspiration::Hero;
    let debug_str = format!("{:?}", insp);
    assert_eq!(debug_str, "Hero");
}

#[test]
fn test_inspiration_clone() {
    let insp = Inspiration::Chorus;
    let cloned = insp.clone();
    assert!(matches!(cloned, Inspiration::Chorus));
}

#[test]
fn test_cli_muse_invalid_variant() {
    let args = vec!["glossa", "muse", "tragedy"];
    let result = Cli::try_parse_from(args);
    assert!(result.is_err());
}
