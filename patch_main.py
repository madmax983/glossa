with open('src/main.rs', 'r') as f:
    content = f.read()

new_match_arm = """
        Some(Commands::Scribe { input }) => {
            #[cfg(feature = "nova")]
            glossa::tools::scribe::run_scribe(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'scribe' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }
"""

if 'Commands::Scribe' not in content:
    content = content.replace("Some(Commands::Repl) | None => {", new_match_arm + "\n        Some(Commands::Repl) | None => {")
    with open('src/main.rs', 'w') as f:
        f.write(content)
