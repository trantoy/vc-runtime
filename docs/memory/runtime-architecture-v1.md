# vc-runtime runtime architecture v1

Created: 2026-05-31
Status: draft target architecture, normative guidance under [ADR 0013](../adr/0013-treat-architecture-memory-docs-as-normative-guidance.md)

## Purpose

This document defines the long-term runtime architecture for `vc-runtime` and
the path from the current Phase 0 passthrough into a full realtime voice
conversion platform.

It intentionally designs the final shape early, because the project already
knows the major destinations:

- cross-platform Rust audio runtime;
- deterministic DSP and chunk scheduling;
- ONNX Runtime as the mainline inference path;
- RVC as the first model family;
- daemon-first backend;
- diagnostics-first UX;
- local web UI and later Tauri app;
- optional model families, providers, accelerators, SDK, and plugins.

The implementation must still grow incrementally through evidence gates. This
document defines seams and evolution paths. It does not authorize building empty
frameworks before the phase that needs them.

## Source documents

This document is constrained by accepted ADRs and project memory:

- [architecture-guide.md](architecture-guide.md) - maintainability rules.
- [architecture.md](architecture.md) - short architecture summary.
- [roadmap.md](roadmap.md) - phase sequence and exit gates.
- [mvp-scope.md](mvp-scope.md) - first credible MVP boundary.
- [vision.md](vision.md) - product and technical direction.
- [project-governance.md](project-governance.md) - process rules.
- [../adr/0002-use-rust-for-realtime-runtime.md](../adr/0002-use-rust-for-realtime-runtime.md)
- [../adr/0003-use-cpal-for-phase-0-audio.md](../adr/0003-use-cpal-for-phase-0-audio.md)
- [../adr/0004-use-onnx-runtime-as-mainline-inference.md](../adr/0004-use-onnx-runtime-as-mainline-inference.md)
- [../adr/0005-use-daemon-first-architecture.md](../adr/0005-use-daemon-first-architecture.md)
- [../adr/0011-define-phase-0-audio-metrics-schema.md](../adr/0011-define-phase-0-audio-metrics-schema.md)
- [../adr/0012-build-phase-0-passthrough-with-cpal-rtrb.md](../adr/0012-build-phase-0-passthrough-with-cpal-rtrb.md)
- [../adr/0013-treat-architecture-memory-docs-as-normative-guidance.md](../adr/0013-treat-architecture-memory-docs-as-normative-guidance.md)

If this document conflicts with an accepted ADR, the ADR wins until superseded
by another ADR.

## Architecture drivers

The architecture is optimized for these drivers, in order:

1. Realtime safety: audio callbacks must remain small, bounded, and predictable.
2. Stable latency: queues must not grow silently.
3. Diagnostics: every stage must explain latency, fallback, overload, and
   dropouts.
4. Cross-platform backend: Windows, Linux, and macOS audio should share one
   architecture even when platform-specific implementations are needed.
5. Provider visibility: CPU, CUDA, DirectML, CoreML, OpenVINO, TensorRT, and
   Triton choices must be explicit and measured.
6. Model extensibility: RVC first, later model families through a controlled
   adapter/plugin boundary.
7. UI independence: CLI, web UI, Tauri, and SDK clients must not own realtime
   behavior.
8. Incremental delivery: each phase must produce working software and evidence.
9. Maintainability: modules should change for one reason and have narrow public
   contracts.

## Non-goals

These are not goals for the first architecture implementation:

- training models;
- arbitrary `.pth` loading in the realtime runtime;
- custom virtual microphone driver;
- cloud voice conversion;
- mobile support;
- plugin marketplace;
- TensorRT or Triton as default inference paths;
- all model families from the start;
- GUI-owned audio or inference runtime.

The architecture leaves space for some of these later, but no code should be
added solely to support them before the relevant evidence gate.

## Current baseline

Current Phase 0.1 state:

```text
vc-cli
  -> vc-audio
      -> CPAL input callback
      -> rtrb ring buffer
      -> CPAL output callback
  -> vc-core metrics
```

Current crates:

- `vc-core`: backend-agnostic audio counters and snapshots.
- `vc-audio`: CPAL device listing and CPAL/rtrb passthrough.
- `vc-cli`: `list-devices` and bounded `passthrough` commands.

Current evidence:

- workspace builds, formats, clippy-checks, and tests on the development
  machine;
- device listing works on the Linux development machine;
- one-second passthrough works on the Linux development machine;
- startup underruns were observed and need later explanation.

Current limitations:

- no long-running passthrough proof;
- no Windows or macOS smoke proof;
- no sample-rate conversion;
- no channel mapping;
- no chunk scheduler;
- no model inference;
- no daemon;
- no UI;
- no provider manager.

## End-state overview

The intended final shape is a local-first runtime with multiple clients, one
control boundary, and a realtime data plane that remains independent of UI and
process packaging.

