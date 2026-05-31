Ниже — design doc для проекта под рабочим названием **`vc-runtime`**. Это название лучше, чем `rvc-realtime-runtime`, потому что MVP действительно RVC-first, но граница проекта шире: runtime для разных voice conversion pipelines. Репозиторий можно назвать `vc-runtime`; первый production plugin — `vc-plugin-rvc`.

Актуальные внешние факты я проверил по официальным источникам на 30 мая 2026: ONNX Runtime использует Execution Providers как абстракцию для CPU/GPU/NPU/специализированных ускорителей; среди EP есть CUDA, TensorRT, DirectML, OpenVINO, CoreML, XNNPACK и другие. CPAL сейчас покрывает Windows/WASAPI, macOS/CoreAudio, Linux/ALSA/JACK/PipeWire/PulseAudio и optional ASIO/JACK, но realtime-priority и buffer-size остаются задачей приложения. DirectML в ONNX Runtime сейчас описан как sustained engineering, а для новых Windows ML deployment-сценариев Microsoft продвигает WinML. CoreML EP требует iOS 13+ или macOS 10.15+ и оптимальнее на Apple Neural Engine. OpenVINO EP поддерживает CPU/GPU/NPU и режимы AUTO/HETERO/MULTI. Triton language официально ориентирован на Linux, NVIDIA GPU Compute Capability 8.0+, AMD ROCm 6.2+, CPU находится в разработке. Tauri 2 позиционируется как Rust + web frontend для Windows/macOS/Linux/Android/iOS, а Tauri bundler умеет `.app`, `.dmg`, `.deb`, `.rpm`, `.AppImage`, Windows `.exe`/NSIS и `.msi`/WiX. ([ONNX Runtime][1])

---

# Design Doc: `vc-runtime`

## 0. Assumptions

**Assumption A1.** Основной сценарий — локальный realtime voice conversion для микрофона: Discord, OBS, streaming, virtual cable, monitoring, VTuber tooling.

**Assumption A2.** MVP не обязан создавать собственный virtual audio driver. На Windows пользователь может использовать VB-CABLE/Voicemeeter/SteelSeries Sonar/OBS monitor path; на macOS — BlackHole/Loopback; на Linux — PipeWire/JACK routing. Собственный virtual device — отдельный high-risk проект.

**Assumption A3.** Целевой I/O sample rate — 48 kHz, потому что это типичный realtime/media rate. Внутри pipeline допустимы model-specific rates: например 16 kHz для content/pitch stages и 40/48 kHz для output, но все resampling boundaries должны быть явными.

**Assumption A4.** Первый model family — RVC. Поддержка Beatrice, LLVC, so-vits-svc, DDSP-SVC, Diffusion-SVC закладывается в plugin boundary, но не реализуется в MVP.

**Assumption A5.** ONNX Runtime — production mainline. PyTorch — compatibility/dev/conversion/fallback path. TensorRT/Triton — не фундамент runtime, а optional acceleration после профилирования.

**Assumption A6.** Главный продуктовый дифференциатор — не “ещё один GUI”, а предсказуемый realtime runtime с измеримой latency, стабильным chunk scheduling, понятным provider selection и диагностикой “где именно тормозит”.

---

# 1. Executive summary

`vc-runtime` — cross-platform realtime voice conversion runtime, который принимает аудио с микрофона, стабильно режет его на chunks, прогоняет через model pipeline, собирает выходной поток через overlap/crossfade/SOLA и отдаёт его в output device или virtual cable.

Это не GUI поверх Python pipeline. Цель проекта — построить **deterministic realtime audio + ML inference runtime**, где UI является клиентом control plane, а не центром архитектуры.

Ключевая идея:

```text
Low latency is not one optimization.
Low latency is a property of the whole pipeline:
audio callbacks + queues + chunk scheduler + resampling + model runtime +
provider selection + memory reuse + output smoothing + diagnostics.
```

Существующие realtime voice changer проекты часто дают пользователю параметры вроде chunk size, extra buffer, f0 method, GPU mode, но не объясняют, что реально происходит: растёт input queue, inference ушёл на CPU, DirectML не поддержал часть graph, TensorRT пересобирает engine, audio device работает в 44.1 kHz, output buffer пустеет, Discord добавляет свой jitter, или virtual cable накопил задержку.

`vc-runtime` должен сделать это видимым.

Первый этап — backend/runtime:

```text
Rust core runtime
  + CPAL-based audio passthrough
  + deterministic chunk scheduler
  + ring/jitter buffers
  + resampling
  + SOLA/crossfade
  + ONNX Runtime provider manager
  + RVC ONNX pipeline
  + live profiler
  + benchmark harness
  + local web UI for development
```

UI нужен, но не должен диктовать pipeline. Правильная последовательность: сначала измеряемый audio/runtime core, потом RVC, потом provider diagnostics, потом Tauri shell.

---

# 2. Problem statement

Проекты вроде w-okada/voice-changer уже доказали спрос: это realtime VC client с несколькими voice conversion AI, включая RVC, so-vits-svc, DDSP-SVC и Beatrice, с server-client архитектурой и кроссплатформенной заявкой на Windows/Mac/Linux. ([GitHub][2])

Но типовые проблемы realtime voice changer программ повторяются:

1. **Latency растёт не только из-за модели.**
   Даже если model inference занимает 30 ms, end-to-end latency может стать 300–2000 ms из-за input queue growth, лишнего resampling, output buffering, virtual cable, mismatch sample rate, network/websocket overhead или неверного scheduling.

2. **Choppy audio и crackling часто не являются ML quality problem.**
   Они могут быть результатом output underrun, discontinuity между chunks, неправильного overlap, слишком маленького audio callback buffer, GC/allocation в realtime path, CPU starvation или несовпадения clock rate input/output devices.

3. **“GPU enabled” не значит “pipeline runs on GPU”.**
   В ONNX Runtime Execution Provider может взять только часть graph; unsupported nodes могут уйти на CPU. DirectML имеет свои ограничения по opset/options, TensorRT может fallback’нуться на CUDA/CPU для subgraphs, CoreML может плохо работать с dynamic shapes. ONNX Runtime распределяет nodes/subgraphs между EP через механизм capabilities, поэтому runtime обязан показывать фактическое состояние, а не только requested provider. ([ONNX Runtime][1])

4. **Пользователь не понимает, какой параметр менять.**
   Chunk size, extra buffer, crossfade, f0 method, provider, CPU threads, precision, sample rate — всё взаимосвязано. Без live profiler пользователь “крутит ручки” вслепую.

5. **Cross-platform audio — отдельная сложность.**
   CPAL закрывает базовую абстракцию устройств, но Windows/WASAPI, optional ASIO, macOS/CoreAudio, Linux/PipeWire/JACK/ALSA/PulseAudio имеют разные buffering, permissions, hotplug, realtime-priority и routing semantics. ([GitHub][3])

Главная проблема, которую решает `vc-runtime`: **сделать realtime voice conversion управляемой инженерной системой**, где latency budget, provider choice, queue depth, xruns и per-stage timings видны пользователю и разработчику.

---

# 3. Goals

## 3.1 Product goals

`vc-runtime` должен быть:

* cross-platform backend для Windows, Linux, macOS;
* low-latency realtime audio engine;
* runtime с предсказуемым processing time per chunk;
* framework для разных hardware backends;
* diagnostics-first системой;
* удобным backend для UI, CLI, SDK, daemon и desktop app;
* расширяемой платформой под разные model families.

## 3.2 Technical goals

Целевое состояние:

```text
Input callback never blocks on ML.
Output callback never blocks on ML.
Realtime path does not allocate.
Control plane cannot stall audio plane.
Every chunk has timing metadata.
Every provider decision is observable.
Every queue depth is measurable.
Every fallback is visible.
```

## 3.3 MVP goals

MVP должен доказать:

1. Audio passthrough стабилен на Windows/Linux/macOS.
2. RVC ONNX pipeline работает в realtime хотя бы на Windows/Linux NVIDIA и CPU fallback.
3. Live profiler показывает, где именно тратится время.
4. Provider manager умеет выбрать CUDA/DirectML/CoreML/OpenVINO/CPU по доступности и benchmark’у.
5. Benchmark harness сравнивает настройки и генерирует JSON/CSV/histogram reports.

---

# 4. Non-goals

В первой версии проект **не** делает:

* training новых моделей;
* переписывание PyTorch/cuDNN/cuBLAS/TensorRT;
* поддержку всех VC model families;
* красивый production UI до появления стабильного runtime;
* собственный virtual audio driver;
* магическое ускорение без профилирования;
* network/cloud voice conversion по умолчанию;
* server-grade multi-user inference;
* DRM для моделей;
* hard realtime guarantees уровня embedded/audio DSP hardware.

---

# 5. Target users and use cases

## 5.1 Users

1. **Streamers**
   Нужны стабильность, OBS routing, низкая задержка, быстрый recovery после device changes.

2. **Discord/Zoom/game voice chat users**
   Нужна интеграция через virtual cable, минимум crackling/dropouts, простые presets.

3. **VTubers**
   Нужны hotkeys, voice presets, low-latency monitoring, scene-specific settings.

4. **Developers of voice tools**
   Нужен SDK, headless runtime, programmatic metrics, plugin API.

5. **ML researchers**
   Нужен realtime deployment harness для моделей, offline/online benchmarks, reproducible latency measurements.

6. **Users on weak machines**
   Нужен CPU fallback, auto-tuning, warning “эта модель не тянется в realtime”.

7. **Users on gaming PCs**
   Нужен GPU provider selection, минимальное влияние на игру, priority management.

8. **macOS users**
   Нужен CoreAudio, CoreML/CPU fallback, signed/notarized package.

9. **Linux users with PipeWire/JACK**
   Нужен explicit routing, low-latency audio graph, realtime scheduling guidance.

10. **Windows users with virtual cable/WASAPI/ASIO**
    Нужен routing assistant, WASAPI shared/exclusive mode, optional ASIO.

---

# 6. Latency model

## 6.1 End-to-end pipeline

```text
┌──────────────────────┐
│ Microphone hardware  │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Input device buffer  │
└──────────┬───────────┘
           │ callback frames
           ▼
┌──────────────────────┐
│ Input callback       │  realtime-safe
│ - copy into ring     │
│ - timestamp          │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Input ring buffer    │
│ / jitter buffer      │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Input resampler      │
│ 48k -> model rate    │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Chunk scheduler      │
│ chunk/context/pad    │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Pitch extraction     │
│ RMVPE/Harvest/etc.   │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Content extraction   │
│ HuBERT/ContentVec    │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Index retrieval      │ optional
│ top-k features       │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Model inference      │
│ generator/vocoder    │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Output resampler     │
│ model rate -> device │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ SOLA / crossfade     │
│ chunk stitching      │
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Output ring buffer   │
└──────────┬───────────┘
           │ callback frames
           ▼
┌──────────────────────┐
│ Output callback      │ realtime-safe
└──────────┬───────────┘
           │
           ▼
┌──────────────────────┐
│ Virtual cable /      │
│ speakers / OBS       │
└──────────────────────┘
```

## 6.2 Latency terms

**Device latency**
Hardware + OS audio buffer latency.

**Algorithmic latency**
Required context, lookahead, chunk size, overlap, model receptive field.

**Processing latency**
Actual compute time per chunk.

**Queue latency**
Frames waiting in input/output queues.

**Accumulated delay**
Queue growth caused by average processing time exceeding emitted audio duration.

**External app latency**
Discord/OBS/virtual cable/app-specific buffering. Runtime should measure its own boundary and estimate external latency where possible, but cannot fully control it.

## 6.3 Latency budget

Для realtime voice changer реалистичнее целиться не в pro-audio 5–10 ms, а в стабильный ML realtime.

### Ideal

```text
End-to-end runtime latency, excluding Discord/OBS:
40–70 ms
```

Условия:

* small chunks: 20–40 ms;
* p95 processing time significantly below chunk duration;
* stable GPU provider;
* no queue growth;
* output buffer 1–2 chunks;
* crossfade 5–10 ms.

### Acceptable

