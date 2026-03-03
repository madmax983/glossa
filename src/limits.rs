//! Centralized compiler limits for ΓΛΩΣΣΑ
//!
//! This module defines limits for recursion depths, collection sizes, and
//! other architectural constraints to prevent DoS attacks, stack overflows,
//! and memory exhaustion.

/// Maximum recursion depth during the parsing phase (CST construction).
/// This is checked via a linear scan before deep recursive descent begins.
pub const MAX_PARSE_DEPTH: usize = 500;

/// Maximum recursion depth during semantic analysis (AST processing).
/// This prevents stack overflows when processing deeply nested phrases or expressions.
pub const MAX_AST_DEPTH: usize = 50;
