//! ΓΛΩΣΣΑ Compiler CLI
//!
//! A compiler for ΓΛΩΣΣΑ - where Ancient Greek morphology encodes programming semantics.

use clap::Parser;
use miette::Result;

#[cfg(not(feature = "nova"))]
use crossterm::style::Stylize;

use glossa::tools::cli::{Cli, Commands};
use glossa::tools::dictionary::lookup_word;
use glossa::tools::repl::run_repl;
use glossa::tools::runner::{bard_file, build_file, check_file, highlight_file, run_file};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // If a file is provided without a subcommand, run it
    if let Some(file) = cli.file {
        return run_file(&file);
    }

    match cli.command {
        Some(Commands::Run { input }) => {
            run_file(&input)?;
        }

        Some(Commands::Mentor) => {
            #[cfg(feature = "nova")]
            glossa::tools::mentor::run_mentor()?;
            #[cfg(not(feature = "nova"))]
            print_nova_required("Μέντωρ (Mentor)", "mentor");
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

        #[allow(unused_variables)]
        Some(Commands::Mosaic { input }) => {
            #[cfg(feature = "nova")]
            glossa::tools::mosaic::run_mosaic(&input)?;
            #[cfg(not(feature = "nova"))]
            print_nova_required("Ψηφιδωτόν (Mosaic)", "mosaic");
        }

        #[allow(unused_variables)]
        Some(Commands::Map { input }) => {
            #[cfg(feature = "nova")]
            glossa::tools::cartographer::run_map(&input)?;
            #[cfg(not(feature = "nova"))]
            print_nova_required("Χαρτογράφησις (Map)", "map");
        }

        #[allow(unused_variables)]
        Some(Commands::Weave { input }) => {
            #[cfg(feature = "nova")]
            glossa::tools::weave::run_weave(&input)?;
            #[cfg(not(feature = "nova"))]
            print_nova_required("Ὕφανσις (Weave)", "weave");
        }

        #[allow(unused_variables)]
        Some(Commands::Alchemist { input }) => {
            #[cfg(feature = "nova")]
            glossa::tools::alchemist::run_alchemist(&input)?;
            #[cfg(not(feature = "nova"))]
            print_nova_required("Χημεία (Alchemist)", "alchemist");
        }

        Some(Commands::Repl) | None => {
            run_repl()?;
        }
    }

    Ok(())
}

#[cfg(not(feature = "nova"))]
fn print_nova_required(tool_name: &str, cli_command: &str) {
    println!();
    println!(
        "   {}",
        format!("Γ Λ Ω Σ Σ Α   {}", tool_name).bold().cyan()
    );
    println!("   {}", "Experimental Tool Disabled".italic().dim());
    println!();
    println!("   {}", "✕ Feature Not Enabled".red().bold());
    println!(
        "   The `{}` tool is experimental and requires the `nova` feature.",
        cli_command
    );
    println!();
    println!("   {}", "To use it, recompile glossa with:".dim());
    println!(
        "   {}",
        format!("cargo run --features nova -- {} [args...]", cli_command).yellow()
    );
    println!();
}
