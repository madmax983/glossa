## 🚰 Smell
Both `GnomonVisitor` (`src/tools/gnomon.rs`) and `AuditorVisitor` (`src/tools/auditor.rs`) were implemented as object-oriented boilerplate classes just to recursively walk the AST. They held very localized state (`current_depth`, `max_depth` for Gnomon, and basic `FxHashMap`/`FxHashSet` accounting for Auditor).

## ✨ Solution
Flattened both object-oriented single-implementation structs into standalone, procedural `visit_statement` and `visit_expr` functions. The minor state needed by each traversal is simply passed recursively down via mutable references (`&mut usize`, `&mut FxHashMap`, etc.). The CLI entrypoints `run_gnomon` and `run_auditor` were updated to construct the minimal state and trigger the functions. Also handled Rust clippy unused parameter warnings by explicitly removing unused state variables passed down via `visit_expr` in the Auditor, reducing overall coupling.

## 🧹 Benefit
Reduced code bloat and boiler-plate. Pure procedural functions are simpler to read and use, fulfilling Razor's goal to destroy speculative generalities and maintain KISS.

## 🛡️ Verification
- Ran `cargo test` explicitly verifying all recursive logic runs correctly.
- Ran `cargo clippy --all-targets --all-features -- -D warnings`
- Ran `cargo fmt --all`
- Ensured CLI commands `glossa audit` and `glossa gnomon` continue to work properly with test outputs.
