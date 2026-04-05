## [Reduction]
**Bloat:** `TraitMethodCall` enum variant in `AnalyzedExprKind` and its specialized processing functions (like `generate_trait_method_call` and `tell_trait_method_call`). It represented speculative generality where a trait method call had identical compilation and parsing paths to a regular object method.
**Cut:** Removed `TraitMethodCall` entirely and consolidated its path into the existing `MethodCall` node, treating trait methods as regular methods dynamically checked in type space.
**Saved:** Dozens of lines of repeated boilerplate matching rules across parsing, codegen, reporting, and narrative tools. Simplifies the AST and standardizes behavior.
## [Reduction]
**Bloat:** `try_parse_trait_method_call` function in `src/semantic/patterns.rs`. The logic was overly specialized for traits, leading to "Speculative Generality", as regular object methods and trait methods share the identical AST representation and compilation behavior.
**Cut:** Refactored `try_parse_trait_method_call` to `try_parse_method_call` in `src/semantic/patterns.rs`, removing the trait-specific checks (`scope.has_trait_method`) to apply parsing broadly for any standalone method call.
**Saved:** Unnecessary trait method logic constraints. Avoids duplicating method parsing logic by generalizing the single abstract rule to a single concrete parsing rule.
## [Reduction]
**Bloat:** `fields.clone()` in `try_parse_struct_instantiation` (src/semantic/patterns.rs) created an unnecessary `Vec<(SmolStr, GlossaType)>` just for sequential iteration to extract field names.
**Cut:** Replaced with `fields.as_slice()`, changing the variable type to `&[(SmolStr, GlossaType)]` and falling back to an empty slice `&[]` instead of a new empty `Vec`.
**Saved:** Unnecessary heap allocations per struct instantiation parsed, keeping the codebase simpler and adhering to zero-cost abstractions by avoiding speculative cloning.
