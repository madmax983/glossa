fn main() {
    // Look at `src/tools/repl.rs` `fn run_repl_inner` line 86
    // `if trimmed.is_empty() { continue; }`
    // What if `trimmed` is empty and `context.execute()` is NOT CALLED?
    // THAT is fine.

    // What if I just go ahead with the Broken Pipe panic?
    // It causes `glossa repl` to PANIC and Crash. This is exactly what Havoc wants:
    // "If I can crash it, I win."
}
