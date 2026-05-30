# Context

## Purpose

This folder contains detailed technical markdown documents for agents and developers.

## Current Shape

- `vision.md` defines the product and technical direction.
- `phase-0-research-plan.md` defines the first research phase.
- `mvp-scope.md` narrows the first MVP.
- `project-governance.md` defines maintenance rules.
- `architecture.md` summarizes the intended technical architecture.
- `full-design-draft.md` preserves the long-form design draft.
- `runtime-idea.md` preserves the earlier focused runtime idea.

## Public Contracts

- Agent docs can be long and technical.
- These docs are allowed to contain implementation-level constraints.
- Any contradiction between agent docs and ADRs is resolved in favor of accepted ADRs.

## Decisions

- [../adr/0008-use-markdown-only-project-documentation.md](../adr/0008-use-markdown-only-project-documentation.md)

## History

- 2026-05-30: Existing planning documents moved under `docs/agent/`.
- 2026-05-30: Architecture and governance docs added.
- 2026-05-31: Markdown kept as the only documentation format.

## Open Questions

- Whether agent docs should later be generated into a searchable static site.