```text
Clients
  CLI
  local web UI
  Tauri app
  SDK / embedding API
  benchmark tools

Control plane
  vc-daemon
  SessionController
  ConfigService
  DeviceService
  ModelRegistry
  ProviderManager
  DiagnosticsService

Realtime data plane
  AudioEngine
  CaptureQueue
  DspPipeline
  ChunkScheduler
  InferenceWorkerPool
  OutputQueue
  Playback callback

Model plane
  ModelBundle
  ModelFamilyAdapter
  RvcPipeline
  future model-family adapters

Provider plane
  OrtRuntime
  CPU EP
  CUDA EP
  DirectML EP experimental
  CoreML EP experimental
  OpenVINO EP experimental
  TensorRT future
  Triton kernels future

Observability
  MetricsRegistry
  DiagnosticsStream
  BenchmarkRunner
  ReportExporter
  ProfilerViewModel
```

## C4 context view

`vc-runtime` is one local system used by human users, developers, and host
applications.

```text
Human user
  -> CLI / web UI / Tauri app
  -> vc-runtime daemon
  -> OS audio backend
  -> local audio devices / virtual cable tools

Developer / integrator
  -> SDK / CLI / benchmark tools
  -> vc-runtime crates and daemon

Model author or converter
  -> model conversion tools
  -> model bundle
  -> vc-runtime model loader

External systems
  OS audio APIs: WASAPI, CoreAudio, ALSA, PipeWire, JACK through CPAL or later
  native backends
  inference providers: ONNX Runtime execution providers and optional future
  accelerator runtimes
```

`vc-runtime` owns local processing and diagnostics. It does not own Discord,
OBS, virtual cable applications, GPU drivers, OS audio stack behavior, or model
training.

## C4 container view

Runtime containers:

```text
vc-cli
  command-line control, smoke tests, benchmarks, conversion entry points

vc-daemon
  local control API, session lifecycle, metrics stream, model/provider/device
  orchestration

runtime library crates
  reusable Rust implementation units for audio, DSP, inference, model family
  pipelines, diagnostics, and benchmarking

local web UI
  development and diagnostics UI over daemon API

Tauri app
  production desktop shell over daemon or embedded library after boundaries are
  stable

model bundle store
  validated local model artifacts and metadata

diagnostics report store
  exported metrics, environment probes, and benchmark results without raw user
  voice audio
```

Container rule:

- clients can call the daemon/control API;
- clients cannot mutate realtime buffers;
- daemon can coordinate runtime crates;
- runtime crates do not know which UI client exists;
- model bundles are data, not executable plugins in the MVP.

## Component view

### Control plane components

`SessionController`

- Owns session lifecycle: create, configure, start, pause, stop, inspect.
- Owns state transitions and validates that session config is complete before
  data-plane start.
- Does not own audio callbacks, model inference implementation, or UI state.

`ConfigService`

- Owns loading, validating, merging, and explaining config.
- Produces immutable runtime snapshots or safe update messages.
- Does not write directly into realtime state.

`DeviceService`

- Owns device enumeration, device capabilities, default-device resolution, and
  user-facing device IDs.
- Delegates backend-specific probing to `vc-audio`.
- Does not run streams.

`ModelRegistry`

- Owns known model bundles, validation state, model metadata, compatibility
  checks, and model-family selection.
- Does not run inference itself.

`ProviderManager`

- Owns provider discovery, preference order, availability, fallback policy, and
  provider diagnostics.
- Owns provider policy in the control plane, but does not instantiate ORT
  sessions directly in the MVP architecture.
- Receives ORT-specific availability and fallback details through model/provider
  abstractions owned below the daemon boundary.
- Does not hide fallback.

`DiagnosticsService`

- Owns metrics aggregation, snapshots, event streams, report export, and
  redaction policy.
- Does not record raw voice audio unless a future explicit user-approved debug
  mode is designed.

### Control-plane ownership matrix

These components coordinate lower-level services, so their non-ownership rules
are as important as their ownership rules.

| Component | Owns | Does not own | Calls / depends on | Public contract risk |
| --- | --- | --- | --- | --- |
| `SessionController` | Session state machine, lifecycle transitions, start/stop orchestration, config revision selected for a session. | Audio callbacks, queue internals, model execution kernels, provider-specific session objects, UI state. | Config, device, model, provider policy, diagnostics, audio session APIs. | Session states and errors become daemon API contracts. |
| `ConfigService` | Config schema, validation, profile merging, migration, config explanations. | Realtime mutation, audio stream handles, model sessions. | Files/env/defaults and domain validators. | Config keys, defaults, and validation errors become user-visible contracts. |
| `DeviceService` | User-facing device inventory, capability reports, stable device identity policy once defined. | Running audio streams, callback code, model routing. | `vc-audio` probes and OS/backend capability data. | Device IDs and capability names can become CLI/API contracts. |
| `ModelRegistry` | Bundle inventory, validation state, model metadata, model-family selection. | Inference execution, provider sessions, audio devices. | `vc-model`, model-family adapters, bundle storage. | Bundle IDs, validation errors, and family names become contracts. |
| `ProviderManager` | Provider preference policy, fallback policy, availability summaries, user-facing provider diagnostics. | ORT session construction in MVP, model-family stage execution, audio deadlines. | Provider probes exposed by lower-level provider/model adapters. | Provider states and fallback decisions become user-visible contracts. |
| `DiagnosticsService` | Metrics aggregation, event stream, redaction, report export, schema compatibility. | Raw voice recording by default, realtime callback formatting/logging, benchmark execution ownership. | Runtime metric snapshots, event channels, report exporters. | Metric names, units, report schema, and redaction policy become contracts. |

