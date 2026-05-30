# Context

## Purpose

This folder will contain Rust crates for the runtime.

## Current Shape

No Rust crates have been created yet.

Expected first crates:

- `vc-core`
- `vc-audio`
- `vc-dsp`
- `vc-ort`
- `vc-rvc`
- `vc-daemon`
- `vc-bench`

## Public Contracts

- Each crate folder must contain its own `context.md`.
- Lower-level crates must not depend on higher-level crates.
- New crates require a short rationale in this file or an ADR if they change architecture.

## Decisions

- [../docs/adr/0002-use-rust-for-realtime-runtime.md](../docs/adr/0002-use-rust-for-realtime-runtime.md)

## History

- 2026-05-30: Crates folder reserved before Phase 0 implementation.

## Open Questions

- Whether Phase 0.1 starts as a single crate or a small workspace.
