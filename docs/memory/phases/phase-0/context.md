# Context

## Purpose

This folder contains Phase 0 plans and results.

Phase 0 turns the `vc-runtime` design into measured facts before the project commits to a larger architecture.

## Current Shape

- `phase-0-research-plan.md` defines parent Phase 0 research scope.
- `phase-0-1-audio-passthrough-plan.md` defines the first implementation plan.
- `results.md` records verified Phase 0 evidence.
- Phase 0.1 implementation currently has device listing and bounded passthrough commands.
- `vc-bench` exists for offline prerecorded-audio benchmark reports and can be
  used as supporting evidence, while live audio soak tests remain Phase 0 work.

## Public Contracts

- Phase 0 work must stay evidence-driven.
- Phase 0.1 does not include ML, ONNX, daemon, UI, Tauri, provider manager, or RVC inference.
- Audio callbacks must not run model inference.
- Implementation steps require strict review after each completed task.
- `results.md` is the rolling Phase 0 evidence log.
- `results.md` must distinguish compile/test verification from hardware/audio runtime verification.

## Decisions

- [../../../adr/0003-use-cpal-for-phase-0-audio.md](../../../adr/0003-use-cpal-for-phase-0-audio.md)
- [../../../adr/0004-use-onnx-runtime-as-mainline-inference.md](../../../adr/0004-use-onnx-runtime-as-mainline-inference.md)
- [../../../adr/0010-store-phase-plans-under-memory-phases.md](../../../adr/0010-store-phase-plans-under-memory-phases.md)
- [../../../adr/0012-build-phase-0-passthrough-with-cpal-rtrb.md](../../../adr/0012-build-phase-0-passthrough-with-cpal-rtrb.md)

## History

- 2026-05-31: Phase 0.1 audio passthrough planning started.
- 2026-05-31: Parent Phase 0 research plan moved into this folder to avoid split ownership.
- 2026-05-31: Phase 0.1 bounded CPAL/rtrb passthrough implemented.
- 2026-05-31: Phase 0 results log started with Phase 0.1 verification.
- 2026-05-31: Offline benchmark prototype promoted into `vc-bench`.

## Open Questions

- Which CPAL device configurations work on the first development machine?
- Whether Phase 0.2 should add channel remapping or keep exact stream-shape matching longer.
- Which `vc-bench` provenance fields are required before comparing laptop and
  main PC performance.
