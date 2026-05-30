# Context

## Purpose

This folder contains `vc-core` Rust source files.

## Current Shape

- `lib.rs` defines the crate root.
- `metrics.rs` defines Phase 0 audio metrics counters and snapshots.

## Public Contracts

- Source in this crate must remain backend-agnostic.
- Public types should be small, unit-explicit, and justified by cross-crate use.
- Snapshot fields must name their units or event semantics explicitly.
- Stream error counters are event counts and must not imply detailed error diagnostics.

## Decisions

- [../../../docs/adr/0002-use-rust-for-realtime-runtime.md](../../../docs/adr/0002-use-rust-for-realtime-runtime.md)
- [../../../docs/adr/0011-define-phase-0-audio-metrics-schema.md](../../../docs/adr/0011-define-phase-0-audio-metrics-schema.md)

## History

- 2026-05-31: Source folder added.
- 2026-05-31: `metrics.rs` added.
- 2026-05-31: `metrics.rs` extended with stream error event counters.

## Open Questions

- Whether callback duration statistics should be raw counters or a separate histogram type.