```text
70–120 ms
```

Условия:

* chunks 40–80 ms;
* stable p95;
* occasional p99 spikes hidden by output buffer;
* no accumulated lag.

### Bad

```text
>150 ms noticeable
>250 ms uncomfortable
>500 ms broken for interactive voice
```

### Unusable

```text
Queue growth over time:
every 10 seconds of speech adds more delay.
```

Типичный симптом: сначала всё нормально, потом голос отстаёт на секунды. Это почти всегда означает, что long-term realtime factor ≥ 1 или output app/virtual cable накапливает buffer.

## 6.4 Per-stage rough budget

Assumption: 48 kHz I/O, RVC-like model, 40 ms emitted chunk.

| Stage               |   Ideal | Acceptable | Bad signal                     |
| ------------------- | ------: | ---------: | ------------------------------ |
| Input callback copy | <0.2 ms |    <0.5 ms | allocation/lock in callback    |
| Input resample      |   <1 ms |      <2 ms | resampler too heavy            |
| Chunk scheduling    | <0.2 ms |    <0.5 ms | lock/contention                |
| Pitch extraction    |  2–8 ms |    8–20 ms | CPU fallback                   |
| Content extraction  | 3–10 ms |   10–25 ms | wrong provider                 |
| Index retrieval     |   <2 ms |      <6 ms | index too large/CPU bottleneck |
| Generator/inference | 5–20 ms |   20–40 ms | RTF near/over 1                |
| Output resample     |   <1 ms |      <2 ms | unnecessary conversion         |
| SOLA/crossfade      |   <1 ms |      <3 ms | search window too large        |
| Output callback     | <0.2 ms |    <0.5 ms | underrun risk                  |

For a 40 ms chunk, p95 total processing should ideally be **≤25–30 ms**, leaving headroom for jitter. If p95 is 39 ms and p99 is 70 ms, audio will eventually crackle unless output buffer is large, which increases latency.

## 6.5 Metrics

For every chunk:

```text
chunk_id
capture_timestamp
scheduler_pop_timestamp
pitch_start/end
feature_start/end
index_start/end
inference_start/end
post_start/end
output_push_timestamp
first_output_callback_timestamp
input_queue_frames
output_queue_frames
observed_provider_assignment
provider_assignment_granularity
dropped_frames
underrun_count
overrun_count
```

Derived:

```text
processing_time_ms = chunk_done - chunk_start
emitted_audio_ms   = output_frames / output_sample_rate * 1000
rtf                = processing_time_ms / emitted_audio_ms
queue_delay_ms     = queue_frames / sample_rate * 1000
accumulated_delay  = input_queue_delay + output_queue_delay + algorithmic_delay
```

Percentiles:

```text
p50: median user experience
p95: normal worst-case
p99: glitch predictor
max: debugging only, not a product KPI
```

Queue growth model:

```text
if processing_time_ms > emitted_audio_ms repeatedly:
    input_queue grows
    accumulated_delay grows
    user hears delayed voice

if output_queue_frames == 0:
    output callback emits silence/last sample
    user hears crackle/dropout

if input_queue overflows:
    frames are dropped
    model receives discontinuous audio
    user hears choppy/robotic output
```

---

# 7. Architecture overview

## 7.1 Control plane vs data plane

The runtime must separate control and audio:

```text
                 CONTROL PLANE
┌───────────────────────────────────────────────┐
│ UI / CLI / SDK                                │
│ - device selection                            │
│ - model loading                               │
│ - presets                                     │
│ - diagnostics                                 │
└───────────────────────┬───────────────────────┘
                        │ JSON/IPC/WebSocket
                        ▼
┌───────────────────────────────────────────────┐
│ Control API / Runtime daemon                  │
│ - session lifecycle                           │
│ - config validation                           │
│ - provider selection                          │
│ - metrics stream                              │
└───────────────────────┬───────────────────────┘
                        │ lock-free commands at boundaries
                        ▼
                 DATA PLANE
┌───────────────────────────────────────────────┐
│ Audio Engine                                  │
│ - input/output callbacks                      │
│ - realtime-safe ring buffers                  │
└───────────────────────┬───────────────────────┘
                        ▼
┌───────────────────────────────────────────────┐
│ DSP Pipeline                                  │
│ - resampling                                  │
│ - chunking                                    │
│ - SOLA/crossfade                              │
└───────────────────────┬───────────────────────┘
                        ▼
┌───────────────────────────────────────────────┐
│ Model Runtime                                 │
│ - RVC pipeline                                │
│ - ONNX Runtime sessions                       │
│ - provider-specific execution                 │
└───────────────────────┬───────────────────────┘
                        ▼
┌───────────────────────────────────────────────┐
│ Diagnostics / Profiler                        │
│ - per-stage timings                           │
│ - queue depths                                │
│ - underruns/overruns                          │
│ - provider status                             │
└───────────────────────────────────────────────┘
```

## 7.2 Process architecture

Recommended MVP:

```text
┌────────────────────────────────────┐
│ Browser / Web UI                   │
│ localhost frontend                 │
└──────────────┬─────────────────────┘
               │ HTTP + WebSocket
               ▼
┌────────────────────────────────────┐
│ vc-daemon                          │
│ Rust process                       │
│                                    │
│ ┌────────────┐ ┌─────────────────┐ │
│ │ Control API│ │ Metrics Stream  │ │
│ └─────┬──────┘ └────────┬────────┘ │
│       │                 │          │
│ ┌─────▼─────────────────▼────────┐ │
│ │ Runtime Session Manager         │ │
│ └─────┬──────────────────────────┘ │
│       │                            │
│ ┌─────▼──────┐ ┌────────────────┐ │
│ │ Audio      │ │ Model Runtime   │ │
│ │ Engine     │ │ ORT/PyTorch     │ │
│ └─────┬──────┘ └────────┬───────┘ │
│       │                 │         │
│ ┌─────▼─────────────────▼───────┐ │
│ │ DSP + Chunk Scheduler          │ │
│ └───────────────────────────────┘ │
└────────────────────────────────────┘
```

Later production Tauri mode:

```text
┌────────────────────────────────────┐
│ Tauri App                          │
│ WebView UI + Rust shell            │
│                                    │
│ Option A: sidecar vc-daemon        │
│ Option B: embedded vc-runtime lib  │
└────────────────────────────────────┘
```

For MVP, sidecar daemon is cleaner because it also supports CLI/headless/SDK use.

## 7.3 Repository shape

```text
vc-runtime/
  crates/
    vc-core/              # common types, time, errors, config
    vc-audio/             # CPAL + platform abstraction
    vc-dsp/               # ring buffers, resampler, SOLA, crossfade
    vc-scheduler/         # chunk scheduler, deadlines
    vc-model/             # model runtime traits
    vc-ort/               # ONNX Runtime integration
    vc-plugin-rvc/        # RVC implementation
    vc-providers/         # provider manager, probing, ranking
    vc-diagnostics/       # metrics, profiler, tracing
    vc-daemon/            # HTTP/WS/local IPC
    vc-cli/               # command-line tools
    vc-bench/             # benchmark harness
  web/
    dev-ui/               # local web UI
  apps/
    tauri/                # later production shell
  tools/
    convert-rvc/          # Python/Rust conversion tooling
    package-provider/     # provider pack tools
  models/
    schemas/              # manifest schemas
  docs/
    design/
    benchmarks/
```

---

# 8. Core components

## 8.1 Audio Device Manager

**Responsibility**

* Enumerate audio hosts/devices.
* Track input/output capabilities.
* Detect sample rates, channel counts, sample formats.
* Handle hotplug.
* Provide stable device IDs where OS allows.
* Expose routing hints.

CPAL supports host/device enumeration and stream configs, but device identity and hotplug behavior vary by backend. It also exposes optional platform backends such as ASIO/JACK/PipeWire/PulseAudio through features. ([GitHub][3])

**Public interface**

```text
list_hosts() -> Vec<AudioHostInfo>
list_devices(host?) -> Vec<AudioDeviceInfo>
get_default_input() -> Option<DeviceId>
get_default_output() -> Option<DeviceId>
get_supported_configs(device) -> Vec<StreamConfigRange>
subscribe_device_events() -> Stream<DeviceEvent>
validate_route(input, output, requested_config) -> RouteValidation
```

**Internal constraints**

* Device enumeration can happen on normal control thread.
* Opening/closing streams must not happen in audio callback.
* Hotplug must trigger graceful session transition, not panic.

**Realtime-safe**

* No enumeration in callback.
* No device open/close in callback.
* Callback sees only prebuilt stream handles and lock-free buffers.

**Normal async/control threads**

* Device refresh.
* Permissions checks.
* Route validation.
* UI hints.

**Failure modes**

* Device disappears.
* Default device changes.
* Unsupported sample rate.
* Exclusive-mode conflict.
* Linux ALSA `DeviceBusy`.
* macOS microphone permission denied.
* Windows virtual cable disabled.

**Metrics**

```text
active_input_device
active_output_device
input_sample_rate
output_sample_rate
input_buffer_size_frames
output_buffer_size_frames
device_clock_drift_ppm
device_hotplug_count
stream_error_count
```

---

## 8.2 Audio Stream Engine

**Responsibility**

* Own input/output streams.
* Run minimal callbacks.
* Copy audio frames to/from ring buffers.
* Timestamp callback boundaries.
* Count underruns/overruns.
* Maintain audio clock.

**Public interface**

```text
start_audio(route, stream_config) -> AudioStreamHandle
stop_audio(handle)
pause_audio(handle)
get_stream_state() -> AudioStreamState
```

**Internal constraints**

* Input and output callbacks are the hottest realtime path.
* Must avoid heap allocation, blocking locks, logging, syscalls where possible.
* Must tolerate callback buffer sizes that differ from requested size.

**Realtime-safe**

Allowed:

```text
copy frames
convert sample format if unavoidable
push/pop lock-free ring
increment atomic counters
read atomic config snapshot
timestamp using monotonic low-overhead clock
```

Forbidden:

```text
model inference
resampling if heavy
file I/O
network I/O
blocking mutex
allocation
JSON serialization
logging formatting
```

**Failure modes**

* Input callback faster/slower than output callback.
* Output ring underrun.
* Input ring overrun.
* Callback receives non-uniform frame counts.
* Device backend changes buffer size.

**Metrics**

```text
input_callback_interval_p50/p95/p99
output_callback_interval_p50/p95/p99
input_overrun_count
output_underrun_count
callback_xrun_count
callback_max_duration_ms
```

---

## 8.3 Ring Buffer / Jitter Buffer

**Responsibility**

* Decouple audio callbacks from model worker.
* Absorb small scheduling jitter.
* Track exact frame counts and timestamps.
* Provide bounded memory behavior.

**Public interface**

```text
push(frames, timestamp) -> PushResult
pop_exact(n_frames) -> PopResult
peek_available() -> FrameCount
latency_frames() -> usize
clear()
set_watermarks(min, target, max)
```

**Internal constraints**

* Prefer SPSC lock-free ring for callback-to-worker and worker-to-callback.
* Use preallocated contiguous buffers.
* Store timestamp spans, not just samples.
* Avoid unbounded queues.

**Realtime-safe**

* `push` from input callback.
* `pop` from output callback.
* Atomic counters only.

**Normal threads**

* Resizing buffers requires session restart or safe swap at boundary.
* Diagnostics can read snapshots.

**Failure modes**

* Overrun: input produces more than worker consumes.
* Underrun: output consumes more than worker produces.
* Timestamp discontinuity.
* Clock drift between input/output devices.

**Metrics**

```text
depth_frames
depth_ms
min_depth_ms
max_depth_ms
overrun_count
underrun_count
dropped_frames
inserted_silence_frames
```

---

## 8.4 Chunk Scheduler

**Responsibility**

* Convert continuous stream into deterministic model chunks.
* Manage context/padding/lookahead.
* Align chunks to model hop size.
* Set processing deadlines.
* Decide skip/drop policy under overload.

**Public interface**

```text
next_chunk() -> Option<AudioChunk>
commit_output(chunk_id, output_frames)
set_chunk_policy(policy)
set_model_timing(model_timing)
```

