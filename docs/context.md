# Context

## Purpose

This folder contains project documentation.

## Current Shape

- `glossary.md` contains shared vocabulary.
- `agent/` contains detailed technical markdown for agents and developers.
- `human/` contains visual HTML documentation for people.
- `adr/` contains numbered architecture decision records.
- `rfc/` is reserved for proposed larger changes before they become implementation work.

## Public Contracts

- Technical source-of-truth documents live in markdown.
- Human HTML documents should be visual summaries and must link back to technical markdown sources.
- Architecture decisions must be recorded in `adr/` when they affect module boundaries, deployment, public APIs, provider strategy, or long-term maintenance.

## Decisions

- [0001. Record architecture decisions with MADR](adr/0001-record-architecture-decisions-with-madr.md)
- [0006. Separate agent docs and human HTML docs](adr/0006-separate-agent-docs-and-human-html-docs.md)
- [0007. Require context files in project folders](adr/0007-require-context-md-in-project-folders.md)

## History

- 2026-05-30: Documentation structure split into agent, human, ADR, and RFC areas.

## Open Questions

- Whether HTML docs should be generated from markdown later or maintained manually as visual summaries.
