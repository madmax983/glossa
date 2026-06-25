# 022. Add Haruspex, Gnomon, and Scholar Tools

Date: 2026-06-25

## Status

Proposed

## Context

We recently added three new developer experience tools in the `nova` feature suite:
- **Haruspex** (`src/tools/haruspex.rs`): A tool that translates the semantic AST of a program into a DOT graph for visualization with Graphviz. This helps compiler developers inspect the raw semantic tree structure.
- **Gnomon** (`src/tools/gnomon.rs`): A tool that estimates the Big-O time complexity of a program by statically analyzing loop depth in the semantic AST.
- **Scholar** (`src/tools/scholar.rs`): An API doc generator that distills structures, traits, and functions into GitHub-flavored Markdown.

## Decision

We integrated these experimental tools into `src/tools/`, exposed them via `src/tools/mod.rs`, and gated them under the `nova` feature flag. We also updated `src/main.rs` and the CLI module to register these commands.

## Consequences

- These tools provide greater introspection into semantic assembly and complexity.
- Increases the CLI surface area and maintainance overhead, which is mitigated by keeping them behind the `nova` experimental feature flag.
