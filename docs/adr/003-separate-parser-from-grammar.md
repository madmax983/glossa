# 3. Separate Parser from Grammar

Date: 2024-10-24
Status: Accepted

## Context

Originally, the compiler's frontend logic was heavily concentrated in the `grammar` module. This module was responsible for both defining the PEG grammar (using `pest`) and constructing the Abstract Syntax Tree (AST) from the parse results.

As the language grew (adding Traits, Lambda expressions, and complex morphological rules), the complexity of converting the Concrete Syntax Tree (CST) to the AST increased. Mixing tokenization logic with AST construction logic made the code harder to navigate and test.

## Decision

We have separated the AST construction logic into a dedicated `parser` module (`src/parser`), while keeping the grammar definition and tokenization in the `grammar` module (`src/grammar`).

- `src/grammar`: Contains `glossa.pest` and normalization logic. Responsible for producing a stream of tokens.
- `src/parser`: Contains the logic to consume those tokens and build the `Program` AST.

## Consequences

- **Separation of Concerns**: The `grammar` module focuses solely on the "shape" of the text, while `parser` focuses on the "structure" of the program.
- **Maintainability**: Changes to the AST builder do not require touching the grammar definition, and vice versa.
- **Clarity**: The architecture diagram now explicitly reflects this split, making the pipeline easier to understand for new contributors.