### Data plane components

`AudioEngine`

- Owns input/output stream lifecycle and callback boundaries.
- Uses preallocated buffers and bounded communication.
- Delegates device enumeration to `DeviceService`/`vc-audio` APIs.
- Does not know about RVC, UI, daemon protocol, or provider internals.

`CaptureQueue`

- Owns captured frames waiting for DSP/scheduling.
- Reports queue depth, push/drop behavior, and overrun events.

`DspPipeline`

- Owns sample-rate conversion, channel mapping, normalization boundaries,
  framing helpers, and output postprocessing.
- Does not own model-family semantics.

`ChunkScheduler`

- Owns chunk formation, deadlines, worker dispatch, and accumulated-delay
  policy.
- Decides whether to drop, reuse, bypass, or degrade when workers are late.
- Does not know UI or provider-specific APIs.

`InferenceWorkerPool`

- Owns non-callback model execution tasks.
- Runs model-family pipelines through explicit interfaces.
- Reports per-stage timing and deadline misses.

`OutputQueue`

- Owns generated frames waiting for playback.
- Reports underruns, queue depth, and latency contribution.

### Data-plane contract matrix

These contracts prevent realtime queues and schedulers from becoming implicit
global state.

| Boundary | Owner crate | Producer | Consumer | Item type | Capacity unit | Allocation/blocking policy | Drop/overload policy | Required metrics |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `CaptureQueue` | `vc-audio` initially; callback-facing queue remains hidden behind `vc-audio` even if scheduling moves later. | Input audio callback. | DSP/scheduler worker outside callback. | Interleaved `f32` audio frames or a later `AudioFrameBatch` domain type. | Frames, not bytes or samples. | Preallocated before stream start; callback push is nonblocking; no logging or formatting in callback. | On full queue, increment overrun and apply configured bounded policy such as drop-oldest/drop-newest; never grow unbounded. | input callbacks, pushed frames, input queue frames, overrun events, input stream error events. |
| `DspPipeline` | `vc-dsp`. | Scheduler or worker thread. | Scheduler, model worker, or output preparation stage. | `AudioChunk<f32>` target type after Phase 1; current Phase 0 code may use raw frames only. | No unbounded queue owned by DSP; working buffers sized by chunk/frame config. | Working buffers are allocated or pooled outside callbacks; DSP does not block on control plane. | Returns explicit error/deadline status; it does not silently drop audio unless instructed by scheduler policy. | resample in/out ms, channel map ms, SOLA/crossfade ms, DSP error events. |
| `ChunkScheduler` | `vc-dsp` or a later narrow scheduler crate if an ADR accepts the split. | Capture queue and DSP preprocessing. | Inference worker pool and output preparation. | `ScheduledChunk` target type containing audio, timing, deadline, and config revision. | Chunks and deadline window duration. | Runs outside callbacks; dispatch queues are bounded; does not allocate per callback. | Drops, bypasses, reuses, or degrades only through explicit overload policy; records deadline misses. | chunk index, chunk frames, hop frames, schedule ms, deadline miss events, accumulated delay ms. |
| `InferenceWorkerPool` | Model runtime integration crate or daemon-owned worker orchestration; exact owner requires a later scheduler/inference ADR. | Chunk scheduler. | Model-family pipeline and output preparation. | `ModelWorkItem` target type with chunk, model config, provider policy, and diagnostics span. | Work items and max in-flight chunks. | Runs outside callbacks; bounded queue; model/provider warmup before audio start unless explicit muted/bypass startup mode is active. | Late work follows scheduler policy; repeated execution failure escalates session health. | pitch/content/retrieval/generator ms, total model ms, realtime factor, model error events. |
| `OutputQueue` | `vc-audio` callback-facing queue. | DSP/model output worker. | Output audio callback. | Interleaved `f32` output frames or later `AudioFrameBatch`. | Frames, not bytes or samples. | Preallocated before stream start; callback pop is nonblocking. | On empty queue, output silence or safe reused frames, increment underrun, and never wait for model output. | output callbacks, popped frames, output queue frames, underrun events, output stream error events. |

### Model plane components

`ModelBundle`

- Owns one validated local model package and metadata.
- Contains model files, expected sample rate, hop/chunk constraints, provider
  requirements, supported controls, and checksums.
- For MVP, it is a constrained format, not arbitrary Python model import.

`ModelFamilyAdapter`

- Defines how a model family is loaded, validated, configured, and run.
- Defines the model-family pipeline stages and diagnostics labels.
- First concrete adapter: RVC.

