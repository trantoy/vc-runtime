# Phase 0.1 Audio Passthrough Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:test-driven-development` for production code. After each completed task, run a strict subagent review for architecture cleanliness and plan compliance before moving to the next task.

**Goal:** Build the first measurable audio foundation: list devices, then pass audio from input to output through a bounded ring buffer with basic metrics.

**Architecture:** Start with a small Rust workspace. `vc-core` owns shared metrics and counters. `vc-audio` owns CPAL device enumeration and passthrough runtime. `vc-cli` exposes `list-devices` and `passthrough` commands. No ML, ONNX, daemon, UI, provider manager, or RVC code is allowed in this phase.

**Tech Stack:** Rust 1.96, Cargo workspace, `clap`, `cpal`, `rtrb`, `anyhow`, `thiserror`.

---

## Scope

In scope:

- Rust workspace skeleton.
- `vc-core` crate with metrics counters and snapshots.
- `vc-audio` crate with CPAL device listing and passthrough runtime.
- `vc-cli` crate with `list-devices` and `passthrough`.
- Context files for all new folders.
- Unit tests for non-hardware logic.
- Compile, format, clippy, and test verification.

Out of scope:

- RVC.
- ONNX Runtime.
- Daemon/control API.
- Tauri or web UI.
- Provider manager.
- Virtual audio driver.
- Realtime priority tuning.
- Long 30-minute run automation.

## Phase 0.1 Exit Criteria

Phase 0.1 is complete only when:

- the Rust workspace exists and follows the crate boundaries in this plan;
- `vc list-devices` runs without panic;
- `vc passthrough --seconds 1` either runs and exits or returns a clear audio-device error without panic;
- metrics counters exist for callbacks, pushed/popped frames, underruns, and overruns;
- `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` pass;
- every folder has `context.md`;
- `docs/memory/phases/phase-0/results.md` records what was and was not proven.

This does not satisfy the broader Phase 0 evidence target of a 30-minute stable passthrough run. The 30-minute run remains a separate hardware validation result in `results.md`.

## Task 1: Phase Plan Structure

**Files:**

- Create: `docs/memory/phases/context.md`
- Create: `docs/memory/phases/phase-0/context.md`
- Create: `docs/memory/phases/phase-0/phase-0-1-audio-passthrough-plan.md`
- Create: `docs/adr/0010-store-phase-plans-under-memory-phases.md`
- Modify: `README.md`
- Modify: `context.md`
- Modify: `docs/context.md`
- Modify: `docs/memory/context.md`
- Modify: `docs/adr/context.md`

**Acceptance:**

- Phase plans live under `docs/memory/phases/`.
- Every new folder has `context.md`.
- ADR 0010 records the folder decision.
- `git diff --check` passes.

**Review prompt:**

Review only documentation architecture. Be strict about drift, duplicate docs, broken context rules, weak ADR wording, and unclear phase ownership.

## Task 2: Workspace Skeleton

**Files:**

- Create: `Cargo.toml`
- Modify: `.gitignore`
- Modify: `crates/context.md`
- Create: `crates/vc-core/Cargo.toml`
- Create: `crates/vc-core/context.md`
- Create: `crates/vc-core/src/context.md`
- Create: `crates/vc-core/src/lib.rs`
- Create: `crates/vc-audio/Cargo.toml`
- Create: `crates/vc-audio/context.md`
- Create: `crates/vc-audio/src/context.md`
- Create: `crates/vc-audio/src/lib.rs`
- Create: `crates/vc-cli/Cargo.toml`
- Create: `crates/vc-cli/context.md`
- Create: `crates/vc-cli/src/context.md`
- Create: `crates/vc-cli/src/lib.rs`
- Create: `crates/vc-cli/src/main.rs`

**Acceptance:**

- `cargo test --workspace` passes.
- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- Every new folder has `context.md`.

**Review prompt:**

Review workspace boundaries. Be strict about crate dependency direction, overbroad crates, missing context files, premature abstractions, and hidden god-object seeds.

