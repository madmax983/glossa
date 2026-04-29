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
**[String Formatting Optimization in Cartographer]**
**Learning:** Using `.collect::<Vec<String>>().join(", ")` inside loops creates unnecessary intermediate heap allocations for the `Vec` and the intermediate formatted strings.
**Action:** Replace `.collect::<Vec<String>>().join(", ")` with iterative formatting directly into a `String` buffer using `std::fmt::Write`, applying `.push_str(", ")` between elements to achieve zero intermediate allocations.
**[Extract Dependencies Buffer Optimization]**
**Learning:** Recursively creating and extending `Vec<String>` allocations for dependency extraction is inefficient.
**Action:** Refactored `extract_dependencies` to pass a `&mut Vec<String>` buffer down the recursive call tree to eliminate all intermediate collections.
**[Closure Codegen Parameters Iteration]**
**Learning:** When using `quote!` to generate code dynamically in `proc-macro2` or `syn`, it is unnecessary to `.collect::<Vec<_>>()` mapped iterators of identifiers into an intermediate allocation. The `quote!` macro's `#(#iter),*` repetition syntax natively accepts any iterator (that implements `Clone` if used multiple times or passed downwards).
**Action:** Replace `let idents: Vec<_> = params.iter().map(...).collect();` with `let idents = params.iter().map(...);` and ensure helper functions accept `impl Iterator<Item = Ident> + Clone` to maintain zero-cost iterator chains without heap allocations.
**[Quote Macro Iterators]**
**Learning:** `quote!` repetition syntax `#(#var)*` evaluates `var` by reference, meaning `&var` must implement `IntoIterator`. While `&Vec<T>` implements it, `&impl Iterator` does not, because advancing an iterator requires mutable access. Therefore, `.collect::<Vec<_>>()` is actually required before using variables in `quote!` repetitions, and converting it to a lazy iterator will cause a compile-time error.
**Action:** Do not try to remove `.collect::<Vec<_>>()` before `quote!` repetitions. Look for other targets.
**[HashMap to FxHashMap in Internal Environment]**
**Learning:** For internal interpreter environments that do not ingest arbitrary user string keys in a hash-collision scenario, the standard library `HashMap` (using SipHash) introduces unnecessary cryptographic hashing overhead. Switching to `rustc_hash::FxHashMap` provides a measurable speedup for variable resolution and lookups without introducing new dependencies, as it is often already present in compiler toolchains.
**Action:** Replace `HashMap` with `FxHashMap` in internal state environments or symbol tables where HashDoS is not a threat model.
**[Buffer Reuse Optimization in Loop]**\n**Learning:** Creating a new  inside a tight loop creates an unnecessary heap allocation on every iteration. This is a common performance pitfall in graph or dependency extraction algorithms. \n**Action:** Declare a single mutable  (e.g., ) outside the loop, pass it by mutable reference  into inner functions, and call  at the start of each loop iteration to reuse the same memory allocation without reallocating.
**[Buffer Reuse Optimization in Loop]**
**Learning:** Creating a new `Vec::new()` inside a tight loop creates an unnecessary heap allocation on every iteration. This is a common performance pitfall in graph or dependency extraction algorithms.
**Action:** Declare a single mutable `Vec` (e.g., `let mut buffer = Vec::new();`) outside the loop, pass it by mutable reference `&mut buffer` into inner functions, and call `buffer.clear()` at the start of each loop iteration to reuse the same memory allocation without reallocating.
**[Replace HashSet with FxHashSet in cartographer]**
**Learning:** Standard HashSet is secure but slow due to SipHash. Since the keys here are just short predictable string representations within compiler phases and are not exposed to hash DOS attacks from users, using FxHashSet avoids significant hashing overhead.
**Action:** Always prefer FxHashSet over HashSet for short internal keys when not exposed to untrusted inputs.

