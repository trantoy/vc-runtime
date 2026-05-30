# 0012. Build Phase 0 passthrough with CPAL and rtrb

Status: accepted
Date: 2026-05-31

## Context and Problem

Phase 0.1 needs the first measurable realtime audio path before model inference, daemon control, or UI work starts. The runtime must move audio from input callbacks to output callbacks without blocking and must expose basic metrics.

The implementation also needs a narrow public API so CLI code can start and monitor passthrough without owning CPAL streams or callback logic.

## Decision Drivers

- Audio callbacks must avoid locks, blocking calls, and model inference.
- CLI code must not own CPAL callbacks or ring-buffer runtime behavior.
- Metrics must reuse the Phase 0 audio metrics schema from ADR 0011.
- Stream error callbacks must not format or print errors on realtime threads.
- The first implementation must stay small enough for hardware smoke testing.
- Device indices are current enumeration indices and may change between runs.

## Considered Options

- Put passthrough directly in `vc-cli`.
- Create a broad audio manager that owns devices, streams, metrics, and future model hooks.
- Create a narrow `PassthroughSession` in `vc-audio` backed by CPAL streams and an `rtrb` single-producer/single-consumer ring buffer.

## Decision

Create `vc_audio::passthrough::PassthroughSession`.

`PassthroughSession::start()`:

- selects default devices or process-local input/output indices;
- reads default CPAL input and output stream configs;
- requires matching sample rate and channel count for Phase 0.1;
- stores audio in an `rtrb::RingBuffer<f32>`;
- converts backend sample formats at the callback boundary;
- records callbacks, pushed/popped frames, underrun events, overrun events, and stream error events through `vc-core` metrics;
- increments stream error counters in CPAL error callbacks without formatting, printing, or locking;
- applies Phase 0.1 upper bounds to `capacity_frames` and total ring-buffer samples before allocation;
- reports selected input/output device names when passthrough starts;
- keeps CPAL stream handles private inside `vc-audio`.

`vc-cli` owns command parsing, sleeping for the requested duration, and printing one metrics snapshot per second.

## Consequences

Positive:

- Realtime data movement stays out of CLI code.
- The data path has no mutex in the input/output audio callbacks.
- The same metrics schema can be reused by later diagnostics.
- The ring buffer boundary is explicit and testable without real audio hardware.
- Stream errors are visible as counters without blocking stream callbacks.

Negative:

- Phase 0.1 rejects mismatched input/output sample rates and channel counts instead of resampling or remapping channels.
- Device index selection still re-enumerates devices at start, so indices may select a different device if backend order changes between runs.
- The Phase 0.1 total-sample cap assumes conventional device channel counts and may need revision for high-channel-count interfaces.
- Runtime stream errors are counts only; detailed error messages require a later non-realtime diagnostics channel.

## Links

- [0011. Define Phase 0 audio metrics schema](0011-define-phase-0-audio-metrics-schema.md)
- [../memory/phases/phase-0/phase-0-1-audio-passthrough-plan.md](../memory/phases/phase-0/phase-0-1-audio-passthrough-plan.md)
- [../../crates/vc-audio/context.md](../../crates/vc-audio/context.md)
- [../../crates/vc-cli/context.md](../../crates/vc-cli/context.md)
