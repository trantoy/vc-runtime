# vc-runtime roadmap

Created: 2026-05-31

## Purpose

This roadmap turns the project vision into ordered product and engineering phases.

It is intentionally not a task checklist. Detailed execution plans live under
`docs/memory/phases/`. This file defines phase order, exit gates, and what the
project should avoid until earlier evidence exists.

## North star

`vc-runtime` should become a local-first realtime voice conversion runtime that
is faster, easier to diagnose, and easier to embed than current Python-first
voice changer applications.

The runtime wins by proving:

- stable audio callback behavior;
- bounded end-to-end latency;
- explicit model and provider selection;
- visible queue growth, underruns, overruns, and provider fallback;
- reproducible benchmarks across machines;
- UI-independent backend architecture.

## Current state

Phase 0.1 is implemented.

Available today:

- Rust workspace with `vc-core`, `vc-audio`, `vc-cli`, and `vc-bench`.
- `vc list-devices` for CPAL input/output enumeration.
- `vc passthrough --seconds N` for bounded input-to-output passthrough.
- Metrics for callbacks, pushed/popped frames, underruns, overruns, and stream
  error events.
- `vc-bench` offline prerecorded-audio benchmark with report v1 and threshold
  mode.
- Linux development-machine smoke verification.

Not available today:

- long-running passthrough proof;
- Windows and macOS verification;
- resampling or channel remapping;
- chunk scheduler;
- ONNX Runtime integration;
- RVC inference;
- daemon/control API;
- UI;
- packaging.

Next hardware context:

- Work is expected to move from the laptop to a stronger main PC with an NVIDIA
  GPU and 16 GB VRAM. Use that machine for CUDA/provider evidence, not for
  undocumented architecture jumps.

## Phase sequence

### Phase 0: Audio foundation and evidence

Goal: prove the project can move realtime audio safely before adding ML.

Scope:

- keep CPAL as the initial audio backend;
- harden passthrough startup and shutdown;
- record queue depth and callback timing;
- handle common device configuration mismatches;
- run longer soak tests;
- collect first cross-platform audio evidence.

Exit gates:

- 30-minute passthrough on Linux without unbounded queue growth;
- one-second passthrough smoke on Windows and macOS;
- clear behavior for sample-rate and channel mismatches;
- startup underruns measured and explained;
- no blocking work, model inference, or UI/control calls inside audio callbacks.

Next likely phase plans:

- Phase 0.2: passthrough soak test and metrics report.
- Phase 0.3: device config negotiation, channel mapping, and resampling boundary.
- Phase 0.4: cross-platform audio smoke matrix.

### Phase 1: Realtime DSP and scheduler

Goal: create the deterministic audio block pipeline that model inference can use.

Scope:

- define internal `f32` audio frame/chunk types;
- add input/output resampling outside callbacks;
- add mono/stereo channel mapping;
- implement chunk scheduler with explicit deadlines;
- add jitter buffer semantics;
- add SOLA/crossfade boundary for generated chunks;
- expose p50/p95/p99 timing metrics for each DSP stage.

Exit gates:

- scheduler can run synthetic chunk workers with controlled latency;
- xrun metrics identify whether failure came from input, worker, or output;
- output latency stays bounded under simulated slow chunks;
- DSP tests cover sample-rate conversion, channel mapping, and boundary behavior.

### Phase 2: Inference foundation

Goal: integrate ONNX Runtime without coupling inference to audio callbacks.

Scope:

- add `vc-ort` as the provider/runtime boundary;
- implement provider probe for CPU and CUDA first;
- make provider fallback visible;
- define model session lifecycle separate from audio streams;
- run synthetic ONNX models before RVC;
- add benchmark harness for provider load time and inference time.

Exit gates:

- ORT CPU inference works in isolation;
- ORT CUDA probe reports clear available/unavailable state;
- provider availability and observed assignment are visible at the strongest
  granularity proven by ORT evidence;
- no inference API leaks into `vc-audio`;
- benchmarks are reproducible from CLI.

### Phase 3: Daemon and control API foundation

Goal: make the backend usable by CLI, tests, and later UI clients without
putting UI logic in the realtime path.

Scope:

- add `vc-daemon`;
- define session lifecycle: create, configure, start, stop, inspect;
- expose device list, provider probe state, model registry state, and metrics
  stream;
- validate config before runtime start;
- use bounded channels or immutable config swaps into realtime components;
- keep CLI smoke/benchmark commands available without UI.

Exit gates:

- CLI can control daemon sessions;
- metrics stream works without recording or exporting user voice audio;
- invalid config fails before audio starts;
- daemon restart does not require changing lower-level crate contracts;
- direct CLI harnesses used in earlier phases are clearly marked as internal
  smoke/benchmark tools, not the product control boundary.

### Phase 4: RVC MVP backend

Goal: run one strict RVC ONNX bundle through the daemon-controlled runtime with
measured realtime behavior.

Scope:

