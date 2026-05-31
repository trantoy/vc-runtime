# Context: offline-audio-bench

## Purpose

Prototype an offline prerecorded-audio benchmark before promoting the concept to
the future `vc-bench` crate.

## Scope

- Location: `experiments/offline-audio-bench/`
- This is experiment code, not production runtime code.
- The experiment has a local `[workspace]` and is not part of the root workspace.
- It must not depend on live audio devices.
- It should emit reproducible JSON reports that can later inform `vc-bench`.

## Current decisions

- Use WAV input for the actual benchmark runner.
- Do not commit large raw audio fixtures by default.
- Use a source manifest and preparation script for public-domain speech fixtures.
