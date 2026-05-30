# 0004. Use ONNX Runtime as mainline inference

Status: accepted
Date: 2026-05-30

## Context and Problem

The project needs a production inference path that can target multiple hardware backends without making Python/PyTorch the realtime deployment foundation.

## Decision Drivers

- The runtime should work across Windows, Linux, and macOS.
- Provider selection must be observable.
- CUDA should be first-class, with CPU fallback everywhere.
- DirectML, CoreML, and OpenVINO should be possible without rewriting the whole runtime.
- PyTorch should remain available for conversion and reference validation.

## Considered Options

- PyTorch as production runtime.
- ONNX Runtime as production runtime.
- TensorRT-only runtime.
- Custom kernels/Triton as runtime foundation.

## Decision

Use ONNX Runtime as the mainline production inference path.

PyTorch is for conversion, validation, and development fallback. TensorRT and Triton are optional post-MVP accelerators after profiling.

## Consequences

Positive:

- One inference API can target multiple execution providers.
- Provider strategy can be exposed in diagnostics.
- CPU fallback is practical.

Negative:

- RVC export and shape stability must be proven.
- Provider packaging and operator support vary by platform.
- Some model families may not map cleanly to ONNX.

## Links

- [../memory/architecture.md](../memory/architecture.md)
- [../memory/phases/phase-0/phase-0-research-plan.md](../memory/phases/phase-0/phase-0-research-plan.md)
