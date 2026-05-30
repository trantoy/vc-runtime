# 0006. Separate agent docs and human HTML docs

Status: accepted
Date: 2026-05-30

## Context and Problem

The project needs detailed technical memory for agents and developers, but humans also need more visual, navigable documentation. A single markdown document is not ideal for both needs.

## Decision Drivers

- Agents need long markdown documents with technical constraints.
- Humans benefit from diagrams, tables, visual summaries, and interactive explanations.
- HTML docs can contain SVG and JavaScript visualizations.
- Documentation should avoid source-of-truth conflicts.

## Considered Options

- Markdown-only documentation.
- HTML-only documentation.
- Separate technical markdown and human HTML summaries.
- Generate all docs from one source immediately.

## Decision

Keep detailed technical docs in `docs/agent/` as markdown.

Keep visual human-facing docs in `docs/human/` as HTML with local assets.

HTML docs summarize and link to technical markdown sources. ADRs remain authoritative for accepted decisions.

## Consequences

Positive:

- Agents get rich technical context.
- Humans get easier visual navigation.
- HTML pages can later include richer pipeline and latency visualizations.

Negative:

- Docs can drift if not maintained.
- HTML summaries require discipline to link back to source markdown.

## Links

- [../agent/context.md](../agent/context.md)
- [../human/context.md](../human/context.md)
