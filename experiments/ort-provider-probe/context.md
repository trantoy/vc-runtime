# Context

## Purpose

This prototype explores a minimal, low-risk path for probing ONNX Runtime provider
assignment assumptions before production integration.

## Scope

- Standalone Rust prototype under `experiments/ort-provider-probe/`.
- Probe intent and output contracts only; no changes to runtime crates.
- Output schema must support `requested_provider`, `provider_probe_status`,
  `observed_provider_assignment`, `provider_assignment_granularity`, and `errors`.
- No edits outside this folder.
- No root workspace membership; this is a local experiment workspace.

## Open Questions Addressed

- What is provable in a stubbed probe environment without CUDA/ORT installed?
- What part of the observed assignment claim is safe to report as evidence?
- How to represent assignment granularity to avoid over-claiming per-stage detail.

## Design Choice

The experiment intentionally includes:

- A deterministic default dry-run mode (no runtime deps needed).
- A fixture-driven mode to model host/provider availability without touching
  hardware.
- A schema-first output for easy integration into later diagnostics/telemetry.
