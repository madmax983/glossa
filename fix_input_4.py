import re

with open("src/main.rs", "r") as f:
    content = f.read()

old = """        Some(Commands::Gnomon { input }) => {
            #[cfg(feature = "nova")]
            glossa::tools::gnomon::run_gnomon(&input)?;

            #[cfg(not(feature = "nova"))]
            miette::bail!(
                "The 'gnomon' command is experimental. Recompile glossa with '--features nova' to enable it."
            );
        }"""

new = """        Some(Commands::Gnomon { input }) => {
            #[cfg(feature = "nova")]
            glossa::tools::gnomon::run_gnomon(&input)?;

            #[cfg(not(feature = "nova"))]
            {
                let _ = input;
                miette::bail!(
                    "The 'gnomon' command is experimental. Recompile glossa with '--features nova' to enable it."
                );
            }
        }"""

content = content.replace("Some(Commands::Gnomon { input: _ }) => {", "Some(Commands::Gnomon { input }) => {")
content = content.replace("        Some(Commands::Gnomon { input }) => {\n            #[cfg(feature = \"nova\")]\n            glossa::tools::gnomon::run_gnomon(&input)?;\n\n            #[cfg(not(feature = \"nova\"))]\n            miette::bail!(\n                \"The 'gnomon' command is experimental. Recompile glossa with '--features nova' to enable it.\"\n            );\n        }", new)
with open("src/main.rs", "w") as f:
    f.write(content)
