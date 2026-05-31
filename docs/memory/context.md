# Context

## Purpose

This folder contains detailed technical project memory.

## Current Shape

- `vision.md` defines the product and technical direction.
- `roadmap.md` defines ordered project phases, exit gates, and deferred work.
- `mvp-scope.md` narrows the first MVP.
- `project-governance.md` defines maintenance rules.
- `architecture.md` summarizes the intended technical architecture.
- `architecture-guide.md` defines recurring maintainability rules for boundaries, APIs, reviews, and agents.
- `runtime-architecture-v1.md` defines the target runtime architecture and phase-by-phase evolution path.
- `main-pc-handoff.md` preserves current progress and next steps for moving work
  to the stronger NVIDIA development machine.
- `full-design-draft.md` preserves the long-form design draft.
- `runtime-idea.md` preserves the earlier focused runtime idea.
- `phases/` contains phase-specific plans and results.

## Public Contracts

- Memory docs can be long and technical.
- These docs are allowed to contain implementation-level constraints.
- Any contradiction between memory docs and ADRs is resolved in favor of accepted ADRs.
- Phase plans and phase results live under `phases/`, not directly in `docs/memory/`.

## Decisions

- [../adr/0008-use-markdown-only-project-documentation.md](../adr/0008-use-markdown-only-project-documentation.md)
- [../adr/0010-store-phase-plans-under-memory-phases.md](../adr/0010-store-phase-plans-under-memory-phases.md)
- [../adr/0013-treat-architecture-memory-docs-as-normative-guidance.md](../adr/0013-treat-architecture-memory-docs-as-normative-guidance.md)

## History

- 2026-05-30: Existing planning documents moved under `docs/agent/`.
- 2026-05-30: Architecture and governance docs added.
- 2026-05-31: Markdown kept as the only documentation format.
- 2026-05-31: Folder renamed from `docs/agent/` to `docs/memory/`.
- 2026-05-31: Phase-specific planning moved under `docs/memory/phases/`.
- 2026-05-31: `phase-0-research-plan.md` moved into `phases/phase-0/` as parent Phase 0 scope.
- 2026-05-31: Roadmap added to connect vision, MVP scope, and phase plans.
- 2026-05-31: Maintainable architecture guide added with external source notes.
- 2026-05-31: Runtime architecture v1 added as the target architecture and evolution map.
- 2026-05-31: ADR 0013 made architecture memory docs normative guidance under ADR precedence.
- 2026-05-31: Main PC handoff added after `vc-bench` promotion.

## Open Questions

- Whether memory docs should later be indexed by a local retrieval tool.
