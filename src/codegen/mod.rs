//! Code generation for ΓΛΩΣΣΑ
//!
//! This module handles the translation of the Analyzed Program into executable Rust code.
//!
//! # Why compile to Rust?
//!
//! ΓΛΩΣΣΑ compiles to Rust rather than LLVM IR or Assembly for several strategic reasons:
//!
//! 1. **Safety**: We inherit Rust's borrow checker and memory safety guarantees. If the
//!    ΓΛΩΣΣΑ compiler generates valid Rust, we are guaranteed a memory-safe binary.
//! 2. **Ecosystem**: Users get instant access to `cargo`, `crates.io`, and the entire
//!    Rust ecosystem.
//! 3. **Performance**: `rustc` is an incredibly sophisticated optimizing compiler. We don't
//!    need to write our own optimization passes.
//!
//! # The Mapping Strategy
//!
//! | ΓΛΩΣΣΑ Concept | Rust Equivalent | Notes |
//! |----------------|-----------------|-------|
//! | `εἶδος` (Struct) | `struct` | Generates `#[derive(Clone, Debug, ...)]` |
//! | `χαρακτήρ` (Trait) | `trait` | Maps methods directly |
//! | `ἀποτέλεσμα` (Result) | `Result<T, E>` | Uses standard Rust Result |
//! | `εὑρεθείη` (Option) | `Option<T>` | The "Optative" mood |
//! | `μετά` (Mutable) | `mut` | Variable mutability |
//! | `διὰ ...` (For loop) | `for ... in ...` | Standard iterator loop |
//! | `εἰ ...` (If) | `if ...` | Standard control flow |
//!
//! # Example Translation
//!
//! **ΓΛΩΣΣΑ Source:**
//! ```text
//! ξ πέντε ἔστω.
//! εἰ ξ πέντε ἰσοῦται,
//!     «ἰσχύει» λέγε.
//! ```
//!
//! **Generated Rust:**
//! ```rust,ignore
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

mod rust;
pub mod types;
pub(crate) mod utils;

pub use rust::*;
