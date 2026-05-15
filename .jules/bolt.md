**[Optimizing Collections in iterators]**
**Learning:** Checking for `.is_empty()` after `.collect::<Vec<_>>()` forces an unnecessary heap allocation. We can check if an iterator is empty dynamically using `.peekable()`. Additionally, if strings from `&'a str` need to be converted to `Cow<'static, str>`, we must use `.to_string()` as `.into()` or `Cow::Borrowed` will result in static lifetime compilation errors.
**Action:** Replace `Vec<_> = iter.collect()` with `.peekable()` for emptiness checking whenever possible, and be mindful of `Cow` lifetimes.

**[Optimizing AST Node Cloning with `std::mem::replace`]**
**Learning:** During iterative AST tree building (e.g., when wrapping an expression in successive `MethodCall`s like `.iter().map().filter().collect()`), the previous expression `current_expr` had to be passed into the new node. Because it's stored in a `Box`, using `Box::new(current_expr.clone())` causes an expensive deep clone of the entire recursive AST structure for each method in the chain. However, since the old node is entirely moved into the new node (and we overwrite `current_expr` immediately after), we can use `std::mem::replace(&mut current_expr, AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unknown })` to safely extract the old tree without a single allocation, effectively a zero-cost transfer of ownership.
**Action:** Use `std::mem::replace` to avoid deep cloning of AST nodes when building recursive or iterative tree structures in place.
**[Optimizing recursive type formatting]**
**Learning:** Using `format!` recursively (e.g., in `to_rust_type` for nested types like `Result<Option<Vec<String>>, i64>`) creates multiple intermediate heap-allocated `String`s that are immediately concatenated and dropped.
**Action:** Replace recursive `format!` calls with a `write!` macro approach using `std::fmt::Write`. Pre-allocate a single `String` buffer (e.g., `String::with_capacity`) and pass a mutable reference to it down the recursive tree to drastically reduce allocations.

## Optimize String allocation in tester.rs
**Learning:** When capturing raw standard error or failure outputs line by line in testing tools, using `String::new()` forces the string to undergo multiple heap reallocations as it expands. `String::with_capacity()` can eliminate these intermediate allocations. Even for unpredictable sizes like panic stack traces, estimating a small but reasonable capacity (e.g., 1024 bytes) prevents early micro-allocations, and when an upper bound is exactly known (e.g., `raw_stderr.len()`), it provides an exact O(1) allocation.
**Action:** Always prefer `String::with_capacity()` when aggregating string parts inside a loop, especially in critical paths or output parsers.
