# Context

## Purpose

This folder contains tests for the ORT provider probe schema experiment.

## Current Shape

- `schema.rs` checks report serialization and fixture fallback behavior.

## Public Contracts

- These tests verify experiment behavior only.
- Passing tests do not prove real ONNX Runtime provider behavior.

## Decisions

- [../../../docs/memory/runtime-architecture-v1.md](../../../docs/memory/runtime-architecture-v1.md)

## History

- 2026-05-31: Provider probe schema tests added.

## Open Questions

- Which live-probe tests should be added once ORT integration is attempted.
