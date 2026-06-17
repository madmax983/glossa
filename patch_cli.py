with open("src/tools/cli.rs", "r") as f:
    content = f.read()

new_cmd = """    /// Generate an API documentation Markdown file (Requires "nova" feature)
    Scholar {
        /// Input file (.γλ)
        input: PathBuf,
    },

    /// Generate TypeScript interface declarations from types (Requires "nova" feature)
    Sibyl {
        /// Input file (.γλ)
        input: PathBuf,
    },
}"""
content = content.replace("    /// Generate an API documentation Markdown file (Requires \"nova\" feature)\n    Scholar {\n        /// Input file (.γλ)\n        input: PathBuf,\n    },\n}", new_cmd)
with open("src/tools/cli.rs", "w") as f:
    f.write(content)
