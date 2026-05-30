# Context

## Purpose

This repository contains `vc-runtime`: a local-first realtime voice conversion runtime focused on low latency, stable audio scheduling, provider transparency, and diagnostics-first model inference.

## Current Shape

The repository is in Phase 0. It currently contains planning documents, project memory rules, ADRs, and documentation scaffolding. Runtime code has not started yet.

## Public Contracts

- Every folder must contain a `context.md` file.
- Detailed technical project memory lives in `docs/memory/`.
- Architectural decisions are recorded as numbered ADRs in `docs/adr/`.
- Shared vocabulary is recorded in `docs/glossary.md`.

## Decisions

- [0001. Record architecture decisions with MADR](docs/adr/0001-record-architecture-decisions-with-madr.md)
- [0002. Use Rust for realtime runtime](docs/adr/0002-use-rust-for-realtime-runtime.md)
- [0003. Use CPAL for Phase 0 audio](docs/adr/0003-use-cpal-for-phase-0-audio.md)
- [0004. Use ONNX Runtime as mainline inference](docs/adr/0004-use-onnx-runtime-as-mainline-inference.md)
- [0005. Use daemon-first architecture](docs/adr/0005-use-daemon-first-architecture.md)
- [0006. Separate agent docs and human HTML docs](docs/adr/0006-separate-agent-docs-and-human-html-docs.md) - superseded
- [0007. Require context files in project folders](docs/adr/0007-require-context-md-in-project-folders.md)
- [0008. Use markdown-only project documentation](docs/adr/0008-use-markdown-only-project-documentation.md)
- [0009. Rename docs/agent to docs/memory](docs/adr/0009-rename-agent-docs-to-memory.md)

## History

- 2026-05-30: Repository initialized.
- 2026-05-30: Planning docs imported from personal notes.
- 2026-05-30: Documentation architecture, glossary, folder contexts, and ADR process added.
- 2026-05-31: Human HTML documentation layer removed; markdown kept as the single project documentation format.
- 2026-05-31: `docs/agent/` renamed to `docs/memory/`.

## Open Questions

- Which RVC model bundle format should be supported first?
- Which exact CPAL backend behavior is acceptable for Phase 0 passthrough?
- Which provider path becomes the second production target after CUDA?
