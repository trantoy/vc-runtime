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

- [Vision](docs/vision.md)
- [Phase 0 research plan](docs/phase-0-research-plan.md)
- [MVP scope](docs/mvp-scope.md)
- [Full design draft](docs/full-design-draft.md)
- [Original runtime idea](docs/runtime-idea.md)
