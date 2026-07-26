**[Optimizing Collections in iterators]**
**Learning:** Checking for `.is_empty()` after `.collect::<Vec<_>>()` forces an unnecessary heap allocation. We can check if an iterator is empty dynamically using `.peekable()`. Additionally, if strings from `&'a str` need to be converted to `Cow<'static, str>`, we must use `.to_string()` as `.into()` or `Cow::Borrowed` will result in static lifetime compilation errors.
**Action:** Replace `Vec<_> = iter.collect()` with `.peekable()` for emptiness checking whenever possible, and be mindful of `Cow` lifetimes.

**[Optimizing AST Node Cloning with `std::mem::replace`]**
**Learning:** During iterative AST tree building (e.g., when wrapping an expression in successive `MethodCall`s like `.iter().map().filter().collect()`), the previous expression `current_expr` had to be passed into the new node. Because it's stored in a `Box`, using `Box::new(current_expr.clone())` causes an expensive deep clone of the entire recursive AST structure for each method in the chain. However, since the old node is entirely moved into the new node (and we overwrite `current_expr` immediately after), we can use `std::mem::replace(&mut current_expr, AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unknown })` to safely extract the old tree without a single allocation, effectively a zero-cost transfer of ownership.
**Action:** Use `std::mem::replace` to avoid deep cloning of AST nodes when building recursive or iterative tree structures in place.
**[Optimizing recursive type formatting]**
**Learning:** Using `format!` recursively (e.g., in `to_rust_type` for nested types like `Result<Option<Vec<String>>, i64>`) creates multiple intermediate heap-allocated `String`s that are immediately concatenated and dropped.
**Action:** Replace recursive `format!` calls with a `write!` macro approach using `std::fmt::Write`. Pre-allocate a single `String` buffer (e.g., `String::with_capacity`) and pass a mutable reference to it down the recursive tree to drastically reduce allocations.

**[Optimizing recursive tree formatting]**
**Learning:** When formatting a recursive enum representing a tree structure (like `GlossaType`), returning a new `String` at each level of recursion and combining them with `format!` causes many intermediate string allocations. A zero-cost abstraction passes a mutable reference `&mut String` buffer down the recursive stack and appends directly to it.
**Action:** Replaced recursive `format!` calls in `tell_type` with a `write_tell_type(ty, buf: &mut String)` helper, and pre-allocated the top-level buffer with `String::with_capacity`.

**[Clippy useless borrows in formatting]**
**Learning:** Passing an explicit reference to `format!` (e.g. `format!("{}", &var)`) when the variable is already a reference or owned value triggers the `clippy::useless_borrows_in_formatting` lint.
**Action:** Removed redundant `&` from `format!` macro arguments in `src/semantic/conversion.rs`.