**[String Formatting Optimization in Dictionary]**
**Learning:** Using `.collect::<Vec<String>>().join(", ")` to concatenate simple strings is inefficient and causes unnecessary heap allocations.
**Action:** Replace `.collect::<Vec<String>>().join(", ")` with a single pre-allocated `String` buffer (or simply reuse an existing one) and use `std::fmt::Write` trait with `write!` or `push_str` to format elements directly into the buffer, eliminating intermediate collections.
**[Optimizing Scholar Docs Generator String Formats]
**Learning:** Found an unnecessary intermediate heap allocation and `Vec` creation in `src/tools/scholar.rs` resulting from `.map(|t| format!("{}", t)).collect::<Vec<_>>().join(", ")`, alongside many intermediate `format!()` strings being pushed to the buffer.
**Action:** Replaced `md.push_str(&format!(...))` and `.collect::<Vec<_>>().join(...)` with direct `std::fmt::Write` macro usage (`write!`, `writeln!`) directly into a pre-allocated `String::with_capacity(4096)` buffer.
**[Removing unnecessary format! for integers]
**Learning:** Cell::new takes T: ToString. Using format!("{}", num) creates an unnecessary String allocation and formatting overhead compared to passing the integer directly which uses fast itoa conversion.
**Action:** Pass integers directly to Cell::new instead of formatting them first.
**Optimization of string join patterns**
**Learning:** Removing intermediate heap allocations for string concatenations using string formatting in a pre-allocated `String` with `write!` eliminates memory overhead when formatting output strings. However, `[T]::join` on array slice already allocates a single `String` directly and avoiding it doesn't help. We should only avoid `.collect::<Vec<_>>().join(" ")`.
**Action:** Replace `format!(..., parts.join(" "))` with a `String::with_capacity` buffer and `write!` macro directly, avoiding intermediate `Vec` allocations.

**[HashMap to FxHashMap in tools/catalog.rs]**
**Learning:** `std::collections::HashMap` introduces unnecessary hashing overhead for internal keys (`PartOfSpeech` enum variants). Switching to `rustc_hash::FxHashMap` is a safe, zero-cost optimization since the map is entirely internal and does not process unvalidated string inputs that could be vulnerable to HashDoS.
**Action:** Replaced `std::collections::HashMap` with `rustc_hash::FxHashMap` in `src/tools/catalog.rs` for `entries_by_pos`.
**[Optimizing String Conversions and Format Macros]**
**Learning:** Calling `.to_string()` on slices or `&str` references directly causes heap allocations and trips `clippy::str_to_string` lints. Further, utilizing `to_string()` on elements already implementing `Display` inside formatting macros (like `format!()` or inside `Cell::new`) trips `clippy::to_string_in_format_args` and generates unneeded temporary string allocations.
**Action:** Replace `.to_string()` with `String::from()` or `.to_owned()` on string literals and slices to accurately convey intent. Remove `.to_string()` when passing values to `Cell::new` or within `format!` and simply pass the `Display`-implementing variables directly.
**[Vec Allocations Optimization]
**Learning:** Chained  calls inside mapping closures or aggregating elements into intermediate vectors can lead to unnecessary heap allocations. Using  informed by sequence lengths (e.g.  and combined literal/phrase lengths) along with  entirely eliminates these reallocations.
**Action:** Always pre-calculate capacity limits when lengths are known and use  instead of relying on default  behavior, particularly inside parsing or semantic phases which recurse deeply.

**[Vec Allocations Optimization]**
**Learning:** Chained `.collect()` calls inside mapping closures or aggregating elements into intermediate vectors can lead to unnecessary heap allocations. Using `Vec::with_capacity()` informed by sequence lengths (e.g. `block_stmts.len()` and combined literal/phrase lengths) along with `.extend()` entirely eliminates these reallocations.
**Action:** Always pre-calculate capacity limits when lengths are known and use `.extend()` instead of relying on default `.collect()` behavior, particularly inside parsing or semantic phases which recurse deeply.
