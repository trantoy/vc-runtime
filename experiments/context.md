# Context

## Purpose

This folder contains throwaway or semi-throwaway prototypes used to answer
architecture questions before production implementation.

## Current Shape

Experiment subfolders are created as needed. Each experiment must have its own
`context.md` and should clearly state whether its code is disposable,
reference-only, or intended to be promoted later.

## Public Contracts

- Experiment code is not production code.
- Experiments must not be added to the root Cargo workspace unless a later phase
  plan explicitly promotes them.
- Rust experiments with their own `Cargo.toml` should include a local
  `[workspace]` table so Cargo treats them as separate workspaces.
- Experiments must not edit production crates unless their prompt explicitly
  allows it.
- Results should be summarized in the experiment README or report file before
  any production architecture decision depends on them.

## Decisions

- [../docs/memory/runtime-architecture-v1.md](../docs/memory/runtime-architecture-v1.md)
- [../docs/memory/architecture-guide.md](../docs/memory/architecture-guide.md)

## History

- 2026-05-31: Experiments folder added for architecture prototypes.
- 2026-05-31: Added audio soak, scheduler, provider-probe, and offline audio benchmark prototypes.

## Open Questions

- Which experiment outputs should be promoted from prototypes into `vc-bench`.
