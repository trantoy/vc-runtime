# Glossary

Created: 2026-05-30

This glossary keeps frequent and project-specific words stable across code, docs, issues, and agent sessions.

## Audio Runtime

**ALSA**
Advanced Linux Sound Architecture. The main Linux audio backend used by CPAL in Phase 0.1.

**Audio callback**
Function called by the OS audio backend to provide input frames or request output frames. It must be fast and must not block on model inference.

**Audio frame**
One timestamped set of samples across all channels. A stereo frame has two samples.

**Chunk**
A fixed-size block of audio passed through the voice conversion pipeline. Chunk size strongly affects latency, overhead, and quality.

**CPAL**
Cross-platform Rust audio I/O library used for Phase 0.1 device listing and passthrough.

**Crossfade**
Short blend between adjacent generated audio chunks to hide discontinuities.

**Data plane**
The realtime path that moves audio frames through buffers, DSP, and model inference workers.

**Capture queue**
Bounded queue that stores input frames after capture and before DSP or chunk scheduling.

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

**Passthrough**
Runtime mode that forwards input audio to output audio without voice conversion. It is used to prove device, callback, buffer, and metrics behavior before model inference.

**Sample rate**
Number of audio frames per second, measured in hertz.

**Sample format**
Numeric representation of a single audio sample, such as `f32`, `i16`, or `u16`.

**Stream error event**
Invocation of an audio backend stream error callback. In Phase 0.1 it is counted without storing formatted error text to keep callbacks realtime-safe.

**Xrun**
Generic audio realtime failure covering underruns and overruns.

**Output queue**
Bounded queue that stores generated output frames before the playback callback consumes them.

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

**Model bundle**
Validated local package containing model artifacts and metadata needed to load a model family without executing arbitrary training or conversion code at runtime.

**Model family adapter**
Boundary that defines how one model family is validated, configured, loaded, run, and reported through common runtime contracts.

## Inference and Providers

**ONNX Runtime / ORT**
Inference runtime used as the main production path.

**Execution Provider / EP**
ONNX Runtime backend for a device or accelerator, such as CUDA, TensorRT, DirectML, CoreML, OpenVINO, or CPU.

**Provider fallback**
When a requested provider cannot run a graph or subgraph and execution moves to another provider. Fallback must be visible to the user.

**Provider probe**
Runtime check that reports whether an execution provider is unavailable, available, usable for a target model, partially falling back, or failed.

**Provider assignment granularity**
How precisely the runtime can prove which provider executed work, such as provider availability only, graph/session-level assignment, or later stage-level assignment.

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

**Control plane**
The non-realtime part of the runtime that owns config, session lifecycle, device/model/provider selection, daemon APIs, and diagnostics coordination.

**Extension point**
Deliberate boundary where future implementations can be added without changing the realtime core, such as model-family adapters or provider packs.

**Internal extension seam**
In-repository interface that keeps a future extension point testable before committing to a public plugin ABI.

**Plugin ABI**
Stable binary or protocol contract for external plugins. It is a future public contract and requires a dedicated ADR before implementation.

**Normative guidance**
Project documentation that contributors and agents must follow during design and review, while remaining subordinate to accepted ADRs.

**Roadmap**
Ordered product and engineering phase map. It defines sequence, exit gates, and deferred work, but does not replace detailed phase plans.

**Evidence gate**
Concrete proof required before moving to a later phase or making a stronger support promise. Examples include soak tests, cross-platform smoke tests, benchmark reports, and provider probes.

**Prerecorded-audio benchmark**
Offline benchmark that reads a stable audio file, slices it into deterministic chunks, runs one or more measured processing stages, and emits a reproducible report. It does not use live audio devices.

**Architecture fitness check**
Manual or automated check that keeps architecture rules true over time, such as dependency direction checks, file-size reports, public API drift checks, and benchmark regression checks.

**C4 model**
Architecture documentation model that describes a system at context, container, component, and code levels.

**Hyrum's Law**
Observation that when enough users depend on an API, every observable behavior can become a dependency, even if it was not documented as a formal contract.