**Internal constraints**

* Chunk sizes must be model-compatible.
* Context frames must be explicit in metadata.
* Scheduler must know algorithmic latency introduced by lookback/lookahead.
* It should not silently increase latency.

**Realtime-safe**

* Scheduler runs on model worker, not audio callback.
* It can use preallocated buffers and bounded queues.

**Normal threads**

* Auto-tuning can request new chunk policy.
* Config updates applied at chunk boundary.

**Failure modes**

* Not enough input frames.
* Model worker falls behind.
* Chunk misalignment causes artifacts.
* Context too short reduces quality.
* Context too long increases latency.

**Metrics**

```text
chunk_size_ms
context_left_ms
context_right_ms
scheduler_wait_ms
missed_deadline_count
dropped_chunk_count
rtf
```

---

## 8.5 Resampler

**Responsibility**

* Convert between device rate and model rate.
* Maintain streaming state.
* Provide deterministic latency.
* Avoid repeated unnecessary conversions.

**Public interface**

```text
create_resampler(src_rate, dst_rate, channels, quality) -> Resampler
process(input_frames) -> output_frames
latency_frames() -> usize
reset()
```

**Internal constraints**

* Use streaming resampler with known group delay.
* Prefer one internal canonical layout: `f32`, planar or interleaved chosen consistently.
* Avoid resampling in audio callback unless trivial and proven safe.

**Realtime-safe**

* Callback may do lightweight sample format conversion only.
* Resampling should run on DSP/model worker.

**Failure modes**

* Sample rate mismatch not handled.
* Resampler state reset creates clicks.
* Wrong channel mapping.
* Excessive CPU for high-quality mode.

**Metrics**

```text
input_resample_time_ms
output_resample_time_ms
resampler_delay_ms
src_rate
dst_rate
quality_mode
```

---

## 8.6 DSP Pre/Post Processor

**Responsibility**

Pre:

* gain;
* mono/stereo conversion;
* DC blocker;
* optional noise gate/VAD;
* normalization/clipping protection;
* silence detection.

Post:

* output gain;
* limiter;
* de-click;
* fade-in/out on start/stop;
* clipping stats.

**Public interface**

```text
process_input(chunk, params) -> DspInput
process_output(frames, params) -> DspOutput
update_params(params)
```

**Internal constraints**

* Must be deterministic.
* No ML here except optional lightweight VAD.
* Avoid hidden latency unless reported.

**Realtime-safe**

* Runs on DSP/model worker.
* Params read via atomic snapshot or boundary update.

**Failure modes**

* Noise gate cuts speech onset.
* Limiter pumps audio.
* Gain causes clipping.
* Stereo routing misconfigured.

**Metrics**

```text
input_peak_dbfs
output_peak_dbfs
clipped_sample_count
vad_state
noise_gate_reduction_db
dsp_time_ms
```

---

## 8.7 SOLA / Crossfade Engine

**Responsibility**

* Stitch model output chunks into continuous audio.
* Align phase/waveform near boundaries.
* Hide model discontinuities.
* Maintain fixed output cadence.

**Public interface**

```text
stitch(previous_tail, new_chunk, policy) -> StitchedFrames
set_crossfade_ms(ms)
set_sola_search_ms(ms)
reset()
```

**Internal constraints**

* Search window must be bounded.
* Crossfade length must be short enough not to smear speech.
* All introduced latency must be measured.
* Must handle silence differently from voiced audio.

**Realtime-safe**

* Runs on model worker.
* Uses preallocated scratch buffers.

**Failure modes**

* Too small crossfade: clicks.
* Too large crossfade: muffled/phasiness.
* SOLA wrong alignment: flanging/robotic transitions.
* Output chunk shorter than expected.

**Metrics**

```text
sola_time_ms
crossfade_time_ms
chosen_offset_frames
boundary_energy_delta
click_risk_score
```

---

## 8.8 Model Runtime Interface

**Responsibility**

* Abstract model execution from pipeline logic.
* Support ONNX Runtime first.
* Keep PyTorch as fallback/dev path.
* Expose provider, shapes, precision, warmup, profiling.

**Public interface**

```text
trait ModelSession {
    model_id() -> ModelId
    stage_name() -> StageName
    input_spec() -> TensorSpec
    output_spec() -> TensorSpec
    provider_status() -> ProviderStatus
    run(inputs, deadline) -> Result<Outputs>
    warmup(sample_inputs) -> WarmupReport
    metrics() -> ModelSessionMetrics
}
```

**Internal constraints**

* Static shapes preferred for realtime.
* Avoid session creation in active audio path.
* Preallocate tensors where possible.
* Use I/O binding for GPU paths where it materially reduces copies. ONNX Runtime notes that with non-CPU providers, arranging inputs/outputs on target device avoids implicit CPU↔device copies around `Run()`. ([ONNX Runtime][4])

**Realtime-safe**

* Model execution is not audio-callback-safe.
* It can run on dedicated model worker thread.

**Failure modes**

* Provider unavailable.
* Unsupported op fallback.
* Shape mismatch.
* First-run compilation spike.
* GPU OOM.
* CPU thread oversubscription.

**Metrics**

```text
model_run_time_ms
provider_requested
provider_actual
device_id
precision
input_copy_time_ms
output_copy_time_ms
session_warmup_time_ms
gpu_memory_bytes
```

---

## 8.9 RVC Pipeline implementation

**Responsibility**

Implement RVC realtime inference pipeline:

```text
audio chunk
  -> input resample
  -> f0/pitch extraction
  -> content features
  -> optional index retrieval
  -> generator/inference
  -> output reconstruction
  -> SOLA/crossfade
```

RVC’s official WebUI documentation references required pre-models such as HuBERT, pretrained assets and RMVPE; it also documents `rmvpe.pt` and `rmvpe.onnx` for AMD/Intel users. ([GitHub][5])

**Public interface**

```text
load_rvc_model(manifest) -> RvcPipeline
process_chunk(audio_chunk, live_params) -> ConvertedChunk
update_live_params(params)
describe_latency_requirements() -> LatencySpec
```

**Internal constraints**

* All model components expose stage-level metrics.
* RVC params must update at chunk boundary.
* Feature/pitch/generator rates must be encoded in manifest.
* Index retrieval must be optional and bounded.

**Realtime-safe**

* No Python inside production audio path.
* No model loading during active processing.
* Live param update must be lock-free or boundary-swapped.

**Failure modes**

* `.pth` cannot be converted.
* ONNX graph unsupported by provider.
* f0 method too slow.
* index file incompatible.
* model output sample rate mismatch.
* speaker id mismatch.

**Metrics**

```text
rvc_pitch_ms
rvc_feature_ms
rvc_index_ms
rvc_generator_ms
rvc_total_ms
f0_method
index_enabled
index_rate
speaker_id
transpose
```

---

## 8.10 Provider Manager

**Responsibility**

* Detect available providers.
* Validate provider compatibility per model stage.
* Rank providers by policy and benchmark.
* Expose actual provider usage.
* Handle fallback explicitly.

**Public interface**

```text
probe_providers() -> Vec<ProviderCandidate>
rank(model_stage, policy) -> RankedProviders
create_session(model, provider_policy) -> ModelSession
benchmark_provider(stage, provider, sample_inputs) -> BenchmarkReport
explain_selection(session) -> ProviderExplanation
```

**Internal constraints**

* Provider probing must not crash the daemon; use isolation where needed.
* Provider choice must be reproducible and logged.
* Fallback must be explicit: “requested CUDA, actual CPU fallback for 12 nodes” if discoverable.

**Provider policy example**

```text
policy:
  prefer:
    - TensorRT
    - CUDA
    - DirectML
    - CoreML
    - OpenVINO
    - CPU
  max_p95_ms: 30
  allow_cpu_fallback: false
  precision:
    fp16: auto
    int8: off
```

**Failure modes**

* DLL/shared library missing.
* Driver mismatch.
* CUDA/cuDNN mismatch.
* DirectML unsupported opset.
* CoreML shape issue.
* OpenVINO environment not configured.
* TensorRT engine build too slow.

**Metrics**

```text
available_providers
observed_provider_assignment
provider_assignment_granularity
fallback_reason
provider_init_time_ms
provider_benchmark_p50/p95/p99
provider_memory_bytes
```

---

## 8.11 Model Registry / Model Loader

**Responsibility**

* Store imported models.
* Validate files.
* Maintain model manifest.
* Manage converted ONNX files and provider caches.
* Track licenses and dependencies.

**Public interface**

```text
import_model(path_or_bundle) -> ImportReport
list_models() -> Vec<ModelSummary>
load_model(model_id, slot) -> LoadReport
validate_model(model_id, target_provider) -> CompatibilityReport
delete_model(model_id)
```

**Internal constraints**

* Model loading is control-plane operation.
* Runtime session swap must happen atomically.
* Model cache keyed by content hashes.

**Failure modes**

* Missing config.
* Unsupported architecture.
* Hash mismatch.
* Dependency missing.
* License unknown.
* ONNX export invalid.

**Metrics**

```text
model_count
loaded_model_id
model_load_time_ms
cache_hit_rate
model_size_bytes
dependency_status
```

---

## 8.12 Control API / IPC

**Responsibility**

* Provide daemon API for UI/CLI/SDK.
* Manage sessions.
* Stream metrics/events.
* Support future streaming audio over IPC/network.

**Public interface**

REST-like control:

```text
GET  /v1/health
GET  /v1/devices
POST /v1/session
POST /v1/session/start
POST /v1/session/stop
POST /v1/models/import
POST /v1/models/{id}/load
PATCH /v1/session/params
GET  /v1/metrics/snapshot
GET  /v1/logs/recent
```

WebSocket topics:

```text
/ws/metrics
/ws/events
/ws/logs
/ws/audio-preview       optional
```

Native IPC later:

```text
Windows: named pipes
macOS/Linux: Unix domain sockets
Shared memory: audio streaming fast path
```

**Internal constraints**

* Control API cannot call into audio callback.
* Commands applied at safe boundaries.
* Long operations are cancellable.

**Failure modes**

* UI disconnect.
* Invalid config.
* Model import halfway failed.
* Session already running.
* IPC auth token mismatch.

**Metrics**

```text
api_request_count
api_error_count
ws_client_count
control_command_latency_ms
```

---

## 8.13 Diagnostics / Profiler

**Responsibility**

* Collect chunk-level timings.
* Collect queue depths.
* Track provider usage.
* Report xruns.
* Produce live dashboard and benchmark reports.

**Public interface**

```text
record_stage(chunk_id, stage, start, end)
record_queue_depth(name, frames)
record_provider(stage, provider_status)
snapshot() -> MetricsSnapshot
stream() -> MetricsEventStream
export_trace(format) -> TraceFile
```

**Internal constraints**

* Metrics path must be low overhead.
* Use fixed-size ring of metrics.
* No string formatting in hot path.
* High-cardinality labels avoided.

**Realtime-safe**

* Audio callback can increment atomics and push compact metric events to lock-free buffer.
* Full aggregation happens off realtime thread.

**Failure modes**

* Metrics overload.
* Clock skew between threads.
* Provider logs unavailable.
* GPU metrics API unavailable.

**Metrics**

The profiler itself should expose overhead:

```text
profiler_event_drop_count
profiler_queue_depth
profiler_overhead_us
```

---

## 8.14 Config System

**Responsibility**

* Define typed config schema.
* Merge defaults, user config, model manifest, session overrides.
* Validate before applying.
* Support config export for bug reports.

**Public interface**

```text
load_config(path) -> Config
validate_config(config) -> ValidationReport
apply_session_patch(patch) -> ApplyReport
export_effective_config(redact_secrets=true) -> Config
```

**Internal constraints**

* No untyped “bag of JSON” in core.
* Config changes classified:

  * live-safe;
  * chunk-boundary;
  * session-restart;
  * daemon-restart.

**Failure modes**

* Invalid sample rate.
* Provider unavailable.
* Chunk size incompatible with model.
* Hot config update requires restart.

**Metrics**

```text
config_reload_count
config_validation_errors
last_applied_config_version
```

