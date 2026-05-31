# Context: vc-bench

## Purpose

`vc-bench` owns production benchmark harnesses and report generation.

## Current Shape

The first implementation is an offline prerecorded-audio benchmark promoted from
`experiments/offline-audio-bench`.

## Public Contracts

- `vc-bench` is part of the root Cargo workspace.
- It must not depend on live audio devices.
- Benchmark reports must use documented schema versions.
- Large audio fixtures do not live in this crate.

## Decisions

- Start with WAV input, simple stages, and threshold mode.
- Keep experiment code independent; production code does not import from
  `experiments/`.

## History

- 2026-05-31: Initial crate added from the offline audio benchmark prototype.

## Open Questions

- Which future stage should be promoted next: resampling, SOLA simulation, pitch,
  or ONNX inference.
