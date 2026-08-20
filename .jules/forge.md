**Refactored Cartographer's generate_map**
**Learning:** Found a god object function > 100 lines handling struct rendering, trait rendering, dependencies, and implementations.
**Action:** Created clear, small helpers (`format_structs`, `format_traits`, `format_dependencies`, `format_trait_impls`) and passed mutable states down.

**[Refactoring God Functions in AST Visitors]**
**Learning:** AST Visitors like `haruspex.rs` tend to become "God Functions" containing massive `match` blocks over every AST node variant, violating readability principles.
**Action:** Extract grouped branches (like literals, or basic statements) into focused helper functions to keep the main match statements clean and under 50 lines.
