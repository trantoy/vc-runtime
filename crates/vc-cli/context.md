# Context

## Purpose

`vc-cli` exposes Phase 0.1 command-line tools.

## Current Shape

The crate exposes the `list-devices` command. `passthrough` will be added in a later Phase 0.1 task.

## Public Contracts

- CLI code may depend on `vc-audio` and `vc-core`.
- CLI parsing and terminal output stay here.
- CLI code must not own audio callback logic.
- CLI behavior should report clear errors without panics.
- Device indices printed by CLI are process-local listing indices.
- Device listing warnings are printed as warnings, not treated as fatal command errors.
- Backend probe limitations from the audio crate should stay in the warning section.

## Decisions

- [../../docs/adr/0002-use-rust-for-realtime-runtime.md](../../docs/adr/0002-use-rust-for-realtime-runtime.md)

## History

- 2026-05-31: Skeleton crate added for Phase 0.1.
- 2026-05-31: `list-devices` command added.

## Open Questions

- Whether command implementations live in `vc-cli` only or move into reusable library functions.
