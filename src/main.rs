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

    match cli.command {
        Some(Commands::Run { input }) => run_file(&input)?,
        Some(Commands::Mentor) => run_mentor()?,
        Some(Commands::Build { input, output }) => build_file(&input, output.as_deref())?,
        Some(Commands::Check { input }) => check_file(&input)?,
        Some(Commands::Report { input }) => report_file(&input)?,
        Some(Commands::Highlight { input }) => highlight_file(&input)?,
        Some(Commands::Bard { input }) => bard_file(&input)?,
        Some(Commands::Lookup { word }) => lookup_word(&word)?,
        Some(Commands::Test { input }) => glossa::tools::tester::run_tests(&input)?,
        Some(Commands::Mosaic { input }) => run_mosaic(&input)?,
        Some(Commands::Map { input }) => run_map(&input)?,
        Some(Commands::Labyrinth { input }) => run_labyrinth(&input)?,
        Some(Commands::Weave { input }) => run_weave(&input)?,
        Some(Commands::Alchemist { input }) => run_alchemist(&input)?,
        Some(Commands::Papyrus { input }) => run_papyrus(&input)?,
        Some(Commands::Haruspex { input }) => run_haruspex(&input)?,
        Some(Commands::Audit { input }) => run_auditor(&input)?,
        Some(Commands::Catalog) => run_catalog()?,
        Some(Commands::Gnomon { input: _input }) => run_gnomon(&_input)?,
        Some(Commands::Scholar { input }) => run_scholar(&input)?,
        Some(Commands::Repl) | None => run_repl()?,
    }

    Ok(())
}

#[cfg(feature = "nova")]
fn run_mentor() -> Result<()> {
    glossa::tools::mentor::run_mentor()
}

#[cfg(not(feature = "nova"))]
fn run_mentor() -> Result<()> {
    miette::bail!(
        "The 'mentor' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_mosaic(input: &std::path::Path) -> Result<()> {
    glossa::tools::mosaic::run_mosaic(input)
}

#[cfg(not(feature = "nova"))]
fn run_mosaic(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'mosaic' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_map(input: &std::path::Path) -> Result<()> {
    glossa::tools::cartographer::run_map(input)
}

#[cfg(not(feature = "nova"))]
fn run_map(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'map' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_labyrinth(input: &std::path::Path) -> Result<()> {
    glossa::tools::labyrinth::run_labyrinth(input)
}

#[cfg(not(feature = "nova"))]
fn run_labyrinth(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'labyrinth' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_weave(input: &std::path::Path) -> Result<()> {
    glossa::tools::weave::run_weave(input)
}

#[cfg(not(feature = "nova"))]
fn run_weave(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'weave' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_alchemist(input: &std::path::Path) -> Result<()> {
    glossa::tools::alchemist::run_alchemist(input)
}

#[cfg(not(feature = "nova"))]
fn run_alchemist(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'alchemist' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_papyrus(input: &std::path::Path) -> Result<()> {
    glossa::tools::papyrus::run_papyrus(input)
}

#[cfg(not(feature = "nova"))]
fn run_papyrus(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'papyrus' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_haruspex(input: &std::path::Path) -> Result<()> {
    glossa::tools::haruspex::run_haruspex(input)
}

#[cfg(not(feature = "nova"))]
fn run_haruspex(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'haruspex' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_auditor(input: &std::path::Path) -> Result<()> {
    glossa::tools::auditor::run_auditor(input)
}

#[cfg(not(feature = "nova"))]
fn run_auditor(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'audit' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_catalog() -> Result<()> {
    glossa::tools::catalog::run_catalog()
}

#[cfg(not(feature = "nova"))]
fn run_catalog() -> Result<()> {
    miette::bail!(
        "The 'catalog' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_gnomon(input: &std::path::Path) -> Result<()> {
    glossa::tools::gnomon::run_gnomon(input)
}

#[cfg(not(feature = "nova"))]
fn run_gnomon(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'gnomon' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}

#[cfg(feature = "nova")]
fn run_scholar(input: &std::path::Path) -> Result<()> {
    glossa::tools::scholar::run_scholar(input)
}

#[cfg(not(feature = "nova"))]
fn run_scholar(_input: &std::path::Path) -> Result<()> {
    miette::bail!(
        "The 'scholar' command is experimental. Recompile glossa with '--features nova' to enable it."
    )
}
