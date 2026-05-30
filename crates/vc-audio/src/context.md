# Context

## Purpose

This folder contains `vc-audio` Rust source files.

## Current Shape

- `lib.rs` defines the crate root.
- `devices.rs` lists input and output devices using CPAL and reports partial failures as warnings.

## Public Contracts

- Source here owns audio-specific behavior only.
- Do not add CLI parsing, model runtime, or daemon control logic here.
- Realtime callback code must avoid blocking operations and heavy allocation.
- Public structs should avoid exposing CPAL types outside this crate.
- Device reports use process-local indices and best-effort warning strings.
- CPAL device-name errors and known ALSA probe-noise limitations are surfaced through warning strings.

## Decisions

- [../../../docs/adr/0003-use-cpal-for-phase-0-audio.md](../../../docs/adr/0003-use-cpal-for-phase-0-audio.md)

## History

- 2026-05-31: Source folder added.
- 2026-05-31: `devices.rs` added.

## Open Questions

- Where to draw the boundary between device selection and stream runtime.
