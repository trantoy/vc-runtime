# Phase 0 Results

Created: 2026-05-31

## Purpose

This file is the rolling evidence log for Phase 0. It records what was actually verified, what failed, and what remains unproven.

## Phase 0.1 Audio Passthrough

Date: 2026-05-31

Implementation commits:

- `fde7a0c` Add phase planning structure
- `c2f3086` Add Rust workspace skeleton
- `30a4a06` Add audio metrics counters
- `46f65e1` Add audio device listing command
- `3ab3ceb` Add audio passthrough runtime

Results-log commit:

- Current commit containing this file. Resolve with `git log -- docs/memory/phases/phase-0/results.md`.

Static verification:

- `cargo fmt --check`: passed.
- `cargo test --workspace`: passed. Observed unit/doc test coverage included `vc-core`, `vc-audio`, and `vc-cli`.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `git diff --check`: passed.
- `git ls-files -z | xargs -0 -n1 dirname | sort -u | while read -r dir; do test -f "$dir/context.md" || echo "missing context: $dir"; done`: passed with no missing `context.md` output.

Hardware/runtime verification:

- `cargo run -p vc-cli -- list-devices`: passed on the development machine.
- `cargo run -p vc-cli -- passthrough --seconds 1`: passed on the development machine.

Observed `list-devices` result on the development machine:

- Command exited successfully.
- Input and output sections were printed.
- CPAL/ALSA reported `default`, `pipewire`, `pulse`, `jack`, and `HD-Audio Generic` for both directions.
- ALSA can still print raw probe diagnostics to stderr for unavailable built-in devices; Phase 0.1 reports that limitation as a warning but does not intercept ALSA stderr.

Observed `passthrough --seconds 1` result on the development machine:

- Command exited successfully.
- Passthrough started with input device `default`, output device `default`, sample rate `44100`, `2` channels, and `48000` capacity frames.
- A one-second metrics line was printed:
  - `input_cb=23`
  - `output_cb=24`
  - `pushed_frames=43249`
  - `popped_frames=43248`
  - `underrun_events=3`
  - `overrun_events=0`
  - `input_stream_error_events=0`
  - `output_stream_error_events=0`
- The observed run reported underrun events near startup and zero stream error events.

Observed command failures:

- None in the final Task 6 verification set.

What this proves:

- The Rust workspace builds and passes format, tests, and clippy on the development machine.
- The crate boundaries exist: `vc-core` owns metrics, `vc-audio` owns CPAL/rtrb audio runtime, and `vc-cli` owns command parsing/output.
- `vc list-devices` runs without panic on the development machine.
- `vc passthrough --seconds 1` runs without panic on the development machine.
- Metrics exist for callbacks, pushed/popped frames, underrun/overrun events, and stream error events.

What this does not prove:

- It does not prove a stable 30-minute passthrough run.
- It does not prove behavior on Windows or macOS.
- It does not prove behavior on machines with mismatched input/output sample rates or channel counts.
- It does not prove low-latency performance under CPU load.
- It does not prove model inference, ONNX Runtime, daemon control, UI, or voice conversion behavior.

Current limitations:

- Phase 0.1 rejects mismatched input/output sample rates and channel counts instead of resampling or remapping.
- Device indices are current enumeration indices and may change between runs.
- Stream error callbacks count events only; detailed backend error text needs a later non-realtime diagnostics channel.
- The one-second passthrough run had startup underruns, so buffer warmup and startup sequencing need later measurement.
- ALSA probe diagnostics may still appear on stderr before structured CLI output.

Next evidence target:

- Run and record a 30-minute passthrough test on at least one Linux development machine.
- Repeat basic device listing and one-second passthrough on Windows and macOS.

## Prototype Findings

Date: 2026-05-31

Prototype artifacts:

- `experiments/audio-soak-metrics/`
- `experiments/chunk-scheduler-sim/`
- `experiments/ort-provider-probe/`

Independent review:

- Initial review rejected the prototypes because `audio-soak-metrics` could mask
  parser failures as zero-valued evidence rows, its CSV schema had a stale field,
  and `chunk-scheduler-sim` initially changed policy labels more than actual
  policy behavior.
- Follow-up fixes added strict log parsing, CSV/JSON schema alignment, scheduler
  policy behavior tests, and a zero-duration parser regression.
- Follow-up review approved the corrected prototype evidence with no remaining
  must-fix findings.

What became clear:

- Diagnostics must come before accelerator work. Optimizing Triton kernels,
  ONNX Runtime providers, or DSP internals before per-stage evidence risks
  speeding up the wrong component.
- Voice-changer latency is not one scalar number. The system must separately
  report queue growth, compute deadline misses, audible underruns, stale chunk
  drops, callback health, and provider fallback.
