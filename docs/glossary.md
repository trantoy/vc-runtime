# Glossary

Created: 2026-05-30

This glossary keeps frequent and project-specific words stable across code, docs, issues, and agent sessions.

## Audio Runtime

**ALSA**
Advanced Linux Sound Architecture. The main Linux audio backend used by CPAL in Phase 0.1.

**Audio callback**
Function called by the OS audio backend to provide input frames or request output frames. It must be fast and must not block on model inference.

**Chunk**
A fixed-size block of audio passed through the voice conversion pipeline. Chunk size strongly affects latency, overhead, and quality.

**CPAL**
Cross-platform Rust audio I/O library used for Phase 0.1 device listing and passthrough.

**Crossfade**
Short blend between adjacent generated audio chunks to hide discontinuities.

**Data plane**
The realtime path that moves audio frames through buffers, DSP, and model inference workers.

**Jitter buffer**
Buffer that absorbs timing variation between producer and consumer stages.

**Ring buffer**
Fixed-size circular buffer used to move audio between realtime callbacks and worker threads.

**SOLA**
Synchronized overlap-add. A technique for aligning adjacent chunks before overlap/crossfade to reduce artifacts.

**Underrun**
Output device requests audio but the runtime has too few frames ready. Usually heard as crackling, silence, or dropout.

**Overrun**
Input produces more frames than downstream stages can consume. Usually causes dropped input frames or growing latency.

**Xrun**
Generic audio realtime failure covering underruns and overruns.

## Voice Conversion

**RVC**
Retrieval-based Voice Conversion. The first model family targeted by this project.

**F0 / pitch**
Fundamental frequency estimate used by many voice conversion models to preserve melody and intonation.

**RMVPE**
A pitch extraction model commonly used in RVC pipelines.

**ContentVec / HuBERT**
Content feature extractors used to represent linguistic content independently from speaker identity.

**Generator**
The model stage that produces converted audio or acoustic representation from features, pitch, and speaker controls.

**Index retrieval**
Optional RVC step that retrieves similar feature vectors from an index to improve timbre similarity.

## Inference and Providers

**ONNX Runtime / ORT**
Inference runtime used as the main production path.

**Execution Provider / EP**
ONNX Runtime backend for a device or accelerator, such as CUDA, TensorRT, DirectML, CoreML, OpenVINO, or CPU.

**Provider fallback**
When a requested provider cannot run a graph or subgraph and execution moves to another provider. Fallback must be visible to the user.

**CUDA EP**
ONNX Runtime provider for NVIDIA GPUs through CUDA.

**DirectML EP**
ONNX Runtime provider for Windows GPU compatibility through DirectML.

**CoreML EP**
ONNX Runtime provider for Apple platforms through CoreML.

**OpenVINO EP**
ONNX Runtime provider for Intel CPU/GPU/NPU targets through OpenVINO.

**TensorRT**
NVIDIA inference optimizer/runtime. Treated as optional post-MVP acceleration, not the first production path.

**Triton kernels**
Custom GPU kernels written with the Triton language. Useful only for specific profiled CUDA/ROCm hotspots, not as the portable runtime foundation.

## Project Process

**ADR**
Architecture Decision Record. A numbered file that records one important decision, its context, considered options, and consequences.

**MADR**
Markdown Architectural Decision Records. The ADR style used by this project.

**Context file**
`context.md` file in each folder. It records purpose, current shape, contracts, decisions, history, and open questions for that folder.

**God object**
A module or type that accumulates unrelated responsibilities and becomes hard to change safely.

**Public contract**
A module boundary, file format, API, protocol, or invariant that other parts of the project rely on.
