//! Code generation for ΓΛΩΣΣΑ
//!
//! This module handles the translation of the High-Level Intermediate Representation (HIR)
//! into executable Rust code.
//!
//! # Architecture
//!
//! The codegen process uses the `quote` crate to construct a Rust AST (TokenStream)
//! from the HIR. This ensures that the generated code is syntactically valid Rust.
//!
//! 1. **Lowering**: The semantic analyzer produces an `AnalyzedProgram`.
//! 2. **HIR Conversion**: `ir::lower_to_hir` converts this to `HirProgram`.
//! 3. **Code Generation**: `codegen::generate_rust` takes the `HirProgram` and
//!    produces a Rust source string.
//!
//! # Example
//!
//! ```ignore
//! // HIR
//! let stmt = HirStatement::Print { args: vec![HirExpr::StringLit("Hello".into())] };
//!
//! // Generated Rust
//! println!("{}", "Hello");
//! ```

mod rust;

pub use rust::*;
