# Context

## Purpose

This folder contains visual human-facing documentation in HTML.

## Current Shape

- `index.html` is the entry point.
- `architecture.html` summarizes system architecture with visual diagrams.
- `phase-0.html` explains the first research phase.
- `glossary.html` presents important vocabulary.
- `assets/` contains CSS and JavaScript used by these pages.

## Public Contracts

- Human docs should be readable without knowing the whole codebase.
- Human docs should link back to source markdown in `../agent/` and `../glossary.md`.
- Human docs may use SVG and JavaScript visualizations.
- Human docs are summaries; accepted ADRs and agent docs remain the technical source of truth.

## Decisions

- [../adr/0006-separate-agent-docs-and-human-html-docs.md](../adr/0006-separate-agent-docs-and-human-html-docs.md)

## History

- 2026-05-30: Human documentation skeleton added.

## Open Questions

- Whether these pages should be hand-maintained or generated from markdown plus structured metadata.
