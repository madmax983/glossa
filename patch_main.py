content = open('src/main.rs').read()
content = content.replace(
    "    match cli.command {",
    "    match cli.command {\n        Some(Commands::Ambassador { input }) => {\n            #[cfg(feature = \"nova\")]\n            glossa::tools::ambassador::run_ambassador(&input)?;\n\n            #[cfg(not(feature = \"nova\"))]\n            {\n                let _ = input;\n                miette::bail!(\n                    \"The 'ambassador' command is experimental. Recompile glossa with '--features nova' to enable it.\"\n                );\n            }\n        }\n"
)
open('src/main.rs', 'w').write(content)
