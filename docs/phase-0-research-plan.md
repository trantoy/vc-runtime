# vc-runtime phase 0 research plan

Created: 2026-05-30

## Goal

Phase 0 should turn the design idea into measured facts before building a large architecture.

The output is not a polished app. The output is evidence:

- what audio backend behavior looks like;
- whether RVC ONNX is viable;
- where the latency is;
- which provider paths are realistic;
- which assumptions in [vc-dis-dok.md](vc-dis-dok.md) are wrong or need narrowing.

## Timebox

Target: 1-2 focused weeks.

Stop Phase 0 when the exit criteria are met or when a core assumption is disproved.

## Workstream 1: audio passthrough spike

Build a small Rust prototype:

```text
input device -> ring buffer -> output device
```

Requirements:

- CPAL input/output device selection;
- fixed sample format internally, preferably `f32`;
- queue depth metrics;
- underrun/overrun counters;
- callback timing;
- no ML;
- no UI beyond CLI logs or minimal local page.

Exit criteria:

- 30-minute passthrough run on the primary development machine;
- visible input/output queue depth;
- no unbounded latency growth;
- sample-rate mismatch handled or explicitly reported.

Questions to answer:

- Is CPAL enough for first passthrough?
- What callback sizes are realistic?
- Does Linux need realtime priority setup immediately?
- How painful is device hotplug?

## Workstream 2: RVC ONNX offline path

Before realtime, prove the model path offline.

Prototype:

```text
recorded speech file
  -> load RVC bundle
  -> pitch/content extraction
  -> generator inference
  -> output wav
```

Requirements:

- pick one known RVC model;
- document exact model files;
- prefer ONNX for generator, pitch and content where available;
- keep PyTorch only as reference/conversion fallback;
- record per-stage timings.

Exit criteria:

- one input wav converts successfully;
- output is listenable enough for technical validation;
- per-stage timings are captured;
- unsupported pieces are listed explicitly.

Questions to answer:

- Which RVC file format is the first supported format?
- Can generator, RMVPE and content extractor all run through ONNX?
- Which stages force PyTorch or custom conversion?
- Are shapes stable enough for realtime?

## Workstream 3: metrics schema

Define the metrics before building the UI.

Minimum chunk/session metrics:

```text
capture_callback_ms
input_queue_frames
resample_in_ms
chunk_schedule_ms
pitch_ms
content_ms
generator_ms
resample_out_ms
sola_ms
output_queue_frames
underrun_count
overrun_count
total_chunk_ms
realtime_factor
provider_per_stage
```

Exit criteria:

- one JSON schema or example report;
- benchmark output can be compared across runs;
- metrics include p50/p95/p99 where relevant.

## Workstream 4: provider probe

Build a small tool that reports ONNX Runtime provider reality.

Requirements:

- list available providers;
- attempt a tiny test model;
- attempt the chosen RVC ONNX components;
- report requested provider vs actual success/failure;
- include dependency errors.

Initial providers:

- CPU;
- CUDA on NVIDIA machines;
- DirectML on Windows if available;
- CoreML on macOS if available;
- OpenVINO only as optional probe.

Exit criteria:

- provider report is exportable as JSON;
- failure messages are understandable;
- CPU fallback always works for the probe.

## Workstream 5: baseline against existing tool

Use `w-okada/voice-changer` as a baseline where practical.

Measure or document:

- model used;
- chunk size;
- f0 method;
- GPU mode/provider;
- visible latency;
- dropouts/crackling;
- CPU/GPU usage;
- any built-in timing fields.

Exit criteria:

- baseline notes exist;
- at least one comparable scenario is defined;
- obvious pain points are documented.

## Phase 0 exit criteria

Phase 0 is done when:

- CPAL passthrough has a measured result;
- one RVC offline ONNX path is proven or rejected with evidence;
- a metrics schema exists;
- provider probing works at least for CPU and one GPU path;
- first MVP scope can be narrowed with facts.

## Phase 0 failure modes

If RVC ONNX is too fragile, narrow MVP to:

- audio runtime;
- profiler;
- offline conversion tooling;
- one highly constrained model bundle format.

If CPAL is not enough, decide which platform-specific backend is needed first instead of pretending cross-platform audio is solved.

