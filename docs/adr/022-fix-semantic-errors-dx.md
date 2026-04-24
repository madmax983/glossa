# 022. Fix Semantic Error Bypasses and Improve DX

Date: 2026-04-24

## Status

Proposed

## Context

The compiler was experiencing several critical bugs related to semantic error handling and developer experience (DX), as raised by the Echo persona:
1. **Undefined Variables:** Undefined variables were silently evaluating to `0` or `Unknown` without producing any semantic error, causing user confusion and invalid code execution.
2. **Double Subjects:** The `DoubleSubject` check within the assembler (`src/semantic/assembly/mod.rs`) was incorrectly bypassing verbs matching `is_print_verb` or `is_binding_verb`, allowing statements with multiple subjects (like `ὁ ἄνθρωπος ὁ θεὸς λέγει.`) to compile silently.
3. **Missing Verbs:** Statements missing a verb completely (like `ὁ ἄνθρωπος.`) were bypassing the `MissingVerb` semantic check, resulting in Internal Compiler Errors (ICE) and raw `rustc` aborts during the code generation phase.

## Decision

We enforced stricter semantic validation across the pipeline:
1. **Strict Undefined Checks:** Enforced explicit `!scope.is_defined()` checks in `try_print_default` and `extract_object_fallback` within `src/semantic/conversion.rs`. Valid variables, trait fields (`ξ`, `ψ`, etc.), and property modifiers (`self`, `selfου`) were explicitly exempted to preserve functionality.
2. **Uniform Verb Checks:** Corrected the `DoubleSubject` enforcement to apply to all verbs uniformly, strictly yielding `AssemblyError::DoubleSubject`. Added exemptions specifically for filter constructs (`is_filter_pattern`).
3. **Proper Missing Verb Flow:** Corrected the assembler's fallback matching criteria so that standard phrases properly yield `AssemblyError::MissingVerb` and return friendly Greek error messages ("Ῥῆμα οὐχ εὑρέθη!") before reaching `rustc`.

## Consequences

- Improved compilation stability and predictable developer feedback with localized error strings.
- Removed Internal Compiler Errors associated with invalid missing verbs.
- Code generation no longer attempts to resolve completely invalid AST nodes.
- Maintained exact behavior for trait implementation parameters (`trait_tests.rs`) and code coverage mechanisms (`warden_coverage.rs`).
