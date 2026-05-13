1. Review the memory on `The Blob` files. It mentions converting large files like `src/semantic/conversion.rs` and `src/morphology/lexicon.rs` into directories to improve cohesion and reduce file size.
2. We moved `lexicon.rs` to `src/morphology/lexicon/` and split its `LEXICON` initialization into `data.rs` and its tests into `tests.rs`.
3. We moved `conversion.rs` to `src/semantic/conversion/` and moved its tests into `tests.rs`.
4. We verified that tests still pass and the architecture is preserved.
5. All tests run perfectly and no dead code was found.
6. Commit the changes following Atlas's PR format, updating the `.jules/atlas.md` log.