- define the first RVC bundle format;
- support explicit model validation;
- implement pitch/content/generator stages only as needed for the first bundle;
- support CPU fallback with clear latency warning;
- support CUDA as the first production accelerator path;
- record per-stage timings: pitch, content, generator, postprocess, total chunk;
- keep index retrieval optional and disabled until the base path is stable;
- keep any direct CLI RVC path as an internal benchmark harness unless an ADR
  accepts it as public product behavior.

Exit gates:

- one known-good RVC bundle runs end-to-end through a daemon-controlled session;
- at least one NVIDIA Windows or Linux machine achieves credible realtime or
  near-realtime latency;
- CPU fallback runs and clearly explains expected high-latency limitations;
- profiler explains the bottleneck for every run;
- provider fallback is never silent.

### Phase 5: Diagnostics, benchmarks, and tuning

Goal: make latency and audio failures explainable enough for users and developers.

Scope:

- add benchmark report format;
- add diagnostics export without raw audio;
- add profiler dashboard data model;
- add automatic chunk/buffer tuning experiments;
- track queue growth, accumulated delay, provider fallback, and xruns;
- add regression thresholds for core benchmarks.

Exit gates:

- a user can tell whether lag comes from audio, DSP, provider, or model stage;
- benchmark reports can compare machines and commits;
- tuning suggestions are based on measured bottlenecks, not generic presets.

### Phase 6: Cross-platform production backend

Goal: turn the backend into a dependable cross-platform runtime.

Scope:

- stabilize Windows, Linux, and macOS audio behavior;
- make Windows/Linux CUDA the first production GPU path;
- keep CPU fallback available everywhere;
- evaluate DirectML, CoreML, and OpenVINO as experimental provider packs;
- define packaging and runtime dependency checks.

Exit gates:

- supported platform matrix is explicit and tested;
- missing drivers and providers produce actionable diagnostics;
- backend can be installed and smoke-tested without developer tooling.

### Phase 7: UI client

Goal: build a serious control surface over the daemon.

Scope:

- start with a local web UI for development speed;
- show devices, sessions, model loading, provider state, and profiler;
- add Tauri only after the backend API is stable enough to package;
- keep all realtime behavior in the backend.

Exit gates:

- UI never bypasses the daemon/control API;
- profiler view maps directly to backend metrics;
- users can export diagnostics and reproduce a report from CLI.

### Phase 8: Ecosystem and advanced acceleration

Goal: grow beyond the first RVC MVP after the runtime is proven.

Scope:

- SDK for embedding;
- model-family plugin boundary;
- model conversion tools;
- optional TensorRT path for profiled NVIDIA deployments;
- optional Triton kernels for proven kernel-level hotspots;
- plugin governance and compatibility rules.

Exit gates:

- plugin boundary does not let plugins own audio devices directly;
- advanced accelerators are justified by benchmark deltas;
- ecosystem features do not weaken realtime diagnostics or core portability.

## Near-term priority

The next meaningful work should stay inside Phase 0.

Recommended order:

1. Record a Phase 0.2 plan for long-running passthrough and richer metrics.
2. Run a 30-minute Linux passthrough soak and update `phase-0/results.md`.
3. Add timestamped callback and queue-depth metrics if the current metrics are
   not enough to explain startup underruns.
4. Decide whether resampling/channel mapping belongs in Phase 0.3 or Phase 1.0.
5. Run Windows/macOS smoke tests before committing to more audio API shape.

## Defer deliberately

Do not spend major time on these until earlier gates pass:

- Tauri shell;
- custom virtual microphone driver;
- arbitrary `.pth` import;
- all model families;
- plugin ABI;
- TensorRT default path;
- Triton kernels;
- mobile support;
- cloud voice conversion.

These may become important later, but they are not allowed to hide unresolved
audio, scheduling, and diagnostics risks.

## Decision gates

The project should pause for an ADR before:

- changing the audio backend strategy;
- making a provider production-supported;
- adding a daemon protocol with compatibility promises;
- defining the RVC bundle format;
- adding plugin ABI or SDK promises;
- adding a custom accelerator path as a recommended default.

## Risks to track

Audio risk:

- CPAL behavior may differ sharply across platforms and host APIs.
- Startup underruns may be harmless or may reveal bad stream sequencing.
- Device enumeration indices are not stable identifiers.

Inference risk:

- ONNX conversion quality may become the real bottleneck before runtime speed.
- Provider fallback can make "GPU enabled" misleading unless diagnostics are
  strict.
- CPU fallback may be functional but not useful for realtime conversion.

Architecture risk:

- A daemon/session manager can become a god object if it owns audio, models,
  providers, UI state, and benchmarks directly.
- UI pressure can tempt direct mutation of realtime state.
- Advanced accelerators can fragment the runtime before the portable path is
  credible.

Process risk:

- Large agent-written PRs can create hidden contracts without ADRs.
- Missing `context.md` updates can make future agents repeat old decisions.
- Benchmarks without fixed reports can become anecdotes.
