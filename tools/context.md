# Context

## Purpose

This folder will contain project tools, conversion utilities, and scripts.

## Current Shape

No tools have been created yet.

Likely future tools:

- RVC model conversion experiments;
- provider probing helpers;
- benchmark report generation;
- documentation checks.

## Public Contracts

- Tools must not become hidden production runtime dependencies unless promoted through an ADR.
- Tools that generate artifacts should document inputs, outputs, and reproducibility.

## Decisions

- [../docs/adr/0004-use-onnx-runtime-as-mainline-inference.md](../docs/adr/0004-use-onnx-runtime-as-mainline-inference.md)

## History

- 2026-05-30: Tools folder reserved before Phase 0 implementation.

## Open Questions

- Which model conversion tooling should be Python and which should be Rust.
