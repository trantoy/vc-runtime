# 0010. Store phase plans under docs/memory/phases

Status: accepted
Date: 2026-05-31

## Context and Problem

`docs/memory/` is the project's durable technical memory. Phase plans and execution results will grow quickly. If every plan lives directly in `docs/memory/`, the folder will become noisy and hard to scan.

The user asked for a dedicated subfolder for plans and phases.

## Decision Drivers

- Keep `docs/memory/` readable.
- Keep phase plans close to phase results.
- Preserve local context with `context.md` in each phase folder.
- Avoid mixing long-running design memory with short-lived implementation plans.

## Considered Options

- Keep all plans directly in `docs/memory/`.
- Use `docs/plans/`.
- Use `docs/memory/phases/`.

## Decision

Store phase plans and phase results under `docs/memory/phases/`.

Existing phase-level plans are migrated into their phase folder. For example, the parent Phase 0 research plan lives at `docs/memory/phases/phase-0/phase-0-research-plan.md`, while narrower implementation plans live beside it.

Use one subfolder per phase:

```text
docs/memory/phases/
  context.md
  phase-0/
    context.md
    phase-0-research-plan.md
    phase-0-1-audio-passthrough-plan.md
    results.md
```

## Consequences

Positive:

- `docs/memory/` stays navigable.
- Phase-specific history and results have a clear home.
- Future phases can grow without polluting top-level memory.

Negative:

- Links must include deeper relative paths.
- Agents must read both top-level memory context and phase-local context.

## Links

- [../memory/phases/context.md](../memory/phases/context.md)
- [../memory/phases/phase-0/context.md](../memory/phases/phase-0/context.md)
