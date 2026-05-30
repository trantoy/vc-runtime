# Context

## Purpose

`vc-core` contains small shared metric and unit types that are independent of audio backends, model runtimes, CLI, daemon, and UI.

## Current Shape

The crate is an empty skeleton for Phase 0.1.

## Public Contracts

- `vc-core` must not depend on CPAL, ONNX Runtime, CLI, daemon, UI, or model-specific crates.
- Shared units and metric types should live here only when at least two runtime crates need them.
- Do not add generic helpers or unrelated utilities here.

## Decisions

- [../../docs/adr/0002-use-rust-for-realtime-runtime.md](../../docs/adr/0002-use-rust-for-realtime-runtime.md)

## History

- 2026-05-31: Skeleton crate added for Phase 0.1.

## Open Questions

- Which metrics belong in `vc-core` versus `vc-audio`.
