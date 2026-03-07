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

/// Maximum recursion depth during semantic validation.
/// This prevents stack overflows from evaluating deeply nested expressions in the AST.
pub const MAX_EXPRESSION_DEPTH: usize = 200;

// Semantic limit constants (from assembly/model.rs)
/// Maximum number of adjectives allowed per statement
pub const MAX_ADJECTIVES: usize = 1024;
/// Maximum number of literals (strings/numbers) allowed per statement
pub const MAX_LITERALS: usize = 1024;
/// Maximum number of nominatives (subjects/function names) allowed per statement
pub const MAX_NOMINATIVES: usize = 256;
/// Maximum number of genitives (possessors) allowed per statement
pub const MAX_GENITIVES: usize = 256;
/// Maximum number of array literals allowed per statement
pub const MAX_ARRAYS: usize = 256;
/// Maximum number of index accesses allowed per statement
pub const MAX_INDEX_ACCESSES: usize = 256;
/// Maximum number of property accesses allowed per statement
pub const MAX_PROPERTY_ACCESSES: usize = 256;
/// Maximum number of nested phrases (parenthesized calls) allowed per statement
pub const MAX_NESTED_PHRASES: usize = 256;
/// Maximum number of participles (lambdas) allowed per statement
pub const MAX_PARTICIPLES: usize = 256;
/// Maximum number of unwrap operations allowed per statement
pub const MAX_UNWRAPS: usize = 256;
/// Maximum number of binary operators allowed per statement
pub const MAX_OPERATORS: usize = 256;
/// Maximum number of block expressions allowed per statement
pub const MAX_BLOCKS: usize = 256;

// Semantic limit constants (from control_flow.rs)
/// Maximum depth of nested control flow structures
pub const MAX_CONTROL_FLOW_DEPTH: usize = 100;
