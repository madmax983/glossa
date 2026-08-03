# 022. Add Gnomon Tool to Developer Experience Tools

Date: 2026-07-08

## Status

Proposed

## Context

The ΓΛΩΣΣΑ compiler translates abstract structural meaning from source, but lacks automated ways to introspect runtime algorithmic boundaries early on. To provide more insight into the execution characteristics of loops and recursive branching, we required a way to statically gauge complexity depth via the AST.

## Decision

We have added the "Gnomon" (`ὁ Γνώμων`) tool in `src/tools/gnomon.rs` to the "Developer Experience (Nova)" toolset. The Gnomon estimates Big-O time complexity by statically analyzing the loop depth and branching logic in the semantic AST.

## Consequences

*   **Positive:** Developers gain immediate complexity feedback (`O(1)`, `O(N)`, `O(N^2)`, etc.) without runtime execution.
*   **Positive:** Encourages performant algorithm design during the early stages of Glossa code authoring.
*   **Negative:** Adds a new tool that must be maintained as part of the `glossa` toolset.
