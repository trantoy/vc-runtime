# Project governance

Created: 2026-05-30

## Purpose

These rules exist to keep `vc-runtime` from collapsing into a large unmaintainable object after a few fast PRs.

## Required Project Memory

Every folder must contain a `context.md` file with:

- purpose;
- current shape;
- public contracts;
- related decisions;
- short history;
- open questions.

Agents must read the nearest `context.md` before editing files in that folder.

## Glossary Rule

Frequent or special terms must be added to [../glossary.md](../glossary.md).

Add glossary entries when introducing:

- model or provider terms;
- audio/DSP terms;
- project-specific names;
- abbreviations;
- words that could mean different things in different domains.

## ADR Rule

Write or update an ADR when a change affects:

- crate boundaries;
- daemon/control protocol;
- model bundle format;
- provider strategy;
- public metrics schema;
- long-term dependency choice;
- folder/documentation rules;
- any decision likely to be re-litigated later.

## God Object Prevention

Do not create a manager that owns unrelated responsibilities.

Bad shape:

```text
RuntimeManager
  owns audio devices
  owns model loading
  owns provider probing
  owns UI state
  owns config files
  owns benchmarks
```

Better shape:

```text
AudioEngine
ModelRegistry
ProviderManager
SessionController
DiagnosticsSink
```

Each type should have one reason to change.

## Size and Boundary Triggers

When a file grows beyond roughly 400-600 lines, the next PR touching it must ask:

- Is this file holding multiple responsibilities?
- Is there a hidden public contract?
- Can a small type or module be extracted without changing behavior?
- Should this boundary be recorded in `context.md` or an ADR?

This is a trigger for design attention, not an automatic rewrite rule.

## Dependency Guardrails

- Lower-level crates cannot depend on higher-level crates.
- `vc-core` cannot depend on CPAL, ONNX Runtime, UI, or model-specific code.
- `vc-audio` cannot depend on `vc-rvc`.
- `vc-dsp` cannot depend on daemon or UI code.
- Model plugins cannot control audio devices directly.
- UI/client code cannot bypass the control API to mutate realtime state.

## PR Checklist

Every meaningful PR should answer:

- Which `context.md` files were read or updated?
- Did this introduce a new glossary term?
- Did this require an ADR?
- Did this expand a manager/god object?
- Did this change a public contract?
- Did this add metrics for new runtime behavior?
- Are tests or benchmarks enough for the risk?

## Refactoring Rule

Prefer small behavior-preserving refactors while working near a boundary.

Do not bundle large refactors with behavior changes unless the refactor is required to make the behavior change safe.

If a refactor changes a public contract, write an ADR or update the folder `context.md`.

## Architecture Fitness Checks

The project should eventually automate:

- dependency direction checks;
- clippy warnings as errors for core crates;
- docs link checks;
- file-size reports;
- public API drift reports;
- metrics schema compatibility checks;
- benchmark regression reports.

Manual discipline comes first; automation follows once patterns stabilize.
