## [Reduction]
**Bloat:** [Silent error-swallowing for Double Subject and Missing Verb via fallthroughs]
**Cut:** [Explicit targeted checks within `src/semantic/assembly/mod.rs` to return `AssemblyError::DoubleSubject` and `AssemblyError::MissingVerb` directly, without touching the complex trait resolution machinery that currently depends on `try_print_default` swallowing undefined variable errors.]
**Saved:** [Prevented a massive refactoring of the parser and trait resolution pipeline while successfully resolving 2 out of 3 major semantic bugs reported by Echo.]
