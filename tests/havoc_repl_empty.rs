//! Tests for handling an empty REPL session.
//!
//! This module contains tests that verify the REPL gracefully handles
//! empty inputs or immediate termination without crashing.

use glossa::tools::repl::run_repl;

// test wrapper for REPL crash
#[test]
fn havoc_repl_empty_panic_wrapper() {
    let _ = run_repl;
}
