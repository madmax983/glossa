import sys

def replace_in_file(filepath):
    with open(filepath, 'r') as f:
        content = f.read()

    search_str = """        Some(Commands::Scholar { input }) => {
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

        Some(Commands::Repl) | None => {"""

    replace_str = """        Some(Commands::Scholar { input }) => {
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

        Some(Commands::Sibyl { input }) => {
            #[cfg(feature = "nova")]
            glossa::tools::sibyl::run_sibyl(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'sibyl' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }

        Some(Commands::Repl) | None => {"""

    if search_str in content:
        content = content.replace(search_str, replace_str)
        with open(filepath, 'w') as f:
            f.write(content)
        print("Success")
    else:
        print("Search string not found")

replace_in_file('src/main.rs')
