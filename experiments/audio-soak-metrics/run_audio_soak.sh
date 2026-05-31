#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

DEFAULT_DURATION_SECONDS=5
DEFAULT_RUNS=1
DEFAULT_FORMAT="both"
DEFAULT_CAPACITY_FRAMES=48000
DEFAULT_OUTPUT_PREFIX="audio-soak"

DURATION_SECONDS="$DEFAULT_DURATION_SECONDS"
RUNS="$DEFAULT_RUNS"
INPUT_INDEX=""
OUTPUT_INDEX=""
CAPACITY_FRAMES=""
FORMAT="$DEFAULT_FORMAT"
OUTPUT_DIR="$SCRIPT_DIR"
OUTPUT_PREFIX="$DEFAULT_OUTPUT_PREFIX"
DRY_RUN=0
FIXTURE_FILE=""
RERUN_DELAY_SECONDS=0

usage() {
  cat <<'EOF'
run_audio_soak.sh

Run bounded passthrough for N runs and summarize CLI metrics output.

Usage:
  run_audio_soak.sh [options]

Options:
  --duration <seconds>         passthrough duration per run (default: 5)
  --runs <n>                   number of runs (default: 1)
  --input-index <n>            passthrough --input-index
  --output-index <n>           passthrough --output-index
  --capacity-frames <n>        passthrough --capacity-frames
  --format <jsonl|csv|both>    output summary format (default: both)
  --output-dir <path>          where logs and summaries are written
  --output-prefix <name>        output file prefix (default: audio-soak)
  --dry-run                    do not touch audio devices
  --fixture <path>             parse fixture instead of running CLI (works with --dry-run)
  --rerun-delay <seconds>      sleep between runs (default: 0)
  -h, --help                  show this help

Examples:
  ./run_audio_soak.sh --duration 5 --runs 2
  ./run_audio_soak.sh --dry-run --fixture sample-passthrough-metrics.txt
  ./run_audio_soak.sh --duration 1800 --format both --output-dir ./artifacts
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --duration)
      DURATION_SECONDS="$2"
      shift 2
      ;;
    --runs)
      RUNS="$2"
      shift 2
      ;;
    --input-index)
      INPUT_INDEX="$2"
      shift 2
      ;;
    --output-index)
      OUTPUT_INDEX="$2"
      shift 2
      ;;
    --capacity-frames)
      CAPACITY_FRAMES="$2"
      shift 2
      ;;
    --format)
      FORMAT="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --output-prefix)
      OUTPUT_PREFIX="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --fixture)
      FIXTURE_FILE="$2"
      DRY_RUN=1
      shift 2
      ;;
    --rerun-delay)
      RERUN_DELAY_SECONDS="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ "$FORMAT" != "jsonl" && "$FORMAT" != "csv" && "$FORMAT" != "both" ]]; then
  echo "--format must be one of: jsonl, csv, both" >&2
  exit 1
fi

if ! [[ "$DURATION_SECONDS" =~ ^[0-9]+$ ]] || (( DURATION_SECONDS < 1 )); then
  echo "--duration must be a positive integer" >&2
  exit 1
fi

if ! [[ "$RUNS" =~ ^[0-9]+$ ]] || (( RUNS < 1 )); then
  echo "--runs must be a positive integer" >&2
  exit 1
fi

if ! [[ "$RERUN_DELAY_SECONDS" =~ ^[0-9]+$ ]]; then
  echo "--rerun-delay must be >= 0" >&2
  exit 1
fi

if [[ "$DRY_RUN" -eq 1 && -n "$FIXTURE_FILE" && ! -f "$SCRIPT_DIR/$FIXTURE_FILE" && ! -f "$FIXTURE_FILE" ]]; then
  echo "--fixture file not found: $FIXTURE_FILE" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"

JSONL_PATH="${OUTPUT_DIR}/${OUTPUT_PREFIX}.summary.jsonl"
CSV_PATH="${OUTPUT_DIR}/${OUTPUT_PREFIX}.summary.csv"

CSV_HEADER="run_id,status,timestamp_utc,duration_requested_sec,duration_reported_sec,sample_rate_hz,channels,capacity_frames,input_cb,output_cb,pushed_frames,popped_frames,input_stream_error_events,output_stream_error_events,underrun_events,overrun_events,input_cb_rate_per_sec,output_cb_rate_per_sec,push_minus_pop,queue_backlog_last,queue_backlog_max,queue_growth,queue_growth_per_sec,startup_underruns,startup_overruns,metric_sample_count"

if [[ "$FORMAT" == "csv" || "$FORMAT" == "both" ]]; then
  if [[ ! -f "$CSV_PATH" ]]; then
    echo "$CSV_HEADER" > "$CSV_PATH"
  fi
fi

build_cli_command() {
  local -a cmd=(cargo run -p vc-cli -- passthrough --seconds "$DURATION_SECONDS")
  if [[ -n "$INPUT_INDEX" ]]; then
    cmd+=(--input-index "$INPUT_INDEX")
  fi
  if [[ -n "$OUTPUT_INDEX" ]]; then
    cmd+=(--output-index "$OUTPUT_INDEX")
  fi
  if [[ -n "$CAPACITY_FRAMES" ]]; then
    cmd+=(--capacity-frames "$CAPACITY_FRAMES")
  fi
  printf '%s\0' "${cmd[@]}"
}