## Task 3: Core Metrics

**Files:**

- Modify: `crates/vc-core/src/lib.rs`
- Create: `crates/vc-core/src/metrics.rs`

**Behavior:**

- `AudioCounters` stores atomic counters for input callbacks, output callbacks, pushed frames, popped frames, underruns, and overruns.
- `AudioMetricsSnapshot` is a copyable report type.
- Unit tests verify increments and snapshots.

**Acceptance:**

- `cargo test -p vc-core` passes.
- No dependency from `vc-core` to CPAL, CLI, UI, or model code.

**Review prompt:**

Review metric API minimality and thread-safety. Be strict about unnecessary dependencies, vague names, counters that hide units, and future god-object risk.

## Task 4: Device Listing

**Files:**

- Modify: `crates/vc-audio/src/lib.rs`
- Modify: `crates/vc-audio/Cargo.toml`
- Create: `crates/vc-audio/src/devices.rs`
- Modify: `crates/vc-cli/Cargo.toml`
- Modify: `crates/vc-cli/src/lib.rs`
- Modify: `crates/vc-cli/src/main.rs`

**Behavior:**

- `vc list-devices` prints input and output devices.
- Device IDs are process-local numeric indices.
- Device listing is best-effort and reports host/device errors clearly.

**Acceptance:**

- `cargo test --workspace` passes.
- `cargo run -p vc-cli -- list-devices` runs without panic. It may report no devices if the environment has no audio devices.
- Output clearly separates input and output devices.

**Review prompt:**

Review device-listing boundaries and CLI behavior. Be strict about leaking CPAL types into CLI, unstable IDs being misrepresented as stable, unclear errors, and missing context updates.

## Task 5: Passthrough Runtime

**Files:**

- Modify: `crates/vc-audio/src/lib.rs`
- Modify: `crates/vc-audio/Cargo.toml`
- Create: `crates/vc-audio/src/passthrough.rs`
- Modify: `crates/vc-cli/Cargo.toml`
- Modify: `crates/vc-cli/src/lib.rs`
- Modify: `crates/vc-cli/src/main.rs`

**Behavior:**

- `vc passthrough --seconds 10` starts default input-to-output passthrough.
- Optional `--input-index`, `--output-index`, and `--capacity-frames` select devices and ring capacity.
- Runtime uses an `rtrb` single-producer/single-consumer ring buffer.
- Input callback pushes samples and counts overruns when the ring is full.
- Output callback pops samples and writes silence/counts underruns when the ring is empty.
- CLI prints metrics once per second.

**Acceptance:**

- `cargo test --workspace` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.
- `cargo run -p vc-cli -- passthrough --seconds 1` either runs and exits or returns a clear audio-device error without panic.

**Review prompt:**

Review realtime audio architecture. Be harsh about blocking work in callbacks, allocations in callbacks, mutexes in callbacks, unclear sample-rate assumptions, god-object coupling, and missing metrics.

## Task 6: Phase 0 Result Notes

**Files:**

- Create: `docs/memory/phases/phase-0/results.md`
- Modify: `docs/memory/phases/phase-0/context.md`
- Modify: `context.md`

**Behavior:**

- Record commands run, results, failures, and current limits.
- Do not claim hardware passthrough success unless an actual passthrough command ran successfully.

**Acceptance:**

- Results distinguish compile/test verification from hardware/audio verification.
- `git diff --check` passes.

**Review prompt:**

Review evidence quality. Be strict about unsupported success claims, missing command output summaries, and vague next steps.

## Final Verification

Run:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git diff --check HEAD
git ls-files -z | xargs -0 -n1 dirname | sort -u | while read -r dir; do test -f "$dir/context.md" || echo "missing context: $dir"; done
git status --short --branch
```

Expected:

- format clean;
- clippy clean;
- tests pass;
- no whitespace errors;
- no missing `context.md`;
- branch clean after commit.
