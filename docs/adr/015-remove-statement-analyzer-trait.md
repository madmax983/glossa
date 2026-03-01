# 15. Remove StatementAnalyzer Trait

Date: 2026-02-28

## Status

Accepted

## Context

In the earlier architecture, the `StatementAnalyzer` trait was introduced to break circular dependencies between the main semantic orchestrator (`src/semantic/mod.rs` and its internal logic) and the specialized submodules (`control_flow.rs`, `declarations.rs`). This allowed the submodules to receive a generic reference to an orchestrator (`&mut impl StatementAnalyzer`) without tightly coupling them to the concrete `Analyzer` implementation.

However, over time, `Analyzer` became the sole and exclusive implementer of this trait. Following the "Razor" persona's philosophy of essentialism and YAGNI (You Aren't Gonna Need It), maintaining a single-implementation interface added unnecessary abstraction overhead and decreased architectural clarity without providing any tangible benefit.

## Decision

We have decided to remove the `StatementAnalyzer` trait entirely (`src/semantic/traits.rs` was deleted). The trait's methods have been flattened directly into the concrete `Analyzer` struct. Function signatures in submodules (`control_flow.rs`, `declarations.rs`) were updated to accept a concrete `&mut Analyzer` reference instead of `&mut impl StatementAnalyzer`.

## Consequences

*   **Simplicity:** The semantic analysis pipeline is simpler and easier to understand, with fewer layers of abstraction.
*   **Performance:** While minimal, removing dynamic trait boundaries or monomorphization overhead can marginally improve compile times.
*   **Clarity:** The relationship between the orchestrator (`Analyzer`) and its delegates is now explicit and direct.
*   **Documentation:** Architecture diagrams and module-level comments have been updated to reflect `Analyzer` as the concrete orchestrator, removing references to the legacy trait.
