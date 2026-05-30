# 0009. Rename docs/agent to docs/memory

Status: accepted
Date: 2026-05-31

## Context and Problem

After removing the human HTML documentation layer, the name `docs/agent/` became misleading. The folder is not only for agents. It contains the project's durable technical memory: architecture, plans, governance, scope, and long-form design notes.

The user proposed `memory` as a clearer name.

## Decision Drivers

- Folder names should describe purpose, not audience.
- The project already treats documentation as local project memory.
- `docs/memory/` fits the required `context.md` model.
- The name should remain useful for both humans and agents.

## Considered Options

- Keep `docs/agent/`.
- Rename to `docs/technical/`.
- Rename to `docs/memory/`.

## Decision

Rename `docs/agent/` to `docs/memory/`.

Use `docs/memory/` for detailed markdown project memory:

- architecture;
- project governance;
- phase plans;
- MVP scope;
- long-form design notes.

## Consequences

Positive:

- The folder name matches the intended role.
- The documentation model becomes easier to explain.
- Markdown remains the only documentation format.

Negative:

- Existing links must be updated.
- Older ADR 0006 still references the former concept historically.

## Links

- [0008. Use markdown-only project documentation](0008-use-markdown-only-project-documentation.md)
- [../memory/context.md](../memory/context.md)
