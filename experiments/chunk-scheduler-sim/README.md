# Chunk Scheduler Simulator

This is a lightweight prototype for validating scheduler policies in isolation.

## What it simulates

- Synthetic chunks arrive every `--hop-ms`.
- Each chunk has a target output time: `chunk_ms + chunk_index * hop_ms`.
- A single worker processes queued chunks with configurable durations:
  - `--worker-ms` constant
  - `--worker-ms-pattern` comma-separated repetition pattern
  - `--worker-ms-jitter` deterministic jitter around base/pattern value
- A bounded input queue (`--queue-capacity`) is maintained.
- On output deadlines the simulator applies one policy:
  - `drop-oldest`: cancel stale pending/in-flight work at the missed deadline.
  - `silence-on-underrun`: emit silence and allow stale work to finish, then drop
    it as late.
  - `reuse-last`: replay the last delivered chunk when available and still track
    the compute deadline miss.

## Metrics

The final JSON summary reports:

- `accumulated_delay_ms` — sum of lateness `(completion - deadline)` for every processed chunk.
- `deadline_miss_events` — number of output time slots without an on-time output chunk.
- `underrun_events` — number of audible/silent underrun slots; `reuse-last` can
  hide a missed deadline from this counter when a previous chunk exists.
- `dropped_chunks` — chunks dropped due to queue overflow, stale completion, or
  policy cancellation.
- `max_queue_depth` — max input queue depth seen during simulation.

## Run

```bash
cd experiments/chunk-scheduler-sim
cargo run -- \
  --duration-ms 1000 \
  --chunk-ms 100 \
  --hop-ms 50 \
  --worker-ms 120 \
  --worker-ms-pattern 120,180,90 \
  --policy drop-oldest \
  --queue-capacity 6 \
  --trace
```

## Scenarios tested

### S1 — fast worker keeps up

- `--duration-ms 200 --chunk-ms 40 --hop-ms 50 --worker-ms 30`
- Expect no misses, no drops, small queue depth.

### S2 — slow worker misses deadlines

- `--duration-ms 400 --chunk-ms 100 --hop-ms 100 --worker-ms 300`
- Expect repeated underruns and deadline misses, non-zero accumulated delay and drops.

### S3 — reuse vs silence policy

- Compare `--policy reuse-last` and `--policy silence-on-underrun` with identical timing to inspect output behavior in miss intervals.

### S4 — stale work shedding

- Compare `--policy drop-oldest` and `--policy silence-on-underrun` with
  `--duration-ms 800 --chunk-ms 100 --hop-ms 100 --worker-ms 260`.
- Expected shape: both miss deadlines, but `drop-oldest` keeps less queued stale
  work and lower accumulated computation lateness.

## Interpretation for ADR

- If `deadline_miss_events` grows quickly while queue is not yet full, bounded latency at the output stage is the main issue, not input ingress.
- `max_queue_depth` tracks pressure before output effects become irreversible.
- `accumulated_delay_ms` helps estimate practical latency growth under sustained overload.
- Replacing miss events with last chunk (`reuse-last`) gives smoother output continuity than silence but does not reduce true lateness of computation.
- Dropping stale work (`drop-oldest`) can bound compute latency, but the audible
  result is still discontinuous; it is a recovery policy, not a quality win.
