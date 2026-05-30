# 0008. Use markdown-only project documentation

Status: accepted
Date: 2026-05-31

## Context and Problem

The project initially split documentation into detailed markdown for agents and HTML pages for human-facing visual summaries. The user found markdown easier to read and wanted to remove duplicate documentation surfaces.

Maintaining both markdown and HTML would create drift before the project even has runtime code.

## Decision Drivers

- The user prefers markdown.
- The project is early and should minimize documentation duplication.
- Agents work well with markdown source files.
- ADRs, glossary, and context files already provide structure without HTML.
- Visual docs can be reintroduced later if a concrete need appears.

## Considered Options

- Keep markdown and HTML docs.
- Generate HTML from markdown immediately.
- Remove HTML docs and keep markdown as the single documentation format.

## Decision

Use markdown as the only project documentation format for now.

Remove `docs/human/` and keep:

- `context.md` files for local project memory;
- `docs/glossary.md` for shared terms;
- `docs/memory/` for detailed technical project memory;
- `docs/adr/` for architecture decisions;
- `docs/rfc/` for larger proposals.

## Consequences

Positive:

- Less duplication.
- Lower maintenance cost.
- Markdown remains easy for both the user and agents.
- ADR/context/glossary rules stay intact.

Negative:

- No interactive HTML visualizations for now.
- Architecture diagrams must be represented as markdown tables, Mermaid, ASCII diagrams, or linked generated artifacts if added later.

## Links

- [0006. Separate agent docs and human HTML docs](0006-separate-agent-docs-and-human-html-docs.md)
- [../memory/project-governance.md](../memory/project-governance.md)