- `deadline_miss_events` and audible `underrun_events` are different signals.
  A policy such as `reuse-last` can hide an audible hole while computation still
  missed its deadline.
- `reuse-last` is useful only as a short-gap concealment policy. It does not
  reduce true compute lateness and must not be treated as a performance fix.
- `drop-oldest` can bound stale work and accumulated compute delay, but it
  creates discontinuities. It is a recovery policy, not a quality improvement.
- `silence-on-underrun` is semantically honest and safe, but frequent silence
  gaps are user-visible failures.
- Provider telemetry must be conservative. The runtime should report
  `observed_provider_assignment` and `provider_assignment_granularity` instead
  of claiming per-stage or per-op placement before there is evidence.
- `audio-soak-metrics` is enough for early long-run passthrough evidence, but
  the production runtime still needs a direct queue-depth gauge instead of only
  deriving queue pressure from `pushed_frames - popped_frames`.
- Benchmark reports must be reproducible artifacts, not UI-only observations.
  They should be generated by CLI tools and kept comparable across machines,
  commits, providers, and model stages.

What this does not prove:

- The prototypes do not prove model inference performance.
- They do not prove real ORT execution-provider placement.
- They do not prove Windows or macOS audio behavior.
- They do not prove subjective voice quality.
- They do not replace a real prerecorded-audio benchmark or a real 30-minute
  hardware audio soak.

Next evidence targets:

- Build an offline prerecorded-audio benchmark experiment that takes a stable WAV
  fixture, chunks it deterministically, runs a measured processing stage, and
  emits a JSON report.
- Use public-domain speech audio via a manifest and preparation script; do not
  commit large raw audio fixtures by default.
- Add a production direct queue-depth metric after the prototype evidence proves
  which queue fields are needed.

## Offline Prerecorded-Audio Benchmark Prototype

Date: 2026-05-31

Prototype artifact:

- `experiments/offline-audio-bench/`

Fixture source:

- `The Wonderful Wizard of Oz, Chapter 1`, LibriVox via Wikimedia Commons.
- Manifest: `experiments/offline-audio-bench/fixtures/sources/wizard-of-oz-01.json`.
- Prepared local WAV path: `experiments/offline-audio-bench/fixtures/audio/wizard-of-oz-01-16k-mono.wav`.
- Large generated `.ogg` and `.wav` files are ignored by git.

Static verification:

- `cargo test` in `experiments/offline-audio-bench`: passed with six tests.
- `cargo fmt --check` in `experiments/offline-audio-bench`: passed.
- `bash -n scripts/fetch_wizard_of_oz_fixture.sh`: passed.

Runtime verification:

- `./scripts/fetch_wizard_of_oz_fixture.sh --duration-seconds 10`: passed and
  generated a local 10-second 16 kHz mono WAV.
- `cargo run -- --input fixtures/audio/wizard-of-oz-01-16k-mono.wav --chunk-ms 100 --hop-ms 50 --stage rms --pretty`: passed.
- `cargo run -- --input fixtures/audio/wizard-of-oz-01-16k-mono.wav --source-id librivox-wizard-of-oz-01 --chunk-ms 100 --hop-ms 50 --stage copy --max-realtime-factor 0.01 --max-deadline-misses 0`: passed.
- `cargo run -- --input fixtures/audio/wizard-of-oz-01-16k-mono.wav --chunk-ms 100 --hop-ms 50 --stage copy --max-realtime-factor 0 --max-deadline-misses 0`: failed with exit code `1` and a threshold failure message, as expected.

Observed report shape from the 10-second fixture:

- `sample_rate_hz=16000`
- `channels=1`
- `input_frames=160000`
- `duration_ms=10000`
- `chunk_ms=100`
- `hop_ms=50`
- `chunk_count=199`
- `stage=rms`
- `deadline_miss_events=0`
- `accumulated_delay_ms=0.0`
- `source_id` is supported for manifest-based fixture identity.
- `input_content_checksum` is supported for prepared-audio identity.
- `build_profile` is reported as `debug` or `release`.

What this proves:

- The project can run an offline benchmark on prerecorded speech without audio
  hardware.
- The benchmark can prepare a license-clean public-domain speech fixture from a
  manifest and script.
- The benchmark report can capture deterministic chunking, per-chunk timing
  percentiles, realtime factor, deadline misses, accumulated delay, and a
  checksum.
- The report schema can document field names, units, and compatibility rules
  before the format is promoted to `vc-bench`.
- Threshold mode can turn a benchmark report into a regression check with
  non-zero exit status.

What this does not prove:

- `rms` is only a harness-validation stage, not a voice-conversion workload.
- The result does not prove model inference, pitch extraction, SOLA, resampling,
  or provider performance.
- The local 10-second run is a smoke check; longer fixture durations and multiple
  machines are still needed for comparative benchmark evidence.
