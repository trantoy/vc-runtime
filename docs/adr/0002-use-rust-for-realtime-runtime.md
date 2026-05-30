# 0002. Use Rust for realtime runtime

Status: accepted
Date: 2026-05-30

## Context and Problem

The project needs a low-latency audio runtime with predictable memory behavior, explicit ownership, and cross-platform deployment. The existing voice changer ecosystem often uses Python as orchestration around ML libraries, which is useful for experimentation but less attractive for the realtime audio data plane.

## Decision Drivers

- Audio callbacks should avoid garbage collection and unpredictable runtime pauses.
- The backend should be embeddable, daemon-friendly, and cross-platform.
- The project should be able to expose Rust, C ABI, CLI, and daemon interfaces later.
- The core runtime should separate realtime audio from Python model experimentation.

## Considered Options

- Python runtime.
- C++ runtime.
- Rust runtime.
- TypeScript/Electron runtime.

## Decision

Use Rust for the realtime runtime and core backend.

Python may still be used for conversion tools, experiments, and reference implementations.

## Consequences

Positive:

- Strong ownership and concurrency model for audio buffers and worker boundaries.
- Good fit for CLI, daemon, and future Tauri integration.
- Easier to enforce low-level module boundaries than in a dynamic runtime.

Negative:

- Some ML tooling has better first-class Python support.
- ONNX Runtime and provider packaging will need careful Rust integration.
- Contributors need Rust experience.

## Links

- [../memory/architecture.md](../memory/architecture.md)
