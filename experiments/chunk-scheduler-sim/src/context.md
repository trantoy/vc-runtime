# Context

## Purpose

This folder contains the Rust source for the chunk scheduler simulator
experiment.

## Current Shape

- `lib.rs` contains deterministic simulation logic and tests.
- `main.rs` exposes a small CLI over the simulator.

## Public Contracts

- This is experiment code, not production scheduler code.
- The simulator may influence future scheduler ADRs, but its types are not
  runtime public contracts.
- Keep the simulation deterministic so policy comparisons are reproducible.

## Decisions

- [../../../docs/memory/runtime-architecture-v1.md](../../../docs/memory/runtime-architecture-v1.md)

## History

- 2026-05-31: Chunk scheduler simulator prototype added.

## Open Questions

- Which simulator policies should be promoted into a Phase 1 scheduler ADR.
