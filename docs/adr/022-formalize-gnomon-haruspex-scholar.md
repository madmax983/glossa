# 22. Formalize Gnomon, Haruspex, and Scholar Tools

Date: 2024-11-15
Status: Proposed

## Context

The ΓΛΩΣΣΑ compiler repository recently introduced three new tools inside the `src/tools/` directory, operating within the Developer Experience (Nova) toolset. These tools consume the semantic AST (`AnalyzedProgram`) to provide distinct insights and capabilities:

1. **Gnomon (`src/tools/gnomon.rs`)**: Analyzes loop depth to estimate Big-O time complexity.
2. **Haruspex (`src/tools/haruspex.rs`)**: Inspects the raw semantic AST and generates Graphviz DOT graphs for visualization.
3. **Scholar (`src/tools/scholar.rs`)**: Auto-generates GitHub-flavored Markdown API documentation from `εἴδη`, `χαρακτῆρες`, and `ἔργα`.

However, the addition of these components was not formally captured in the architecture documentation. To adhere to the project's Architectural Transparency and "Documentation as Code" philosophy, their existence, purpose, and relationship to the compiler must be recorded and illustrated.

## Decision

We formally recognize **Gnomon**, **Haruspex**, and **Scholar** as official tools within the "Developer Experience (Nova)" boundary.

Their definitions are recorded in this Architectural Decision Record, and they will be represented as containers inside the System Architecture (C4 Container Level) diagram. They all consume the `AnalyzedProgram` emitted by the Semantic Analyzer.

## Consequences

### Positive
- **Architectural Clarity**: Implicit components are now explicitly tracked, avoiding undocumented, "shadow" subsystems.
- **Enhanced Discoverability**: Future developers can easily locate and understand the responsibilities of these tools from standard documentation rather than discovering them ad-hoc.
- **Clear Boundaries**: Demonstrates how modular the compiler design is, with additional insights derived purely from the shared AST format (`AnalyzedProgram`).

### Negative
- **Diagram Density**: The addition of three tools into the C4 architecture diagram increases its size and density.
- **Maintenance Surface**: The tools must be continuously maintained alongside any structural updates made to the `AnalyzedProgram` types.
