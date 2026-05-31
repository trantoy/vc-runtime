# Audio soak + queue metrics experiment

## Purpose

This folder contains a throwaway prototype for Phase 0.2: long-running
passthrough soak measurement and lightweight interpretation of queue health from
existing `vc-cli` metrics output.

## Scope

- Keep this prototype isolated under `experiments/audio-soak-metrics/`.
- Do not modify production crates (`vc-core`, `vc-audio`, `vc-cli`) or root
  workspace files.
- Do not require any additional dependencies outside `bash`, `cargo`, and core
  Unix tools.
- Produce machine-readable summaries (JSONL/CSV) and enough text output to make
  long soak runs inspectable.

## Notes

- The script reads existing CLI logs in the format currently printed by
  `vc-cli::format_passthrough_metrics`.
- It is intentionally lightweight and does not change runtime behavior.

## Open Questions (for architecture review)

- Should startup underruns be treated as a warmup artifact or a regression?
- Is derived queue growth (`pushed_frames - popped_frames`) sufficient as a
  queue-depth proxy until queue depth gauge exists?
- What threshold of sustained queue growth is unacceptable for Phase 0.2 acceptance?
