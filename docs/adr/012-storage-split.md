# Decouple Storage from Core

Date: 2026-07-08
Status: Proposed

## Context
Circular dependencies were causing build failures.

## Decision
Move persistence logic to a dedicated crate.

## Consequences
Build times improve, but FFI complexity increases.