`RvcPipeline`

- Owns RVC-specific pitch/content/generator/index logic.
- Calls provider runtime through `vc-ort` abstractions.
- Does not own audio devices.

### Provider plane components

`OrtRuntime`

- Owns ONNX Runtime environment/session setup and execution provider binding.
- Exposes provider availability, actual provider assignment, and fallback.
- Does not own model-family semantics beyond model input/output execution.

`ProviderProbe`

- Owns hardware/provider checks and produces actionable diagnostics.
- Must distinguish unavailable provider, provider loaded but graph unsupported,
  and partial provider fallback.

`AcceleratorAdapter`

- Future boundary for TensorRT engines or Triton kernels.
- Must be justified by benchmarked ORT bottlenecks.
- Must not replace the portable ORT path as the baseline.

### Observability components

`MetricsRegistry`

- Owns stable metric names, units, snapshots, and compatibility rules.

`DiagnosticsStream`

- Owns live event/snapshot delivery to CLI, daemon clients, and UI.

`BenchmarkRunner`

- Owns reproducible offline benchmarks for audio, DSP, inference, and full
  pipeline runs.

`ReportExporter`

- Owns redacted diagnostic reports that users can share without voice audio.

## Target crate map

Current crates:

```text
crates/
  vc-core
  vc-audio
  vc-cli
```

Expected target crates:

```text
crates/
  vc-core
  vc-audio
  vc-dsp
  vc-ort
  vc-rvc
  vc-daemon
  vc-bench
  vc-diagnostics
  vc-config
  vc-model
  vc-sdk
```

Do not create all target crates at once. Create a crate when:

- a real boundary exists;
- at least two callers need the boundary;
- the crate can be tested independently;
- dependency direction becomes clearer by extracting it;
- an ADR or phase plan explains the split.

### Crate responsibilities

`vc-core`

- Owns shared units, identifiers, small value types, common errors, and stable
  metric types that multiple runtime crates need.
- Must remain backend-agnostic.
- Must not become a dumping ground for helpers.

`vc-audio`

- Owns OS audio integration, device probes, stream setup, callback-safe buffer
  movement, and audio runtime metrics.
- May hide CPAL or later native backend details.
- Must not depend on model, daemon, UI, or provider crates.

`vc-dsp`

- Owns resampling, channel mapping, framing, SOLA, crossfade, jitter buffering,
  and chunk-boundary helpers.
- May depend on `vc-core`.
- Must not depend on daemon, UI, RVC, or ORT.

`vc-ort`

- Owns ONNX Runtime environment/session/provider integration.
- May expose model-execution primitives and provider diagnostics.
- Must not depend on audio, daemon, UI, or RVC-specific policy unless an ADR
  accepts a narrow model integration helper.

`vc-model`

- Owns model bundle metadata, validation, checksums, model-family registry
  traits, and compatibility reporting.
- Does not run RVC itself.

`vc-rvc`

- Owns the RVC adapter and pipeline.
- May depend on `vc-core`, `vc-dsp`, `vc-model`, and `vc-ort`.
- Must not own audio devices or daemon protocol.

`vc-diagnostics`

- Owns metrics aggregation, event schema, redaction, reports, and compatibility
  helpers.
- Must keep raw audio out of default diagnostic export.

`vc-config`

- Owns persisted config schema, validation, profiles, and migration.
- Produces validated values for control-plane use.
- Does not mutate realtime state directly.

`vc-daemon`

- Owns local control API, session lifecycle, service composition, process
  lifecycle, and metrics streaming.
- Coordinates lower-level crates without absorbing their responsibilities.

`vc-bench`

- Owns benchmark harnesses, benchmark data models, and report generation.
- May depend on crates under test.
- Must not depend on UI.

`vc-cli`

- Owns command parsing, terminal output, smoke commands, benchmark commands, and
  conversion entry points.
- Calls daemon or lower-level crates depending on the phase.
- Must not own audio callbacks or provider internals.

`vc-sdk`

- Future stable embedding API.
- Should be introduced after daemon/session/model contracts have stabilized.

## Dependency direction

Target dependency direction:

```text
vc-cli -> vc-daemon
vc-cli -> vc-bench
vc-cli -> selected lower-level crates for pre-daemon phase tools

vc-daemon -> vc-config
vc-daemon -> vc-audio
vc-daemon -> vc-dsp
vc-daemon -> vc-model
vc-daemon -> vc-rvc
vc-daemon -> vc-diagnostics

vc-rvc -> vc-model
vc-rvc -> vc-dsp
vc-rvc -> vc-ort
vc-rvc -> vc-core

vc-model -> vc-core
vc-diagnostics -> vc-core
vc-config -> vc-core
vc-audio -> vc-core
vc-dsp -> vc-core
vc-ort -> vc-core

vc-bench -> crates under test
vc-sdk -> stable public facade over daemon/library contracts
```

Forbidden:

