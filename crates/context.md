# Context

## Purpose

This folder will contain Rust crates for the runtime.

## Current Shape

The first Phase 0.1 crates have been created:

- `vc-core`
- `vc-audio`
- `vc-cli`

Expected later crates:

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
- 2026-05-31: Initial `vc-core`, `vc-audio`, and `vc-cli` skeleton crates added.

## Open Questions

- Which crate should own runtime configuration once passthrough behavior exists.
