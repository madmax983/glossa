## [Reduction]
**Bloat:** `TraitMethodCall` enum variant in `AnalyzedExprKind` and its specialized processing functions (like `generate_trait_method_call` and `tell_trait_method_call`). It represented speculative generality where a trait method call had identical compilation and parsing paths to a regular object method.
**Cut:** Removed `TraitMethodCall` entirely and consolidated its path into the existing `MethodCall` node, treating trait methods as regular methods dynamically checked in type space.
**Saved:** Dozens of lines of repeated boilerplate matching rules across parsing, codegen, reporting, and narrative tools. Simplifies the AST and standardizes behavior.
## [Reduction]
**Bloat:** `try_parse_trait_method_call` function in `src/semantic/patterns.rs`. The logic was overly specialized for traits, leading to "Speculative Generality", as regular object methods and trait methods share the identical AST representation and compilation behavior.
**Cut:** Refactored `try_parse_trait_method_call` to `try_parse_method_call` in `src/semantic/patterns.rs`, removing the trait-specific checks (`scope.has_trait_method`) to apply parsing broadly for any standalone method call.
**Saved:** Unnecessary trait method logic constraints. Avoids duplicating method parsing logic by generalizing the single abstract rule to a single concrete parsing rule.
## [Reduction]
**Bloat:** Re-implementing redundant null-checks across error types via empty match variants for Double Subject and Missing Verb in `Assembly`.
**Cut:** Moved Double Subject and Missing Verb checking rules to `classify_expression` within `conversion.rs` where the actual context determines if it's really an error instead of using arbitrary syntax matching that breaks control flow logic.
**Saved:** Multiple blank lines and misleading comments from `assembly.rs` as well as avoiding unnecessary `unwrap` and recursive macro workarounds.
