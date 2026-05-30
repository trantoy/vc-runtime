# Context

## Purpose

`vc-cli` exposes Phase 0.1 command-line tools.

## Current Shape

The crate exposes `list-devices` and `passthrough` Phase 0.1 commands.

## Public Contracts

- CLI code may depend on `vc-audio` and `vc-core`.
- CLI parsing and terminal output stay here.
- CLI code must not own audio callback logic.
- CLI behavior should report clear errors without panics.
- Device indices printed by CLI are process-local listing indices.
- Device listing warnings are printed as warnings, not treated as fatal command errors.
- Backend probe limitations from the audio crate should stay in the warning section.
- The `passthrough` command may start audio sessions but must not own CPAL stream or callback logic.
- Metrics output is a CLI formatting concern over `vc-core` snapshots.
- Passthrough startup output should include the selected input/output device names because numeric indices may change between runs.

## Decisions

- [../../docs/adr/0002-use-rust-for-realtime-runtime.md](../../docs/adr/0002-use-rust-for-realtime-runtime.md)
- [../../docs/adr/0012-build-phase-0-passthrough-with-cpal-rtrb.md](../../docs/adr/0012-build-phase-0-passthrough-with-cpal-rtrb.md)

## History

- 2026-05-31: Skeleton crate added for Phase 0.1.
- 2026-05-31: `list-devices` command added.
- 2026-05-31: `passthrough` command added.

## Open Questions

- When to split command handlers into separate modules as CLI surface grows.
