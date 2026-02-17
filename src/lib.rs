//! ΓΛΩΣΣΑ (GLOSSA) - A compiler where Ancient Greek morphology encodes programming semantics
//!
//! # The Philosophy
//!
//! ΓΛΩΣΣΑ is not just a language with translated keywords. It is an exploration of how
//! natural language grammar—specifically Ancient Greek—can map directly to programming
//! language semantics.
//!
//! In most programming languages, **position** determines meaning: `func(a, b)` is different from `func(b, a)`.
//! In Ancient Greek, **morphology** (word endings) determines meaning.
//!
//! * **Nominative Case** (-ος, -η, -α) marks the **Subject** (Agent).
//! * **Accusative Case** (-ον, -ην, -αν) marks the **Object** (Patient).
//! * **Verb Aspects** encode execution semantics (Aorist = Immediate, Present = Continuous).
//! * **Grammatical Agreement** serves as the type system.
//!
//! This allows for **Free Word Order**:
//!
//! ```glossa
//! ὁ ἄνθρωπος τὸν λόγον λέγει.  // The man says the word.
//! τὸν λόγον λέγει ὁ ἄνθρωπος.  // The word says the man.
//! ```
//!
//! # The Compiler Pipeline
//!
//! The compiler follows a standard multi-pass architecture, but with a unique "Assembler" phase:
//!
//! 1. **Parsing** ([`grammar`]):
//!    * Uses a PEG grammar to tokenize the input.
//!    * Normalizes polytonic Greek (with accents/breathings) to monotonic forms using [`text`].
//!
//! 2. **Morphological Analysis** ([`morphology`]):
//!    * Analyzes each word to determine its part of speech, case, gender, number, etc.
//!    * Uses a built-in lexicon for core vocabulary.
//!    * Identifies participles for lambda formation.
//!
//! 3. **Semantic Assembly** ([`semantic`]):
//!    * **The Core Innovation**: A slot-based assembler that mimics how the human brain processes Greek.
//!    * Words are routed to "slots" (Subject, Object, Verb) based on case.
//!    * Performs agreement checks (Subject-Verb, Adjective-Noun).
//!
//! 4. **Code Generation** ([`codegen`]):
//!    * Transpiles the Analyzed Program into valid Rust code.
//!    * Uses the `quote` crate to ensure syntactical correctness.
//!
//! # Quick Start: The Hero's Journey
//!
//! Here is a simple program that defines a user struct and greets them.
//!
//! ```
//! // Define a type (struct)
//! // εἶδος Χρήστης ὁρίζειν {
//! //    ὄνομα ὀνόματος.      // field: String
//! //    ἡλικία ἀριθμοῦ.   // field: i64
//! // }.
//!
//! // Create a new user instance
//! // "user" (nominative) "new" (adjective) "User" (type) ...
//! // χρήστης νέον Χρήστης
//! //    «Σωκράτης»
//! //    70
//! // ἔστω.
//!
//! // Access property and print
//! // "of the user" (genitive) "name" (nominative) "say" (verb)
//! // χρήστου ὄνομα λέγε.
//! ```
//!
//! # Module Guide
//!
//! * [`ast`]: **The Skeleton** - Abstract Syntax Tree definitions that preserve the original Greek text.
//! * [`codegen`]: **The Translator** - Logic that turns Greek semantics into Rust code.
//! * [`errors`]: **The Oracle** - Greek-native error messages and diagnostics using `miette`.
//! * [`grammar`]: **The Gatekeeper** - PEG parser that defines the valid syntax.
//! * [`highlight`]: **The Scribe** - Semantic syntax highlighting for the CLI.
//! * [`morphology`]: **The Analyst** - Word analysis, lexicon lookup, and participle parsing.
//! * [`parser`]: **The Builder** - Constructs the AST from the raw parse tree.
//! * [`report`]: **The Chronicler** - Report generation and statistics.
//! * [`semantic`]: **The Assembler** - The slot-based engine that assembles sentences from words.
//! * [`text`]: **The Sizer** - Text utilities and normalization (polytonic -> monotonic).

pub mod ast;
pub mod codegen;
pub mod errors;
pub mod grammar;
pub mod morphology;
pub mod parser;
pub mod report;
pub mod semantic;
pub mod text;
pub mod tools;

pub use tools::highlight;
