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
- Human HTML docs should link to these files instead of duplicating every technical detail.
- Any contradiction between agent docs and ADRs is resolved in favor of accepted ADRs.

## Decisions

- [../adr/0006-separate-agent-docs-and-human-html-docs.md](../adr/0006-separate-agent-docs-and-human-html-docs.md)

## History

- 2026-05-30: Existing planning documents moved under `docs/agent/`.
- 2026-05-30: Architecture and governance docs added.

## Open Questions

- Whether agent docs should later be generated into a searchable static site.
