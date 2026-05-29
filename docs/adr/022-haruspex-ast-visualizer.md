# 22. Introduce Haruspex (AST Visualizer)

Date: 2026-05-29

## Status

Proposed

## Context

Developers needed a way to inspect the raw semantic tree structure of a ΓΛΩΣΣΑ program to understand exactly how expressions are nested and typed. Existing tools like Cartographer and Labyrinth serve different purposes (architecture and control flow, respectively).

## Decision

Introduce "Haruspex", a new tool that translates the semantic AST (`AnalyzedProgram`) into a DOT graph for visualization with Graphviz.

## Consequences

Compiler developers gain deeper visibility into the AST structure. This requires Graphviz for rendering the resulting DOT output.
