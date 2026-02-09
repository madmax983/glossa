# Architecture

This document describes the high-level architecture of the ΓΛΩΣΣΑ (GLOSSA) programming language compiler.

## System Context (C4 Level 1)

The following diagram illustrates how ΓΛΩΣΣΑ fits into the development environment.

```mermaid
C4Context
    title System Context Diagram for ΓΛΩΣΣΑ

    Person(programmer, "Programmer", "Writes code in Ancient Greek")
    System(glossa, "ΓΛΩΣΣΑ Compiler", "Compiles Greek code to Rust")
    System_Ext(rustc, "Rust Compiler", "Compiles generated Rust to machine code")

    Rel(programmer, glossa, "Writes Source (.γλ)", "File System")
    Rel(glossa, rustc, "Generates Rust Code", "File System")
    Rel(rustc, glossa, "Reports Errors", "StdOut/StdErr")
```

## Compiler Pipeline (C4 Container Level)

The compiler is organized as a pipeline of modules, transforming source text into Rust code.

```mermaid
C4Container
    title Container Diagram for ΓΛΩΣΣΑ Compiler

    Container(lexer, "Lexer", "src/grammar.rs", "Tokenizes source, handling Unicode normalization")
    Container(parser, "Parser", "src/parser", "Constructs AST, enforcing recursion limits (max depth 500)")
    Container(morphology, "Declension Resolver", "src/morphology", "Analyzes case, gender, number, and resolves agreement")
    Container(semantic, "Semantic Analyzer", "src/semantic", "Checks types, aspect, voice, and ownership")
    Container(codegen, "Code Generator", "src/codegen", "Generates Rust source code")

    Rel(lexer, parser, "Stream<Token>")
    Rel(parser, morphology, "AST (Unresolved)")
    Rel(morphology, semantic, "AST (Resolved Morphology)")
    Rel(semantic, codegen, "Analyzed Program")
```

## Semantic Analysis (C4 Component Level)

The semantic analysis phase is unique due to the slot-based assembler which enables free word order.

```mermaid
C4Component
    title Component Diagram for Semantic Analysis

    Container_Boundary(semantic, "Semantic Analysis") {
        Component(declarations, "Declarations", "src/semantic/declarations.rs", "Analyzes Types, Traits, Functions, Tests")
        Component(assembler, "Assembler", "src/semantic/assembler.rs", "Routes words to slots based on Case (Nom, Acc, Dat)")
        Component(converter, "Converter", "src/semantic/conversion.rs", "Interprets assembled slots into statements")
        Component(patterns, "Pattern Matcher", "src/semantic/patterns.rs", "Identifies high-level constructs (Iterators, Structs)")
        Component(model, "Semantic Model", "src/semantic/model.rs", "Type-checked HIR (AnalyzedStatement)")
        Component(oracle, "Oracle", "src/semantic/oracle.rs", "Generates human-readable explanations")
    }

    Container(morphology, "Morphology", "src/morphology", "Provides Case/Gender/Number analysis")

    Rel(declarations, model, "Produces AnalyzedStatement")
    Rel(morphology, assembler, "Feeds MorphAnalysis")
    Rel(assembler, converter, "Produces AssembledStatement")
    Rel(converter, patterns, "Delegates complex patterns")
    Rel(patterns, model, "Produces AnalyzedStatement")
    Rel(converter, model, "Produces AnalyzedStatement")
    Rel(oracle, assembler, "Uses (via assemble_statement)")
```
