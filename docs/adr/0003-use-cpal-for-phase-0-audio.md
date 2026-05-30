# 0003. Use CPAL for Phase 0 audio

Status: accepted
Date: 2026-05-30

## Context and Problem

Phase 0 must prove basic audio passthrough across desktop platforms without starting with custom platform-specific audio backends.

## Decision Drivers

- The first audio spike should be small.
- Windows, Linux, and macOS should be explored early.
- Platform-specific escape hatches should remain possible.
- The project needs actual measurements before deciding whether CPAL is enough.

## Considered Options

- CPAL-first audio layer.
- Separate platform-native backends from day one.
- JACK/PipeWire-first Linux-only prototype.
- PortAudio.

## Decision

Use CPAL for Phase 0 audio passthrough and device enumeration.

Do not assume CPAL solves every production audio issue. Treat it as the first measurement layer.

## Consequences

Positive:

- Fast Rust-native start for desktop audio.
- Early visibility into cross-platform device behavior.
- Keeps Phase 0 focused on evidence.

Negative:

- CPAL backend quirks may require later platform-specific layers.
- Realtime priority and buffer tuning remain application responsibilities.

## Links

- [../memory/phase-0-research-plan.md](../memory/phase-0-research-plan.md)
- [../memory/mvp-scope.md](../memory/mvp-scope.md)
