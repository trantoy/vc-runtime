# Realtime voice changer runtime

Created: 2026-05-30

## Idea

Build one focused library for low-latency realtime voice conversion instead of rewriting the whole voice changer stack.

Working name:

- `rvc-realtime-runtime`

The first target should be **RVC-only**, because RVC is popular, the pipeline is relatively clear, and it is a better optimization target than trying to support every model family at once.

## Core thesis

Do not rewrite PyTorch, HuBERT, RMVPE, CUDA kernels, the whole GUI, or every supported voice conversion model.

Rewrite the realtime runtime around the model:

- audio chunk buffering;
- ring buffer and jitter buffer;
- fixed-size chunk scheduling;
- `int16 <-> float32` conversion;
- mono/stereo conversion;
- normalization;
- slicing, padding, and context windows;
- input and output resampling;
- SOLA, overlap-add, and crossfade;
- binary audio transport boundary;
- ONNX Runtime wrapper for model inference where possible.

The goal is not "Rust instead of Python" as a slogan. The goal is a predictable low-latency audio path where each chunk has stable processing time and does not accumulate delay.

## Why this may reduce lag

Realtime voice conversion lag is usually the sum of multiple stages:

```text
microphone buffer
  -> input transport
  -> resampling
  -> context/padding
  -> pitch/features
  -> model inference
  -> SOLA/crossfade
  -> output transport
  -> speaker/virtual cable buffer
```

A focused runtime library can reduce the non-model parts of this path:

- fewer audio buffer copies;
- faster and more predictable resampling;
- less Python orchestration on every chunk;
- stable ring-buffer behavior under small timing spikes;
- fewer dropouts when one chunk is slower than average;
- less accumulated delay when processing is close to realtime;
- cleaner integration with ONNX Runtime, CUDA, DirectML, or TensorRT providers.

The concrete user-visible improvements should be:

- lower end-to-end latency;
- less crackling and dropout;
- less choppy audio;
- less delay growth over time;
- more stable realtime factor;
- better behavior with small chunk sizes.

## What it will not automatically fix

This library will not automatically make every model fast.

It will not solve:

- HuBERT/contentvec being slow if they still run through a heavy PyTorch path;
- RMVPE/CREPE pitch extraction being the bottleneck;
- bad GPU provider selection, for example ONNX or PyTorch silently running on CPU;
- very heavy Diffusion-SVC-style models;
- audio routing problems in Discord, OBS, virtual cable, or OS devices;
- bad model weights or poor conversion quality.

If pitch extraction or feature extraction dominates the profile, those stages must be moved to ONNX/TensorRT/CUDA-friendly paths or optimized separately.

## Proposed boundary

The library should expose a small streaming API:

```text
init(config, model_paths, provider)
push_input_audio(samples, sample_rate)
process_until_output_ready()
pull_output_audio()
update_settings(pitch, index_rate, chunk_size, crossfade, provider)
shutdown()
```

The GUI and high-level server can stay outside the library. Python or TypeScript can still manage settings, model selection, and UI. The runtime owns the hot path.

## Implementation direction

Use Rust for:

- realtime audio buffers;
- deterministic chunk scheduler;
- resampler integration;
- binary protocol boundary;
- memory reuse;
- SOLA and crossfade;
- ONNX Runtime integration through a Rust binding if the target model path supports it.

Keep Python for:

- experiment orchestration;
- compatibility with existing model families;
- model conversion scripts;
- fallback paths;
- quick UI/server glue.

The first useful prototype can be a sidecar process rather than an in-process replacement.

Possible shape:

```text
client/audio worklet
  -> binary websocket or local IPC
  -> rvc-realtime-runtime sidecar
  -> ONNX Runtime / GPU provider
  -> processed audio stream
```

## Measurement plan

Before rewriting, benchmark the current pipeline per stage:

- receive/decode audio;
- input resample;
- feature extraction;
- pitch extraction;
- model inference;
- output resample;
- SOLA/crossfade;
- send/encode audio;
- total chunk wall time;
- queue depth over time.

The runtime is successful only if the numbers improve. Important metrics:

- p50/p95/p99 chunk processing time;
- end-to-end latency in milliseconds;
- underrun count;
- overrun count;
- queue depth growth;
- CPU usage;
- GPU utilization;
- number of CPU/GPU copies per chunk.

## Relation to `w-okada/voice-changer`

This note came from inspecting and discussing the `w-okada/voice-changer` project on 2026-05-30.

Relevant local files in the cloned project:

- `~/Gits/voice-changer/server/voice_changer/VoiceChangerV2.py`
- `~/Gits/voice-changer/server/voice_changer/RVC/RVCr2.py`
- `~/Gits/voice-changer/server/voice_changer/RVC/inferencer/InferencerManager.py`
- `~/Gits/voice-changer/client/lib/src/client/VoiceChangerWorkletNode.ts`

The current conclusion is a design hypothesis, not a benchmark result. The next step is profiling the real per-stage latency before choosing exactly which code to replace.

