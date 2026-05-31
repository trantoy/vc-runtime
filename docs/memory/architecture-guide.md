# Maintainable architecture guide

Created: 2026-05-31
Status: normative guidance, accepted by [ADR 0013](../adr/0013-treat-architecture-memory-docs-as-normative-guidance.md)

## Purpose

This guide defines how `vc-runtime` should grow without turning into an
unmaintainable runtime manager, GUI wrapper, or model demo.

It is a practical project rulebook. It does not replace accepted ADRs, phase
plans, or the roadmap. It explains how to make everyday architecture choices
while working inside this repository.

## Working definition

In this project, maintainable architecture means:

- the realtime data plane stays small, measurable, and hard to misuse;
- the control plane can grow without leaking into audio callbacks;
- model, provider, DSP, daemon, benchmark, and UI code change for different
  reasons and live behind explicit contracts;
- important decisions are recorded before they become invisible assumptions;
- every new phase adds evidence, not just code.

Clean architecture is not a diagram style and not a fixed folder layout. It is
the ability to keep changing the system safely over time.

## Source principles

This guide is based on these external ideas, adapted for `vc-runtime`:

- Long-lived software is different from short-lived programming. The core
  question is whether the system can respond to future changes in requirements,
  dependencies, platforms, and scale.
- A codebase tends toward a big ball of mud unless growth pressure is balanced
  by boundaries, refactoring, tests, and decision records.
- Simple design and refactoring are not opposites. YAGNI works only when the
  code stays easy to change.
- Microservices, plugins, daemons, and UI shells do not automatically create
  good architecture. Bad boundaries become worse when distributed.
- Public APIs need extra care because users can depend on observable behavior
  that was never intended as a promise.
- Architecture documentation should be light enough to keep current and concrete
  enough to guide implementation.

## Project architecture stance

`vc-runtime` should be built as a modular local runtime first.

The product may later expose a daemon, SDK, web UI, Tauri app, model plugins,
and optional accelerators, but the first durable boundary is still the runtime
core:

```text
control clients
  -> control API / daemon
  -> session controller
  -> audio engine
  -> DSP and scheduler
  -> model pipeline
  -> provider runtime
  -> diagnostics
```

The UI is a client. It is not allowed to own realtime behavior.

The daemon is a control boundary. It is not allowed to become a god object.

The audio callback is a realtime boundary. It is not allowed to run inference,
block on locks, allocate large objects, perform file/network I/O, or call UI
code.

## Boundary rules

Every meaningful boundary must answer these questions before implementation:

- What does this unit own?
- What does it explicitly not own?
- Who calls it?
- What lower-level crates or modules may it depend on?
- What public types, commands, files, metrics, or protocol messages does it
  expose?
- What happens on error, overload, or missing dependency?
- How is the boundary tested without the full application?
- Does this decision need an ADR?
- Which `context.md` should be updated?

If those questions are hard to answer, the boundary is not ready.

Good boundary examples:

- `vc-audio` owns device enumeration, CPAL stream setup, callback-safe buffer
  movement, and audio runtime metrics.
- `vc-dsp` owns resampling, channel mapping, chunk scheduling, jitter handling,
  SOLA, and crossfade behavior.
- `vc-ort` owns ONNX Runtime provider probing, session setup, provider fallback,
  and provider-level diagnostics.
- `vc-rvc` owns the RVC model pipeline and bundle interpretation.
- `vc-daemon` owns session lifecycle, config validation, control API, and metrics
  streaming.
- `vc-bench` owns reproducible benchmark execution and report generation.

Bad boundary examples:

- `RuntimeManager` owns audio devices, model loading, provider probing, UI state,
  config files, benchmark reports, and plugin lifecycle.
- `vc-audio` knows about RVC model settings.
- `vc-rvc` opens CPAL devices.
- UI code mutates realtime buffers directly.
- provider code silently falls back to CPU without diagnostics.

## Dependency rules

Lower-level crates must not depend on higher-level crates.

Allowed direction:

```text
vc-daemon -> vc-rvc -> vc-ort
vc-daemon -> vc-audio
vc-daemon -> vc-dsp
vc-rvc -> vc-dsp
vc-audio -> vc-core
vc-dsp -> vc-core
vc-ort -> vc-core
vc-bench -> runtime crates under test
UI clients -> daemon/control API
```

Forbidden direction:

```text
vc-core -> any project-specific runtime crate
vc-audio -> vc-rvc
vc-audio -> vc-daemon
vc-dsp -> vc-daemon
vc-dsp -> UI clients
vc-ort -> vc-daemon
model plugin -> audio device ownership
UI clients -> realtime internals
```

If a lower-level crate needs a concept from a higher-level crate, the concept is
probably either:

