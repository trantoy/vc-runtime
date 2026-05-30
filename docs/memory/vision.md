# vc-runtime vision

Created: 2026-05-30

## Summary

`vc-runtime` is a local-first cross-platform runtime for realtime voice conversion.

The project should not start as another GUI around an existing Python voice changer. The core product is a reliable realtime audio and inference backend with clear diagnostics:

- low and stable latency;
- predictable chunk processing time;
- explicit provider selection;
- visible queue depth and dropouts;
- reproducible benchmarks;
- UI-independent runtime.

Source notes:

- [full-design-draft.md](full-design-draft.md) - full design doc draft.
- [runtime-idea.md](runtime-idea.md) - earlier focused runtime idea.

## Core thesis

`vc-runtime` is a realtime audio system that happens to run voice conversion models, not a model demo that happens to capture audio.

This framing matters because most user-visible failures are system failures, not only model failures:

- latency grows because queues grow;
- crackling appears because output underruns;
- choppy audio appears because chunk boundaries are unstable;
- "GPU enabled" can still mean some stages run on CPU;
- users cannot fix settings when they cannot see the bottleneck.

## Product shape

The backend should be useful in several forms:

- Rust library for embedding;
- daemon for desktop apps and headless usage;
- CLI tools for benchmarks and model conversion;
- local web UI for development;
- Tauri app later for production desktop packaging.

The UI should be a client of the runtime. It should not own the realtime path.

## Technical direction

Main choices:

- Rust for audio/data-plane runtime.
- CPAL first for cross-platform audio.
- ONNX Runtime first for production inference.
- RVC first as the initial model family.
- CUDA first-class for NVIDIA production path.
- CPU fallback everywhere.
- DirectML, CoreML and OpenVINO as experimental/provider-pack paths at first.
- PyTorch only for conversion, validation and development fallback.
- TensorRT and Triton only after profiling proves a specific need.

## Non-goals for early project

Do not start with:

- training;
- all model families;
- custom virtual audio driver;
- Tauri polish;
- TensorRT by default;
- Triton kernels;
- plugin marketplace;
- cloud voice conversion;
- arbitrary `.pth` support without conversion constraints.

## Differentiator

The project wins if users and developers can answer:

- What is my end-to-end latency?
- Which stage is slow?
- Is my model actually running on GPU?
- Is the output crackling because of model quality or buffer underrun?
- Is delay accumulating over time?
- What setting should I change and why?

Diagnostics are not a developer-only feature. They are part of the product.

## Long-term shape

The long-term platform can support multiple model families and richer UI, but only after the runtime proves itself.

The intended growth path:

1. measurable realtime audio loop;
2. RVC ONNX path;
3. provider manager and profiler;
4. local web UI;
5. production Tauri shell;
6. plugin model families;
7. advanced accelerators;
8. SDK/ecosystem.
