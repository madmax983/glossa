1. **Explore Codebase**: Investigate areas susceptible to crashes based on Havoc's targets: `unwrap`, `expect`, `unsafe`, thread synchronization, deep recursion, and memory exhaustion.
2. **Identify Weakness**: I've successfully identified a vulnerability in the `proc_macro2` dependency used by the `codegen` module. Even though `stacker::maybe_grow` was correctly implemented in parser recursive structures, it wasn't implemented everywhere, and the Semantic AST `AnalyzedExpr`'s derived `Clone` causes a crash when deep recursion happens.
3. **Draft the report**: I've written `HAVOC_ISSUE.md` describing the crash with precise details.
4. **Pre-commit**: Complete pre-commit steps to ensure proper testing, verification, review, and reflection are done.
5. **Submit**: Submit `HAVOC_ISSUE.md` (or directly via PR) to present the wreckage without "fixing" the bug as strictly enforced by Havoc.
