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

    Container(lexer, "Lexer", "src/grammar", "Tokenizes source, handling Unicode normalization")
    Container(parser, "Parser", "src/grammar", "Constructs AST from tokens, handling flexible word order")
    Container(morphology, "Declension Resolver", "src/morphology", "Analyzes case, gender, number, and resolves agreement")
    Container(semantic, "Semantic Analyzer", "src/semantic", "Checks types, aspect, voice, and ownership")
    Container(ir, "IR Generator", "src/ir", "Lowers AST to Intermediate Representation")
    Container(codegen, "Code Generator", "src/codegen", "Generates Rust source code")

    Rel(lexer, parser, "Stream<Token>")
    Rel(parser, morphology, "AST (Unresolved)")
    Rel(morphology, semantic, "AST (Resolved Morphology)")
    Rel(semantic, ir, "Typed AST")
    Rel(ir, codegen, "IR")
```

## Core-Storage Decoupling (Class Level)

The following diagram illustrates the decoupled relationship between Core and Storage.

```mermaid
classDiagram
  class Core
  class Storage
  Core --> Storage : Uses (Trait Bound)
  %% Removed the circular dependency arrow
```
