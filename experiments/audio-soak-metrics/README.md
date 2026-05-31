# Audio Soak + Queue Metrics Prototype

## Architectural question this prototype answers

For `Phase 0.2`, should the evidence that long-running passthrough is stable
be based on:

1. `underrun_events` and `overrun_events` staying near-zero,
2. bounded and non-growing queue pressure (`pushed_frames - popped_frames`),
3. monotonic callback health (`input_callbacks`, `output_callbacks`) over time,
4. and the startup warmup shape (first 1–3 seconds separately)?

This prototype answers that question without changing production code by strictly
parsing existing `vc-cli` output from `vc passthrough --seconds ...` and
producing structured summaries.

## What it measures today

The prototype reads logs in the CLI format already produced by `vc-cli`:

- `input_cb`
- `output_cb`
- `pushed_frames`
- `popped_frames`
- `underrun_events`
- `overrun_events`
- `input_stream_error_events`
- `output_stream_error_events`

From these it derives:

- `duration_reported` (from last `t=N`)
- callback rates (`input_cb_rate`, `output_cb_rate`)
- queue proxy:
  - `queue_backlog = pushed_frames - popped_frames`
  - `queue_backlog_last`
  - `queue_backlog_max`
  - `queue_growth = backlog_last - backlog_first`
  - `queue_growth_per_sec`
- startup snapshots:
  - `startup_underruns`
  - `startup_overruns`
- optional `capacity_frames`, `sample_rate_hz`, `channels` from the `Passthrough started` line.

The parser intentionally fails the run when required fields are missing. A
truncated or format-drifted log must not become a valid all-zero evidence row.

## Why this is enough for Phase 0.2 decision

- If startup underruns are only in the first second and disappear, this points to
  warmup sequencing rather than sustained runtime failure.
- If `queue_growth_per_sec > 0` consistently across long runs, the implementation
  is falling behind and should trigger investigation in callback scheduling or
  queue sizing.
- If callback rates are stable and backlog is bounded while underruns/overruns are
  zero, it is a strong signal for stable long-run passthrough health.

## How to run

```bash
cd /home/cordis/Gits/vc-runtime/experiments/audio-soak-metrics
./run_audio_soak.sh --help
```

Default is a **safe short dry check**:

- `duration=5`
- `runs=1`
- both `JSONL` and `CSV` output

### Short verification path (no hardware needed)

```bash
./run_audio_soak.sh --dry-run --fixture sample-passthrough-metrics.txt --duration 5
```

### Prototype tests

```bash
bash test_audio_soak.sh
```

The test covers a valid fixture, CSV schema stability, and rejection of a metric
line with a missing required field.

### Real pass (single 5-second run)

```bash
./run_audio_soak.sh --duration 5
```

### Real pass (long run)

```bash
./run_audio_soak.sh --duration 1800 --runs 1 --format both --output-dir ./artifacts
```

### Useful flags

- `--input-index`, `--output-index`, `--capacity-frames` map directly to
  `vc passthrough` arguments.
- `--rerun-delay` spaces runs when `--runs > 1`.
- `--format jsonl|csv|both` controls output summary files.

## Files produced

- `audio-soak.summary.jsonl`: one JSON line per run.
- `audio-soak.summary.csv`: CSV table with one row per run.
- `audio-soak.run-<n>.log`: raw CLI output for each run.
- `JSONL`/`CSV` include summary fields listed above.

## Manual follow-up required for real 30-minute run

To run the full Phase 0.2 soak on development hardware:

1. Ensure stable default device selection and enough headroom on the host.
2. Run:
   ```bash
   ./run_audio_soak.sh --duration 1800 --runs 1 --format both --output-dir ./artifacts
   ```
3. Record:
   - hardware + OS + sample rate/device indices,
   - summary files,
   - any audible artifacts / speaker loop behavior.
4. Compare:
   - `queue_growth_per_sec` trend,
   - `startup_underruns` vs later `underrun_events`,
   - `queue_backlog_max`.

If startup underruns dominate and queues stay bounded, likely fix is start-up
sequencing (warmup). If backlog grows and underruns accumulate, prioritize
callback pressure and callback-to-callback timing investigation before moving to
inference/model stages.
