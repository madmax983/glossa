# 022. Record Missing Developer Tools

Date: 2026-05-26

## Status

Proposed

## Context

During a routine architectural review, we discovered that several new developer tools were added to the `src/tools` module but were never formally recorded in the system architecture diagrams or ADRs. This violates the "Architectural Transparency" directive.

The newly discovered tools are:
- `gnomon.rs`: The Gnomon - A Big-O complexity estimator.
- `haruspex.rs`: The Haruspex - A Graphviz AST visualizer.
- `scholar.rs`: The Scholar - An API Documentation generator.

Without recording these tools, our architectural maps are incomplete, leaving developers without a clear mental model of the available utilities within the "Nova" developer experience boundary.

## Decision

We will explicitly document these tools in our living architecture diagram (`docs/architecture.md`) under the Developer Experience boundary.
The tools have been appended to the C4 Container diagram to reflect their presence and purpose.

## Consequences

- The C4 architecture diagrams now accurately reflect the codebase's current state.
- The Gnomon, Haruspex, and Scholar are officially recognized as part of the compiler's developer tools suite.
- Future developers will have better visibility into the available toolchain.
