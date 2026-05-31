# Context: chunk-scheduler-sim

## Purpose

Prototype simulator for the Phase 1 ADR question: how chunk scheduling behaves when model workers become slow relative to stream cadence.

This prototype is intentionally non-production. It simulates chunk arrival, a bounded input queue, a single worker, and playback/output deadline handling.

## Scope

- Location: `experiments/chunk-scheduler-sim/`
- Output: runnable Rust CLI + deterministic tests
- No production crates are modified.
- No dependency on the root workspace; the experiment has its own local `[workspace]` in `Cargo.toml`.

## Constraints kept in this prototype

- Inputs are synthetic chunks only (no audio, no ML model).
- Worker time is configurable as:
  - constant value (`--worker-ms`) or
  - repeating pattern (`--worker-ms-pattern`) or
  - constant + deterministic jitter (`--worker-ms-jitter` + seed)
- Policies evaluated:
  - `drop-oldest`
  - `silence-on-underrun`
  - `reuse-last`

## Outputs produced

The simulator prints JSON lines and a final summary line with:

- `accumulated_delay_ms`
- `deadline_miss_events`
- `underrun_events`
- `dropped_chunks`
- `max_queue_depth`

Event lines (`--trace`) include per-chunk scheduled output attempts.
