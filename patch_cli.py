content = open('src/tools/cli.rs').read()
content = content.replace(
    "pub enum Commands {",
    "pub enum Commands {\n    /// Export Glossa structs to JSON Schema (Requires \"nova\" feature)\n    Ambassador {\n        /// Input file (.γλ)\n        input: std::path::PathBuf,\n    },\n"
)
open('src/tools/cli.rs', 'w').write(content)
