# 022. Decouple Storage from Core

Date: 2026-05-30

## Status

Proposed

## Context

Circular dependencies were causing build failures between the core and storage layers, making the system fragile.

## Decision

Move persistence logic to a dedicated storage crate.

## Consequences

Build times improve, but FFI complexity increases.
