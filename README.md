# vc-runtime

`vc-runtime` is a local-first realtime voice conversion runtime.

The project focuses on low latency, stable audio scheduling, provider
transparency, and diagnostics-first model inference. It is intended to be a
runtime backend first, not another GUI wrapper around an existing Python voice
changer.

## Direction

- Rust runtime for the realtime audio/data plane.
- CPAL-first cross-platform audio experiments.
- ONNX Runtime-first production inference path.
- RVC-first model support.
- CUDA as the first production GPU path.
- CPU fallback everywhere.
- DirectML, CoreML and OpenVINO as experimental provider paths.
- Tauri app later, after the backend is measurable and stable.

## Current Stage

This repository is in Phase 0: research and validation.

The immediate goal is to prove:

- stable audio passthrough;
- RVC ONNX viability;
- basic provider probing;
- per-stage metrics;
- reproducible benchmark reports.

## Documents

- [Repository context](context.md)
- [Glossary](docs/glossary.md)
- [Project memory](docs/memory/context.md)
- [Phase plans](docs/memory/phases/context.md)
- [Architecture decisions](docs/adr/context.md)

## Documentation Rules

- Every project folder must contain a `context.md` file.
- Project documentation is markdown-first.
- Detailed technical project memory lives in `docs/memory/`.
- Shared project terms live in `docs/glossary.md`.
- Architecture decisions live in numbered ADR files under `docs/adr/`.
- New large boundaries, providers, daemon protocols, or cross-crate dependencies require an ADR.
