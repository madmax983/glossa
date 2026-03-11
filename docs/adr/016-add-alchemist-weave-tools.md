# 16. Add Alchemist and Weave Tools

Date: 2026-03-11
Status: Accepted

## Context

The `alchemist` and `weave` tools have been added under `src/tools/` behind the `nova` feature flag. The `alchemist` is a Python transpiler that proves the independence of the semantic phase from Rust codegen by providing an alternative export format. The `weave` tool generates a Rosetta Stone Markdown document combining Glossa source code, semantic assembly logic, and generated Rust code.

## Decision

Record the addition of these two new tools (`alchemist` and `weave`) to the compiler tool ecosystem.

## Consequences

### Positive
- **Export Options**: Provides alternative compilation targets (Python).
- **Documentation**: Generates rich Rosetta Stone documents linking Greek syntax to semantics and generated code.
- **Developer Experience**: Enhances the overall tool ecosystem under the Nova banner.

### Negative
- **Complexity**: Increases the surface area of tools that need maintenance.
