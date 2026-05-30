# Context

## Purpose

`vc-cli` exposes Phase 0.1 command-line tools.

## Current Shape

The crate is an empty skeleton. `list-devices` and `passthrough` commands will be added in later Phase 0.1 tasks.

## Public Contracts

- CLI code may depend on `vc-audio` and `vc-core`.
- CLI parsing and terminal output stay here.
- CLI code must not own audio callback logic.
- CLI behavior should report clear errors without panics.

## Decisions

- [../../docs/adr/0002-use-rust-for-realtime-runtime.md](../../docs/adr/0002-use-rust-for-realtime-runtime.md)

## History

- 2026-05-31: Skeleton crate added for Phase 0.1.

## Open Questions

- Whether command implementations live in `vc-cli` only or move into reusable library functions.
