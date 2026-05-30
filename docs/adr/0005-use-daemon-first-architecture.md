# 0005. Use daemon-first architecture

Status: accepted
Date: 2026-05-30

## Context and Problem

The project needs to support CLI, local web UI, future Tauri UI, and possibly SDK/headless usage. If UI owns the runtime, the architecture will be harder to test and reuse.

## Decision Drivers

- The runtime should be UI-independent.
- Metrics and control APIs should work in headless mode.
- A daemon can be used by web UI, CLI, tests, and future Tauri sidecar packaging.
- Audio and model workers should survive UI reconnects where possible.

## Considered Options

- Tauri app owns runtime from day one.
- Web UI owns runtime through a local server.
- Daemon-first backend with UI as a client.
- Library-only runtime first.

## Decision

Use daemon-first architecture for the first product boundary.

The daemon can internally use library crates. Tauri can later wrap the same daemon or embed the library after boundaries stabilize.

## Consequences

Positive:

- Clear separation between runtime and UI.
- Easier CLI/headless testing.
- Better fit for diagnostics and benchmark automation.

Negative:

- Requires local API and process lifecycle design.
- Packaging sidecar daemons adds desktop distribution complexity later.

## Links

- [../agent/architecture.md](../agent/architecture.md)