emit_synthetic_log() {
  local output_file="$1"
  local sec=1
  local input_cb=0
  local output_cb=0
  local pushed=0
  local popped=0
  local underruns=2

  {
    echo "Passthrough started: input_device=\"DryRunMic\" output_device=\"DryRunSpk\" sample_rate_hz=48000 channels=2 capacity_frames=48000"
    while (( sec <= DURATION_SECONDS )); do
      input_cb=$((input_cb + 21 + (sec - 1)))
      output_cb=$input_cb
      pushed=$((pushed + 10000 + (sec - 1) * 5))
      popped=$((popped + 9900 + (sec - 1) * 3))
      if (( sec >= 3 )); then
        underruns=2
      fi
      echo "t=${sec}s input_cb=${input_cb} output_cb=${output_cb} pushed_frames=${pushed} popped_frames=${popped} underrun_events=${underruns} overrun_events=0 input_stream_error_events=0 output_stream_error_events=0"
      sec=$((sec + 1))
    done
  } > "$output_file"
}

parse_metrics_file() {
  local run_id="$1"
  local status="$2"
  local timestamp="$3"
  local log_file="$4"
  local duration_requested="$5"

  awk -v run_id="$run_id" \
      -v status="$status" \
      -v timestamp="$timestamp" \
      -v duration_requested="$duration_requested" '
	function fail(message) {
	  printf("parse error: %s\n", message) > "/dev/stderr"
	  exit 2
	}

	function require_number(line, field,    pattern, start, raw) {
	  pattern = field "=[0-9]+"
	  if (!match(line, pattern)) {
	    fail("missing required metric field " field " at line " NR)
	  }
	  start = RSTART + length(field) + 1
	  raw = substr(line, start, RLENGTH - length(field) - 1)
	  return raw + 0
	}

	function require_elapsed(line,    raw) {
	  if (!match(line, /^t=[0-9]+s /)) {
	    fail("missing required metric field t at line " NR)
	  }
	  raw = substr(line, 3, RLENGTH - 4)
	  return raw + 0
	}

	BEGIN {
	  sample_count = 0
	  duration_reported = 0
	  sample_rate_hz = 0
	  channels = 0
	  capacity_frames = 0
	  seen_start = 0

	  input_cb = 0
	  output_cb = 0
  pushed_frames = 0
  popped_frames = 0
  input_stream_error_events = 0
  output_stream_error_events = 0
  underrun_events = 0
  overrun_events = 0

  startup_underruns = 0
  startup_overruns = 0
  queue_backlog_max = 0
  queue_backlog_last = 0
  queue_backlog_first = 0
  seen_first_metric = 0
}

	/^Passthrough started:/ {
	  seen_start = 1
	  sample_rate_hz = require_number($0, "sample_rate_hz")
	  channels = require_number($0, "channels")
	  capacity_frames = require_number($0, "capacity_frames")
	  next
	}

	/^t=[0-9]+s / {
	  elapsed = require_elapsed($0)

	  input_cb = require_number($0, "input_cb")
	  output_cb = require_number($0, "output_cb")
	  pushed_frames = require_number($0, "pushed_frames")
	  popped_frames = require_number($0, "popped_frames")
	  underrun_events = require_number($0, "underrun_events")
	  overrun_events = require_number($0, "overrun_events")
	  input_stream_error_events = require_number($0, "input_stream_error_events")
	  output_stream_error_events = require_number($0, "output_stream_error_events")

	  if (!seen_first_metric) {
	    startup_underruns = underrun_events
	    startup_overruns = overrun_events
	    queue_backlog_first = pushed_frames - popped_frames
	    queue_backlog_max = queue_backlog_first
	    seen_first_metric = 1
	  }

	  queue_backlog_last = pushed_frames - popped_frames
	  if (queue_backlog_last > queue_backlog_max) {
	    queue_backlog_max = queue_backlog_last
	  }

  duration_reported = elapsed
  sample_count++
	}

	END {
	  if (!seen_start) {
	    fail("missing Passthrough started line")
	  }
	  if (sample_count == 0) {
	    fail("missing metric sample lines")
	  }
	  if (duration_reported <= 0) {
	    fail("duration_reported_sec must be > 0")
	  }

	  input_cb_rate = input_cb / duration_reported
	  output_cb_rate = output_cb / duration_reported
	  push_minus_pop = pushed_frames - popped_frames
	  queue_growth = queue_backlog_last - queue_backlog_first
	  queue_growth_per_sec = queue_growth / duration_reported

	  printf("{\"run_id\":%d", run_id + 0)
	  printf(",\"status\":\"%s\"", status)
	  printf(",\"timestamp_utc\":\"%s\"", timestamp)
	  printf(",\"duration_requested_sec\":%d", duration_requested + 0)
	  printf(",\"duration_reported_sec\":%d", duration_reported)
	  printf(",\"sample_rate_hz\":%d", sample_rate_hz)
	  printf(",\"channels\":%d", channels)
	  printf(",\"capacity_frames\":%d", capacity_frames)
	  printf(",\"input_cb\":%d", input_cb)
	  printf(",\"output_cb\":%d", output_cb)
	  printf(",\"pushed_frames\":%d", pushed_frames)
	  printf(",\"popped_frames\":%d", popped_frames)
	  printf(",\"underrun_events\":%d", underrun_events)
	  printf(",\"overrun_events\":%d", overrun_events)
	  printf(",\"input_stream_error_events\":%d", input_stream_error_events)
	  printf(",\"output_stream_error_events\":%d", output_stream_error_events)
	  printf(",\"input_cb_rate_per_sec\":%.6f", input_cb_rate)
	  printf(",\"output_cb_rate_per_sec\":%.6f", output_cb_rate)
	  printf(",\"push_minus_pop\":%d", push_minus_pop)
	  printf(",\"queue_backlog_last\":%d", queue_backlog_last)
	  printf(",\"queue_backlog_max\":%d", queue_backlog_max)
	  printf(",\"queue_growth\":%d", queue_growth)
	  printf(",\"queue_growth_per_sec\":%.6f", queue_growth_per_sec)
	  printf(",\"startup_underruns\":%d", startup_underruns)
	  printf(",\"startup_overruns\":%d", startup_overruns)
	  printf(",\"metric_sample_count\":%d", sample_count)
	  printf("}\n")

	  printf("%d,\"%s\",\"%s\",%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%d,%.6f,%.6f,%d,%d,%d,%d,%.6f,%d,%d,%d\n",
	    run_id + 0,
	    status,
	    timestamp,
	    duration_requested + 0,
	    duration_reported,
	    sample_rate_hz,
	    channels,
	    capacity_frames,
	    input_cb,
	    output_cb,
	    pushed_frames,
	    popped_frames,
	    input_stream_error_events,
	    output_stream_error_events,
	    underrun_events,
	    overrun_events,
	    input_cb_rate,
	    output_cb_rate,
	    push_minus_pop,
	    queue_backlog_last,
	    queue_backlog_max,
	    queue_growth,
	    queue_growth_per_sec,
	    startup_underruns,
	    startup_overruns,
	    sample_count)
	}
' "$log_file"
}

