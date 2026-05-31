#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

run_valid_fixture() {
  local tmp
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN

  "$SCRIPT_DIR/run_audio_soak.sh" \
    --dry-run \
    --fixture sample-passthrough-metrics.txt \
    --duration 5 \
    --format both \
    --output-dir "$tmp" >/dev/null

  grep -q '"duration_reported_sec":5' "$tmp/audio-soak.summary.jsonl" ||
    fail "valid fixture did not report duration_reported_sec=5"

  local header
  header="$(head -n 1 "$tmp/audio-soak.summary.csv")"
  [[ "$header" != *"passthrough_timing_sec"* ]] ||
    fail "CSV header still exposes stale passthrough_timing_sec field"
  [[ "$header" == *"duration_reported_sec"* ]] ||
    fail "CSV header is missing duration_reported_sec"
}

reject_missing_required_metric() {
  local tmp fixture stderr_file
  tmp="$(mktemp -d)"
  fixture="$tmp/missing-popped-frames.log"
  stderr_file="$tmp/stderr.log"
  trap 'rm -rf "$tmp"' RETURN

  cat > "$fixture" <<'LOG'
Passthrough started: input_device="Mic" output_device="Spk" sample_rate_hz=48000 channels=2 capacity_frames=48000
t=1s input_cb=10 output_cb=10 pushed_frames=48000 underrun_events=0 overrun_events=0 input_stream_error_events=0 output_stream_error_events=0
LOG

  if "$SCRIPT_DIR/run_audio_soak.sh" \
    --dry-run \
    --fixture "$fixture" \
    --duration 1 \
    --format jsonl \
    --output-dir "$tmp/out" >/dev/null 2>"$stderr_file"; then
    fail "invalid fixture succeeded"
  fi

  grep -q "missing required metric field" "$stderr_file" ||
    fail "invalid fixture did not explain the missing metric"
}

reject_zero_duration_metric() {
  local tmp fixture stderr_file
  tmp="$(mktemp -d)"
  fixture="$tmp/zero-duration.log"
  stderr_file="$tmp/stderr.log"
  trap 'rm -rf "$tmp"' RETURN

  cat > "$fixture" <<'LOG'
Passthrough started: input_device="Mic" output_device="Spk" sample_rate_hz=48000 channels=2 capacity_frames=48000
t=0s input_cb=10 output_cb=10 pushed_frames=48000 popped_frames=48000 underrun_events=0 overrun_events=0 input_stream_error_events=0 output_stream_error_events=0
LOG

  if "$SCRIPT_DIR/run_audio_soak.sh" \
    --dry-run \
    --fixture "$fixture" \
    --duration 1 \
    --format jsonl \
    --output-dir "$tmp/out" >/dev/null 2>"$stderr_file"; then
    fail "zero-duration fixture succeeded"
  fi

  grep -q "duration_reported_sec must be > 0" "$stderr_file" ||
    fail "zero-duration fixture did not explain invalid duration"
}

run_valid_fixture
reject_missing_required_metric
reject_zero_duration_metric
echo "audio-soak tests passed"
