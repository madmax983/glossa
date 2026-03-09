1. We're getting E0603 module is private errors in doctests.
2. The reason is because the doctests are referencing modules that are now `pub(crate) mod`.
3. Doctests are treated like external crates (`tests/`), so they can't access `pub(crate)` directly if it's imported via its full path `use glossa::morphology::conjugation::conjugate;` when `conjugation` is `pub(crate)`.
4. We need to update the doctests to use the publicly exported interfaces from the parent module (e.g. `use glossa::morphology::conjugate;`) or we might need to hide some doctests if they're purely internal, by making them not execute.
5. In Rust, you can't run doctests on non-public items easily unless you do some tricks. Since `conjugate` etc. are exported via `pub use conjugation::*;` in `src/morphology/mod.rs`, the doctests can just import them directly from `glossa::morphology`.
6. Actually, let's fix all the doctests by running `sed` on the source files to adjust their imports inside the doc comments.
