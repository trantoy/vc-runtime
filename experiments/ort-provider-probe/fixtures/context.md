# Context

## Purpose

This folder contains deterministic fixture data for the ORT provider probe
experiment.

## Current Shape

- `host-provider-matrix.json` simulates host provider availability and fallback.
- `sample-report.json` documents the candidate output schema.

## Public Contracts

- Fixtures are synthetic evidence only.
- Fixture results must not be described as real provider availability.

## Decisions

- [../../../docs/memory/runtime-architecture-v1.md](../../../docs/memory/runtime-architecture-v1.md)

## History

- 2026-05-31: Provider probe fixtures added.

## Open Questions

- Which real host/provider matrices should be captured after live probing exists.
