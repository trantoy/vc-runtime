# Context

## Purpose

This folder contains `vc-audio` Rust source files.

## Current Shape

- `lib.rs` defines the crate root.

## Public Contracts

- Source here owns audio-specific behavior only.
- Do not add CLI parsing, model runtime, or daemon control logic here.
- Realtime callback code must avoid blocking operations and heavy allocation.

## Decisions

- [../../../docs/adr/0003-use-cpal-for-phase-0-audio.md](../../../docs/adr/0003-use-cpal-for-phase-0-audio.md)

## History

- 2026-05-31: Source folder added.

## Open Questions

- Where to draw the boundary between device selection and stream runtime.
