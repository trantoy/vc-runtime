# Context

## Purpose

This folder contains `vc-core` Rust source files.

## Current Shape

- `lib.rs` defines the crate root.

## Public Contracts

- Source in this crate must remain backend-agnostic.
- Public types should be small, unit-explicit, and justified by cross-crate use.

## Decisions

- [../../../docs/adr/0002-use-rust-for-realtime-runtime.md](../../../docs/adr/0002-use-rust-for-realtime-runtime.md)

## History

- 2026-05-31: Source folder added.

## Open Questions

- What is the minimal metric API for Phase 0.1.
