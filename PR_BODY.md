💡 What: Switched `Vec::new()` to `Vec::with_capacity(8)` in `analyze_noun_all` and `analyze_verb_all`.
🎯 Why: These functions are called heavily during morphological disambiguation, and typically return between 1 and 8 analyses for ambiguous endings. Pre-allocating capacity avoids intermediate heap reallocations.
📊 Impact: Reduces intermediate heap allocations for every ambiguous noun/verb analyzed during compilation.
🔬 Measurement: Run `cargo test` and observe identical functionality with fewer allocs under the hood.
