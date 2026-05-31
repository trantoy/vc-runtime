# 0013. Treat architecture memory docs as normative guidance

Status: accepted
Date: 2026-05-31

## Context and Problem

`docs/memory/architecture-guide.md`, `docs/memory/roadmap.md`, and
`docs/memory/runtime-architecture-v1.md` contain recurring architecture rules,
phase gates, dependency constraints, and extension-point policy. These documents
use terms such as "must", "forbidden", and "requires".

Without an ADR, those rules would look like informal notes even though future
agents and contributors are expected to follow them.

## Decision Drivers

- Architecture rules must be discoverable and enforceable during review.
- Accepted ADRs must remain the highest-priority durable architecture records.
- Long-form memory documents need room for detailed guidance without turning
  every rule into its own ADR.
- Draft target architecture must not override accepted decisions.

## Considered Options

- Keep architecture memory documents as non-binding notes.
- Convert every rule in memory documents into a separate ADR.
- Treat selected architecture memory documents as normative guidance subordinate
  to accepted ADRs.

## Decision

Treat these documents as normative project guidance:

- `docs/memory/architecture-guide.md`
- `docs/memory/roadmap.md`
- `docs/memory/runtime-architecture-v1.md`
- `docs/memory/project-governance.md`

They may contain recurring rules, review criteria, phase gates, and target
architecture constraints.

Accepted ADRs still take precedence over all memory documents. If an accepted
ADR conflicts with a memory document, the ADR wins until a later ADR supersedes
it.

Draft target architecture sections are binding as review guidance, not as proof
that the described components already exist or that they should all be
implemented immediately.

## Consequences

Positive:

- Architecture review can cite long-form memory documents as project rules.
- Agents can follow detailed guidance without re-litigating the same principles.
- ADRs stay focused on significant decisions instead of every repeated rule.
- Draft architecture can guide implementation while remaining subordinate to
  accepted ADRs and phase evidence.

Negative:

- Memory documents must be maintained with the same care as decision records.
- Contributors must distinguish target architecture from implemented code.
- Broad guidance can still become stale if phase results disprove assumptions.

## Links

- [../memory/architecture-guide.md](../memory/architecture-guide.md)
- [../memory/roadmap.md](../memory/roadmap.md)
- [../memory/runtime-architecture-v1.md](../memory/runtime-architecture-v1.md)
- [../memory/project-governance.md](../memory/project-governance.md)
- [0001. Record architecture decisions with MADR](0001-record-architecture-decisions-with-madr.md)
