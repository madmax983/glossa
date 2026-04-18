# 20. Document early structural error checking

Date: 2026-04-18
Status: Proposed

## Context

The compiler was failing to report expected error messages described in the "Troubleshooting" guide of the documentation. Specifically:
- `DoubleSubject` ("Διπλοῦν ὑποκείμενον"): `ὁ ἄνθρωπος ὁ θεὸς λέγει.` compiled silently instead of throwing an error.
- `UndefinedName` ("Οὐκ οἶδα τὸ ὄνομα"): `ἄγνωστος λέγε.` compiled silently and defaulted to `0` instead of throwing an error.

The root cause for the `DoubleSubject` issue was that the check was happening too late (inside `Assembler::finalize()`), where it could be bypassed by certain verb classifications (like `is_print_verb` or `is_binding_verb`) or completely missed if the assembler didn't flag it under specific operator conditions.
For `UndefinedName`, there was a missing validation in the value extraction step for standalone noun fallbacks.

These issues fall under the purview of bugfixes affecting core logic semantics.

## Decision

We propose that the semantic phase must handle early structural and reference error checking during the conversion phase rather than relying on delayed checks or fallback structures.
- The `DoubleSubject` enforcement must be moved to the very beginning of the expression classification phase (`classify_expression` in `src/semantic/conversion.rs`) to ensure multiple nominatives are strictly caught early.
- For `UndefinedName`, we propose adding explicit `extract_subject_fallback` checks during value extraction to verify that a noun acting as a subject (or object fallback) actually exists in the current scope, and throw an error immediately if absent.

As Codex, I have created this ADR to formally record the architectural decision and updated the Mermaid map, but I will not directly alter the compiler logic, leaving the bug fix execution to a different persona or a future task to ensure adherence to scope boundaries.

## Consequences

- We now formally recognize `conversion.rs` as the correct layer for these structural error checks.
- The `docs/architecture.md` map has been updated to clarify that `conversion` validates early error enforcements (DoubleSubject, Undefined).
- This record serves as the architectural foundation for subsequent PRs that will implement these fixes.