---

## 8.15 Preset System

**Responsibility**

* Save user-friendly combinations of model + params + routing + latency mode.
* Separate voice preset from hardware preset.
* Provide safe defaults.

**Public interface**

```text
save_preset(name, scope)
load_preset(id)
list_presets()
validate_preset(id, current_hardware)
```

**Preset scopes**

```text
Voice preset:
  model_id
  transpose
  index_rate
  f0_method
  formant/protect params if supported

Runtime preset:
  chunk_size
  provider_policy
  buffer_policy
  precision

Routing preset:
  input_device
  output_device
  monitor_device
```

**Failure modes**

* Preset references missing device.
* Preset references unsupported provider.
* Model deleted.
* Preset too aggressive for current hardware.

**Metrics**

```text
preset_load_count
preset_validation_warning_count
```

---

## 8.16 Logging / Crash Reporting

**Responsibility**

* Structured logs.
* Crash dumps.
* User-shareable diagnostic bundles.
* Opt-in crash reports.

**Public interface**

```text
set_log_level(level)
export_diagnostics_bundle()
enable_crash_reports(opt_in)
```

**Internal constraints**

* Logs must never record raw microphone audio by default.
* Logs must redact paths/usernames where possible.
* Crash reporting is opt-in.

**Failure modes**

* Log spam affects performance.
* Sensitive data leak.
* Crash during provider init.

**Metrics**

```text
log_error_count
panic_count
crash_report_opt_in
diagnostic_bundle_size
```

---

## 8.17 Packaging / Updates

**Responsibility**

* Ship runtime, UI, provider packs, model dependencies.
* Manage updates.
* Verify signatures/checksums.
* Keep GPU dependencies modular.

**Public interface**

```text
check_updates(channel)
install_provider_pack(provider)
verify_package(package)
repair_installation()
```

**Internal constraints**

* Base package should run CPU passthrough without GPU dependencies.
* GPU provider packs should be optional.
* TensorRT should not be mandatory because of size/licensing/driver complexity.

**Failure modes**

* Missing DLL/shared library.
* Broken CUDA path.
* Unsigned macOS binary blocked.
* Linux dependency mismatch.
* Partial update.

**Metrics**

```text
installed_version
provider_pack_versions
update_channel
package_integrity_status
```

---

# 9. Audio backend strategy

## 9.1 CPAL as base

CPAL is the right MVP base because it provides a Rust-native low-level audio abstraction and supports the relevant desktop backends: Windows/WASAPI with optional ASIO/JACK, macOS/CoreAudio with optional JACK, Linux/ALSA/JACK/PipeWire/PulseAudio. ([GitHub][3])

Use CPAL for:

* initial device enumeration;
* basic input/output streams;
* Windows WASAPI shared mode;
* macOS CoreAudio;
* Linux PipeWire/PulseAudio/JACK/ALSA experiments;
* cross-platform audio passthrough MVP.

Do not assume CPAL solves:

* virtual device creation;
* best possible exclusive-mode latency;
* all hotplug edge cases;
* OS-specific routing UX;
* professional ASIO workflows;
* Linux realtime scheduling permissions.

## 9.2 Windows

### Base

* CPAL + WASAPI shared mode.
* Optional WASAPI exclusive mode later.
* Optional ASIO for pro users.

### Virtual cable reality

The app should not ship a driver in MVP. It should detect common virtual devices and help route:

```text
Input: physical mic
Output: VB-CABLE / Voicemeeter / Sonar / OBS virtual mic
Monitor: headphones
```

### ASIO

ASIO can reduce latency, but:

* requires ASIO driver;
* CPAL ASIO feature has extra build requirements;
* consumer users often do not have ASIO devices;
* ASIO routing into Discord still requires bridge/virtual cable.

Use ASIO as optional advanced backend, not MVP default.

## 9.3 macOS

### Base

* CPAL + CoreAudio.
* Microphone permission flow.
* Device hotplug via CoreAudio events where CPAL is insufficient.

### Virtual cable

Recommend external virtual devices:

* BlackHole;
* Loopback;
* OBS virtual camera/audio workflows where applicable.

Custom virtual audio driver is non-goal for MVP.

### Provider implication

Apple Silicon path should test CoreML EP, but fallback to ORT CPU must exist. CoreML EP supports compute unit options such as CPUOnly, CPUAndNeuralEngine, CPUAndGPU, ALL, and dynamic shapes may negatively affect performance unless constrained. ([ONNX Runtime][6])

## 9.4 Linux

### Preferred

* PipeWire first for desktop Linux.
* JACK for pro-audio users.
* PulseAudio fallback.
* ALSA direct fallback only when user understands device locking.

CPAL docs explicitly note that when PipeWire/PulseAudio holds ALSA default exclusively, opening ALSA default can fail; native PipeWire/PulseAudio features or bridge devices are preferable. ([GitHub][3])

### Realtime priority

Use CPAL `realtime`/`realtime-dbus` features where possible. CPAL notes Linux/BSD RT priority may require `rtprio` limits or rtkit via D-Bus, and smaller buffers reduce latency but increase glitch risk. ([GitHub][3])

## 9.5 Buffer size selection

Runtime should expose three levels:

```text
Safe:
  device default or 1024 frames
  lower crackle risk
  higher latency

Balanced:
  256–512 frames when supported
  good default for most users

Aggressive:
  64–128 frames
  only after calibration
  high CPU/glitch risk
```

Never present “buffer size” as a magic quality slider. Show:

```text
requested buffer
actual callback frame distribution
underrun count
callback p95/p99
```

## 9.6 Sample rate mismatch

Policy:

```text
1. Prefer matching input/output device sample rate.
2. Prefer 48 kHz device I/O.
3. Resample once into model rate.
4. Resample once back to output rate.
5. Never chain hidden resamplers.
```

UI must show:

```text
Input device: 48 kHz
Model content: 16 kHz
Model output: 40 kHz
Output device: 48 kHz
Resamplers: input 48->16, output 40->48
```

## 9.7 Where platform-specific layers may be needed

CPAL is enough for MVP passthrough and many users.

Platform-specific code may be needed for:

| Area                              | Why                          |
| --------------------------------- | ---------------------------- |
| Windows WASAPI exclusive          | tighter control over latency |
| Windows device role/session APIs  | better routing UX            |
| Windows audio endpoint IDs        | stable device identity       |
| ASIO                              | pro-audio latency            |
| macOS CoreAudio aggregate devices | advanced routing             |
| macOS permissions                 | UX and recovery              |
| Linux PipeWire graph              | routing assistant            |
| JACK transport/routing            | pro users                    |
| realtime priority                 | better diagnostics and setup |

---

# 10. Model runtime strategy

## 10.1 Mainline: ONNX Runtime

ONNX Runtime is the main production path because it gives a consistent API over multiple hardware Execution Providers. It supports provider-level graph partitioning and a broad EP ecosystem. ([ONNX Runtime][1])

Mainline design:

```text
RVC model components exported to ONNX
  -> ORT session per stage
  -> provider-specific session options
  -> static shapes where possible
  -> warmup benchmark
  -> live per-stage metrics
```

## 10.2 CUDA EP

**Use for:** NVIDIA mainline on Windows/Linux.

Pros:

* mature NVIDIA GPU path;
* lower complexity than TensorRT;
* good first target for RVC ONNX;
* supports CUDA-specific tuning options.

Risks:

* CUDA/cuDNN version compatibility;
* packaging DLL/shared libraries;
* user PATH/LD_LIBRARY_PATH problems;
* CPU↔GPU copies if I/O binding not used.

ONNX Runtime CUDA docs note CUDA/cuDNN version compatibility and that GPU packages require matching CUDA/cuDNN runtime dependencies unless using packaging paths that preload compatible libraries. ([ONNX Runtime][7])

## 10.3 TensorRT EP

**Use for:** optional NVIDIA acceleration after ONNX graphs are stable.

Pros:

* potentially lower inference latency than CUDA EP;
* FP16/INT8 options;
* engine caches reduce repeated build cost.

Risks:

* engine build can be slow;
* operator coverage;
* dynamic shape complexity;
* per-GPU/per-driver cache invalidation;
* packaging and support burden;
* first-run latency spike.

ONNX Runtime TensorRT EP can fallback subgraphs to other EPs, supports FP16/INT8 options, and has cache/engine-related settings; NVIDIA also warns that TensorRT engine files are executable artifacts and should only be deserialized if built or received over trusted channels. ([ONNX Runtime][8])

Recommendation:

```text
MVP:
  CUDA EP first.

Post-MVP:
  TensorRT optional provider pack.
  Build engines during model optimization step, not during live session.
  Cache key includes:
    model_hash
    provider_version
    TensorRT_version
    CUDA_version
    GPU_name
    compute_capability
    precision
    static_shape
```

## 10.4 DirectML / WinML

**Use for:** Windows compatibility fallback for AMD/Intel/NVIDIA when CUDA is unavailable.

Pros:

* broad Windows GPU support;
* no vendor-specific CUDA installation;
* useful for AMD/Intel users.

Risks:

* ONNX Runtime DirectML is now documented as sustained engineering, with WinML suggested for new Windows deployments;
* opset/operator limits;
* requires DirectX 12 capable device;
* must disable memory pattern optimizations and use sequential execution;
* same session cannot be called concurrently from multiple threads. ([ONNX Runtime][9])

Recommendation:

```text
MVP:
  Support DirectML as compatibility EP on Windows.
  Show it as "Windows GPU compatibility mode", not "fastest mode".

Research:
  Evaluate WinML EP selection APIs for future Windows builds.
```

## 10.5 CoreML EP

**Use for:** macOS Apple Silicon path.

Pros:

* access to Apple CPU/GPU/ANE through CoreML;
* good power/performance potential on Apple Silicon.

Risks:

* CoreML EP build/distribution details for macOS C/C++ need verification per ONNX Runtime release;
* dynamic shapes may hurt performance;
* unsupported ops fallback;
* Apple Neural Engine behavior can be opaque.

Recommendation:

```text
MVP:
  macOS audio passthrough + ORT CPU baseline.
  CoreML EP experimental provider.

Post-MVP:
  Static-shape RVC ONNX/CoreML path.
  Apple Silicon benchmark matrix.
```

CoreML EP requires macOS 10.15+ and recommends Apple Neural Engine devices for best performance; it exposes compute unit and static shape options. ([ONNX Runtime][6])

## 10.6 OpenVINO EP

**Use for:** Intel CPU/iGPU/NPU on Windows/Linux.

Pros:

* supports CPU/GPU/NPU;
* AUTO/HETERO/MULTI modes;
* good Intel deployment story.

Risks:

* OpenVINO environment variables/installers;
* device-specific behavior;
* NPU availability varies;
* packaging redistributables.

OpenVINO EP docs list CPU, GPU, NPU and modes like AUTO/HETERO/MULTI. ([ONNX Runtime][10])

Recommendation:

```text
MVP:
  CPU EP universally.
  OpenVINO optional provider pack for Intel.

Post-MVP:
  Intel-specific tuning and NPU experiments.
```

## 10.7 XNNPACK / CPU

**Use for:** CPU fallback and possibly mobile/web later.

Pros:

* optimized CPU kernels;
* useful for ARM/x86/mobile/WebAssembly paths.

Risks:

* desktop packaging/support must be verified per ORT build;
* operator coverage;
* may not accelerate full RVC graph.

ONNX Runtime describes XNNPACK as optimized for Arm, WebAssembly and x86 platforms and especially relevant to Android/iOS/WASM. ([ONNX Runtime][11])

## 10.8 PyTorch

**Use for:**

* dev fallback;
* conversion;
* reference correctness;
* model import validation;
* unsupported model family experiments.

Do not use PyTorch as main realtime path in MVP because:

* dependency size;
* Python environment complexity;
* GPU library conflicts;
* hard to guarantee callback-safe / stable low-jitter runtime;
* provider diagnostics become less uniform.

PyTorch can run in:

```text
tools/convert-rvc
dev compatibility mode
offline benchmark reference
```

## 10.9 `torch.compile`

**Use for:** optional Python fallback optimization.

