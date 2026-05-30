# 0001. Record architecture decisions with MADR

Status: accepted
Date: 2026-05-30

## Context and Problem

`vc-runtime` will contain long-running architectural choices around realtime audio, inference providers, model formats, daemon protocols, and documentation rules. These decisions should not live only in chat history or broad design documents.

## Decision Drivers

- Decisions must be easy for agents and humans to find.
- The project needs a stable memory of why choices were made.
- The format should be lightweight enough to use frequently.
- Rejected and superseded decisions should remain auditable.

## Considered Options

- No formal decision records.
- Free-form design notes.
- Nygard-style ADRs.
- MADR-style markdown ADRs.

## Decision

Use numbered markdown ADRs based on MADR-style structure in `docs/adr/`.

## Consequences

Positive:

- Important decisions become searchable and reviewable.
- Future agents can avoid re-litigating settled choices.
- Superseded decisions preserve architectural history.

Negative:

- Contributors must maintain small decision records.
- Some small changes will require judgment about whether an ADR is necessary.

## Links

- [template.md](template.md)
- [../memory/project-governance.md](../memory/project-governance.md)
