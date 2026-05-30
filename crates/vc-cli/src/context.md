# Context

## Purpose

This folder contains `vc-cli` Rust source files.

## Current Shape

- `lib.rs` defines the crate root.
- `main.rs` defines the binary entry point.

## Public Contracts

- CLI source owns argument parsing and user-facing terminal output.
- Do not put CPAL callbacks or ring-buffer runtime code here.
- `main.rs` should stay thin and delegate behavior to the library portion of this crate.

## Decisions

- [../../../docs/adr/0002-use-rust-for-realtime-runtime.md](../../../docs/adr/0002-use-rust-for-realtime-runtime.md)

## History

- 2026-05-31: Source folder added.

## Open Questions

- What command structure is sufficient before introducing a daemon.
