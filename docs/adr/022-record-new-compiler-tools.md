# Record New Compiler Tools: Gnomon, Haruspex, Scholar

## Status
Proposed

## Context
Three new tools have been added to the compiler's toolbelt to improve the developer experience:
- **The Gnomon (`src/tools/gnomon.rs`)**: A Big-O complexity estimator that casts a shadow over the program's AST to estimate its execution time complexity.
- **The Haruspex (`src/tools/haruspex.rs`)**: A Graphviz AST visualizer that translates the `AnalyzedProgram` into a DOT graph, allowing compiler developers to inspect the raw semantic tree structure.
- **The Scholar (`src/tools/scholar.rs`)**: An API documentation generator that parses `.γλ` files and translates structs, traits, and functions into standard Markdown documentation (`doc.md`).

However, these structural additions were not previously recorded in the Architecture Decision Records (ADRs) nor reflected in our central architecture map (`docs/architecture.md`).

## Decision
We will formally adopt and document these three tools as components of the compiler's Developer Experience boundary (`tools` module).
- `docs/architecture.md` will be updated to include `gnomon`, `haruspex`, and `scholar` in the System Context and Container C4 models.
- This ADR serves as the historical record of their inclusion.

## Consequences
- The compiler toolbelt expands, providing better performance analysis, debugging, and documentation capabilities for developers writing in ΓΛΩΣΣΑ.
- Maintenance surface area increases within the `tools` directory.
- Architectural transparency is maintained, ensuring diagrams remain a true reflection of the codebase.
