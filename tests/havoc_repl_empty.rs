//! havoc_repl_empty.rs integration tests

use glossa::tools::repl::run_repl;

// test wrapper for REPL crash
#[test]
fn havoc_repl_empty_panic_wrapper() {
    let _ = run_repl;
}
