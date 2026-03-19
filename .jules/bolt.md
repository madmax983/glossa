**[Replacing .lines().collect() with .lines().peekable()]**
**Learning:** When parsing text output streams (like from `rustc`), avoid collecting `output.lines()` into an intermediate `Vec<&str>` just to iterate using index variables. This creates an unnecessary O(N) heap allocation. Instead, use a `.peekable()` iterator and process the stream in place, manually managing `.next()` and `.peek()` calls. Be careful with mutable references when using iterator `by_ref()` and `peek()` simultaneously within the same block to avoid borrow checker errors (`cannot borrow as mutable more than once at a time`).
**Action:** Use `output.lines().peekable()` and a `while let Some(line) = lines.next()` structure for stream processing instead of `Vec` collection.
**[Slice Refactoring over Vec Collection in AST Parsing]**
**Learning:** Avoid replacing `Vec::new()` with `Vec::with_capacity()` using arbitrary heuristics (e.g., total statement counts). `Vec::new()` is a zero-cost abstraction, whereas eager capacity allocation causes immediate heap allocations, which can lead to performance regressions if the vectors remain empty. Only use `Vec::with_capacity` when the exact required size is known in advance. When parsing or traversing ASTs (e.g., in `semantic/control_flow.rs`), avoid collecting iterator chains into intermediate vectors (e.g., `terms.iter().skip(1).collect::<Vec<_>>()`). Instead, use slice references (e.g., `&terms[1..]`) whenever possible to prevent unnecessary `O(N)` heap allocations.
**Action:** Identify and replace unnecessary `.collect::<Vec<_>>()` calls with slices, and only use `Vec::with_capacity` with an exact calculated size.

**[Optimize string parameter joining in reports]**
**Learning:** Using `.map(|t| t.to_string()).collect::<Vec<_>>().join(", ")` inside loops causes two performance issues: an unnecessary intermediate `Vec` allocation to store string items, and temporary `String` allocations for each item created by `.to_string()`.
**Action:** Instead, iterate manually over the items, use a single pre-allocated (or dynamically growing) `String` buffer, and use `write!(&mut string_buf, "{}", item)` from `std::fmt::Write` to format the objects directly into the buffer, thereby avoiding both the vector heap allocation and the intermediate string creations.

**[Optimize string parameter joining and coverage]**
**Learning:** When replacing `.collect::<Vec<_>>().join(", ")` with manual iteration and `std::fmt::Write`, ensure that test cases actually exercise the multiple-item branches (e.g., `if i > 0`). Otherwise, code coverage gates (like Codecov patch coverage) will fail due to untested conditionals.
**Action:** Add test data (like a function with multiple parameters) when modifying iteration logic to maintain high code coverage.
