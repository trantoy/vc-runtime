# Offline Audio Benchmark Prototype

## Architectural question this prototype answers

Can we make benchmark evidence reproducible before the realtime voice-conversion
pipeline exists?

This experiment answers that with an offline prerecorded-audio harness:

- no live audio devices,
- deterministic chunk/hop slicing,
- measured per-chunk processing time,
- JSON report output suitable for comparing commits and machines.

The report schema is documented in `schemas/benchmark-report-v1.md`. It fixes
field names, units, and compatibility rules so reports can be read by scripts,
CI, future UI, and later `vc-bench` without guessing semantics.

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
  --source-id librivox-wizard-of-oz-01 \
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

## Current stages

Current stages are intentionally simple harness-validation stages, not model
benchmarks:

- `copy`: baseline chunk scan that sums samples without changing them.
- `gain`: simple DSP-like multiply stage.
- `rms`: simple analysis stage.

These prove that WAV loading, chunk slicing, timing, deadline accounting, and
report generation work before adding heavier DSP/model stages.

## Report fields

- `sample_rate_hz`, `channels`, `input_frames`, `duration_ms`
- `source_id`, `input_content_checksum`, `build_profile`
- `chunk_ms`, `hop_ms`, `chunk_frames`, `hop_frames`, `chunk_count`
- `total_processing_ms`, `realtime_factor`
- `chunk_processing_p50_us`, `chunk_processing_p95_us`, `chunk_processing_p99_us`
- `deadline_miss_events`, `accumulated_delay_ms`
- `checksum`

See `schemas/benchmark-report-v1.md` for field units and compatibility rules.

## Regression thresholds

Threshold mode makes the benchmark act like a test:

```bash
cargo run -- \
  --input fixtures/audio/wizard-of-oz-01-16k-mono.wav \
  --source-id librivox-wizard-of-oz-01 \
  --stage copy \
  --max-realtime-factor 0.01 \
  --max-deadline-misses 0
```

If a threshold fails, the command exits non-zero after producing the report.

## Promotion criteria

Promote this idea into `vc-bench` only after the report shape survives at least
one real fixture and one later non-trivial stage such as resampling, SOLA, pitch,
or model inference.
