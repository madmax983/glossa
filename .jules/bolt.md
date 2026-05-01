**[Title] Eliminate Intermediate Iterator Collections**
**Learning:** Found multiple places in `src/semantic/conversion.rs` and `src/semantic/patterns.rs` where iterators were being fully `.collect()`ed into `Vec` inside functions only to be iterated again, or creating intermediate vecs instead of using `Vec::with_capacity` and extending.
**Action:** Replace `.collect()` with `Vec::with_capacity` and `.extend()` to save allocations, especially on potentially hot paths like semantic conversion and AST node instantiation.


**[Title] Eliminate Intermediate Iterator Collections**
**Learning:** Found multiple places in `src/semantic/conversion.rs` and `src/semantic/patterns.rs` where iterators were being fully `.collect()`ed into `Vec` inside functions only to be iterated again, or creating intermediate vecs instead of using `Vec::with_capacity` and extending.
**Action:** Replace `.collect()` with `Vec::with_capacity` and `.extend()` to save allocations, especially on potentially hot paths like semantic conversion and AST node instantiation.

**[Title] Eliminate Intermediate Iterator Collections**
**Learning:** Found multiple places in `src/semantic/conversion.rs` and `src/semantic/patterns.rs` where iterators were being fully `.collect()`ed into `Vec` inside functions only to be iterated again, or creating intermediate vecs instead of using `Vec::with_capacity` and extending.
**Action:** Replace `.collect()` with `Vec::with_capacity` and `.extend()` to save allocations, especially on potentially hot paths like semantic conversion and AST node instantiation.
