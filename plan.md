1. *Create `src/morphology/lexicon/` directory*
2. *Extract `LexiconEntry` and related DTOs, as well as `LEXICON` LazyLock and `lookup`/`entries` into `src/morphology/lexicon/data.rs`*
3. *Extract the helper functions (e.g. `is_verb`, `is_binding_verb`, etc.) into `src/morphology/lexicon/mod.rs`, which also re-exports items from `data.rs`.*
4. *Update `src/morphology/mod.rs` to use `pub(crate) mod lexicon;`.*
5. *Update imports across the codebase where `crate::morphology::lexicon::*` was used if necessary.*
6. *Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.*
7. *Submit the change.*
