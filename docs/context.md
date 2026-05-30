# Context

## Purpose

This folder contains project documentation.

## Current Shape

- `glossary.md` contains shared vocabulary.
- `memory/` contains detailed technical project memory.
- `adr/` contains numbered architecture decision records.
- `rfc/` is reserved for proposed larger changes before they become implementation work.

## Public Contracts

- Technical source-of-truth documents live in markdown.
- Architecture decisions must be recorded in `adr/` when they affect module boundaries, deployment, public APIs, provider strategy, or long-term maintenance.

## Decisions

- [0001. Record architecture decisions with MADR](adr/0001-record-architecture-decisions-with-madr.md)
- [0006. Separate agent docs and human HTML docs](adr/0006-separate-agent-docs-and-human-html-docs.md) - superseded
- [0008. Use markdown-only project documentation](adr/0008-use-markdown-only-project-documentation.md)
- [0009. Rename docs/agent to docs/memory](adr/0009-rename-agent-docs-to-memory.md)
- [0007. Require context files in project folders](adr/0007-require-context-md-in-project-folders.md)

## History

- 2026-05-30: Documentation structure added with agent, human, ADR, and RFC areas.
- 2026-05-31: Human HTML documentation layer removed to avoid duplicate docs; markdown kept as the only documentation format.
- 2026-05-31: `agent/` renamed to `memory/`.

## Open Questions

- Whether generated diagrams should be added later inside markdown docs.