```text
vc-core -> any other project crate
vc-audio -> vc-rvc
vc-audio -> vc-daemon
vc-dsp -> vc-daemon
vc-dsp -> vc-rvc
vc-ort -> vc-rvc
vc-ort -> vc-daemon
vc-daemon -> vc-ort
vc-rvc -> vc-audio
UI/web/Tauri -> data-plane internals
plugins -> audio devices
```

MVP rule: the daemon may own provider policy and diagnostics, but it must not
instantiate ORT sessions directly. ORT execution belongs below the model-family
adapter boundary. A later shared provider facade can change this only through an
ADR.

## Data flow

Target realtime data flow:

```text
input callback
  -> CaptureQueue
  -> input resampler / channel mapper
  -> ChunkScheduler
  -> model worker request
  -> pitch stage
  -> content stage
  -> optional retrieval stage
  -> generator stage
  -> output resampler / channel mapper
  -> SOLA / crossfade
  -> OutputQueue
  -> output callback
```

Callback path:

- input callback moves frames into a bounded queue and records callback metrics;
- output callback pops frames from a bounded queue and records underruns;
- callbacks do not wait for inference.

Worker path:

- scheduler forms chunks from available input frames;
- worker pool runs DSP/model stages outside callbacks;
- output frames are queued before playback deadlines;
- late chunks are handled by explicit overload policy.

Control path:

- session configuration is validated before stream start;
- runtime snapshots cross into data plane through bounded channels or immutable
  swaps;
- structural changes restart a session instead of mutating unsafe live state.

## Control flow

Target session lifecycle:

```text
CreateSession
  -> validate static config
  -> resolve devices
  -> validate model bundle
  -> probe providers
  -> build runtime graph
  -> load and warm up providers/model before audio start
  -> start audio streams
  -> stream diagnostics
  -> accept safe live updates
  -> stop streams
  -> release model/provider/session resources
```

Important control-plane guarantees:

- invalid config fails before realtime start;
- provider fallback is reported before and during inference;
- model/provider load and warmup complete before audio callback start;
- the only exception is an explicit muted/bypass startup mode that emits
  diagnostics and does not claim converted realtime output until warmup ends;
- UI disconnect does not necessarily stop a session;
- daemon restart behavior is explicit and tested.

## Configuration model

Target config groups:

```text
AudioConfig
  input_device_id
  output_device_id
  sample_rate_hz
  input_channels
  output_channels
  callback_buffer_frames
  queue_capacity_frames

DspConfig
  internal_sample_rate_hz
  chunk_frames
  hop_frames
  crossfade_frames
  sola_enabled
  channel_policy

ModelConfig
  model_bundle_id
  model_family
  speaker_id
  pitch_mode
  index_retrieval_enabled
  model_quality_latency_profile

ProviderConfig
  provider_preference_order
  allow_cpu_fallback
  require_gpu_for_realtime
  provider_options

DiagnosticsConfig
  metrics_interval_ms
  export_redaction_level
  include_environment_probe
  include_benchmark_summary
```

Config update classes:

- startup-only: devices, stream shape, model bundle, provider runtime;
- chunk-boundary update: speaker controls, pitch shift, mix controls, selected
  latency/quality profile if model-compatible;
- immediate control-plane update: diagnostics interval, UI preferences;
- restart-required: anything changing graph topology, stream config, or model
  session shape.

## Public contracts

Public contracts should be introduced intentionally and tracked with ADRs when
they stabilize.

MVP-era contracts:

- CLI command names and machine-readable flags where added.
- Device list shape and device ID semantics.
- Model bundle format.
- Metrics schema and units.
- Diagnostics report format.
- Benchmark report format.
- Daemon API once clients depend on it.

Future contracts:

- SDK API.
- Model-family adapter trait.
- Public plugin ABI after explicit ADR.
- Provider-pack interface after explicit ADR.
- Tauri app update and sidecar lifecycle contract.

Rule: do not expose internal structs as public API just because they are easy to
return. Use narrow DTOs or snapshots when data crosses process, crate, or user
boundaries.

## Latency model

Total runtime latency is the sum of fixed buffering, scheduling, model work, and
queue growth.

```text
total_runtime_latency_ms =
  input_backend_buffer_ms
  + capture_queue_ms
  + input_resample_ms
  + chunk_wait_ms
  + pitch_ms
  + content_ms
  + retrieval_ms
  + generator_ms
  + output_resample_ms
  + sola_crossfade_ms
  + output_queue_ms
  + output_backend_buffer_ms
```

Each stage must report p50, p95, and p99 timing once it exists.

Queue growth rule:

```text
if average_processing_time_ms > emitted_audio_duration_ms:
  queue_depth grows
  accumulated_delay grows
  realtime_factor > 1.0
```

The runtime must report this as delay growth, not as vague "lag".

Initial budget targets are phase-specific and must be measured before being
treated as support claims. A useful starting model:

```text
capture callback:          < 1 ms callback work
input queue target:        bounded, no upward trend
DSP preprocess:            measured per chunk
pitch/content/generator:   must fit inside chunk/hop budget or degrade
postprocess:               measured per chunk
output queue target:       enough for jitter, not enough to hide delay growth
output callback:           < 1 ms callback work
```

