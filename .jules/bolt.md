**[Optimizing Collections in iterators]**
**Learning:** Checking for `.is_empty()` after `.collect::<Vec<_>>()` forces an unnecessary heap allocation. We can check if an iterator is empty dynamically using `.peekable()`. Additionally, if strings from `&'a str` need to be converted to `Cow<'static, str>`, we must use `.to_string()` as `.into()` or `Cow::Borrowed` will result in static lifetime compilation errors.
**Action:** Replace `Vec<_> = iter.collect()` with `.peekable()` for emptiness checking whenever possible, and be mindful of `Cow` lifetimes.

**[Optimizing AST Node Cloning with `std::mem::replace`]**
**Learning:** During iterative AST tree building (e.g., when wrapping an expression in successive `MethodCall`s like `.iter().map().filter().collect()`), the previous expression `current_expr` had to be passed into the new node. Because it's stored in a `Box`, using `Box::new(current_expr.clone())` causes an expensive deep clone of the entire recursive AST structure for each method in the chain. However, since the old node is entirely moved into the new node (and we overwrite `current_expr` immediately after), we can use `std::mem::replace(&mut current_expr, AnalyzedExpr { expr: AnalyzedExprKind::None, glossa_type: GlossaType::Unknown })` to safely extract the old tree without a single allocation, effectively a zero-cost transfer of ownership.
**Action:** Use `std::mem::replace` to avoid deep cloning of AST nodes when building recursive or iterative tree structures in place.

**[Optimizing Recursive String Construction]**
**Learning:** Using `format!` in recursive functions (like AST or type serialization) forces an unnecessary heap allocation at every level of depth. Instead, passing a pre-allocated mutable `String` buffer and using `std::fmt::Write` with the `write!` macro can reduce allocations to roughly O(1).
**Action:** Replace recursive string concatenations and formats with a single mutable buffer passed by reference when traversing structures.
