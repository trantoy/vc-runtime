# Context

## Purpose

`vc-audio` owns audio backend integration for Phase 0.1.

## Current Shape

The crate exposes CPAL-backed device listing. Passthrough runtime will be added in a later Phase 0.1 task.

## Public Contracts

- `vc-audio` may depend on `vc-core`.
- `vc-audio` must not depend on model crates, daemon code, UI code, or CLI code.
- Audio callbacks must remain minimal and must not run model inference.
- CPAL types should not leak into CLI-facing APIs unless a later ADR accepts that boundary.
- Device indices are process-local listing indices and must not be documented as stable IDs.
- Device listing is best-effort: one failed direction should become a warning, not hide the other direction.
- CPAL-returned device-name failures must be preserved as warnings rather than replaced silently.
- ALSA can print low-level probe diagnostics to stderr; Phase 0.1 reports this limitation as a structured warning instead of adding unsafe stderr interception.

## Decisions

- [../../docs/adr/0003-use-cpal-for-phase-0-audio.md](../../docs/adr/0003-use-cpal-for-phase-0-audio.md)

## History

- 2026-05-31: Skeleton crate added for Phase 0.1.
- 2026-05-31: CPAL-backed device listing added.

## Open Questions

- Which CPAL stream configuration should be selected by default.
