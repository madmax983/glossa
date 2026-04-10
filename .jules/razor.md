## [Reduction]
**Bloat:** `TraitMethodCall` enum variant in `AnalyzedExprKind` and its specialized processing functions (like `generate_trait_method_call` and `tell_trait_method_call`). It represented speculative generality where a trait method call had identical compilation and parsing paths to a regular object method.
**Cut:** Removed `TraitMethodCall` entirely and consolidated its path into the existing `MethodCall` node, treating trait methods as regular methods dynamically checked in type space.
**Saved:** Dozens of lines of repeated boilerplate matching rules across parsing, codegen, reporting, and narrative tools. Simplifies the AST and standardizes behavior.

## [Reduction]
**Bloat:** `try_parse_trait_method_call` function in `src/semantic/patterns.rs`. The logic was overly specialized for traits, leading to "Speculative Generality", as regular object methods and trait methods share the identical AST representation and compilation behavior.
**Cut:** Refactored `try_parse_trait_method_call` to `try_parse_method_call` in `src/semantic/patterns.rs`, removing the trait-specific checks (`scope.has_trait_method`) to apply parsing broadly for any standalone method call.
**Saved:** Unnecessary trait method logic constraints. Avoids duplicating method parsing logic by generalizing the single abstract rule to a single concrete parsing rule.

## [Reduction]
**Bloat:** Unused deprecated `expressions()` method in `Statement` struct in `src/ast.rs`.
**Cut:** Deleted the `expressions()` method.
**Saved:** 13 lines of code.

## [Reduction]
**Bloat:** Speculative and unenforced structural checks in Glossa's compiler semantic layer regarding the definition and usage of verbless statements, undefined variables, and double subjects, which falsely advertised compiler capabilities in the README.
**Cut:** Implemented literal semantic evaluation in `classify_expression` to directly fail and raise standard errors like `MissingVerb` and `DoubleSubject` for invalid structural phrases rather than silencing them.
**Saved:** Mental load for debugging silent compilation ICEs by giving end users immediate and direct error feedback for missing verbs, undefined fallback bindings, and double subjects.
