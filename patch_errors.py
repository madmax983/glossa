with open("src/errors/assembly.rs", "r") as f:
    content = f.read()

if "MissingVerb" not in content:
    print("MissingVerb not in content!")

    new_error = """
    /// Missing verb in a statement
    ///
    /// # Example
    /// `ὁ ἄνθρωπος.` (The man.)
    #[error("Ῥῆμα οὐχ εὑρέθη! Πᾶσα πρότασις ῥῆμα αἰτεῖ.")]
    #[diagnostic(code(glossa::assembly::missing_verb))]
    MissingVerb,
"""
    content = content.replace("DoubleVerb,\n", "DoubleVerb,\n" + new_error)
    with open("src/errors/assembly.rs", "w") as f:
        f.write(content)
