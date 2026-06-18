content = open('src/tools/ambassador.rs').read()
content = content.replace(
"""    let source = std::fs::read_to_string(input).map_err(|e| {
        status.error("Σφάλμα ἀρχείου (File Error)");
        miette::miette!("Failed to read file: {}", e)
    })?;""",
"""    let source = match std::fs::read_to_string(input) {
        Ok(s) => s,
        Err(e) => {
            status.error("Σφάλμα ἀρχείου (File Error)");
            return Err(miette::miette!("Failed to read file: {}", e));
        }
    };""")
open('src/tools/ambassador.rs', 'w').write(content)