Not MVP mainline.

Risks:

* warmup/compile time;
* dynamic shape sensitivity;
* version-specific behavior;
* not cross-platform deployment primitive.

## 10.10 Triton kernels

Triton is appropriate only for specific profiled GPU hot spots, not as a general runtime replacement. Official Triton compatibility currently lists Linux, NVIDIA GPUs with Compute Capability 8.0+, AMD GPUs with ROCm 6.2+, and CPU as under development. ([GitHub][12])

Use Triton for:

* small custom tensor transforms that dominate p95;
* GPU-side postprocessing;
* custom interpolation/gather/scatter;
* possible pitch/index hot spots after profiling.

Do not use Triton for:

* audio callback path;
* cross-platform MVP;
* macOS;
* Windows-first acceleration;
* replacing ONNX Runtime;
* speculative optimization before profiler proves need.

---

# 11. RVC-first MVP

## 11.1 MVP RVC pipeline

```text
Input 48k mono/stereo
  -> mono/select channel
  -> resample to content/f0 rate
  -> pitch extractor
  -> content extractor
  -> optional index retrieval
  -> RVC generator
  -> output waveform
  -> resample to output device rate
  -> SOLA/crossfade
  -> output ring
```

## 11.2 Model loading

Supported in MVP:

```text
Preferred:
  .vcrt bundle
    manifest.json
    generator.onnx
    pitch.onnx optional
    content.onnx optional
    index.vci optional
    license.txt
    preview.wav optional

Import:
  RVC .pth + config + optional .index
  converted offline to .vcrt
```

Do not make arbitrary `.pth` loading part of realtime session startup. Import/conversion is a tools workflow.

## 11.3 Manifest sketch

```json
{
  "schema_version": "0.1",
  "family": "rvc",
  "model_id": "sha256:...",
  "display_name": "example voice",
  "sample_rates": {
    "input_preferred": 48000,
    "content": 16000,
    "generator_output": 40000
  },
  "hop": {
    "content_hop_ms": 20,
    "f0_hop_ms": 10
  },
  "files": {
    "generator": "generator.onnx",
    "pitch": "rmvpe.onnx",
    "content": "contentvec.onnx",
    "index": "speaker.index"
  },
  "providers": {
    "generator": ["CUDA", "TensorRT", "DirectML", "CoreML", "OpenVINO", "CPU"],
    "pitch": ["CUDA", "DirectML", "CPU"],
    "content": ["CUDA", "CoreML", "OpenVINO", "CPU"]
  },
  "latency": {
    "min_chunk_ms": 40,
    "recommended_chunk_ms": 80,
    "context_left_ms": 120,
    "lookahead_ms": 0
  }
}
```

## 11.4 Pitch extractor

MVP order:

```text
1. RMVPE ONNX if compatible.
2. Lightweight CPU fallback for weak machines.
3. PyTorch RMVPE only in dev/fallback mode.
```

Pitch extraction is often the hidden bottleneck. It must be independently timed and provider-labelled.

Metrics:

```text
f0_method
f0_provider
f0_time_ms
f0_voiced_ratio
f0_nan_count
```

## 11.5 Content extractor

MVP order:

```text
1. ContentVec/HuBERT ONNX static shape.
2. ORT CUDA/CPU/OpenVINO/CoreML depending platform.
3. PyTorch reference only for conversion/dev.
```

RVC docs reference HuBERT/pretrained dependencies as required assets. ([GitHub][5])

## 11.6 Index retrieval

Index retrieval improves speaker similarity but can be a CPU latency trap.

MVP policy:

```text
Phase 2:
  index disabled by default
  support no-index RVC first

Phase 3:
  import .index
  convert to internal .vci format
  bounded top-k search
  metric: index_time_ms
```

Implementation options:

| Option                  | Pros                                | Cons                           |
| ----------------------- | ----------------------------------- | ------------------------------ |
| FAISS C++               | compatible with common RVC `.index` | packaging/native deps          |
| hnswlib-like Rust crate | easier packaging                    | conversion needed              |
| flat top-k SIMD         | deterministic, simple               | slow for large index           |
| GPU index               | fast                                | overkill and provider-specific |

Recommendation: start with **no-index + internal converted index format**. Treat original `.index` import as conversion, not runtime dependency.

## 11.7 Generator / inference

Production path:

```text
generator.onnx
  static chunk shape
  ORT session
  provider selected by Provider Manager
  warmup before session start
  preallocated input/output tensors
```

Need profile:

* fp32 vs fp16 quality;
* static vs dynamic chunk;
* batch size always 1;
* CUDA EP vs TensorRT EP;
* DirectML operator fallback;
* CoreML static shape options.

## 11.8 Output reconstruction

RVC generator often emits waveform already. If vocoder is separate in a future family, model plugin exposes separate stages:

```text
acoustic_model -> vocoder -> postprocess
```

For RVC MVP:

```text
generator output -> post gain/limiter -> output resample -> SOLA/crossfade
```

## 11.9 Live parameter update

Live-safe params:

```text
input_gain
output_gain
transpose
index_rate
protect
f0_method only if preloaded
speaker_id if model supports
monitor_mix
```

Not live-safe without session rebuild:

```text
provider
precision
chunk size
model file
content extractor architecture
generator shape
```

Parameter updates apply at chunk boundary:

```text
control thread sends ParamPatch
model worker receives patch
scheduler applies patch to next chunk
metrics emits param_version
```

---

# 12. Hardware/provider matrix

| Platform            | Preferred inference provider                            | Audio backend                                | Expected limitations                                                                    | Fallback                          | Packaging concern                                                      |
| ------------------- | ------------------------------------------------------- | -------------------------------------------- | --------------------------------------------------------------------------------------- | --------------------------------- | ---------------------------------------------------------------------- |
| Windows + NVIDIA    | CUDA EP first; TensorRT optional                        | CPAL/WASAPI; optional ASIO                   | CUDA/cuDNN mismatch; laptop GPU selection; TensorRT engine build                        | DirectML, CPU                     | Separate CUDA build/provider pack; Visual C++ runtime; DLL search path |
| Windows + AMD       | DirectML/WinML research path                            | CPAL/WASAPI                                  | DirectML sustained engineering; opset/operator limits; sequential execution requirement | CPU                               | DirectML DLL/NuGet-style deps; DX12 capable GPU                        |
| Windows + Intel     | OpenVINO for Intel CPU/iGPU/NPU; DirectML compatibility | CPAL/WASAPI                                  | NPU availability variable; OpenVINO setup; DirectML may be slower                       | CPU                               | OpenVINO redistributables; Visual C++ runtime                          |
| Linux + NVIDIA      | CUDA EP; TensorRT optional                              | PipeWire first; JACK optional; ALSA fallback | CUDA/cuDNN install; driver mismatch; realtime permissions                               | CPU                               | CUDA/cuDNN libs, `LD_LIBRARY_PATH`, distro differences                 |
| Linux + AMD         | Experimental ROCm/PyTorch path later; CPU in MVP        | PipeWire/JACK                                | ROCm support matrix fragile; ORT ROCm/MIGraphX packaging risk                           | CPU                               | ROCm driver/runtime; render/video groups                               |
| Linux + Intel       | OpenVINO EP                                             | PipeWire/JACK                                | Intel GPU/NPU drivers; OpenVINO env setup                                               | CPU                               | OpenVINO runtime package                                               |
| macOS Apple Silicon | CoreML EP experimental; CPU baseline                    | CoreAudio                                    | CoreML operator/static shape issues; ANE behavior opaque                                | CPU                               | universal binaries; code signing; notarization                         |
| macOS Intel         | CPU EP; CoreML CPU/GPU if useful                        | CoreAudio                                    | weak/old GPUs; limited acceleration                                                     | CPU                               | notarization; x86_64 build                                             |
| CPU-only            | ORT CPU; XNNPACK where validated                        | platform default                             | larger chunks required; pitch extraction bottleneck                                     | lighter f0/no-index/lower quality | base package must always work                                          |

Important: “preferred” does not mean “force”. Provider Manager should benchmark and select per-stage.

---

# 13. Diagnostics-first design

## 13.1 Live profiler dashboard

Profiler must show:

```text
End-to-end:
  estimated runtime latency
  accumulated delay
  realtime factor
  chunk p50/p95/p99

Audio:
  capture latency
  input queue depth
  output queue depth
  underrun count
  overrun count
  callback interval p95/p99
  actual device sample rate
  actual buffer size

DSP:
  input resample time
  output resample time
  SOLA time
  crossfade time
  clipping count

Model:
  pitch time
  feature extraction time
  index retrieval time
  model inference time
  provider per stage
  requested vs actual provider
  precision per stage

System:
  CPU usage
  CPU thread count
  GPU memory
  GPU utilization if available
  process memory
  thermal/power warning if available
```

## 13.2 Profiler UI concept

```text
┌──────────────────────────────────────────────────────┐
│ Realtime Health: WARNING                             │
│ Runtime latency: 118 ms                              │
│ Accumulated delay: +42 ms and growing                │
│ RTF p95: 1.12                                        │
├──────────────────────┬────────┬────────┬────────────┤
│ Stage                │ p50    │ p95    │ Provider   │
├──────────────────────┼────────┼────────┼────────────┤
│ Input queue wait     │ 8 ms   │ 22 ms  │ audio      │
│ Input resample       │ 0.4 ms │ 0.8 ms │ CPU        │
│ Pitch RMVPE          │ 19 ms  │ 44 ms  │ CPU        │
│ ContentVec           │ 7 ms   │ 12 ms  │ CUDA       │
│ RVC generator        │ 12 ms  │ 18 ms  │ CUDA       │
│ Output resample      │ 0.5 ms │ 0.9 ms │ CPU        │
│ SOLA/crossfade       │ 0.7 ms │ 1.4 ms │ CPU        │
│ Output queue         │ 16 ms  │ 18 ms  │ audio      │
└──────────────────────┴────────┴────────┴────────────┘

Diagnosis:
  Pitch extractor is running on CPU and exceeds budget.
  Try RMVPE ONNX on GPU or increase chunk size from 40 ms to 80 ms.
```

## 13.3 How diagnostics help users

Without diagnostics:

```text
"Sound is choppy. What setting do I change?"
```

With diagnostics:

```text
"Output underruns occur because generator p99 is 87 ms
while chunk size is 40 ms. CUDA is unavailable because
onnxruntime-gpu did not load cuDNN. Current provider is CPU."
```

or:

```text
"Model inference is fine, but output device callback p99 is 35 ms
and PipeWire quantum is 1024 frames. Audio backend is the bottleneck."
```

or:

```text
"Runtime is stable at 60 ms, but Discord adds buffering.
vc-runtime output queue is not growing."
```

## 13.4 Developer diagnostics

Developer needs:

* trace export;
* reproducible benchmark JSON;
* provider init logs;
* ORT profiling output;
* model manifest;
* device config;
* timing histograms;
* audio discontinuity markers.

Diagnostic bundle:

```text
diagnostics.zip
  effective_config.redacted.json
  model_manifest.json
  provider_report.json
  metrics_last_120s.jsonl
  latency_histograms.csv
  logs.txt
  ort_profile_pitch.json optional
  ort_profile_generator.json optional
```

No raw mic audio unless explicitly enabled.

## 13.5 Provider transparency

UI must distinguish:

```text
Requested provider:
  CUDA

Session provider:
  CUDAExecutionProvider

Actual graph assignment:
  96% nodes CUDA
  4% nodes CPU fallback
```

If exact node assignment is not available in stable API, show confidence level:

```text
Provider status:
  CUDA requested and session created.
  ORT profiling enabled for exact node provider attribution.
```

---

# 14. Auto-tuning

## 14.1 Calibration run

Calibration happens before live session or on user request.

Inputs:

```text
selected model
selected devices
available providers
target latency mode:
  low / balanced / stable
representative audio:
  synthetic voiced signal
  optional recorded speech sample
```

Steps:

```text
1. Probe audio devices and supported buffer sizes.
2. Probe providers per model stage.
3. Warm up candidate providers.
4. Benchmark fixed candidate chunk sizes.
5. Estimate p50/p95/p99 and RTF.
6. Select config with headroom.
7. Save hardware profile.
```

