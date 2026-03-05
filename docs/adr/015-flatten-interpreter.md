# 15. Flatten Interpreter Module into Tools

Date: 2026-03-04
Status: Accepted

## Context

The interpreter simulator was previously located in its own directory at `src/experimental/interpreter.rs`. This structure violated the Razor persona's KISS/YAGNI principles by introducing unnecessary nesting for a single-file directory, adding bloat without structural benefit. While the interpreter is experimental, grouping it under a dedicated `experimental/` folder fragmented the tool ecosystem and deepened the module hierarchy unnecessarily.

## Decision

We have moved the interpreter module from `src/experimental/interpreter.rs` to `src/tools/interpreter.rs` and deleted the `src/experimental` directory.

The module remains guarded behind the `#[cfg(feature = "nova")]` feature flag in `src/tools/mod.rs` to signify its experimental, non-production nature without requiring a separate directory.

## Consequences

### Positive
- **Reduced Bloat**: Eliminates the single-file `src/experimental` directory, adhering to the principle of flattened hierarchies.
- **Cohesion**: Groups the interpreter alongside other developer experience tools in the `src/tools/` module, treating it as a standard compiler tool.

### Negative
- **Visibility**: The "experimental" nature of the module is no longer visible from the file path alone; it relies entirely on the `nova` feature flag in `src/tools/mod.rs` to convey its status.
