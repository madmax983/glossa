//! Intermediate Representation for ΓΛΩΣΣΑ
//!
//! This module defines the High-Level Intermediate Representation (HIR),
//! which serves as the bridge between the semantic analysis phase and the
//! code generation phase.
//!
//! # Why HIR?
//!
//! The semantic analysis produces an `AnalyzedProgram` which is rich in
//! linguistic information (Greek-specific constructs, case usage, etc.).
//! However, for code generation, we need something closer to the target
//! language (Rust).
//!
//! The HIR strips away the Greek-specific details (like case endings and
//! word order) and presents a clean, imperative structure that maps
//! 1-to-1 with Rust constructs.
//!
//! # Pipeline
//!
//! `AnalyzedProgram` → `lower_to_hir()` → `HirProgram` → `generate_rust()`

mod hir;

pub use hir::*;
