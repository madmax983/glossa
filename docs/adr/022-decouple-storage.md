# 022. Decouple Storage from Core

Date: 2026-05-09

## Status

Proposed

## Context

Circular dependencies were causing build failures and preventing isolated testing of the core compiler logic. The core engine and storage mechanism were tightly coupled.

## Decision

Move persistence logic to a dedicated `storage` crate. The core module will now interface with storage through strict trait bounds rather than direct concrete dependencies.

## Consequences

Build times improve significantly due to decoupled compilation units, and the architecture is cleaner. However, FFI complexity increases slightly due to the trait boundary.
