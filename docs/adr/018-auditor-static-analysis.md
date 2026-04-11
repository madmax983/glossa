# 018. Add Auditor Static Analysis Tool

## Status

Accepted

## Context

The ΓΛΩΣΣΑ compiler handles parsing, type checking, and translation of Ancient Greek source code into Rust. While semantic analysis detects missing bindings and type mismatches, it does not currently evaluate code quality metrics or detect common smells like unused variables or variables declared as mutable but never reassigned. These logic bugs, though technically valid syntactically, degrade developer experience and can hide serious structural flaws in user programs. There is a need for a dedicated tool to traverse the analyzed semantic model (HIR) and report these specific patterns to the programmer.

## Decision

We have added "The Auditor" (`ὁ Λογιστής`) to the "Developer Experience (Nova)" toolset (`src/tools/auditor.rs`). It acts as a static analysis phase that runs over the `AnalyzedProgram`. The Auditor leverages a custom AST visitor to collect variable declarations and track their usages and re-assignments throughout the symbol tree. Based on this collected data, it emits warnings for:
1. Variables that are declared but never accessed.
2. Variables that are declared as mutable but never reassigned.

## Consequences

* **Developer Experience:** Improved code quality tools for developers. Programmers will receive actionable feedback for unutilized code paths and unnecessary mutability markers.
* **Architecture:** Adheres to the established compiler pipeline by cleanly operating as a separate module traversing the `AnalyzedProgram` instead of complicating the core `src/semantic` engine.
* **Complexity:** Adds a new pass over the HIR, slightly increasing total compile/analysis time when the tool is explicitly invoked via the CLI (`Audit` command). It introduces a dependency on traversing the entire resolved AST which must be maintained if the AST structure changes.
