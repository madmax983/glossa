# 022. Enforce Semantic Strictness For Missing Verbs, Double Subjects, and Undefined Names

Date: 2026-05-10

## Status

Accepted

## Context

The compiler was allowing several invalid or structurally unsound semantic conditions to pass silently without generating appropriate error messages, creating confusion as documented in `ECHO_ISSUE.md`. Specifically:
1. "Missing verb" statements with a single undefined subject triggered an Internal Compiler Error (codegen panic) instead of failing gracefully.
2. "Double subject" combinations erroneously bypassed validation due to overly broad exemptions for function/property accesses.
3. Undefined variables used alone (e.g. `ἄγνωστος λέγε.`) silently compiled and were treated as numeric zero during execution, hiding logical errors.

## Decision

1. Repaired the logic in `Assembler::check_missing_verb` to ensure `AssemblyError::MissingVerb` is correctly thrown when appropriate, bypassing only explicitly valid match-arm fallback variables (e.g. `μηδεν`).
2. Corrected the `DoubleSubject` checks inside `Assembler::finalize` to restrict exemptions only to valid binary ops, binding verbs, or actual function calls, removing unintended exemptions that previously masked multiple subjects.
3. Enhanced semantic interpretation in `src/semantic/conversion.rs` (`try_print_default` and `classify_expression`) to verify the existence of the AST `subject` or `object` variable in the `Scope`. If a variable is completely undefined, the compilation correctly halts with `GlossaError::UndefinedName`. The `in_trait` property was added to `Scope` to ensure we do not falsely flag implicit struct properties referenced inside trait methods as "undefined."

## Consequences

- Improved developer experience; the compiler now communicates accurate, Greek-styled syntax and semantic errors instead of silent failures or rustc panics.
- Modifies `Scope` by adding the `in_trait` state, ensuring property accesses within traits continue working as expected without being flagged as undefined standalone variables.
- Ensures all integration and `havoc` tests accurately confirm error pathways without regressing valid behavior.
