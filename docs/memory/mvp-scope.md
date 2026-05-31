# vc-runtime MVP scope

Created: 2026-05-30

## MVP principle

The first MVP should prove that `vc-runtime` can run a measured realtime voice conversion pipeline, not that it can support every model and every machine.

The MVP should be small enough to finish and strict enough to be credible.

## Target architecture

```text
local web UI / CLI
  -> vc-daemon
  -> audio engine
  -> DSP/chunk scheduler
  -> RVC ONNX pipeline
  -> ONNX Runtime provider manager
  -> diagnostics stream
```

The daemon is the first product boundary. Tauri can wrap the same daemon later.

## Supported platforms

MVP audio passthrough target:

- Windows;
- Linux;
- macOS.

MVP production RVC target:

- Windows + NVIDIA CUDA;
- Linux + NVIDIA CUDA.

MVP fallback:

- CPU on all platforms, with clear latency warning.

Experimental only:

- Windows DirectML;
- macOS CoreML;
- Intel OpenVINO.

Not production in MVP:

- Linux AMD ROCm;
- TensorRT;
- Triton;
- custom virtual audio devices.

## Must have

Audio:

- input/output device selection;
- CPAL-based audio streams;
- passthrough mode;
- ring buffer;
- queue depth metrics;
- underrun/overrun counters;
- graceful start/stop.

DSP/runtime:

- deterministic chunk scheduler;
- input/output resampling;
- `f32` internal audio path;
- SOLA/crossfade;
- no blocking locks in audio callbacks;
- no heap allocation in audio callbacks where practical.

Model:

- one explicit RVC bundle format;
- ONNX generator;
- ONNX pitch/content where proven viable;
- CPU fallback;
- CUDA provider path.

Diagnostics:

- per-stage timing;
- p50/p95/p99 chunk time;
- provider availability and observed provider assignment at the strongest proven
  granularity;
- accumulated delay / queue growth;
- diagnostic export.

Tools:

- offline benchmark harness;
- provider probe;
- model validation/conversion notes.

UI:

- local web UI;
- device page;
- model loader;
- profiler dashboard;
- provider diagnostics.

## Should have

- basic auto-tuning for chunk size and buffer size;
- DirectML probe on Windows;
- CoreML probe on macOS;
- OpenVINO probe for Intel;
- routing assistant notes for virtual cable tools;
- model dependency manager for known pretrained assets.

## Not in MVP

- Tauri production app;
- training;
- all model families;
- plugin ABI;
- plugin marketplace;
- custom virtual microphone driver;
- TensorRT default path;
- Triton kernels;
- network audio streaming;
- mobile support;
- arbitrary model import without validation.

## MVP exit criteria

A credible MVP is done when:

1. Passthrough runs for 30 minutes without unbounded queue growth.
2. RVC ONNX works end-to-end on at least one NVIDIA Windows or Linux machine.
3. CPU fallback works and reports expected high-latency limitations.
4. The profiler explains the bottleneck for every run.
5. Provider fallback is never silent.
6. Benchmark reports are reproducible.
7. Audio callbacks never run model inference.
8. User can export diagnostics without recording voice audio.

## First repository modules

Start with fewer crates than the full design doc suggests:

```text
crates/
  vc-core
  vc-audio
  vc-dsp
  vc-ort
  vc-rvc
  vc-daemon
  vc-bench
web/
  dev-ui
tools/
  convert-rvc
```

Split more only when boundaries become real.

## Decision gates

Before moving beyond MVP, answer:

- Is ONNX RVC stable enough for users?
- Is CUDA path clearly better than Python baseline?
- Is CPAL enough for the supported audio paths?
- Are diagnostics understandable to non-developers?
- Which provider should become second production target?
