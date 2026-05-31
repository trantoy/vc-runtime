# ORT Provider Probe Prototype

## Architecture question this prototype answers

The ADR direction (`0004`) requires provider visibility but we should not claim
stage-level provider assignment until evidence exists.

This prototype intentionally answers only:

1. **provider availability** — what providers are modeled as available by a host
   probe source (dry-run fixture or real runtime probe in future),
2. **usable provider** — which provider can be used for a request given requested
   provider and fallback chain, and
3. **observed assignment granularity** — the confidence level of where assignment
   was observed (`dry_run`, `session`, etc.), instead of claiming fine-grained
   per-stage certainty.

## Output schema

Every run emits a JSON object with this contract:

```json
{
  "requested_provider": "CUDAExecutionProvider",
  "provider_probe_status": "requested_provider_unavailable_with_fallback",
  "observed_provider_assignment": [
    {
      "scope": "session",
      "requested_provider": "CUDAExecutionProvider",
      "observed_provider": "CPUExecutionProvider",
      "observed_with": "fixture",
      "evidence": "requested provider missing in fixture; selected first matching provider from fallback_chain"
    }
  ],
  "provider_assignment_granularity": "session",
  "errors": [
    "requested provider \"CUDAExecutionProvider\" was not available in fixture"
  ]
}
```

`provider_assignment_granularity` intentionally reflects *observability depth*, not
an optimistic promise of per-layer truth.

## Commands

Run from this folder:

```bash
cd /home/cordis/Gits/vc-runtime/experiments/ort-provider-probe
cargo run -- --pretty
```

Dry-run output example (no fixture):

```bash
cargo run -- --requested-provider CUDAExecutionProvider --pretty
```

Fixture-driven simulation:

```bash
cargo run -- \
  --requested-provider CUDAExecutionProvider \
  --fixture fixtures/host-provider-matrix.json \
  --pretty
```

Test:

```bash
cargo test
```

## Future next-step command shape (once real ORT crate is wired)

The current prototype is intentionally read-only in terms of provider evidence:

- `--fixture` drives deterministic simulation.

If/when a real ORT probe path is added, keep this CLI contract and add a
`--live-probe` mode with a call like:

```bash
# expected shape after integration
cargo run -- --requested-provider CUDAExecutionProvider --live-probe /path/model.onnx --pretty
```

Expected output fields remain the same; only `provider_probe_status`,
`observed_provider_assignment`, and `provider_assignment_granularity` should become
more concrete (`session`/`op`/`node`) when runtime evidence exists.

## What is safe to learn without CUDA/ORT

- JSON schema shape and deterministic fallback behavior with fixtures.
- Difference between dry-run and session-level granularity.
- Whether requested provider aliasing is normalized and how many probes errors are
  represented.

## What requires a machine + ORT/Provider setup

- Discovering real installed execution providers from ONNX Runtime API.
- Detecting provider availability after real session creation.
- Verifying op/node-level assignment and fallback behavior from actual runtime
  execution traces.

## Why this layout is safe for future integration

The result contract is explicitly conservative and includes `provider_assignment_granularity`,
so production integration can migrate from:

`NotProbedDryRun` -> `fixture` simulation -> real ORT-backed probe

without changing consumers that only depend on the schema shape.