- a lower-level abstraction that belongs in `vc-core`;
- a callback/trait boundary that must be made explicit;
- a sign that the proposed feature is crossing responsibilities.

Do not fix dependency direction problems by adding an `utils`, `common`, or
`manager` module with mixed ownership.

## Realtime rules

Realtime audio code has stricter rules than normal application code.

Audio callbacks may:

- read/write preallocated buffers;
- perform scalar sample-format conversion or copying at the callback boundary;
- update lock-free or bounded metrics counters;
- use bounded nonblocking communication;
- return silence or drop frames when required to preserve realtime behavior.

Audio callbacks must not:

- run model inference;
- wait on mutexes that can block behind control-plane work;
- allocate large buffers;
- parse config;
- resample, remap channels, run SOLA/crossfade, or perform model-family DSP;
- log formatted strings on the hot path;
- call network, filesystem, UI, or daemon code;
- resize shared structures in place.

Any exception requires an ADR and a benchmark proving why it is safe.

## API design rules

Rust public APIs should follow these project defaults:

- Use meaningful domain types instead of raw `usize`, `u32`, `bool`, or
  `Option<T>` when the value has domain meaning.
- Prefer `From`, `TryFrom`, `AsRef`, and `AsMut` over ad hoc conversion methods.
- Make errors specific enough to explain what failed and what the user can do.
- Implement `Debug` for public types.
- Keep fields private unless direct construction is part of the contract.
- Avoid exposing implementation details that would prevent later replacement.
- Use builders for config objects that have multiple optional or validated
  fields.
- Keep stable public dependencies small and intentional.
- Treat every CLI output format, model bundle format, metrics field, daemon
  protocol message, and exported report as a public contract once users can rely
  on it.

Do not stabilize an API just because it was easy to expose.

## Configuration rules

Configuration must be validated before realtime work starts.

A valid config should make these things explicit:

- selected input and output devices;
- sample rate, channels, and buffer sizes;
- model bundle and model family;
- provider preference and fallback behavior;
- latency/quality tradeoff settings;
- diagnostics/export settings.

Invalid config should fail in the control plane, not inside an audio callback.

Live config updates must cross into the data plane through a safe boundary:

- immutable snapshot swap;
- bounded channel;
- chunk-boundary update;
- session restart for structural changes.

## Diagnostics rules

Diagnostics are part of the architecture, not an afterthought.

Every new runtime stage should expose:

- input size;
- output size;
- queue depth or backlog;
- per-stage timing;
- error count;
- fallback state;
- overload/drop behavior.

For realtime voice conversion, a feature is not complete if users cannot tell
whether lag comes from:

- audio capture;
- resampling;
- chunk scheduling;
- pitch extraction;
- content extraction;
- generator inference;
- output buffering;
- provider fallback;
- external routing outside `vc-runtime`.

Do not hide failure behind "low quality model" or "GPU enabled" messages.

## Refactoring rules

Refactoring is expected. Large rewrites are not.

Prefer behavior-preserving refactors near the code already being changed:

- extract a focused type;
- split a module by responsibility;
- move a lower-level concept down into `vc-core`;
- replace a boolean with a domain enum;
- add a test around existing behavior before changing it;
- update `context.md` when a boundary becomes clearer.

Do not:

- rewrite a whole subsystem because its current shape is annoying;
- combine a large refactor with new behavior unless the refactor is required for
  safety;
- introduce a generic framework before two concrete use cases exist;
- split crates just to make the tree look architectural;
- leave compatibility-breaking changes undocumented.

When a refactor changes a public contract, write an ADR.

## Growth rules

The project should grow by evidence gates.

Before adding a new layer, crate, provider, plugin boundary, or UI surface,
answer:

- What current limitation does this remove?
- What evidence proves the limitation exists?
- What new complexity does this introduce?
- How will we test it?
- How will we remove or replace it if it is wrong?
- Does the roadmap say this belongs now?

Do not add future architecture for hypothetical scale. Add enough structure to
make the current phase safe, measurable, and easy to extend.

## Documentation rules

Use different documents for different jobs:

- `context.md`: local memory for a folder.
- ADR: one accepted or superseded architecture decision.
- Phase plan: concrete implementation plan for a phase.
- Results log: evidence from tests, runs, benchmarks, and failures.
- Roadmap: order of phases and exit gates.
- Architecture guide: recurring rules for maintainability.

Good documentation is short at the decision point and detailed at the execution
point.

When a document and an accepted ADR disagree, the ADR wins until superseded by a
new ADR.

## Review checklist

Every meaningful PR should answer these questions:

- Did the change preserve dependency direction?
- Did it keep realtime code out of the control plane and UI?
- Did it keep control-plane work out of audio callbacks?
- Did it add or update diagnostics for new runtime behavior?
- Did it introduce a public contract?
- Did it need an ADR?
- Did it update the nearest `context.md`?
- Did it introduce terms that belong in the glossary?
- Did it expand a manager or god object?
- Did it add tests or benchmarks proportional to the risk?
- Can the changed module be understood without reading the whole project?
- Can the implementation change later without breaking callers?