## Overload behavior

Overload is expected. Silent overload is a bug.

Overload sources:

- input queue overrun;
- output queue underrun;
- inference worker misses deadline;
- provider fallback to slower provider;
- resampler or DSP stage exceeds budget;
- UI/control load delays diagnostics;
- external routing adds latency outside `vc-runtime`.

Allowed strategies:

- drop oldest input chunks before accumulated delay becomes unbounded;
- reuse previous output or controlled silence on output underrun;
- bypass optional retrieval;
- lower quality/latency profile when configured;
- stop session if realtime requirement cannot be met;
- report CPU fallback as non-realtime risk;
- require user confirmation for unsafe fallback policies.

Forbidden strategies:

- grow queues without limit;
- hide CPU fallback;
- block audio callbacks waiting for model output;
- report success when output is underrunning continuously;
- keep a session running with invalid model/provider state.

## Diagnostics model

Diagnostics must answer four questions:

1. Is audio healthy?
2. Is processing faster than realtime?
3. Is the requested provider actually being used?
4. Which setting should the user change?

Metric groups:

```text
audio
  input_callbacks
  output_callbacks
  input_stream_error_events
  output_stream_error_events
  pushed_frames
  popped_frames
  underrun_events
  overrun_events
  input_queue_frames
  output_queue_frames

scheduler
  chunk_index
  chunk_frames
  hop_frames
  chunk_deadline_ms
  chunk_schedule_ms
  deadline_miss_events
  accumulated_delay_ms

dsp
  resample_in_ms
  channel_map_in_ms
  resample_out_ms
  channel_map_out_ms
  sola_ms
  crossfade_ms

model
  pitch_ms
  content_ms
  retrieval_ms
  generator_ms
  total_model_ms
  model_realtime_factor

provider
  requested_provider
  observed_provider_assignment
  provider_assignment_granularity
  provider_fallback_events
  provider_probe_status
  provider_error_code

session
  state
  config_revision
  model_bundle_id
  session_uptime_ms
  diagnostics_revision
```

Diagnostics output forms:

- live metrics snapshots for CLI and UI;
- event stream for warnings and state changes;
- benchmark report for reproducible performance comparisons;
- redacted diagnostic export for issue reports.

Default diagnostics must not include raw voice audio.

## Failure model

Device failure:

- device missing: config validation fails or session stops with actionable
  message;
- device format mismatch: control plane proposes conversion or rejects startup;
- backend stream error: realtime callback increments counter and non-realtime
  diagnostics channel records detail where possible.

Audio overload:

- input overrun: increment counter, drop according to configured policy, report;
- output underrun: output silence/reused safe frames, increment counter, report;
- repeated xrun: escalate session health status.

Inference failure:

- model load failure: fail before audio start;
- model execution failure: stop or bypass according to explicit mode;
- shape mismatch: model bundle validation failure;
- slow inference: report realtime factor and apply overload policy.

Provider failure:

- unavailable requested provider: fail or fallback depending on config;
- partial graph fallback: report observed provider assignment at the granularity
  the provider layer can prove; stage-level reporting requires ORT evidence and
  a metrics/schema ADR before becoming a support promise;
- provider crash or repeated execution error: stop session and preserve report.

Control failure:

- invalid live update: reject without changing data-plane snapshot;
- UI disconnect: session keeps running unless client owns session lifecycle;
- daemon API error: return structured error and preserve session state.

Packaging failure:

- missing runtime dependency: preflight check fails with install guidance;
- incompatible GPU driver: provider probe reports expected vs found state.

## Model bundle architecture

The first model bundle format must be strict.

MVP RVC bundle should contain:

- manifest file with schema version;
- model family: `rvc`;
- expected input/output sample rate;
- chunk/hop constraints;
- speaker controls;
- pitch mode requirements;
- model artifact paths;
- checksums;
- supported execution providers if known;
- conversion tool version;
- optional index artifact metadata.

Bundle rules:

- the runtime validates before session start;
- arbitrary Python code is not executed from bundles;
- PyTorch `.pth` is not a runtime format unless a later ADR accepts a constrained
  import path;
- conversion tools may use Python, but production runtime consumes validated
  artifacts.

Future model-family adapters should reuse the bundle envelope but define
family-specific required artifacts and stage names.

## Provider architecture

ONNX Runtime is the mainline provider boundary.

Provider flow:

```text
ProviderManager
  -> ProviderProbe
  -> OrtRuntime
  -> ORT session options
  -> execution provider binding
  -> observed provider assignment report
```

Provider states:

- unavailable: dependency missing or unsupported platform;
- available: provider can be loaded;
- usable: provider can run the target model or stage;
- partial fallback: provider runs only part of the graph;
- failed: provider failed during load or execution.

Provider assignment reporting is staged:

- Phase 2 may report provider availability, requested provider, selected session
  options, and graph/session-level assignment if ORT exposes enough evidence.
