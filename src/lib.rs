//! ΓΛΩΣΣΑ - A compiler where Ancient Greek morphology encodes programming semantics
//!
//! Case endings determine semantic roles, verb aspects encode execution semantics,
//! and grammatical agreement serves as type checking.

pub mod grammar;
pub mod ast;
pub mod morphology;
pub mod semantic;
pub mod ir;
pub mod codegen;
pub mod errors;
