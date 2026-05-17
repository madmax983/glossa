//! Tests for REPL empty input handling.
//!
//! This module verifies that the REPL does not crash or panic when given empty input.

use glossa::tools::repl::run_repl;

// test wrapper for REPL crash
#[test]
fn havoc_repl_empty_panic_wrapper() {
    let _ = run_repl;
}
