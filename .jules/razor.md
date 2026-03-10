## [Reduction]
**Bloat:** The `Trait` system (`χαρακτήρ` and `ἐμπίπτειν`), including Trait nodes in AST, Semantic Models, Parsers, Codegen, and Tools.
**Cut:** Complete removal of all Trait-related logic. Traits were an overly complex abstraction that violated YAGNI. They added significant overhead to the parser, semantic analyzer, scope resolver, and codegen for a feature that wasn't strictly necessary for the core language functionality. De-abstracted by relying purely on concrete Types and Functions.
**Saved:** Hundreds of lines of code across AST, parser, semantic analyzer, codegen, and tools. Simplified the mental model of the compiler significantly.
