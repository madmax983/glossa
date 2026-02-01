# 2. Diagrams as Code

Date: 2024-10-24
Status: Accepted

## Context

Architecture diagrams are essential for understanding the system, but binary images (PNG, JPG) are difficult to maintain, version control, and keep up-to-date. They often become stale and misleading.

## Decision

We will use **Mermaid.js** for all architectural diagrams.
- All diagrams must be text-based.
- Diagrams must be embedded directly in Markdown files.
- Binary images for architecture diagrams are forbidden.

## Consequences

- Diagrams are version-controlled alongside code.
- Diagrams are easier to update (just edit text).
- Rendering depends on the platform (GitHub supports Mermaid natively).
