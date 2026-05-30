# 0011. Define Phase 0 audio metrics schema

Status: accepted
Date: 2026-05-31

## Context and Problem

Phase 0.1 needs basic audio health metrics before device listing and passthrough behavior can be useful. Governance requires an ADR for changes affecting the public metrics schema.

The first schema must be small enough for passthrough, explicit about units, and independent of CPAL so it can be shared by `vc-audio`, `vc-cli`, and later diagnostics code.

## Decision Drivers

- Metrics must be cheap to update from audio callbacks.
- Field names must make units and event semantics clear.
- Snapshot reads are for health reporting, not strict transactional accounting.
- `vc-core` must remain backend-agnostic.
- The schema should be minimal for Phase 0.1.

## Considered Options

- Keep metrics private in `vc-audio`.
- Publish raw atomic counters from `vc-core`.
- Publish `AudioCounters` plus copyable `AudioMetricsSnapshot`.

## Decision

Define `AudioCounters` and `AudioMetricsSnapshot` in `vc-core`.

Initial snapshot fields:

```text
input_callbacks: u64
output_callbacks: u64
pushed_frames: u64
popped_frames: u64
underrun_events: u64
overrun_events: u64
```

`underrun_events` and `overrun_events` count events, not dropped frames or samples.

`snapshot()` returns an approximate non-transactional report. Fields are loaded independently with relaxed atomic ordering. This is acceptable for realtime health display and unsuitable for deriving strict cross-field invariants.

## Consequences

Positive:

- Audio callbacks can update counters without locks.
- CLI can print a stable first metrics shape.
- Event counters are unit-explicit.

Negative:

- The snapshot is not internally consistent at a single instant.
- Later latency histograms will need another schema decision.

## Links

- [../memory/phases/phase-0/phase-0-1-audio-passthrough-plan.md](../memory/phases/phase-0/phase-0-1-audio-passthrough-plan.md)
- [../../crates/vc-core/context.md](../../crates/vc-core/context.md)
