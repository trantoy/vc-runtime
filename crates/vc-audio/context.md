# Context

## Purpose

`vc-audio` owns audio backend integration for Phase 0.1.

## Current Shape

The crate is an empty skeleton. CPAL device listing and passthrough runtime will be added in later Phase 0.1 tasks.

## Public Contracts

- `vc-audio` may depend on `vc-core`.
- `vc-audio` must not depend on model crates, daemon code, UI code, or CLI code.
- Audio callbacks must remain minimal and must not run model inference.
- CPAL types should not leak into CLI-facing APIs unless a later ADR accepts that boundary.

## Decisions

- [../../docs/adr/0003-use-cpal-for-phase-0-audio.md](../../docs/adr/0003-use-cpal-for-phase-0-audio.md)

## History

- 2026-05-31: Skeleton crate added for Phase 0.1.

## Open Questions

- Which CPAL stream configuration should be selected by default.
