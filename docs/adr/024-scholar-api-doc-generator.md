# 024. Scholar API Doc Generator

Date: 2026-06-30
Status: Accepted

## Context

A language ecosystem cannot thrive without accessible documentation. Forcing developers to read Ancient Greek source files to understand an API's shape is prohibitive. We needed a bridge between the raw semantic analysis and human-readable reference material.

## Decision

We created the "Scholar" (ὁ Σχολαστικός) tool in `src/tools/scholar.rs`. It parses a ΓΛΩΣΣΑ program and automatically distills defined structures (εἴδη), traits (χαρακτῆρες), and functions (ἔργα) into comprehensive, GitHub-flavored Markdown API documentation (`doc.md`).

## Consequences

- **Positive:** Automates the creation of clean, standardized API documentation.
- **Positive:** Significantly lowers the barrier to entry for consuming Glossa libraries.
- **Negative:** The tool must be continuously updated to support new language features and syntax additions to ensure accurate documentation.
