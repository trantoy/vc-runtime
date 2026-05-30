# Context

## Purpose

This folder contains assets for human HTML documentation.

## Current Shape

- `site.css` defines the visual language for documentation pages.
- `pipeline.js` renders small SVG diagrams for the pipeline cards.

## Public Contracts

- Assets must be local and not depend on external CDNs.
- HTML docs should remain readable if JavaScript fails.
- JavaScript visualizations should enhance the page, not hide core content.

## Decisions

- [../../adr/0006-separate-agent-docs-and-human-html-docs.md](../../adr/0006-separate-agent-docs-and-human-html-docs.md)

## History

- 2026-05-30: Initial human docs assets added.

## Open Questions

- Whether to add generated diagrams from a structured architecture model later.