run_once() {
  local run_id="$1"
  local timestamp
  local log_file
  local cmd_status="ok"
  local cmd_display=""
  local -a cli_command
  local raw_input_status="real"

  timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  log_file="${OUTPUT_DIR}/${OUTPUT_PREFIX}.run-${run_id}.log"

  if [[ "$DRY_RUN" -eq 1 ]]; then
    if [[ -n "$FIXTURE_FILE" ]]; then
      if [[ -f "$SCRIPT_DIR/$FIXTURE_FILE" ]]; then
        cp "$SCRIPT_DIR/$FIXTURE_FILE" "$log_file"
      elif [[ -f "$FIXTURE_FILE" ]]; then
        cp "$FIXTURE_FILE" "$log_file"
      else
        echo "Fixture missing in dry-run: $FIXTURE_FILE" >&2
        exit 1
      fi
    else
      emit_synthetic_log "$log_file"
      raw_input_status="synthetic"
    fi
    cmd_status="dry-run (${raw_input_status})"
    echo "[dry-run] ${run_id}/${RUNS} log=$log_file"
  else
    IFS=$'\0' read -r -d '' -a cli_command <<< "$(build_cli_command)"
    cmd_display="${cli_command[*]}"
    echo "run=${run_id}/${RUNS} duration=${DURATION_SECONDS}s command: ${cmd_display}"

    set +e
    if (cd "$PROJECT_DIR" && "${cli_command[@]}") 2>&1 | tee "$log_file"; then
      cmd_status="ok"
    else
      cmd_status="failed"
    fi
    set -e
  fi

  parse_output="$(parse_metrics_file "$run_id" "$cmd_status" "$timestamp" "$log_file" "$DURATION_SECONDS")"
  json_line="${parse_output%%$'\n'*}"
  csv_line="${parse_output#*$'\n'}"

  if [[ "$FORMAT" == "jsonl" || "$FORMAT" == "both" ]]; then
    printf '%s\n' "$json_line" >> "$JSONL_PATH"
  fi

  if [[ "$FORMAT" == "csv" || "$FORMAT" == "both" ]]; then
    echo "$csv_line" >> "$CSV_PATH"
  fi

  echo "run=${run_id}/${RUNS} status=${cmd_status} summary json=${json_line}"
}

run_index=1
while (( run_index <= RUNS )); do
  run_once "$run_index"
  if (( run_index < RUNS && RERUN_DELAY_SECONDS > 0 )); then
    sleep "$RERUN_DELAY_SECONDS"
  fi
  run_index=$(( run_index + 1 ))
done

if [[ "$FORMAT" == "jsonl" || "$FORMAT" == "both" ]]; then
  echo "JSONL summary: $JSONL_PATH"
fi
if [[ "$FORMAT" == "csv" || "$FORMAT" == "both" ]]; then
  echo "CSV summary: $CSV_PATH"
fi
