# Context

## Purpose

This folder contains `vc-audio` Rust source files.

## Current Shape

- `lib.rs` defines the crate root.
- `devices.rs` lists input and output devices using CPAL and reports partial failures as warnings.
- `passthrough.rs` starts default or indexed CPAL streams and moves samples through an `rtrb` ring buffer.

## Public Contracts

- Source here owns audio-specific behavior only.
- Do not add CLI parsing, model runtime, or daemon control logic here.
- Realtime callback code must avoid blocking operations and heavy allocation.
- Public structs should avoid exposing CPAL types outside this crate.
- Device reports use process-local indices and best-effort warning strings.
- CPAL device-name errors and known ALSA probe-noise limitations are surfaced through warning strings.
- Audio callbacks in `passthrough.rs` must avoid locks, heap allocation, sleeps, and model inference.
- CPAL stream error callbacks in `passthrough.rs` also avoid printing and only increment counters.
- Phase 0.1 supports passthrough only when input/output default configs have the same sample rate and channel count.

## Decisions

- [../../../docs/adr/0003-use-cpal-for-phase-0-audio.md](../../../docs/adr/0003-use-cpal-for-phase-0-audio.md)
- [../../../docs/adr/0012-build-phase-0-passthrough-with-cpal-rtrb.md](../../../docs/adr/0012-build-phase-0-passthrough-with-cpal-rtrb.md)

## History

- 2026-05-31: Source folder added.
- 2026-05-31: `devices.rs` added.
- 2026-05-31: `passthrough.rs` added.

## Open Questions

- Whether channel remapping belongs in passthrough or a later DSP crate.
