# Context

## Purpose

`vc-core` contains small shared metric and unit types that are independent of audio backends, model runtimes, CLI, daemon, and UI.

## Current Shape

The crate currently exposes `metrics` with:

- `AudioCounters`
- `AudioMetricsSnapshot`

## Public Contracts

- `vc-core` must not depend on CPAL, ONNX Runtime, CLI, daemon, UI, or model-specific crates.
- Shared units and metric types should live here only when at least two runtime crates need them.
- Do not add generic helpers or unrelated utilities here.
- Metrics snapshots are approximate health reports, not transactional state.

## Decisions

- [../../docs/adr/0002-use-rust-for-realtime-runtime.md](../../docs/adr/0002-use-rust-for-realtime-runtime.md)
- [../../docs/adr/0011-define-phase-0-audio-metrics-schema.md](../../docs/adr/0011-define-phase-0-audio-metrics-schema.md)

## History

- 2026-05-31: Skeleton crate added for Phase 0.1.
- 2026-05-31: Phase 0 audio metrics counters added.

## Open Questions

- Whether later latency histogram types belong here or in a diagnostics crate.
