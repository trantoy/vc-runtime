# Context

## Purpose

This folder contains phase-specific project plans, results, and local memory.

## Current Shape

- `phase-0/` contains Phase 0 research and implementation planning.

## Public Contracts

- Phase folders keep plans close to results.
- Large phase plans should not live directly in `docs/memory/`.
- Every phase folder must contain its own `context.md`.
- Phase plans must state scope, exit criteria, verification commands, and review expectations.
- Each phase should keep one rolling `results.md` file unless an experiment is large enough to need its own subfolder.

## Decisions

- [../../adr/0010-store-phase-plans-under-memory-phases.md](../../adr/0010-store-phase-plans-under-memory-phases.md)

## History

- 2026-05-31: Phase planning folder added to prevent `docs/memory/` from becoming a flat dumping ground.

## Open Questions

- What threshold should trigger a dedicated experiment subfolder instead of appending to phase `results.md`.