- Stage-level assignment is a target diagnostic, not a current guarantee.
- Per-stage provider reporting becomes a public metric only after provider
  evidence and a schema ADR define what can be proven on each provider.

Production priority:

1. CPU fallback everywhere.
2. CUDA for Windows/Linux NVIDIA production path.
3. DirectML, CoreML, and OpenVINO as experimental provider packs.
4. TensorRT only after ORT CUDA bottlenecks are measured.
5. Triton kernels only for specific profiled GPU kernel bottlenecks.

Provider fallback policy must be explicit:

- `allow_cpu_fallback=true`: session may run but diagnostics must warn.
- `require_gpu_for_realtime=true`: session fails if requested GPU provider is not
  usable.
- fallback must be visible at the strongest proven granularity; per-stage
  fallback is the target, not an MVP assumption.

## UI architecture

UI is a client of the daemon/control API.

Initial UI:

- local web UI for development speed;
- device page;
- model loader;
- provider diagnostics;
- profiler dashboard;
- export diagnostics.

Later UI:

- Tauri shell for production packaging;
- same daemon/control API where possible;
- sidecar or embedded runtime only after an ADR.

UI must not:

- own CPAL streams;
- own model workers;
- mutate realtime buffers;
- hide provider fallback;
- invent metrics not backed by runtime diagnostics;
- become the only way to configure sessions.

The CLI must remain able to run smoke tests, benchmarks, and diagnostics without
the UI.

## SDK and plugin architecture

The SDK is future-facing. It should appear only after daemon/session contracts
stabilize.

SDK goals:

- embed runtime sessions in other apps;
- expose stable config/session/diagnostics APIs;
- avoid exposing internal crate topology;
- preserve realtime safety rules.

Internal extension seam goals:

- add model families;
- add conversion helpers;
- add optional provider-pack experiments;
- add UI panels only through daemon-defined data after the UI API exists.

Internal extension seam non-goals:

- extensions do not own audio devices;
- extensions do not run code in audio callbacks;
- extensions do not bypass diagnostics;
- extensions do not define their own incompatible session lifecycle.

Public plugin ABI is explicitly out of scope until after at least one internal
model-family adapter is stable and a second real extension need exists. The
first model-family boundary should be an internal Rust trait or adapter. Public
plugin ABI requires a later ADR, compatibility policy, security model, and
versioning plan.

## Evolution path

### Phase 0: Audio foundation

Existing:

- `vc-core`;
- `vc-audio`;
- `vc-cli`;
- CPAL/rtrb passthrough.

Architecture goal:

- prove callback, buffer, device, and metrics behavior before adding model work.

Must remain true:

- no ML;
- no daemon;
- no UI;
- no provider manager;
- no model inference inside callbacks.

Next transitions:

- add longer soak metrics;
- add queue-depth and callback timing if needed;
- decide resampling/channel mapping boundary;
- smoke test Windows and macOS.

### Phase 1: DSP and scheduler

New likely crate:

- `vc-dsp`.

New stable seams:

- `AudioFrame`/`AudioChunk` domain types;
- resampling boundary;
- channel mapping policy;
- chunk scheduler interface;
- output queue health metrics.

Implementation should use synthetic workers before real model inference.

Evidence gate:

- scheduler keeps bounded output latency under controlled slow-worker tests;
- DSP operations are independently tested.

### Phase 2: ORT and provider foundation

New likely crate:

- `vc-ort`.

New stable seams:

- provider probe result;
- provider fallback policy;
- ORT session wrapper;
- synthetic model benchmark.

Evidence gate:

- CPU ORT inference works;
- CUDA probe is explicit;
- provider availability and observed assignment are reported at the strongest
  granularity proven by ORT evidence.

### Phase 3: Daemon and control API foundation

New likely crates:

- `vc-config`;
- `vc-daemon`;
- possible `vc-diagnostics` if metrics and events have outgrown `vc-core`.

New stable seams:

- session lifecycle API;
- config validation model;
- diagnostics stream;
- daemon error model;
- internal CLI harness vs daemon-controlled product boundary.

Evidence gate:

- CLI can control daemon sessions;
- invalid config fails before audio start;
- UI-independent diagnostics work;
- direct CLI harnesses are marked internal unless a later ADR makes them public.

### Phase 4: RVC MVP backend

New likely crates:

- `vc-model`;
- `vc-rvc`;
- possible expansion of `vc-diagnostics`.

New stable seams:

- RVC bundle manifest;
- model-family adapter interface;
- RVC stage labels;
- per-stage timing schema;
- first daemon-controlled end-to-end pipeline graph.

Evidence gate:

- one known-good RVC bundle runs end-to-end through a daemon-controlled session;
- CUDA path is measured against CPU and Python baseline where possible;
- profiler explains bottlenecks;
- provider fallback is never silent.

### Phase 5: Diagnostics and tuning

New likely stabilization:

- `vc-diagnostics`;
- `vc-bench`.

New stable seams:

- benchmark report format;
- redacted diagnostics export;
- profiler data model;
- tuning recommendation inputs.

Evidence gate:

