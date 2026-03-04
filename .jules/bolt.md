**[Optimized Internal Symbol Table Lookups]**
**Learning:** The `std::collections::HashMap` in Rust defaults to SipHash, a cryptographically secure hash function. This is often overkill for internal compiler symbol tables where keys are small, interned strings (e.g., `SmolStr`), and where HashDoS attacks are not a risk.
**Action:** Always prefer `rustc_hash::FxHashMap` for compiler-internal maps (such as scope resolution) mapping strings to AST nodes or types. It is much faster and deterministic.