Candidate chunk sizes should align to model hop:

```text
20 ms
40 ms
80 ms
120 ms
160 ms
```

Do not allow arbitrary frame counts that break model alignment.

## 14.2 Provider auto-selection

Provider ranking should use:

```text
availability
model compatibility
warmup success
p95 latency
p99 spike behavior
memory usage
expected startup cost
user policy
```

Example:

```text
For Windows + NVIDIA:
  try CUDA
  benchmark CUDA
  try TensorRT only if engine cache exists or user allows build
  use DirectML only if CUDA unavailable or fails
  CPU fallback if all GPU providers fail
```

For Windows + AMD:

```text
try DirectML / future WinML
if DirectML graph unsupported or p95 bad:
  CPU fallback
```

For Intel:

```text
try OpenVINO AUTO:GPU,NPU,CPU
benchmark
if unstable:
  OpenVINO CPU
  ORT CPU
```

## 14.3 Precision auto-selection

Policy:

```text
fp32:
  default correctness baseline

fp16:
  allowed if provider supports and golden audio delta is acceptable
  useful on CUDA/TensorRT/CoreML/OpenVINO GPU

int8:
  not auto-enabled in MVP
  requires calibration and quality validation
```

ONNX Runtime docs note float16 can reduce model size and improve performance on some GPUs with possible accuracy loss; quantization has separate preprocessing/static/dynamic flows. ([ONNX Runtime][13])

## 14.4 Buffer auto-selection

Inputs:

```text
audio callback p95/p99
underrun count
overrun count
CPU load
target latency mode
```

Rules:

```text
if underruns > threshold:
  increase output buffer target by one chunk
  or increase device buffer if callback jitter is source

if input queue grows:
  model pipeline too slow
  increase chunk size or switch provider

if callback interval jitter high:
  increase device buffer
  warn about OS/audio backend
```

## 14.5 Live adaptation

Allowed live changes:

```text
output buffer target within safe range
crossfade length small adjustment
skip index retrieval under overload
temporary silence fast-path
increase model worker queue priority
```

Dangerous live changes:

```text
provider switch
precision switch
chunk size change
sample rate change
model reload
```

Dangerous changes require a controlled transition:

```text
fade out
drain output
rebuild session
warmup
fade in
```

## 14.6 When not to auto-change

Do not auto-change:

* while user is recording benchmark for comparison;
* while output is used in a live stream unless user enabled adaptive mode;
* if preset locks latency;
* if quality mode is fixed;
* if changing would require model/session rebuild;
* if auto-tuner cannot identify bottleneck confidently.

---

# 15. UI strategy

## 15.1 Recommendation

Recommended sequence:

```text
Phase 1–3:
  local web UI for development and diagnostics

Phase 4:
  Tauri shell around same backend

Later:
  plugin/headless SDK
```

Why:

* Web UI is fastest for profiler dashboards.
* Backend remains usable headless.
* Tauri can ship the same web UI with native packaging later.
* Tauri’s Rust side aligns with Rust runtime and desktop packaging story. Tauri supports Windows/macOS/Linux and includes bundling/updater features for desktop distribution. ([Tauri][14])
* Native UI first would slow down runtime work.
* GUI should not become an implicit dependency of audio engine.

## 15.2 UI-to-daemon communication

Development:

```text
Browser UI
  -> localhost HTTP for commands
  -> WebSocket for metrics/logs/events
```

Production Tauri:

```text
Tauri WebView
  -> sidecar vc-daemon via localhost or Unix/named pipe
  -> optional embedded library mode later
```

Use auth token even on localhost:

```text
daemon creates random session token
UI receives token via launch args/env
requests require token
```

## 15.3 Required screens

### Device graph

Shows:

```text
Mic -> vc-runtime -> output device -> target app
Monitor device optional
sample rates
buffer sizes
clock drift
```

### Model manager

Shows:

```text
installed models
format
dependencies
provider compatibility
conversion status
license
```

### Voice presets

Shows:

```text
model
transpose
f0 method
index rate
gain
latency mode
hotkey
```

### Latency/profiler dashboard

Shows p50/p95/p99, queue depth graphs, stage timings.

### Provider diagnostics

Shows requested vs actual provider per stage.

### Routing assistant

Platform-specific guidance:

```text
Windows:
  select VB-CABLE as output
  set Discord input to CABLE Output

macOS:
  select BlackHole as output
  create Multi-Output Device if monitoring

Linux:
  PipeWire routing graph
  JACK ports if JACK mode
```

### Advanced settings

Chunk, crossfade, provider policy, CPU threads, precision, buffer policy.

### Logs/errors

Human-readable errors:

```text
"CUDA provider failed: cuDNN 9 DLL not found.
Using CPU fallback. Install NVIDIA provider pack or switch to DirectML."
```

not:

```text
EP_FAIL 0x...
```

---

# 16. API design

## 16.1 Library API

Rust-like conceptual API:

```text
Runtime::init(config) -> Runtime

runtime.list_devices() -> Vec<AudioDeviceInfo>
runtime.list_providers() -> Vec<ProviderInfo>
runtime.import_model(path) -> ImportReport
runtime.load_model(slot, model_id, provider_policy) -> LoadReport

session = runtime.create_session(session_config)
session.start()
session.update_params(param_patch)
session.metrics_snapshot()
session.stop()
```

## 16.2 Daemon control API

```text
GET /v1/health
GET /v1/version
GET /v1/devices
GET /v1/providers
GET /v1/models

POST /v1/models/import
POST /v1/models/{model_id}/convert
POST /v1/sessions

POST /v1/sessions/{id}/start
POST /v1/sessions/{id}/stop
PATCH /v1/sessions/{id}/params
PATCH /v1/sessions/{id}/routing
PATCH /v1/sessions/{id}/provider-policy

GET /v1/sessions/{id}/metrics
GET /v1/sessions/{id}/diagnostics
POST /v1/sessions/{id}/calibrate
```

## 16.3 Streaming audio API

MVP does not need network audio streaming. Future API:

```text
/ws/audio/input
/ws/audio/output
```

But for low latency local streaming, prefer:

```text
shared memory ring
  + control socket
  + binary frame metadata
```

WebSocket is acceptable for remote experiments, not for lowest-latency local audio.

## 16.4 Plugin API

Conceptual trait:

```text
trait VoiceConversionPlugin {
    fn family_id(&self) -> &'static str;
    fn validate_model(&self, files: ModelFiles) -> ValidationReport;
    fn load(&self, ctx: PluginLoadContext) -> Result<Box<dyn VoicePipeline>>;
    fn supported_providers(&self) -> ProviderSupportMatrix;
    fn latency_requirements(&self) -> LatencySpec;
    fn default_params(&self) -> ParamSchema;
}
```

Pipeline:

```text
trait VoicePipeline {
    fn process_chunk(&mut self, chunk: AudioChunk, params: ParamSnapshot)
        -> Result<ConvertedChunk>;

    fn describe_stages(&self) -> Vec<StageDescriptor>;
    fn metrics(&self) -> PipelineMetrics;
}
```

## 16.5 Config schema

Use JSON schema or TOML/YAML with typed Rust structs.

Example:

```toml
[audio]
input_device = "default"
output_device = "default"
sample_rate = 48000
channels = 1
buffer_mode = "balanced"

[runtime]
chunk_ms = 40
crossfade_ms = 8
sola_search_ms = 12
latency_mode = "balanced"

[provider]
policy = "auto"
allow_cpu_fallback = true
prefer = ["CUDA", "DirectML", "CoreML", "OpenVINO", "CPU"]

[model]
slot = "main"
model_id = "sha256:..."
family = "rvc"

[diagnostics]
profiler = true
metrics_window_sec = 120
```

---

# 17. Plugin architecture

## 17.1 Boundary

A model plugin owns:

```text
model file validation
model-specific preprocessing
model-specific postprocessing
model runtime stages
model-specific params
latency requirements
quality/performance hints
```

Core runtime owns:

```text
audio devices
ring buffers
chunk scheduler
resampling framework
SOLA/crossfade primitives
provider manager
diagnostics
control API
config/presets
packaging
```

## 17.2 Required plugin declarations

Each plugin must declare:

```text
family_id
supported_model_versions
required_files
optional_files
required_dependencies
input_sample_rates
output_sample_rates
min_chunk_ms
recommended_chunk_ms
context_requirements
supported_providers_per_stage
live_params_schema
non_live_params_schema
metrics_schema
```

## 17.3 Plugin examples

### RVC

Stages:

```text
pitch
content
index
generator
```

### Beatrice

Could have different streaming state and model constraints.

### LLVC

Likely designed for low-latency streaming; may have smaller context and different scheduler contract.

### so-vits-svc

May share content/f0/generator concepts but not same manifest.

### DDSP-SVC

May have explicit DSP/vocoder split.

### Diffusion-SVC

Likely higher latency; plugin must honestly report non-realtime or high-latency requirements.

## 17.4 Plugin isolation

MVP:

```text
in-tree Rust plugins only
```

Post-MVP:

```text
native dynamic plugins for trusted plugins
process-isolated plugins for untrusted/experimental plugins
```

Do not load arbitrary third-party native plugins into realtime process without trust model. For sandboxing, process isolation is more realistic than trying to sandbox native dynamic libraries.

---

# 18. Model conversion and packaging

## 18.1 Import existing models

RVC import flow:

```text
User selects:
  model.pth
  config.json optional
  model.index optional

Importer:
  detects architecture
  validates metadata
  computes hashes
  checks dependency availability
  exports ONNX using converter tool
  runs test inference
  writes .vcrt bundle
```

## 18.2 PyTorch weights to ONNX

Conversion should be offline:

```text
tools/convert-rvc
  Python env
  PyTorch model load
  dummy/static inputs
  torch.onnx.export
  ONNX shape fixups
  ONNX checker
  ORT test inference
  optional fp16 conversion
```

Do not require Python for normal runtime.

## 18.3 TensorRT engine cache

Cache path:

```text
cache/tensorrt/
  {model_hash}/
    {gpu_id}_{trt_version}_{cuda_version}_{precision}_{shape}.engine
    build_report.json
```

Never download arbitrary `.engine` files by default. NVIDIA warns TensorRT engine files are executable artifacts and should only be deserialized when built by you or received over a trusted channel. ([NVIDIA Docs][15])

## 18.4 Model storage

Suggested paths:

```text
Windows:
  %APPDATA%\vc-runtime\models
  %LOCALAPPDATA%\vc-runtime\cache

macOS:
  ~/Library/Application Support/vc-runtime/models
  ~/Library/Caches/vc-runtime

Linux:
  $XDG_DATA_HOME/vc-runtime/models
  $XDG_CACHE_HOME/vc-runtime
```

## 18.5 Compatibility validation

Validation report:

```text
Model: ok
Generator ONNX: ok
Content extractor: missing
RMVPE: ok
Index: unsupported format, ignored
CUDA: compatible
DirectML: incompatible, opset/operator issue
CPU: compatible
Estimated latency: unknown until calibration
```

## 18.6 Dependency delivery

Dependencies like HuBERT/ContentVec/RMVPE must be managed as first-class assets:

```text
dependency_id
version
source_url
sha256
license
required_by
provider_compatibility
download_status
```

No silent downloads. UI must show model source and license.

---

# 19. Performance optimization plan

## 19.1 Algorithmic

Optimize first by avoiding unnecessary work:

* skip index retrieval under overload;
* reduce f0 extraction frequency if model quality tolerates it;
* silence fast-path: during silence, avoid full generator or use fade/hold;
* use model-compatible chunk sizes;
* reduce context/lookahead where quality allows;
* precompute static speaker embeddings;
* avoid re-running content extractor over overlapping context when cached features can be reused.

## 19.2 Memory copies

Targets:

```text
audio callback -> ring: one copy
ring -> chunk buffer: one copy or view
CPU -> GPU: one explicit copy per stage boundary unless I/O binding avoids it
GPU -> CPU: only when audio waveform needed for output
```

Use:

