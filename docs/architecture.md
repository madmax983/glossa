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
    Container(highlight, "Highlighter", "src/tools/highlight.rs", "Semantic syntax highlighting")
    Container(narrator, "Narrator", "src/tools/narrator.rs", "Generates English narrative from AST")
    Container(report, "Reporter", "src/report.rs", "Generates statistics and structured reports")
    Container(codegen, "Code Generator", "src/codegen", "Generates Rust source code")

    Rel(lexer, parser, "Stream<Token>")
    Rel(parser, morphology, "AST (Unresolved)")
    Rel(parser, highlight, "AST (Unresolved)")
    Rel(morphology, semantic, "AST (Resolved Morphology)")
    Rel(semantic, report, "Analyzed Program")
    Rel(semantic, narrator, "Analyzed Program")
    Rel(semantic, codegen, "Analyzed Program")
```

## Semantic Analysis (C4 Component Level)

The semantic analysis phase is unique due to the slot-based assembler which enables free word order.

```mermaid
C4Component
    title Component Diagram for Semantic Analysis

    Container_Boundary(semantic, "Semantic Analysis") {
        Component(statements, "Statements", "src/semantic/statements.rs", "Analyzes Declarations & Control Flow")
        Component(expressions, "Expressions", "src/semantic/expressions.rs", "Recursive Expression Analysis")
        Component(resolver, "Resolver", "src/semantic/resolver.rs", "Scope & Name Resolution")
        Component(assembler, "Assembler", "src/semantic/assembler.rs", "Routes words to slots (SVO)")
        Component(converter, "Converter", "src/semantic/conversion.rs", "Converts Slots to Statements")
        Component(patterns, "Patterns", "src/semantic/patterns.rs", "Identifies complex patterns")
        Component(model, "Semantic Model", "src/semantic/model.rs", "Type-checked HIR")
    }

    Container(morphology, "Morphology", "src/morphology", "Lexicon & Analysis")

    Rel(statements, resolver, "Defines Names")
    Rel(statements, expressions, "Uses (Conditions/Values)")
    Rel(statements, assembler, "Uses (Simple Stmts)")
    Rel(statements, converter, "Uses (After Assembly)")
    Rel(statements, model, "Produces AnalyzedStatement")

    Rel(expressions, resolver, "Lookups")
    Rel(expressions, assembler, "Feeds (Recursion)")

    Rel(assembler, morphology, "Uses Analysis")
    Rel(assembler, converter, "Produces AssembledStatement")

    Rel(converter, patterns, "Delegates")
    Rel(converter, model, "Produces AnalyzedStatement")
```
