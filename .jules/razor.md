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
**Bloat:** Undefined variables silently decaying into literal `0` integers deep within semantic parsing (in `extract_value`), and critical sentence structure checks (`DoubleSubject`, `MissingVerb`) being ignored or throwing silent raw Rust codegen errors.
**Cut:** Removed silent variable decay defaults. Flattened statement validation into `classify_assembled_statement` where scoping and AST context are available, actively preventing `DoubleSubject` and explicit `MissingVerb` states. Added a post-classification generic `check_undefined_variables` AST walker that gracefully handles all node types without specific `Option<...>` fallback boilerplates.
**Saved:** Multiple paths that historically led to compiler crashes / malformed semantic models. Unified missing-verb catching, and prevented dozens of lines of unneeded checks later down in the codegen toolchain.
## [Reduction]
**Bloat:** `parse_test_output` in `src/tools/tester.rs`. Using iterator cloning, `split_whitespace`, and manual advances to extract pieces of test output string.
**Cut:** Replaced string whitespace splitting with explicit slice manipulation and string search (`rfind`).
**Saved:** 15 lines of code, reduced parsing complexity and removed iterator state logic. Avoids any edge case bounds issue without needing extra checks.
## [Reduction]
**Bloat:** Speculative Generality via `CaptureMode::Memoize`. The `Memoize` variant for closure capture modes existed in the semantic model to theoretically cache 0-arity closures (Perfect Participles). However, the compiler actually downgraded these to `CaptureMode::Borrow` at AST assembly time to avoid fatal cache-invalidation bugs when arguments were present. The code generator still contained complex, dead logic (`generate_memoized_closure`) full of `RefCell` caching logic.
**Cut:** Eliminated `CaptureMode::Memoize` entirely from the semantic model, `src/codegen.rs`, and `src/tools/narrator.rs`.
**Saved:** Deleted ~40 lines of dangerous, unreachable code, flattened the `CaptureMode` model, and simplified closure generation.

## [Reduction]
**Bloat:** Complex `DoubleSubject` and Missing Verb state checks spread out in `Assembler::finalize()` which were bypassed by certain verbs and nested phrases.
**Cut:** Removed duplicate and misfiring checks in `finalize()`. Placed a single unified `DoubleSubject` check at the beginning of `classify_expression` in `src/semantic/conversion.rs`.
**Saved:** Avoided messy verb classification bypassing and consolidated grammatical validation to where semantic structure is actually clear.
## [Reduction]
**Bloat:** `src/errors/` module structure with single file `assembly.rs` containing `AssemblyError`.
**Cut:** Flattened `assembly.rs` into `errors.rs` by combining the contents, removing the `pub(crate) mod assembly;` and replacing the nested directory.
**Saved:** 1 file, cognitive load of jumping between files, flattened hierarchy.
