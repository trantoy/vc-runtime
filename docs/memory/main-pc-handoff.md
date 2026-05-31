# Main PC Handoff

Created: 2026-05-31

## Purpose

This file preserves the current session state before moving work from the laptop
to the stronger main PC with an NVIDIA GPU.

Read this first after cloning the repository on the main PC, then read:

- [context.md](context.md)
- [roadmap.md](roadmap.md)
- [runtime-architecture-v1.md](runtime-architecture-v1.md)
- [phases/phase-0/results.md](phases/phase-0/results.md)
- [../adr/context.md](../adr/context.md)

## Current Repository State

Important recent commits:

- `5e6337d` Add architecture prototype experiments
- `51dd35c` Enhance offline audio benchmark prototype
- `f8e31d3` Promote offline audio benchmark crate

The branch was ahead of `origin/main` by 9 commits at the time this handoff was
written. Push before cloning elsewhere if the remote has not been updated.

## What Exists Now

Production crates:

- `vc-core`: shared metrics.
- `vc-audio`: CPAL device listing and passthrough.
- `vc-cli`: CLI for device listing and passthrough.
- `vc-bench`: offline prerecorded-audio benchmark runner and JSON reports.

Experiment folders:

- `experiments/audio-soak-metrics/`: parses passthrough logs into JSONL/CSV.
- `experiments/chunk-scheduler-sim/`: simulates chunk deadline policies.
- `experiments/ort-provider-probe/`: dry-run/provider-report schema prototype.
- `experiments/offline-audio-bench/`: fixture preparation and prototype benchmark
  work that preceded `vc-bench`.

Docs that now matter:

- `docs/memory/architecture-guide.md`: maintainability and review rules.
- `docs/memory/runtime-architecture-v1.md`: target architecture and evolution.
- `docs/memory/roadmap.md`: phase order.
- `docs/memory/phases/phase-0/results.md`: evidence log.
- `docs/adr/0013-treat-architecture-memory-docs-as-normative-guidance.md`:
  accepted ADR making architecture memory docs normative guidance.

## What Has Been Proven

- Rust workspace builds and tests on the laptop.
- CPAL/rtrb passthrough works for a short Linux smoke run.
- Passthrough metrics exist for callbacks, pushed/popped frames, underruns,
  overruns, and stream errors.
- Offline prerecorded-audio benchmark can run without audio devices.
- `vc-bench` can emit report v1 with:
  - fixture/source provenance;
  - chunk/hop configuration;
  - per-chunk timing percentiles;
  - `realtime_factor`;
  - `deadline_miss_events`;
  - `accumulated_delay_ms`;
  - checksum fields.
- Threshold mode can make benchmark runs fail on performance regressions.

## What Has Not Been Proven

- No 30-minute passthrough soak has been recorded.
- Windows and macOS audio behavior is unverified.
- Resampling and channel remapping are not implemented.
- ONNX Runtime is not integrated in production.
- CUDA/provider probing is not verified on real NVIDIA hardware.
- RVC inference does not exist yet.
- `copy`, `gain`, and `rms` benchmark stages validate the harness only; they do
  not prove voice-conversion performance.

## Local Files Not In Git

The laptop has generated audio files ignored by git:

- `experiments/offline-audio-bench/fixtures/audio/wizard-of-oz-01.ogg`
- `experiments/offline-audio-bench/fixtures/audio/wizard-of-oz-01-16k-mono.wav`

Recreate them on the main PC:

```bash
cd experiments/offline-audio-bench
./scripts/fetch_wizard_of_oz_fixture.sh --duration-seconds 120
```

The script requires `curl` and `ffmpeg`.

## Main PC Bring-Up

After cloning:

```bash
git status --short --branch
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Prepare the benchmark fixture:

```bash
cd experiments/offline-audio-bench
./scripts/fetch_wizard_of_oz_fixture.sh --duration-seconds 120
cd ../..
```

Run the first production benchmark:

```bash
cargo run -p vc-bench -- \
  --input experiments/offline-audio-bench/fixtures/audio/wizard-of-oz-01-16k-mono.wav \
  --source-id librivox-wizard-of-oz-01 \
  --stage copy \
  --max-realtime-factor 0.01 \
  --max-deadline-misses 0
```

Confirm threshold failure still works:

```bash
cargo run -p vc-bench -- \
  --input experiments/offline-audio-bench/fixtures/audio/wizard-of-oz-01-16k-mono.wav \
  --source-id librivox-wizard-of-oz-01 \
  --stage copy \
  --max-realtime-factor 0
```

Expected: non-zero exit after printing the report.

## Main PC GPU Plan

The NVIDIA machine should be used for evidence that the laptop cannot provide:

1. Record hardware and driver facts:
   - OS and kernel/build;
   - CPU model;
   - GPU model and VRAM;
   - NVIDIA driver version;
   - CUDA availability if installed.
2. Run the current `vc-bench` baseline stages on the same WAV fixture:
   - `copy`;
   - `gain`;
   - `rms`.
3. Add richer provenance to `vc-bench` before using reports for public claims:
   - OS;
   - CPU;
   - git commit;
   - Rust version;
   - dependency lockfile hash;
   - optional GPU/driver fields.
4. Build a real ORT provider probe in production or an experiment:
   - CPU provider first;
   - CUDA provider on the main PC;
   - visible fallback reasons;
   - report `observed_provider_assignment` and
     `provider_assignment_granularity`.
5. Add a synthetic ONNX benchmark stage before any RVC model:
   - model load time;
   - warmup time;
   - inference p50/p95/p99;
   - provider status;
   - threshold mode.
6. Only after ORT evidence exists, decide whether Triton kernels are justified.

## Next Implementation Candidates

Recommended order:

1. `vc-bench` provenance upgrade.
2. ORT provider probe experiment on the NVIDIA PC.
3. Production `vc-ort` crate skeleton.
4. Synthetic ONNX benchmark stage in `vc-bench`.
5. Phase 0.2 audio soak on the main PC.
6. Resampling/channel mapping boundary.

Avoid jumping directly to RVC/Triton until ORT and per-stage benchmark evidence
exist.

## Agent Reminder

When a new agent starts on the cloned repo:

1. Read this file.
2. Read `docs/memory/context.md`.
3. Read `docs/memory/phases/phase-0/results.md`.
4. Check `git status --short --branch`.
5. Recreate ignored audio fixtures if needed.
6. Run verification before making new claims.
