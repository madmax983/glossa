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

    Container(lexer, "Lexer", "src/parser/grammar.rs", "Tokenizes source, handling Unicode normalization")
    Container(parser, "Parser", "src/parser", "Constructs AST, enforcing recursion limits (max depth 500)")
    Container(morphology, "Declension Resolver", "src/morphology", "Analyzes case, gender, number, and resolves agreement")
    Container(semantic, "Semantic Analyzer", "src/semantic", "Checks types, aspect, voice, and ownership")

    Container_Boundary(tools, "Developer Experience (Nova)") {
        Container(alchemist, "Alchemist", "src/tools/alchemist.rs", "Transpiles Analyzed Program to Python source code")
        Container(cache, "Cache", "src/tools/cache.rs", "Incremental compilation cache")
        Container(cartographer, "Cartographer", "src/tools/cartographer.rs", "Generates Mermaid Class Diagrams")
        Container(cli, "CLI", "src/tools/cli.rs", "Command-line interface definition")
        Container(dictionary, "The Lexicon", "src/tools/dictionary.rs", "The Source of Truth for Words (Dictionary)")
        Container(highlight, "Highlighter", "src/tools/highlight.rs", "Semantic syntax highlighting")
        Container(interpreter, "Interpreter", "src/tools/interpreter.rs", "In-memory tree-walk simulator")
        Container(mentor, "Mentor", "src/tools/mentor.rs", "Interactive Tutorial Mode")
        Container(mosaic, "Mosaic", "src/tools/mosaic.rs", "Visualizes Semantic Assembly")
        Container(narrator, "The Bard", "src/tools/narrator.rs", "Generates English narrative ('Scroll of Logic') from AST")
        Container(repl, "REPL", "src/tools/repl.rs", "Interactive Read-Eval-Print Loop")
        Container(report, "Reporter", "src/tools/report.rs", "Generates statistics and structured reports")
        Container(runner, "Runner", "src/tools/runner.rs", "Orchestrates the compilation pipeline")
        Container(tester, "The Judge", "src/tools/tester.rs", "Verifies Correctness (Test Runner)")
        Container(ui, "The Stage", "src/tools/ui.rs", "Presentation Layer & UI Helpers")
        Container(weave, "Weave", "src/tools/weave.rs", "Generates a 'Rosetta Stone' Markdown document")
    }

    Container(codegen, "Code Generator", "src/codegen.rs", "Generates Rust source code")

    Rel(lexer, parser, "Stream<Token>")
    Rel(parser, morphology, "AST (Unresolved)")
    Rel(parser, highlight, "AST (Unresolved)")
    Rel(morphology, semantic, "AST (Resolved Morphology)")
    Rel(semantic, report, "Analyzed Program")
    Rel(semantic, narrator, "Analyzed Program")
    Rel(semantic, cartographer, "Analyzed Program")
    Rel(semantic, mentor, "Analyzed Program")
    Rel(semantic, mosaic, "Analyzed Program")
    Rel(semantic, tester, "Analyzed Program")
    Rel(semantic, interpreter, "Analyzed Program")
    Rel(semantic, alchemist, "Analyzed Program")
    Rel(semantic, weave, "Analyzed Program")
    Rel(semantic, codegen, "Analyzed Program")

    Rel(morphology, dictionary, "Lexicon Data")
    Rel(parser, tester, "AST")
    Rel(codegen, tester, "Rust Source")
```

## Semantic Analysis (C4 Component Level)

The semantic analysis phase is unique due to the slot-based assembler which enables free word order.

```mermaid
C4Component
    title Component Diagram for Semantic Analysis

    Container_Boundary(semantic, "Semantic Analysis") {
        Component(orchestrator, "Orchestrator", "src/semantic/analyzer.rs", "Coordinates analysis pipeline")
        Component(declarations, "Declarations", "src/semantic/declarations.rs", "Analyzes Types, Traits, Functions")
        Component(control_flow, "Control Flow", "src/semantic/control_flow.rs", "Analyzes If, While, Match")
        Component(expressions, "Expressions", "src/semantic/expressions.rs", "Recursively analyzes nested expressions")
        Component(resolver, "Resolver", "src/semantic/resolver.rs", "Manages Scope and Bindings")
        Component(assembly, "Assembly", "src/semantic/assembly/mod.rs", "Routes words to grammatical slots")
        Component(conversion, "Conversion", "src/semantic/conversion.rs", "Interprets assembled slots into statements")
        Component(patterns, "Pattern Matcher", "src/semantic/patterns.rs", "Identifies high-level constructs")
        Component(model, "Semantic Model", "src/semantic/model.rs", "Type-checked HIR (AnalyzedStatement)")
        Component(types, "Type System", "src/semantic/types.rs", "GlossaType definitions and utilities")
    }

    Container(morphology, "Morphology", "src/morphology", "Provides Case/Gender/Number analysis")

    Rel(orchestrator, declarations, "Delegates to")
    Rel(orchestrator, control_flow, "Delegates to")
    Rel(orchestrator, conversion, "Delegates to")

    Rel(declarations, resolver, "Defines Symbols")
    Rel(declarations, types, "Uses")
    Rel(declarations, model, "Produces")

    Rel(control_flow, expressions, "Analyzes conditions")
    Rel(control_flow, model, "Produces")

    Rel(conversion, assembly, "Uses Assembler")
    Rel(assembly, morphology, "Uses")
    Rel(assembly, expressions, "Feeds sub-expressions")

    Rel(conversion, patterns, "Delegates complex patterns")
    Rel(conversion, resolver, "Lookups Symbols")
    Rel(conversion, model, "Produces")

    Rel(expressions, resolver, "Lookups Symbols")
    Rel(expressions, types, "Uses")

    Rel(model, types, "Uses")
```
