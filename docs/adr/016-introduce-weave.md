# 16. Introduce Weave Rosetta Stone Exporter

Date: 2026-03-09
Status: Proposed

## Context

As the ΓΛΩΣΣΑ compiler ecosystem expands, there is a growing need to demystify the semantic assembly process and the relationship between Ancient Greek source code, the compiler's semantic interpretation, and the generated Rust code. Previously, users and contributors had to piece together these phases using discrete tools like `mosaic` (for assembly) and inspecting the compiled `.rs` output. This fragmented developer experience makes learning the language and debugging the compiler more difficult.

## Decision

We have introduced the `Weave` tool (`src/tools/weave.rs`), guarded by the `nova` feature flag, to act as an exporter. "Weave" generates a consolidated 'Rosetta Stone' Markdown document that combines:
1. The original ΓΛΩΣΣΑ source code.
2. The semantic assembly logic (visualized via `mosaic`).
3. The generated Rust code.

## Consequences

### Positive
- **Educational Value:** The generated Rosetta Stone document provides an immediate, side-by-side comparison of the full pipeline, greatly improving the onboarding experience for new users and contributors.
- **Debugging:** It serves as a unified artifact for examining compilation edge cases and assembly mappings.
- **Architectural Alignment:** The tool neatly integrates into the existing "Developer Experience (Nova)" tool ecosystem, reusing the existing `run_mosaic_inner` and `generate_rust_file` interfaces.

### Negative
- **Maintenance Surface:** Adds another developer tool to maintain and test, specifically requiring coverage for Markdown generation and structural rendering logic.
