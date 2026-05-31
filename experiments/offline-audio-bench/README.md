# Offline Audio Benchmark Prototype

## Architectural question this prototype answers

Can we make benchmark evidence reproducible before the realtime voice-conversion
pipeline exists?

This experiment answers that with an offline prerecorded-audio harness:

- no live audio devices,
- deterministic chunk/hop slicing,
- measured per-chunk processing time,
- JSON report output suitable for comparing commits and machines.

## Source audio

The default fixture source is a public-domain LibriVox recording hosted on
Wikimedia Commons:

- `The Wonderful Wizard of Oz, Chapter 1`
- source manifest: `fixtures/sources/wizard-of-oz-01.json`
- prepared local WAV path: `fixtures/audio/wizard-of-oz-01-16k-mono.wav`

The raw audio and prepared WAV are ignored by git. Recreate them locally:

```bash
cd /home/cordis/Gits/vc-runtime/experiments/offline-audio-bench
./scripts/fetch_wizard_of_oz_fixture.sh --duration-seconds 120
```

The script requires `curl` and `ffmpeg`.

## Run

```bash
cargo run -- \
  --input fixtures/audio/wizard-of-oz-01-16k-mono.wav \
  --chunk-ms 100 \
  --hop-ms 50 \
  --stage rms \
  --pretty
```

Optional report file:

```bash
cargo run -- \
  --input fixtures/audio/wizard-of-oz-01-16k-mono.wav \
  --output reports/wizard-of-oz-rms.json
```

## Current stage

`rms` is intentionally simple. It is a harness-validation stage, not a model
benchmark. It proves that WAV loading, chunk slicing, timing, deadline accounting,
and report generation work before adding DSP/model stages.

## Report fields

- `sample_rate_hz`, `channels`, `input_frames`, `duration_ms`
- `chunk_ms`, `hop_ms`, `chunk_frames`, `hop_frames`, `chunk_count`
- `total_processing_ms`, `realtime_factor`
- `chunk_processing_p50_us`, `chunk_processing_p95_us`, `chunk_processing_p99_us`
- `deadline_miss_events`, `accumulated_delay_ms`
- `checksum`

## Promotion criteria

Promote this idea into `vc-bench` only after the report shape survives at least
one real fixture and one later non-trivial stage such as resampling, SOLA, pitch,
or model inference.
