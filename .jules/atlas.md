## [Breaking Internal Module Entanglement]
**Tangle:** The `src/tools/` sub-modules were directly importing from one another using `crate::tools::` paths, violating local cohesion and bypassing the defined module structure.
**Blueprint:** Replaced direct `crate::tools::` usages with relative `super::` imports within the `src/tools/` submodules.
**Stability:** Internalized module dependencies, ensuring the submodules don't leak beyond the tool boundary and rely on localized relative imports instead of global paths.