- reports compare machines and commits;
- users can identify bottleneck category.

### Phase 6: Cross-platform production backend

New likely work:

- platform capability matrix;
- packaging checks;
- provider pack probes;
- optional platform-specific audio backend escape hatches.

Evidence gate:

- supported platform matrix is explicit and tested;
- missing dependencies produce actionable diagnostics.

### Phase 7: UI clients

New likely folders:

```text
web/dev-ui
apps/tauri
```

New stable seams:

- UI API contract;
- profiler view model;
- diagnostics export UX.

Evidence gate:

- UI displays runtime truth from daemon metrics;
- CLI remains capable of the same diagnostics.

### Phase 8: Ecosystem and accelerators

New likely boundaries:

- `vc-sdk`;
- internal model-family extension seam first;
- public model-family plugin ABI only after a later ADR;
- provider-pack ABI only after a later ADR;
- TensorRT/Triton accelerators.

Evidence gate:

- extension point has at least two real implementations or a proven external
  integrator need;
- benchmark deltas justify accelerator complexity.

## Stable seams

These seams should be preserved from early phases:

- UI/client to daemon/control API.
- Control plane to data plane.
- Audio callbacks to queues.
- DSP/chunk scheduler to model worker.
- Model-family adapter to provider runtime.
- Provider manager to provider runtime.
- Runtime metrics to diagnostics stream.
- Model bundle to model loader.
- Benchmark runner to report format.

Breaking one of these seams requires an ADR.

## Extension points

Allowed extension points:

- additional audio backend implementation behind `vc-audio`;
- additional model-family adapter behind `vc-model`;
- additional ORT execution provider pack behind `ProviderManager`;
- optional accelerator behind provider/accelerator adapter after benchmarks;
- UI clients over daemon API;
- benchmark scenarios over `vc-bench`;
- diagnostics exporters over redacted diagnostics data.

Non-extension zones:

- audio callbacks;
- raw realtime buffers;
- internal queue implementation;
- metrics unit semantics;
- model bundle validation;
- provider fallback reporting;
- session lifecycle state machine.

These zones should be boring and constrained.

## ADR backlog

Likely future ADRs:

- Define audio stream identity and stable device ID policy.
- Define resampling and channel mapping boundary.
- Define chunk scheduler and overload policy.
- Define latency metrics and histogram schema.
- Define ONNX Runtime provider manager contract.
- Define provider fallback policy.
- Define RVC model bundle format.
- Define model-family adapter interface.
- Define daemon protocol and transport.
- Define diagnostics report format and redaction policy.
- Define benchmark report format.
- Decide local web UI transport.
- Decide Tauri sidecar vs embedded runtime.
- Decide SDK public API boundary.
- Decide model plugin ABI.
- Decide TensorRT support criteria.
- Decide Triton kernel support criteria.

ADR timing rule:

- write the ADR before the decision becomes a public contract;
- write it no later than the phase plan that implements the decision.

## Architecture fitness checks

Manual checks now:

- `git ls-files -z | xargs -0 -n1 dirname | sort -u | while read -r dir; do test -f "$dir/context.md" || echo "missing context: $dir"; done`
- `cargo fmt --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- review crate dependency directions in `Cargo.toml`;
- review new public structs/enums/functions for accidental contract expansion;
- review callback code for blocking, allocation, logging, and model work.

Future automated checks:

- crate dependency direction;
- forbidden import rules;
- public API diff report;
- metric schema compatibility;
- docs link checker;
- benchmark regression gate;
- realtime callback audit;
- file-size and god-object drift report.

## Open questions

Audio:

- Is CPAL sufficient for production audio paths on all target platforms?
- Which platforms need native audio backend escape hatches?
- What stable device identity can survive device reordering?

DSP:

- Which resampler should be used for realtime quality and latency?
- Which chunk/hop sizes are acceptable for RVC quality and latency?
- How much output buffering is acceptable before perceived lag is too high?

Inference:

- Which RVC subgraphs export cleanly to ONNX?
- Which stages actually benefit from CUDA vs CPU on target machines?
- How much provider fallback can be allowed before realtime claims become false?

Daemon:

- Which transport should the local daemon use first?
- Should CLI talk to the daemon by default once daemon exists, or keep direct
  crate access for smoke tests?
- How should daemon sessions survive UI disconnects and process restarts?

UI:

- How much tuning should be automatic vs exposed to users?
- Which profiler view best explains queue growth without overwhelming users?

Ecosystem:

- When does `vc-sdk` become worth stabilizing?
- Which second model family proves the adapter boundary?
- Which accelerator path justifies complexity after ORT benchmarks?

## Summary

The architecture should grow toward a full local voice conversion platform, but
only through measured phase gates.

The most important decision is the separation of concerns:

- clients control;
- daemon coordinates;
- audio callbacks move frames;
- DSP and scheduler shape chunks;
- model adapters define family semantics;
- providers execute graphs;
- diagnostics explain everything.

This structure leaves room for UI, SDK, plugins, and accelerators without
allowing any of them to take ownership of the realtime path.
