# Benchmark Report v1

## Purpose

This report is the machine-readable contract for offline prerecorded-audio
benchmarks. It exists so local runs, CI checks, future UI views, and later
`vc-bench` reports can compare the same fields without guessing names or units.

## Compatibility

- `schema_version` identifies the report contract.
- Readers must reject unknown major schema versions.
- Field units are part of the contract.
- New optional fields may be added later, but existing field names and units must
  not change within version `1`.

## Fields

| Field | Type | Unit | Meaning |
| --- | --- | --- | --- |
| `schema_version` | integer | none | Report contract version. Current value is `1`. |
| `input_path` | string | path | Input WAV path used for the run. |
| `source_id` | string or null | none | Stable fixture/source id from the source manifest when provided. |
| `input_content_checksum` | integer | none | Deterministic checksum over prepared mono input samples. |
| `build_profile` | string | none | Rust build profile observed by the benchmark binary, currently `debug` or `release`. |
| `sample_rate_hz` | integer | Hz | WAV sample rate after preparation. |
| `channels` | integer | channels | WAV channel count before mono mixdown. |
| `input_frames` | integer | frames | Input audio frame count. |
| `duration_ms` | integer | milliseconds | Input duration derived from frames and sample rate. |
| `chunk_ms` | integer | milliseconds | Configured analysis chunk duration. |
| `hop_ms` | integer | milliseconds | Configured hop between adjacent chunks. |
| `chunk_frames` | integer | frames | Chunk duration converted to frames. |
| `hop_frames` | integer | frames | Hop duration converted to frames. |
| `chunk_count` | integer | chunks | Number of processed chunks. |
| `stage` | string | none | Processing stage name, e.g. `copy`, `gain`, `rms`. |
| `total_processing_ms` | float | milliseconds | Sum of measured per-chunk processing time. |
| `realtime_factor` | float | ratio | `total_processing_ms / duration_ms`; lower is faster. |
| `chunk_processing_p50_us` | integer | microseconds | Median per-chunk processing time. |
| `chunk_processing_p95_us` | integer | microseconds | p95 per-chunk processing time. |
| `chunk_processing_p99_us` | integer | microseconds | p99 per-chunk processing time. |
| `deadline_miss_events` | integer | chunks | Chunks whose processing time exceeded `hop_ms`. |
| `accumulated_delay_ms` | float | milliseconds | Sum of per-chunk time above the hop deadline. |
| `checksum` | integer | none | Deterministic checksum over stage outputs for sanity comparison. |

## Provenance Limits

Version `1` records enough provenance for smoke and early regression checks:
`source_id`, `input_path`, `input_content_checksum`, and `build_profile`.
It does not yet record CPU model, operating system, compiler version, git commit,
or dependency graph. Those fields should be added before making public
cross-machine performance claims.

## Threshold Mode

Threshold mode turns a benchmark into a regression check. The prototype supports:

- `--max-realtime-factor`
- `--max-deadline-misses`

If any threshold fails, the CLI exits non-zero after generating the report. This
lets CI or local scripts fail on performance regressions without parsing logs.
