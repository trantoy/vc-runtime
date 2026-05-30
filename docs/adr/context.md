# Context

## Purpose

This folder stores Architecture Decision Records.

## Current Shape

- `template.md` defines the project ADR format.
- Numbered files `NNNN-title.md` record accepted, proposed, rejected, or superseded decisions.

## Public Contracts

- One ADR records one decision.
- ADR numbers are never reused.
- Accepted ADRs override older conflicting design docs.
- Superseded ADRs stay in the repository with a link to the replacing ADR.
- New architecture-significant decisions must be recorded here before implementation work depends on them.

## Decisions

- [0001. Record architecture decisions with MADR](0001-record-architecture-decisions-with-madr.md)
- [0002. Use Rust for realtime runtime](0002-use-rust-for-realtime-runtime.md)
- [0003. Use CPAL for Phase 0 audio](0003-use-cpal-for-phase-0-audio.md)
- [0004. Use ONNX Runtime as mainline inference](0004-use-onnx-runtime-as-mainline-inference.md)
- [0005. Use daemon-first architecture](0005-use-daemon-first-architecture.md)
- [0006. Separate agent docs and human HTML docs](0006-separate-agent-docs-and-human-html-docs.md) - superseded
- [0007. Require context files in project folders](0007-require-context-md-in-project-folders.md)
- [0008. Use markdown-only project documentation](0008-use-markdown-only-project-documentation.md)
- [0009. Rename docs/agent to docs/memory](0009-rename-agent-docs-to-memory.md)
- [0010. Store phase plans under docs/memory/phases](0010-store-phase-plans-under-memory-phases.md)
- [0011. Define Phase 0 audio metrics schema](0011-define-phase-0-audio-metrics-schema.md)
- [0012. Build Phase 0 passthrough with CPAL and rtrb](0012-build-phase-0-passthrough-with-cpal-rtrb.md)

## History

- 2026-05-30: ADR folder and initial decision records added.
- 2026-05-31: ADR 0008 superseded the earlier human HTML docs decision.
- 2026-05-31: ADR 0009 renamed `docs/agent/` to `docs/memory/`.
- 2026-05-31: ADR 0010 introduced phase-specific plan folders.
- 2026-05-31: ADR 0011 recorded the initial public audio metrics schema.
- 2026-05-31: ADR 0012 recorded the first CPAL/rtrb passthrough runtime boundary.

## Open Questions

- Whether to add ADR linting once the repository has CI.
