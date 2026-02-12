//! ΓΛΩΣΣΑ (GLOSSA) - A compiler where Ancient Greek morphology encodes programming semantics
//!
//! # The Philosophy
//!
//! ΓΛΩΣΣΑ is not just a language with translated keywords. It is an exploration of how
//! natural language grammar—specifically Ancient Greek—can map directly to programming
//! language semantics.
//!
//! * **Case Endings** determine semantic roles (Nominative = Subject, Accusative = Object).
//! * **Verb Aspects** encode execution semantics (Aorist = Immediate, Present = Continuous).
//! * **Grammatical Agreement** serves as the type system.
//!
//! # The Compiler Pipeline
//!
//! The compiler follows a standard multi-pass architecture, but with a unique "Assembler" phase:
//!
//! 1. **Parsing** (`grammar`):
//!    * Uses a PEG grammar (`glossa.pest`) to tokenize the input.
//!    * Normalizes polytonic Greek (with accents/breathings) to monotonic forms.
//!
//! 2. **Morphological Analysis** (`morphology`):
//!    * Analyzes each word to determine its part of speech, case, gender, number, etc.
//!    * Uses a built-in `lexicon` for core vocabulary.
//!    * Identifies participles for lambda formation.
//!
//! 3. **Semantic Assembly** (`semantic`):
//!    * **The Core Innovation**: A slot-based assembler that mimics how the human brain processes Greek.
//!    * Words are routed to "slots" (Subject, Object, Verb) based on case, allowing for free word order.
//!    * Performs agreement checks (Subject-Verb, Adjective-Noun).
//!
//! 4. **Code Generation** (`codegen`):
//!    * Transpiles the Analyzed Program into valid Rust code.
//!    * Uses the `quote` crate to ensure syntactical correctness.
//!
//! # Module Guide
//!
//! * [`ast`]: Abstract Syntax Tree definitions.
//! * [`cli`]: CLI command implementations.
//! * [`codegen`]: Rust code generation logic.
//! * [`errors`]: Greek-native error messages and diagnostics.
//! * [`grammar`]: PEG parser.
//! * [`highlight`]: Semantic syntax highlighting.
//! * [`morphology`]: Word analysis, lexicon, and participle parsing.
//! * [`parser`]: AST Builder and parsing logic.
//! * [`repl`]: Interactive REPL.
//! * [`report`]: Report generation and statistics.
//! * [`semantic`]: The slot-based assembler and semantic analysis.
//! * [`text`]: Text utilities and normalization.

pub mod ast;
pub mod cli;
pub mod codegen;
pub mod errors;
pub mod grammar;
pub mod highlight;
pub mod morphology;
pub mod parser;
pub mod repl;
pub mod report;
pub mod semantic;
pub mod text;
