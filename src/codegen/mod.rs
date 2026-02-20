//! Code generation for ΓΛΩΣΣΑ
//!
//! This module handles the translation of the Analyzed Program into executable Rust code.
//!
//! # The Strategy: Transpilation
//!
//! ΓΛΩΣΣΑ compiles to Rust rather than LLVM IR or Assembly for several strategic reasons.
//! This approach is often called "transpilation" or "source-to-source compilation".
//!
//! ## 1. Safety First (The "Rust" Shield)
//! By targeting Rust, we inherit the most sophisticated borrow checker and memory safety
//! system in the world. If the ΓΛΩΣΣΑ compiler generates valid Rust code, the resulting
//! binary is guaranteed to be memory-safe (no segfaults, no data races).
//!
//! ## 2. The Ecosystem
//! Users get instant access to `cargo`, `crates.io`, and the entire Rust ecosystem.
//! A ΓΛΩΣΣΑ program is just a Rust program with a Greek syntax.
//!
//! ## 3. Performance
//! `rustc` is an incredibly sophisticated optimizing compiler. We don't need to write
//! our own optimization passes (constant folding, loop unrolling, etc.) because `rustc`
//! does it for us.
//!
//! # The Mapping Strategy
//!
//! The mapping between ΓΛΩΣΣΑ and Rust is direct and intentional.
//!
//! | ΓΛΩΣΣΑ Concept | Rust Equivalent | Notes |
//! |----------------|-----------------|-------|
//! | **Types** | | |
//! | `ἀριθμός` (Number) | `i64` | 64-bit signed integer |
//! | `ὄνομα` (String) | `String` | Heap-allocated UTF-8 string |
//! | `ἀληθές/ψεῦδος` | `bool` | Boolean |
//! | `λίστη` | `Vec<T>` | Dynamic array |
//! | `σύνολον` | `HashSet<T>` | Unique collection |
//! | `χάρτης` | `HashMap<K, V>` | Key-value store |
//! | **Control Flow** | | |
//! | `εἰ ...` (If) | `if ...` | Standard control flow |
//! | `εἰ δὲ μή ...` | `else ...` | Else block |
//! | `ἕως ...` (While) | `while ...` | While loop |
//! | `διὰ ...` (For) | `for ... in ...` | Iterator loop |
//! | `παῦε` (Break) | `break` | Break loop |
//! | `συνέχιζε` (Continue)| `continue` | Continue loop |
//! | **Structures** | | |
//! | `εἶδος` (Struct) | `struct` | Generates `#[derive(Clone, Debug, ...)]` |
//! | `χαρακτήρ` (Trait) | `trait` | Maps methods directly |
//! | `ὁρίζειν` (Define) | `impl` | Implementation block |
//! | **Error Handling** | | |
//! | `ἀποτέλεσμα` | `Result<T, E>` | Uses standard Rust Result |
//! | `εὑρεθείη` | `Option<T>` | The "Optative" mood |
//! | `;` (Propagate) | `?` | The Try operator |
//! | `!` (Unwrap) | `.unwrap()` | Panic on failure |
//!
//! # Example Translation
//!
//! **ΓΛΩΣΣΑ Source:**
//! ```text
//! // Define a struct
//! εἶδος Point ὁρίζειν {
//!     x ἀριθμοῦ.
//!     y ἀριθμοῦ.
//! }.
//!
//! // Check equality
//! ξ πέντε ἔστω.
//! εἰ ξ πέντε ἰσοῦται,
//!     «ἰσχύει» λέγε.
//! ```
//!
//! **Generated Rust:**
//! ```rust,ignore
//! #[derive(Clone, Debug, PartialEq)]
//! struct Point {
//!     x: i64,
//!     y: i64,
//! }
//!
//! fn main() {
//!     let x = 5;
//!     if x == 5 {
//!         println!("ἰσχύει");
//!     }
//! }
//! ```
//!
//! # Architecture
//!
//! The codegen process uses the [`quote`] crate to construct a Rust AST (TokenStream)
//! directly from the Semantic Analysis model. This ensures that the generated code
//! is syntactically valid Rust and avoids "stringly typed" code generation errors.
//!
//! 1. **Semantic Analysis**: Produces an [`AnalyzedProgram`](crate::semantic::AnalyzedProgram).
//! 2. **Code Generation**: [`generate_rust`] takes the program and produces a Rust source string.

#![allow(clippy::needless_doctest_main)]

pub mod expressions;
pub mod statements;
pub mod types;
pub mod utils;

use crate::semantic::{AnalyzedProgram, AnalyzedStatement};
use quote::quote;
use self::statements::{generate_statement, statement_uses_collections};

// Re-exports for backward compatibility / public API
pub use self::types::to_rust_type;
pub use self::utils::{sanitize_name, transliterate};

// ==================================================================================
// RUST CODEGEN
// ==================================================================================