If the answer to any of these is unclear, the PR is not ready.

## Agent-specific rules

Agents working on this repository must optimize for future maintainers, not just
for passing the current prompt.

Before editing:

- read the nearest `context.md`;
- read relevant ADRs;
- check the current phase plan and roadmap;
- identify the public contract being touched.

While editing:

- keep changes scoped to one boundary;
- avoid new generic abstractions unless they remove real duplication or protect
  a real contract;
- add tests before behavior changes when practical;
- update docs next to the changed boundary.

Before finishing:

- run the smallest meaningful verification first;
- run broader workspace checks when the change crosses crates;
- review for god-object growth;
- review dependency direction;
- record evidence in the phase results log when the work proves something.

Agents must not use "clean architecture" as an excuse for ceremony. The test is
whether the code becomes easier to change safely.

## Anti-patterns

Avoid these patterns:

- `RuntimeManager` as the owner of everything.
- `common` modules that contain unrelated domain concepts.
- UI state mixed with runtime state.
- provider fallback that is invisible to users.
- benchmark code that depends on UI.
- model code that owns devices.
- config parsing inside realtime paths.
- broad plugin ABI before one stable model path exists.
- TensorRT or Triton integration before benchmarked ORT bottlenecks exist.
- all-platform support claims without smoke evidence.
- "temporary" CLI output that later becomes a hidden contract.
- rewrites justified only by dislike of old code.

## Architecture fitness checks

These checks should eventually become automated:

- crate dependency direction;
- forbidden imports by crate;
- file size and responsibility drift reports;
- clippy warnings as errors;
- Rust API guideline checklist for public crates;
- docs link checks;
- missing `context.md` check;
- ADR requirement check for public contracts;
- metrics schema compatibility check;
- benchmark regression check;
- realtime callback audit for allocation/blocking/logging.

Manual review comes first. Automation should follow once the rules are stable.

## Source notes

Open sources used for this guide:

- [Software Engineering at Google](https://abseil.io/resources/swe-book) -
  long-lived software, sustainable engineering, Hyrum's Law, build/test/change
  practices.
- [Google Engineering Practices: Code Review](https://google.github.io/eng-practices/review/reviewer/) -
  code review as a code-health mechanism.
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Rust API
  naming, interoperability, documentation, predictability, type safety,
  dependability, debuggability, and future-proofing.
- [C4 Model](https://c4model.com/) - lightweight architecture visualization
  using context, container, component, and code levels.
- [Architectural Decision Records](https://adr.github.io/) - decision logs,
  rationale, tradeoffs, and architecture knowledge management.
- [Big Ball of Mud](https://www.laputan.org/mud/) - forces that produce
  unstructured systems and why "keep it working" pressure must be balanced by
  reconstruction.
- [Martin Fowler: Is Design Dead?](https://martinfowler.com/articles/designDead.html) -
  evolutionary design, simple design, YAGNI, and refactoring.
- [Martin Fowler: Monolith First](https://martinfowler.com/bliki/MonolithFirst.html) -
  microservice premium and why distribution is not the first solution for most
  new systems.
- [Stefan Tilkov: Don't start with a monolith](https://martinfowler.com/articles/dont-start-monolith.html) -
  counterpoint on early boundaries when microservices are a known goal.
- [Joel Spolsky: Things You Should Never Do, Part I](https://www.joelonsoftware.com/2000/04/06/things-you-should-never-do-part-i/) -
  warning against rewrite-from-scratch reflexes.
- [The Monolith Strikes Back: Why Istio Migrated from Microservices to a Monolithic Architecture](https://research.google/pubs/the-monolith-strikes-back-why-istio-migrated-from-microservices-to-a-monolithic-architecture/) -
  industrial case where early microservice costs outweighed benefits.
- [Seven Hard-Earned Lessons Learned Migrating a Monolith to Microservices](https://www.infoq.com/articles/lessons-learned-monolith-microservices/) -
  migration lessons around tests, cost, patterns, and whether migration is
  justified.
- [GitHub Architecture & Optimization](https://github.blog/engineering/architecture-optimization/) -
  example of a large monolith maintained through ownership, tooling,
  optimization, and continuous improvement.

Books to incorporate later if a legal local copy is provided:

- John Ousterhout, *A Philosophy of Software Design*.
- Michael Feathers, *Working Effectively with Legacy Code*.
- Martin Fowler, *Refactoring*.
- Robert C. Martin, *Clean Architecture*.
- Sam Newman, *Monolith to Microservices*.
- Michael Nygard, *Release It!*.
