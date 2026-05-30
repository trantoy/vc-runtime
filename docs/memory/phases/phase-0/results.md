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
