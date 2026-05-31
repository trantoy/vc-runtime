# vc-runtime architecture

Created: 2026-05-30

## Architecture Summary

`vc-runtime` is built around a separation between control plane and data plane.

This file is the short architecture summary. The draft target architecture and
phase-by-phase evolution path live in
[runtime-architecture-v1.md](runtime-architecture-v1.md). That document is
normative review guidance under ADR precedence, not proof that every target
component already exists.

```text
CLI / local web UI / future Tauri app
  -> control API
  -> vc-daemon
  -> audio engine
  -> DSP and chunk scheduler
  -> model runtime
  -> provider manager
  -> diagnostics stream
```

The audio callback must stay minimal. It moves frames into or out of buffers and records timing. It does not run model inference, perform blocking I/O, allocate large objects, or call UI/control code.

## Initial Runtime Boundaries

```text
crates/
  vc-core     shared types, errors, metrics, time units
  vc-audio    CPAL device enumeration and streams
  vc-dsp      ring buffers, resampling boundary, SOLA/crossfade
  vc-ort      ONNX Runtime integration
  vc-rvc      RVC pipeline
  vc-daemon   control API and metrics stream
  vc-bench    benchmarks and reports
```

Start with fewer crates than the long-term design suggests. Split more only when a boundary becomes real.

## Control Plane

The control plane owns:

- config validation;
- device selection;
- model loading;
- provider selection;
- session lifecycle;
- metrics collection and export;
- UI/CLI/API integration.

The control plane may use normal async tasks and locks, but it must communicate with the realtime path only through bounded channels or boundary-swapped immutable configs.

## Data Plane

The data plane owns:

- audio callbacks;
- ring buffers;
- chunk scheduling;
- DSP preprocessing and postprocessing;
- model worker deadlines;
- output queue health.

The data plane should be measurable at every stage.

## Diagnostics First

Every major stage must produce timing and health metrics:

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
observed_provider_assignment
provider_assignment_granularity
```

## Dependency Direction

Lower-level crates must not depend on higher-level crates.

Allowed direction:

```text
vc-daemon -> vc-rvc -> vc-ort
vc-daemon -> vc-audio
vc-daemon -> vc-dsp
vc-rvc -> vc-dsp
vc-audio -> vc-core
vc-dsp -> vc-core
vc-ort -> vc-core
```

Forbidden examples:

- `vc-audio` depending on `vc-rvc`;
- `vc-dsp` depending on daemon or UI code;
- `vc-core` depending on provider-specific libraries;
- model plugins controlling audio devices directly.

## First Implementation Target

Phase 0.1 should implement only:

```text
input device -> CPAL callback -> ring buffer -> CPAL output callback
```

No ML. No Tauri. No provider manager. The purpose is to prove the audio loop and metrics path.
