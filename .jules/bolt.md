**Pre-allocate semantic conversions**
**Learning:** Pre-allocating `Vec` instances based on inner AST node sizes is extremely effective for improving AST generation speed during semantic conversion, directly removing multiple unneeded O(log N) heap allocations.
**Action:** Implemented pre-allocation `Vec::with_capacity` optimizations in `src/semantic/declarations.rs`.
**Pre-allocate Semantic and Codegen Vectors**
**Learning:** Pre-allocating `Vec` instances based on exact source elements in `src/semantic/conversion.rs` and heuristics in `src/codegen.rs` directly eliminates unneeded `O(log N)` heap allocations without increasing complexity.
**Action:** Implemented `Vec::with_capacity` optimizations across semantic conversion and codegen phases.
