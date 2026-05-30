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
- [0008. Use markdown-only project documentation](0008-use-markdown-only-project-documentation.md)
- [0009. Rename docs/agent to docs/memory](0009-rename-agent-docs-to-memory.md)

## History

- 2026-05-30: ADR folder and initial decision records added.
- 2026-05-31: ADR 0008 superseded the earlier human HTML docs decision.
- 2026-05-31: ADR 0009 renamed `docs/agent/` to `docs/memory/`.

## Open Questions

- Whether to add ADR linting once the repository has CI.
