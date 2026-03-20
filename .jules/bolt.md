**[Replacing .lines().collect() with .lines().peekable()]**
**Learning:** When parsing text output streams (like from `rustc`), avoid collecting `output.lines()` into an intermediate `Vec<&str>` just to iterate using index variables. This creates an unnecessary O(N) heap allocation. Instead, use a `.peekable()` iterator and process the stream in place, manually managing `.next()` and `.peek()` calls. Be careful with mutable references when using iterator `by_ref()` and `peek()` simultaneously within the same block to avoid borrow checker errors (`cannot borrow as mutable more than once at a time`).
**Action:** Use `output.lines().peekable()` and a `while let Some(line) = lines.next()` structure for stream processing instead of `Vec` collection.
**[Slice Refactoring over Vec Collection in AST Parsing]**
**Learning:** Avoid replacing `Vec::new()` with `Vec::with_capacity()` using arbitrary heuristics (e.g., total statement counts). `Vec::new()` is a zero-cost abstraction, whereas eager capacity allocation causes immediate heap allocations, which can lead to performance regressions if the vectors remain empty. Only use `Vec::with_capacity` when the exact required size is known in advance. When parsing or traversing ASTs (e.g., in `semantic/control_flow.rs`), avoid collecting iterator chains into intermediate vectors (e.g., `terms.iter().skip(1).collect::<Vec<_>>()`). Instead, use slice references (e.g., `&terms[1..]`) whenever possible to prevent unnecessary `O(N)` heap allocations.
**Action:** Identify and replace unnecessary `.collect::<Vec<_>>()` calls with slices, and only use `Vec::with_capacity` with an exact calculated size.

**Cow Optimization for Heavy AST Structs**
**Learning:** Returning a large parsed AST struct like `AssembledStatement` (which contains multiple internal `Vec`s) by value from helper functions causes expensive clones on hot paths like variable binding.
**Action:** Use `std::borrow::Cow<'a, AssembledStatement>` to return a borrowed reference for the happy path and only allocate `Cow::Owned` when modification (like swapping subject/object) is strictly necessary.

**Codecov Patch Gate Drops After Formatting**
**Learning:** Running `cargo fmt` can expand untested, single-line fallback or error branches (like swapping subject/object logic) into multi-line statements. This artificially increases the number of "uncovered" lines in the diff, which can cause the strict `codecov/patch` gate (target 92.94%) to fail even if logic wasn't changed.
**Action:** Use `cargo llvm-cov --show-missing-lines` locally to identify exactly which new lines lack coverage, and write unit tests to exercise those specific edge cases to restore the coverage percentage before pushing.

**Codecov Patch Gates and Manual AST Construction**
**Learning:** When trying to test specific logic branches (like `AssembledStatement` fallbacks in `resolve_binding_target`) to satisfy strict >92% Codecov patch gates, it is often simpler and far less brittle to manually construct the required AST/Semantic structs (`Constituent`, `AssembledStatement`, etc.) and pass them directly to the helper function in a unit test, rather than trying to reverse-engineer a raw ancient Greek string that parses and semantically evaluates perfectly into that precise edge case.
**Action:** For edge-case branch coverage, write targeted unit tests that manually instantiate the data structures needed to trigger the specific `if` statement logic.
## [Refactored intermediate array allocations]
**Learning:** Refactoring uses of `.collect::<Vec<_>>().join(", ")` in string formatting to manual loops calling `write!(f, ...)` or `push_str` prevents unnecessary heap allocations when traversing structures like ASTs. While `.join()` is ergonomic, `.with_capacity()` combined with `for i in 0..items.len()` checks is the preferred zero-cost approach.
**Action:** Replace `.join()` inside formatters entirely.