/// Generate Rust code from an Analyzed Program
pub fn generate_rust(program: &AnalyzedProgram) -> String {
    // Check if we need collection imports
    let needs_collections = uses_collections(program);

    // Separate trait defs, struct defs, trait impls, function defs, tests, and main body statements
    let mut trait_defs = Vec::new();
    let mut struct_defs = Vec::new();
    let mut trait_impls = Vec::new();
    let mut fn_defs = Vec::new();
    let mut test_defs = Vec::new();
    let mut main_stmts = Vec::new();

    for stmt in &program.statements {
        match stmt {
            AnalyzedStatement::TraitDefinition { .. } => trait_defs.push(generate_statement(stmt)),
            AnalyzedStatement::TypeDefinition { .. } => struct_defs.push(generate_statement(stmt)),
            AnalyzedStatement::TraitImplementation { .. } => {
                trait_impls.push(generate_statement(stmt))
            }
            AnalyzedStatement::FunctionDef { .. } => fn_defs.push(generate_statement(stmt)),
            AnalyzedStatement::TestDeclaration { .. } => test_defs.push(generate_statement(stmt)),
            _ => main_stmts.push(generate_statement(stmt)),
        }
    }

    // Generate imports if needed
    let imports = if needs_collections {
        quote! { use std::collections::{HashMap, HashSet}; }
    } else {
        quote! {}
    };

    let output = quote! {
        #imports

        #(#trait_defs)*

        #(#struct_defs)*

        #(#trait_impls)*

        #(#fn_defs)*

        fn main() {
            std::panic::set_hook(Box::new(|info| {
                let msg = if let Some(s) = info.payload().downcast_ref::<&str>() {
                    *s
                } else if let Some(s) = info.payload().downcast_ref::<String>() {
                    &s[..]
                } else {
                    "Unknown error"
                };

                let (greek_msg, english_msg) = match msg {
                    "division by zero" | "division by zero or overflow" =>
                        ("Διαίρεσις διὰ μηδενός", "Division by zero"),
                    "arithmetic overflow" | "attempt to add with overflow" | "attempt to subtract with overflow" | "attempt to multiply with overflow" =>
                        ("Ὑπερχείλισις ἀριθμοῦ", "Arithmetic overflow"),
                    "index out of bounds" | "bounds check" =>
                        ("Δείκτης ἐκτὸς ὁρίων", "Index out of bounds"),
                    s if s.starts_with("index out of bounds") =>
                        ("Δείκτης ἐκτὸς ὁρίων", "Index out of bounds"),
                    _ => ("Σφάλμα ἐκτελέσεως", msg),
                };

                eprintln!("\x1b[31m✕ {}: {}\x1b[0m", greek_msg, english_msg);

                if let Some(location) = info.location() {
                    eprintln!("\x1b[90m  (at line {})\x1b[0m", location.line());
                }
            }));

            #(#main_stmts)*
        }

        #(#test_defs)*
    };

    output.to_string()
}

/// Check if the program uses collection types (HashMap, HashSet)
fn uses_collections(program: &AnalyzedProgram) -> bool {
    for stmt in &program.statements {
        if statement_uses_collections(stmt) {
            return true;
        }
    }
    false
}

/// Generate a complete Rust file with proper formatting
pub fn generate_rust_file(program: &AnalyzedProgram) -> String {
    let code = generate_rust(program);

    // Add a comment header
    format!(
        "// Generated by ΓΛΩΣΣΑ compiler\n// Αὐτόματος κῶδιξ ἀπὸ ΓΛΩΣΣΑ\n#![allow(non_snake_case, unused_imports)]\n\n{}",
        code
    )
}

// Re-export generate_statement_code for testing/usage
pub use self::statements::generate_statement_code;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;
    use crate::semantic::analyze_program;

    // --- Rust Codegen Tests ---

    fn compile(source: &str) -> String {
        let ast = parse(source).unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        generate_rust(&analyzed)
    }

    #[test]
    fn test_generate_hello() {
        let code = compile("«χαῖρε» λέγε.");
        // quote! generates `println !` with space
        assert!(code.contains("println"), "Expected println in: {}", code);
        assert!(code.contains("χαῖρε"));
    }

    #[test]
    fn test_generate_binding() {
        let code = compile("ξ πέντε ἔστω.");
        // Variables are now prefixed with g_ and hex encoded
        assert!(code.contains("let g__u3be_"));
        assert!(code.contains("5"));
    }

    #[test]
    fn test_generate_number() {
        let code = compile("42 λέγε.");
        assert!(code.contains("println"), "Expected println in: {}", code);
        assert!(code.contains("42"));
    }

    #[test]
    fn test_generate_full_program() {
        let code = compile("ξ πέντε ἔστω. ξ λέγε.");
        // Variables are now prefixed with g_ and hex encoded
        assert!(
            code.contains("let g__u3be_ = 5"),
            "Expected binding in: {}",
            code
        );
        assert!(code.contains("println"), "Expected println in: {}", code);
    }

    #[test]
    fn test_generate_statement_code() {
        let ast = parse("«χαῖρε» λέγε.").unwrap();
        let analyzed = analyze_program(&ast).unwrap();
        let stmt = &analyzed.statements[0];
        let code = generate_statement_code(stmt);
        assert!(code.contains("println"));
        assert!(code.contains("χαῖρε"));
    }
}