* preallocated buffers;
* buffer pools;
* pinned host memory where useful;
* ONNX Runtime I/O Binding for GPU paths;
* consistent `f32` internal format;
* avoid interleaved/planar conversions repeatedly.

## 19.3 CPU scheduling

* Audio callback: realtime/high priority.
* Model worker: high priority but not above audio callback.
* Control/UI: normal priority.
* Avoid oversubscribing CPU with ORT intra-op threads.
* Pinning/affinity only after measuring.

## 19.4 GPU provider

* Explicit device selection.
* Warmup before live.
* Static shapes.
* Avoid first-run compilation in live path.
* Track GPU memory.
* Use CUDA EP before TensorRT complexity.

## 19.5 Graph optimizations

Use ORT graph optimization levels and offline optimized model artifacts. ONNX Runtime documents graph optimizations as graph-level transformations from simple eliminations to node fusions/layout optimizations. ([ONNX Runtime][16])

Plan:

```text
export ONNX
run ORT checker
run graph optimization
benchmark optimized vs original
save optimized artifact if stable
```

## 19.6 FP16 / mixed precision

Use only after quality tests:

```text
1. fp32 golden output
2. fp16 candidate
3. compare perceptual metrics + manual listening
4. benchmark p95/p99
5. enable per provider/model
```

## 19.7 Quantization

INT8 is not MVP default.

Use when:

* model is too slow on CPU;
* calibration corpus exists;
* quality loss acceptable;
* provider supports fast INT8 for relevant ops.

## 19.8 TensorRT

Use when:

* ONNX model has stable static shapes;
* CUDA EP already works;
* TensorRT build is offline/cached;
* operator coverage good;
* p95 improves materially.

Do not use TensorRT as first debug target.

## 19.9 I/O binding / zero-copy

Goal:

```text
avoid hidden CPU↔GPU copies around ORT Run()
```

Measure:

```text
input_copy_time_ms
output_copy_time_ms
total_run_time_ms
```

Only keep complexity if p95 improves.

## 19.10 `torch.compile` fallback

Use for:

* dev experiments;
* comparing PyTorch vs ONNX;
* unsupported ONNX export path.

Not production mainline.

## 19.11 Triton kernels

Triton is appropriate after profiling shows a small tensor operation dominates latency and cannot be optimized via ONNX graph/provider. Not for cross-platform MVP because official compatibility is Linux-oriented and excludes macOS/Windows as first-class targets. ([GitHub][12])

Candidate hot spots:

* GPU-side interpolation;
* feature postprocessing;
* custom gather/index blending;
* small fused transforms around model I/O.

Non-candidates:

* audio callback;
* resampler/SOLA unless rewritten as GPU batch is proven useful;
* full model runtime replacement;
* provider abstraction.

---

# 20. Testing strategy

## 20.1 Unit tests

```text
ring buffer:
  wraparound
  overrun
  underrun
  timestamp continuity

jitter buffer:
  target depth
  drift handling
  discontinuity markers

resampler:
  rate conversion correctness
  impulse response
  latency accounting

SOLA/crossfade:
  phase alignment
  boundary click reduction
  silence behavior
```

## 20.2 Deterministic offline pipeline tests

Use recorded input and fixed model artifact:

```text
input.wav
config.json
expected_metrics.json
golden_output.wav or feature checksum
```

Expect numeric tolerance, not bit-perfect output across providers.

## 20.3 Golden audio snapshots

For each provider:

```text
CPU fp32 baseline
CUDA fp32
CUDA fp16
DirectML
CoreML
OpenVINO
```

Compare:

* loudness;
* spectral distance;
* F0 continuity;
* voiced/unvoiced agreement;
* clipping;
* perceptual metrics where useful;
* manual listening set.

## 20.4 Latency regression tests

Synthetic test:

```text
feed virtual input clock
run pipeline for N chunks
assert:
  p95 processing < threshold
  no queue growth
  underruns == 0
```

## 20.5 Stress tests

* CPU load in background.
* GPU load in background.
* device hotplug.
* sample rate change.
* model reload loop.
* start/stop 100 times.
* long run stability.

## 20.6 Provider fallback tests

Test:

```text
CUDA missing DLL
CUDA OOM
DirectML unsupported op
TensorRT build failure
CoreML unavailable
OpenVINO env missing
```

Expected:

```text
clear error
explicit fallback
no daemon crash
diagnostics explain cause
```

## 20.7 Cross-platform CI

CI can test:

* build;
* unit tests;
* CPU offline pipeline;
* config/schema;
* Web UI;
* packaging smoke.

Hardware CI/lab required for:

* CUDA;
* TensorRT;
* DirectML;
* CoreML;
* OpenVINO GPU/NPU;
* PipeWire/JACK audio.

## 20.8 Manual QA scenarios

Windows:

```text
Discord input through VB-CABLE
OBS monitor
game running under GPU load
WASAPI default device changes
```

macOS:

```text
BlackHole routing
mic permission reset
AirPods sample rate quirks
CoreAudio device hotplug
```

Linux:

```text
PipeWire graph routing
JACK low latency
PulseAudio fallback
rtkit unavailable
```

---

# 21. Benchmark suite

## 21.1 Harness goals

`vc-bench` is a separate executable, not hidden inside UI.

It should run:

```text
offline deterministic benchmarks
live audio loopback benchmarks
provider benchmarks
chunk-size sweeps
quality comparisons
baseline comparisons
```

## 21.2 Inputs

```text
synthetic:
  sine
  chirp
  impulses
  silence
  pink noise
  voiced-like harmonic signal

recorded:
  speech corpus
  male/female voices
  quiet/loud speech
  plosives
  laughter
  singing optional
```

## 21.3 Dimensions

```text
sample rates:
  44.1k
  48k
  96k optional

chunk sizes:
  20/40/80/120/160 ms

providers:
  CPU
  CUDA
  TensorRT
  DirectML
  CoreML
  OpenVINO

precision:
  fp32
  fp16
  int8 experimental
```

## 21.4 Outputs

```text
report.json
report.csv
latency_histogram.svg/png
stage_breakdown.csv
provider_report.json
audio_output.wav
```

Report schema:

```json
{
  "runtime_version": "0.1.0",
  "os": "...",
  "cpu": "...",
  "gpu": "...",
  "model": "...",
  "provider": "...",
  "chunk_ms": 40,
  "metrics": {
    "rtf_p50": 0.42,
    "rtf_p95": 0.71,
    "rtf_p99": 0.93,
    "latency_p95_ms": 68,
    "underruns": 0,
    "overruns": 0
  },
  "stages": {
    "pitch_p95_ms": 6.2,
    "feature_p95_ms": 8.1,
    "generator_p95_ms": 13.5
  }
}
```

## 21.5 Baseline comparison

Compare against w-okada/voice-changer where practical:

* same model;
* same f0 method;
* same device route;
* same chunk size;
* measure black-box end-to-end via loopback;
* report limitations.

Do not claim superiority without reproducible benchmark artifacts.

---

# 22. Security/privacy

## 22.1 Local-first

Default policy:

```text
no cloud inference
no audio upload
no telemetry
no crash upload
no model download without user action
```

## 22.2 Model download trust

Every downloadable dependency/model:

```text
source
license
sha256
size
publisher
signature if available
```

Warn on unknown model bundles.

## 22.3 Signed updates

* Signed update manifests.
* Signed binaries.
* Hash verification for provider packs.
* Rollback on failed update.

## 22.4 Plugin sandboxing

MVP:

```text
only bundled/in-tree plugins
```

Future:

```text
external plugin = separate process
restricted IPC
no direct access to mic unless granted through runtime
```

## 22.5 Crash reports

Opt-in only.

Crash reports include:

```text
stack trace
provider status
config
hardware summary
```

Not included by default:

```text
raw mic audio
converted audio
model files
personal paths where avoidable
```

## 22.6 Safe logging

Never log:

* raw PCM;
* transcribed speech;
* full model paths with usernames unless diagnostic export explicitly includes them;
* tokens.

Audio capture for bug reports must require explicit temporary enablement and visible indicator.

---

# 23. Packaging/release

## 23.1 Package types

Base packages:

```text
vc-runtime-cpu
  daemon
  CLI
  local web UI
  ORT CPU
  audio backend
```

Provider packs:

```text
vc-provider-nvidia-cuda
vc-provider-nvidia-tensorrt
vc-provider-windows-dml
vc-provider-intel-openvino
vc-provider-macos-coreml
```

Desktop app:

```text
vc-runtime-desktop
  Tauri shell
  sidecar daemon
  bundled web UI
```

## 23.2 Windows

Ship:

* installer `.exe`/NSIS;
* optional `.msi`;
* portable `.zip`;
* CPU base;
* NVIDIA CUDA pack;
* DirectML/WinML compatibility pack;
* OpenVINO pack.

Concerns:

* Visual C++ runtime;
* CUDA/cuDNN DLL search path;
* antivirus false positives;
* code signing;
* virtual cable not included.

## 23.3 Linux

Ship:

* AppImage;
* `.deb`;
* `.rpm`;
* Flatpak later;
* tarball for advanced users.

Concerns:

* PipeWire/JACK/PulseAudio deps;
* CUDA/cuDNN;
* OpenVINO;
* realtime priority permissions;
* distro compatibility.

## 23.4 macOS

Ship:

* `.dmg`;
* `.app`;
* universal binary if feasible;
* signed/notarized builds.

Concerns:

* microphone permission;
* hardened runtime;
* CoreML EP packaging;
* Apple Silicon vs Intel;
* virtual audio device handled externally.

## 23.5 Tauri packaging note

Tauri’s official repository documents built-in bundling for macOS `.app/.dmg`, Linux `.deb/.rpm/.AppImage`, and Windows `.exe`/NSIS and `.msi`/WiX, plus desktop updater support. ([GitHub][17])

But the backend should also be releasable without Tauri.

## 23.6 Update channels

```text
stable:
  conservative provider packs

beta:
  new providers and model families

nightly:
  benchmark and developer builds
```

Provider pack updates should be independent from UI updates.

---

# 24. Roadmap

## Phase 0: Research / profiling / baseline

**Goal**
Build factual baseline against existing tools and validate RVC pipeline assumptions.

**Deliverables**

* w-okada baseline notes.
* RVC model import experiments.
* ONNX export prototype.
* Audio backend spike with CPAL.
* Provider probing spike.
* Benchmark plan.

**Exit criteria**

* One RVC model can run offline through ONNX.
* Basic stage timings known.
* Target chunk sizes chosen.
* Top 5 risks documented.

**Risks**

* ONNX export fragile.
* RVC variants inconsistent.
* Provider behavior differs by platform.

---

## Phase 1: Audio passthrough runtime

**Goal**
Stable cross-platform realtime audio without ML.

**Deliverables**

* CPAL input/output.
* Ring buffers.
* Jitter buffer.
* Device list.
* Basic profiler.
* CLI passthrough.
* Local web UI device screen.

**Exit criteria**

* 30-minute passthrough with no underruns on test machines.
* Queue depth visible.
* Hotplug failure is graceful.
* Sample rate mismatch handled.

**Risks**

* CPAL backend quirks.
* Linux realtime permissions.
* macOS permissions.
* WASAPI buffer variability.

---

## Phase 2: RVC ONNX MVP

**Goal**
First end-to-end RVC conversion through ONNX.

**Deliverables**

* RVC bundle manifest.
* ONNX generator session.
* RMVPE/content ONNX path where available.
* CPU fallback.
* SOLA/crossfade.
* Basic live params.
* Offline benchmark.

**Exit criteria**

* RVC works from mic to output.
* Per-stage timings visible.
* No hidden Python in production path.
* CPU fallback emits correct audio, even if high latency.

**Risks**

* Content extractor export.
* f0 bottleneck.
* Generator shape issues.
* Audio artifacts at chunk boundaries.

---

## Phase 3: Provider manager + diagnostics

**Goal**
Make provider selection transparent and benchmark-driven.

**Deliverables**

* Provider probe.
* CUDA/CPU production path.
* DirectML experimental path.
* CoreML/OpenVINO probes.
* Provider explanation UI.
* p50/p95/p99 dashboard.
* Diagnostic bundle export.

