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
//! | ΓΛΩΣΣΑ Concept | Rust Equivalent |
//! |----------------|-----------------|
//! | `εἶδος` (Struct) | `struct` |
//! | `χαρακτήρ` (Trait) | `trait` |
//! | `ἐνδέχεται` (Result) | `Result<T, E>` |
//! | `ἴσως` (Option) | `Option<T>` |
//! | `μετά` (Mutable) | `mut` |
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
