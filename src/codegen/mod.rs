//! Code generation for ΓΛΩΣΣΑ
//!
//! This module handles the translation of the Analyzed Program into executable Rust code.
//!
//! # Architecture
//!
//! The codegen process uses the `quote` crate to construct a Rust AST (TokenStream)
//! directly from the Semantic Analysis model. This ensures that the generated code
//! is syntactically valid Rust.
//!
//! 1. **Semantic Analysis**: Produces an `AnalyzedProgram`.
//! 2. **Code Generation**: `codegen::generate_rust` takes the `AnalyzedProgram` and
//!    produces a Rust source string.

pub mod mappings;
mod rust;

pub use rust::*;
