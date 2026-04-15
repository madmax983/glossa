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
**Bloat:** `ScopeGuard` RAII structure with explicit `Drop`, `Deref`, and `DerefMut` trait implementations in `src/semantic/resolver.rs` which added verbosity and abstraction bloat. Explicit `enter_scope()` usage forced callers to manually manage the scope lifetime or use separate code blocks (`{ ... }`).
**Cut:** Replaced `ScopeGuard` and `enter_scope` with a single closure-based higher-order function `with_scope(|scope| { ... })`.
**Saved:** Over 30 lines of boilerplate structs and trait implementations. Reduced cognitive load for scope lifetime management by encapsulating setup and teardown into an idiomatic Rust closure approach.
## [Reduction]
**Bloat:** Undefined variables silently returned `NumberLiteral(0)` inside `extract_value` instead of failing, and missing verbs produced raw rustc outputs instead of `MissingVerb` errors. Furthermore, DoubleSubject wasn't checked on simple Noun/Noun/Verb combinations.
**Cut:** Added proper strict error returns inside `extract_value` for undefined subjects, enforced `MissingVerb` by removing the fallback code when no expressions could be made, and simplified the `DoubleSubject` checks inside `classify_expression` to correctly reject `Noun Noun Verb` patterns that aren't bindings or matches.
**Saved:** Reduced silent error-swallowing bugs and fixed missing verb validation.
