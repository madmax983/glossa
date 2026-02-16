use clap::ValueEnum;

/// The type of inspiration to seek from the Muse.
#[derive(ValueEnum, Clone, Debug)]
pub enum Inspiration {
    /// A simple "Hello World" style program (The Hero's Journey).
    Hero,
    /// A program demonstrating struct definition and usage (The Myth).
    Myth,
    /// A program demonstrating loops and control flow (The Chorus).
    Chorus,
    /// A program demonstrating functions and complexity (The Epic).
    Epic,
}

/// Invokes the Muse to generate a code snippet based on the inspiration type.
pub fn invoke_muse(inspiration: Inspiration) -> String {
    match inspiration {
        Inspiration::Hero => r#"// The Hero's Journey (ὁ τοῦ Ἥρωος βίος)
// A simple program to declare a variable and print it.

// "x" (variable) "five" (value) "let be" (assignment).
// ξ πέντε ἔστω.
ξ πέντε ἔστω.

// "x" (variable) "say" (print).
// ξ λέγε.
ξ λέγε.
"#
        .to_string(),
        Inspiration::Myth => r#"// The Myth (ὁ Μῦθος)
// Defining a Form (struct) and creating an instance.

// Define a type: God
// εἶδος Θεός ὁρίζειν { ... }.
εἶδος Θεός ὁρίζειν {
    ὄνομα ὀνόματος.
    δύναμις ἀριθμοῦ.
}.

// Create an instance: Zeus
// Ζεύς νέον Θεός ... ἔστω.
// "Zeus" (Name) "new" (Keyword) "God" (Type)
Ζεύς νέον Θεός
    «Κρονίδης»
    100
ἔστω.

// Print the name
// Ζεύς ὄνομα λέγε.
Ζεύς ὄνομα λέγε.
"#
        .to_string(),
        Inspiration::Chorus => r#"// The Chorus (ὁ Χορός)
// A loop example (Repetition).

// Loop from 0 to 5 (exclusive)
// ἀπὸ μηδενὸς μέχρι πέντε, ...
ἀπὸ μηδενὸς μέχρι πέντε, ι λέγε.
"#
        .to_string(),
        Inspiration::Epic => r#"// The Epic (τὸ Ἔπος)
// A function definition and usage.

// Define a function: square (τετράγωνον)
// Parameter: x (χ) of type Number (ἀριθμοῦ - genitive)
// Body: return x * x (product - γινόμενον)
// Note: · (middle dot) separates signature from body.
τετράγωνον ὁρίζειν τῷ χ ἀριθμοῦ· δός χ χ γινόμενον.

// Call the function
// ξ τετράγωνον πέντε ἔστω.
ξ τετράγωνον πέντε ἔστω.

// Print the result
ξ λέγε.
"#
        .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_invoke_hero() {
        let code = invoke_muse(Inspiration::Hero);
        assert!(code.contains("ξ πέντε ἔστω."));
        assert!(
            parse(&code).is_ok(),
            "Hero snippet should be valid Glossa code"
        );
    }

    #[test]
    fn test_invoke_myth() {
        let code = invoke_muse(Inspiration::Myth);
        assert!(code.contains("εἶδος Θεός"));
        assert!(
            parse(&code).is_ok(),
            "Myth snippet should be valid Glossa code"
        );
    }

    #[test]
    fn test_invoke_chorus() {
        let code = invoke_muse(Inspiration::Chorus);
        assert!(code.contains("ἀπὸ μηδενὸς"));
        assert!(
            parse(&code).is_ok(),
            "Chorus snippet should be valid Glossa code"
        );
    }

    #[test]
    fn test_invoke_epic() {
        let code = invoke_muse(Inspiration::Epic);
        assert!(code.contains("τετράγωνον ὁρίζειν"));
        assert!(
            parse(&code).is_ok(),
            "Epic snippet should be valid Glossa code"
        );
    }
}
