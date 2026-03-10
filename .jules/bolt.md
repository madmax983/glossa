**Pre-allocate semantic conversions**
**Learning:** Pre-allocating `Vec` instances based on inner AST node sizes is extremely effective for improving AST generation speed during semantic conversion, directly removing multiple unneeded O(log N) heap allocations.
**Action:** Implemented pre-allocation `Vec::with_capacity` optimizations in `src/semantic/declarations.rs`.
