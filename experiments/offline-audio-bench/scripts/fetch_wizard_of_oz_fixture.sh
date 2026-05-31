#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXPERIMENT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

SOURCE_URL="https://commons.wikimedia.org/wiki/Special:Redirect/file/LibriVox%20-%20The%20Wonderful%20Wizard%20of%20Oz%2001.ogg"
OUTPUT_DIR="$EXPERIMENT_DIR/fixtures/audio"
RAW_OGG="$OUTPUT_DIR/wizard-of-oz-01.ogg"
OUTPUT_WAV="$OUTPUT_DIR/wizard-of-oz-01-16k-mono.wav"
DURATION_SECONDS=120

usage() {
  cat <<'EOF'
fetch_wizard_of_oz_fixture.sh

Download a public-domain LibriVox/Wikimedia speech fixture and prepare a
16-bit PCM, 16 kHz, mono WAV for offline-audio-bench.

Usage:
  fetch_wizard_of_oz_fixture.sh [--duration-seconds N]
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --duration-seconds)
      DURATION_SECONDS="$2"
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

if ! [[ "$DURATION_SECONDS" =~ ^[0-9]+$ ]] || (( DURATION_SECONDS < 1 )); then
  echo "--duration-seconds must be a positive integer" >&2
  exit 1
fi

command -v curl >/dev/null || {
  echo "curl is required" >&2
  exit 1
}
command -v ffmpeg >/dev/null || {
  echo "ffmpeg is required to convert the source audio to WAV" >&2
  exit 1
}

mkdir -p "$OUTPUT_DIR"

curl --fail --location --show-error --output "$RAW_OGG" "$SOURCE_URL"
ffmpeg -y -hide_banner -loglevel error \
  -i "$RAW_OGG" \
  -t "$DURATION_SECONDS" \
  -ac 1 \
  -ar 16000 \
  -sample_fmt s16 \
  "$OUTPUT_WAV"

echo "$OUTPUT_WAV"
