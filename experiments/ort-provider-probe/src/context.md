# Context

## Purpose

This folder contains Rust source for the ORT provider probe schema experiment.

## Current Shape

- `lib.rs` contains provider probe data types, fixture logic, and unit tests.
- `main.rs` exposes a small CLI for dry-run and fixture-based reports.

## Public Contracts

- This is experiment code, not production provider code.
- The output shape is a candidate for a future ProviderManager ADR.
- The experiment must not claim live ORT/provider evidence unless a live probe is
  implemented and run.

## Decisions

- [../../../docs/memory/runtime-architecture-v1.md](../../../docs/memory/runtime-architecture-v1.md)
- [../../../docs/adr/0004-use-onnx-runtime-as-mainline-inference.md](../../../docs/adr/0004-use-onnx-runtime-as-mainline-inference.md)

## History

- 2026-05-31: ORT provider probe schema prototype added.

## Open Questions

- Which provider assignment granularity can be proven by real ONNX Runtime APIs.
