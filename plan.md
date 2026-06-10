1.  **Refactor `analyze_verb_all` and `analyze_noun_all` to pre-allocate capacity**
    -   In `src/morphology/conjugation.rs`, modify `analyze_verb_all` to use `Vec::with_capacity(4)` instead of `Vec::new()`.
    -   In `src/morphology/declension.rs`, modify `analyze_noun_all` to use `Vec::with_capacity(4)` instead of `Vec::new()`.
    -   This prevents reallocations on the hot path for morphology matching, which is called frequently during lexical analysis.

2.  **Add documentation explaining the optimization**
    -   Add doc comments with `/// ⚡ Bolt Optimization:` explaining that we pre-allocate to prevent reallocation during morphology analysis.

3.  **Run tests and linters**
    -   Execute `cargo clippy --all-targets --all-features -- -D warnings` and `cargo test` and `cargo fmt --all`.

4.  **Complete pre commit steps**
    -   Ensure proper testing, verification, review, and reflection are done.

5.  **Submit PR**
    -   Create a pull request titled "⚡ Bolt: Pre-allocate capacity in morphology analysis" following the PR format.
