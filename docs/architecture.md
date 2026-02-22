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

    Container(cli, "CLI", "src/tools/cli.rs", "Command-line interface")
    Container(runner, "Runner", "src/tools/runner.rs", "Orchestrates compilation pipeline")
    Container(cache, "Cache", "src/tools/cache.rs", "Incremental compilation cache")

    Container(lexer, "Lexer", "src/parser/grammar.rs", "Tokenizes source, handling Unicode normalization")
    Container(parser, "Parser", "src/parser", "Constructs AST, enforcing recursion limits (max depth 500)")
    Container(morphology, "Declension Resolver", "src/morphology", "Analyzes case, gender, number, and resolves agreement")
    Container(semantic, "Semantic Analyzer", "src/semantic", "Checks types, aspect, voice, and ownership")
    Container(codegen, "Code Generator", "src/codegen/mod.rs", "Generates Rust source code")

    Container(highlight, "Highlighter", "src/tools/highlight.rs", "Semantic syntax highlighting")
    Container(report, "Reporter", "src/tools/report.rs", "Generates statistics and structured reports")

    Rel(cli, runner, "Invokes")
    Rel(cli, highlight, "Invokes")
    Rel(runner, cache, "Checks/Updates")
    Rel(runner, lexer, "Initiates Pipeline")

    Rel(lexer, parser, "Stream<Token>")
    Rel(parser, morphology, "AST (Unresolved)")
    Rel(parser, highlight, "AST (Unresolved)")
    Rel(morphology, semantic, "AST (Resolved Morphology)")
    Rel(semantic, report, "Analyzed Program")
    Rel(semantic, codegen, "Analyzed Program")
```

## Semantic Analysis (C4 Component Level)

The semantic analysis phase is unique due to the slot-based assembler which enables free word order.

```mermaid
C4Component
    title Component Diagram for Semantic Analysis

    Container_Boundary(semantic, "Semantic Analysis") {
        Component(statements, "Statements", "src/semantic/statements.rs", "Analyzes Control Flow and Declarations")
        Component(expressions, "Expressions", "src/semantic/expressions.rs", "Recursively analyzes nested expressions")
        Component(resolver, "Resolver", "src/semantic/resolver.rs", "Manages Scope and Bindings")
        Component(assembler, "Assembler", "src/semantic/assembler.rs", "Routes words to slots based on Case (Nom, Acc, Dat)")
        Component(converter, "Converter", "src/semantic/conversion.rs", "Interprets assembled slots into statements")
        Component(patterns, "Pattern Matcher", "src/semantic/patterns.rs", "Identifies high-level constructs (Iterators, Structs)")
        Component(model, "Semantic Model", "src/semantic/model.rs", "Type-checked HIR (AnalyzedStatement)")
    }

    Container(morphology, "Morphology", "src/morphology", "Provides Case/Gender/Number analysis")

    Rel(statements, model, "Produces AnalyzedStatement")
    Rel(statements, expressions, "Analyzes conditions")
    Rel(statements, resolver, "Defines Symbols")

    Rel(morphology, assembler, "Feeds MorphAnalysis")

    Rel(assembler, expressions, "Feeds sub-expressions")
    Rel(assembler, converter, "Produces AssembledStatement")

    Rel(converter, patterns, "Delegates complex patterns")
    Rel(converter, resolver, "Lookups Symbols")
    Rel(converter, model, "Produces AnalyzedStatement")

    Rel(expressions, resolver, "Lookups Symbols")

    Rel(patterns, model, "Produces AnalyzedStatement")
```