**Exit criteria**

* User can see actual stage bottleneck.
* Requested vs selected provider visible.
* Fallback produces clear message.
* Calibration recommends chunk/provider.

**Risks**

* ORT provider introspection incomplete.
* GPU metrics API differs by vendor.
* DirectML/CoreML/OpenVINO edge cases.

---

## Phase 4: Tauri app

**Goal**
Production desktop shell around stable daemon.

**Deliverables**

* Tauri app.
* Sidecar daemon.
* Installer packages.
* Routing assistant.
* Logs/errors UI.
* Basic updater channel.

**Exit criteria**

* One-click app starts runtime.
* UI disconnect does not kill audio unexpectedly.
* Packaging works on Windows/macOS/Linux.
* Signed/notarized where required.

**Risks**

* Packaging GPU dependencies.
* WebView/platform quirks.
* macOS signing/notarization.

---

## Phase 5: Plugin model families

**Goal**
Generalize beyond RVC.

**Deliverables**

* Stable plugin interface.
* Second model family prototype.
* Model manifest schema v1.
* Plugin validation tests.

**Exit criteria**

* RVC no longer hardcoded in core.
* New model family can be added without touching audio engine.
* Plugin latency requirements visible.

**Risks**

* Plugin ABI instability.
* Too much abstraction too early.
* Model family differences larger than expected.

---

## Phase 6: Advanced accelerators

**Goal**
Optimize after measured bottlenecks.

**Deliverables**

* TensorRT engine cache.
* fp16 policy.
* quantization experiments.
* I/O binding.
* Triton hot spot experiments.
* provider-specific benchmark reports.

**Exit criteria**

* At least one accelerator gives reproducible p95 improvement.
* Quality regression tests pass.
* Fallback remains stable.

**Risks**

* TensorRT build complexity.
* fp16 quality loss.
* Triton portability.
* INT8 artifacts.

---

## Phase 7: SDK/ecosystem

**Goal**
Make runtime usable by other apps/tools.

**Deliverables**

* Rust SDK.
* C ABI.
* Python client for control API.
* Plugin examples.
* Benchmark corpus.
* Documentation.

**Exit criteria**

* External app can embed/control runtime.
* Headless mode stable.
* Plugin author guide exists.

**Risks**

* API support burden.
* Backward compatibility.
* Security of external plugins.

---

# 25. Risks and hard problems

## 25.1 Cross-platform audio is hard

Each OS has different assumptions about devices, buffers, permissions and routing. CPAL reduces but does not eliminate this complexity.

Mitigation:

```text
start with passthrough
measure actual callbacks
keep platform-specific escape hatches
```

## 25.2 Virtual devices are OS-specific

Creating a virtual microphone is a driver-level problem on Windows/macOS and a graph/routing problem on Linux.

Mitigation:

```text
MVP uses existing virtual cable tools
build routing assistant
defer custom drivers
```

## 25.3 ONNX export may be fragile

RVC forks and model variants may differ.

Mitigation:

```text
support explicit known model formats
conversion test harness
manifest schema
do not promise arbitrary .pth support
```

## 25.4 Model quality vs speed tradeoff

Faster f0/content/generator settings may reduce similarity/naturalness.

Mitigation:

```text
separate latency presets from quality presets
show expected impact
benchmark quality and speed together
```

## 25.5 GPU provider compatibility

CUDA, DirectML, CoreML, OpenVINO and TensorRT all have different operator coverage and packaging requirements.

Mitigation:

```text
provider packs
per-stage validation
explicit fallback
benchmark before selection
```

## 25.6 Apple Silicon limitations

CoreML can be fast, but conversion/operator/static-shape constraints can be painful.

Mitigation:

```text
macOS CPU baseline first
CoreML experimental
Apple Silicon benchmark lab
```

## 25.7 AMD Linux situation

ROCm support varies by GPU/distro/driver. This is risky for one-click MVP.

Mitigation:

```text
do not make ROCm mainline in MVP
document experimental status
support CPU fallback
```

## 25.8 TensorRT packaging complexity

TensorRT can improve latency but adds engine build, cache, operator and trust issues.

Mitigation:

```text
optional accelerator
offline engine build
cache validation
CUDA EP remains mainline
```

## 25.9 Realtime thread scheduling

OS scheduling jitter can break audio even when model is fast.

Mitigation:

```text
audio callback minimal
RT priority where available
visible callback p95/p99
safe buffer mode
```

## 25.10 Users expect one-click setup

Provider packs and virtual cable setup can be confusing.

Mitigation:

```text
CPU base always works
routing assistant
provider diagnostics
clear install messages
```

## 25.11 Benchmarking voice quality objectively

Latency is easy to measure; perceived voice quality is not.

Mitigation:

```text
golden audio
perceptual metrics
manual listening panels
artifact labels
no unsupported quality claims
```

---

# 26. Open questions

1. Exact first supported RVC model format:

   * `.pth + config`?
   * `.onnx` only?
   * custom `.vcrt` bundle only?

2. Daemon first or in-process library first:

   * recommendation: daemon first, library internally.

3. CPAL only or early platform-specific backend:

   * recommendation: CPAL first, platform escape hatches later.

4. WebSocket vs shared memory vs native IPC:

   * recommendation: HTTP/WS for control/metrics; shared memory later for audio streaming.

5. Minimum hardware target:

   * CPU-only acceptable with high latency?
   * minimum NVIDIA GPU generation?
   * Apple Silicon baseline?

6. Acceptable latency target:

   * balanced target 70–120 ms?
   * aggressive target 40–70 ms?
   * per-model family target?

7. Supported OS versions:

   * Windows 10/11?
   * macOS 12+ or 10.15+?
   * Linux distro baseline?

8. Licensing:

   * runtime license;
   * bundled model dependencies;
   * RVC-related licenses;
   * provider dependency licenses.

9. Training support later:

   * separate project?
   * plugin tools?
   * never in core runtime?

10. DirectML vs WinML future:

* keep DirectML compatibility pack?
* investigate WinML provider/device policy APIs for Windows builds.

11. CoreML delivery:

* ORT CoreML EP in bundled macOS C/C++ build?
* direct CoreML conversion for some stages?

12. Index support:

* support FAISS `.index` directly?
* convert to internal index?
* skip in MVP?

13. GPU metrics:

* NVML for NVIDIA;
* Windows Performance Counters/DXGI;
* Metal/CoreML visibility;
* AMD GPU metrics availability.

---

# 27. Recommended MVP

Если делать прагматично, первый сильный MVP должен быть таким:

```text
vc-runtime MVP
  Rust daemon
  CPAL audio passthrough
  realtime-safe ring buffers
  deterministic chunk scheduler
  streaming resampler
  SOLA/crossfade
  RVC ONNX path
  ONNX Runtime provider manager
  CPU fallback
  CUDA first-class path
  DirectML experimental Windows compatibility path
  local web UI
  live profiler
  benchmark harness
```

## 27.1 Platform scope

Recommended platform split:

```text
Audio passthrough:
  Windows + Linux + macOS from Phase 1

RVC production path:
  Windows + NVIDIA
  Linux + NVIDIA
  CPU fallback on all three

Experimental:
  Windows DirectML
  macOS CoreML
  Intel OpenVINO
```

Do **not** try to make Windows + NVIDIA + AMD + Intel + macOS Apple Silicon + Linux AMD all equally production-grade in the first RVC release. That will dilute the core runtime work.

## 27.2 MVP feature set

### Must have

```text
Audio:
  input/output device selection
  passthrough mode
  queue depth metrics
  underrun/overrun metrics

Runtime:
  chunk scheduler
  resampler
  SOLA/crossfade
  no allocation in callbacks
  graceful start/stop

Model:
  RVC ONNX bundle
  generator ONNX
  pitch/content ONNX where available
  CPU fallback
  CUDA provider

Diagnostics:
  per-stage p50/p95/p99
  provider per stage
  realtime factor
  accumulated delay
  diagnostic export

UI:
  local web UI
  device graph
  model loader
  profiler dashboard
  provider diagnostics

Bench:
  offline benchmark
  chunk/provider sweep
  JSON/CSV reports
```

### Should have

```text
DirectML experimental
OpenVINO probe
CoreML probe
basic auto-tuning
routing assistant
model dependency manager
```

### Not in MVP

```text
custom virtual audio driver
training
all model families
TensorRT by default
Triton kernels
network streaming
mobile
plugin marketplace
```

## 27.3 MVP exit criteria

A credible MVP is not “it converts voice once”. It is:

```text
1. 30-minute passthrough stable on Windows/Linux/macOS.
2. RVC ONNX works end-to-end on at least Windows NVIDIA and Linux NVIDIA.
3. CPU fallback works and clearly reports expected latency limitations.
4. Profiler explains bottleneck for every test run.
5. Provider manager never silently falls back without telling user.
6. Benchmark harness can reproduce latency numbers.
7. Audio callback has no model inference, no blocking locks, no heap allocation.
8. User can export diagnostics without sharing voice audio.
```

## 27.4 Strategic recommendation

Build the project around this sentence:

> `vc-runtime` is a realtime audio system that happens to run voice conversion models, not a model demo that happens to capture audio.

That framing leads to the right decisions:

* Rust for audio/data-plane stability;
* ONNX Runtime for production inference portability;
* PyTorch for conversion/reference, not the core realtime loop;
* TensorRT/Triton only after profiling;
* diagnostics as a first-class feature;
* UI as a client of a robust daemon;
* RVC first, plugin architecture later;
* provider truth visible to the user.

[1]: https://onnxruntime.ai/docs/execution-providers/ "Execution Providers | onnxruntime"
[2]: https://github.com/w-okada/voice-changer/blob/master/README_en.md "voice-changer/README_en.md at master · w-okada/voice-changer · GitHub"
[3]: https://github.com/RustAudio/cpal "GitHub - RustAudio/cpal: Cross-platform audio I/O library in pure Rust · GitHub"
[4]: https://onnxruntime.ai/docs/performance/tune-performance/iobinding.html?utm_source=chatgpt.com "I/O Binding"
[5]: https://github.com/RVC-Project/Retrieval-based-Voice-Conversion-WebUI "GitHub - RVC-Project/Retrieval-based-Voice-Conversion-WebUI: Easily train a good VC model with voice data <= 10 mins! · GitHub"
[6]: https://onnxruntime.ai/docs/execution-providers/CoreML-ExecutionProvider.html "Apple - CoreML | onnxruntime"
[7]: https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html "NVIDIA - CUDA | onnxruntime"
[8]: https://onnxruntime.ai/docs/execution-providers/TensorRT-ExecutionProvider.html "NVIDIA - TensorRT | onnxruntime"
[9]: https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html "Windows - DirectML | onnxruntime"
[10]: https://onnxruntime.ai/docs/execution-providers/OpenVINO-ExecutionProvider.html "Intel - OpenVINO™ | onnxruntime"
[11]: https://onnxruntime.ai/docs/execution-providers/Xnnpack-ExecutionProvider.html?utm_source=chatgpt.com "XNNPACK Execution Provider - onnxruntime"
[12]: https://github.com/triton-lang/triton "GitHub - triton-lang/triton: Development repository for the Triton language and compiler · GitHub"
[13]: https://onnxruntime.ai/docs/performance/model-optimizations/float16.html?utm_source=chatgpt.com "Create Float16 and Mixed Precision Models"
[14]: https://v2.tauri.app/ "Tauri 2.0 | Tauri"
[15]: https://docs.nvidia.com/deeplearning/tensorrt/latest/getting-started/quick-start-guide.html "https://docs.nvidia.com/deeplearning/tensorrt/latest/getting-started/quick-start-guide.html"
[16]: https://onnxruntime.ai/docs/performance/model-optimizations/graph-optimizations.html?utm_source=chatgpt.com "Graph Optimizations in ONNX Runtime"
[17]: https://github.com/tauri-apps/tauri "GitHub - tauri-apps/tauri: Build smaller, faster, and more secure desktop and mobile applications with a web frontend. · GitHub"
