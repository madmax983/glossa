with open('src/tools/cli.rs', 'r') as f:
    content = f.read()

new_command = """
    /// Export the semantic AST to JSON format (Requires "nova" feature)
    Scribe {
        /// Input file (.γλ)
        input: PathBuf,
    },
"""

content = content.replace("pub enum Commands {\n", "pub enum Commands {\n" + new_command)

with open('src/tools/cli.rs', 'w') as f:
    f.write(content)
