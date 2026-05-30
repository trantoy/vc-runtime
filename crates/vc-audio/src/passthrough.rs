use anyhow::{Context, Result, anyhow, bail, ensure};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, Sample, SampleFormat, SizedSample, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};
use std::sync::Arc;
use vc_core::metrics::{AudioCounters, AudioMetricsSnapshot};

pub const DEFAULT_CAPACITY_FRAMES: usize = 48_000;
pub const MAX_CAPACITY_FRAMES: usize = 960_000;
pub const MAX_CAPACITY_SAMPLES: usize = MAX_CAPACITY_FRAMES * 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PassthroughConfig {
    pub input_index: Option<usize>,
    pub output_index: Option<usize>,
    pub capacity_frames: usize,
}

impl Default for PassthroughConfig {
    fn default() -> Self {
        Self {
            input_index: None,
            output_index: None,
            capacity_frames: DEFAULT_CAPACITY_FRAMES,
        }
    }
}

impl PassthroughConfig {
    pub fn validate(self) -> Result<Self> {
        ensure!(
            self.capacity_frames > 0,
            "capacity-frames must be at least 1"
        );
        ensure!(
            self.capacity_frames <= MAX_CAPACITY_FRAMES,
            "capacity-frames must be at most {MAX_CAPACITY_FRAMES}"
        );
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PassthroughStreamInfo {
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub capacity_frames: usize,
    pub input_device_name: String,
    pub output_device_name: String,
}

pub struct PassthroughSession {
    _input_stream: cpal::Stream,
    _output_stream: cpal::Stream,
    counters: Arc<AudioCounters>,
    stream_info: PassthroughStreamInfo,
}

impl PassthroughSession {
    pub fn start(config: PassthroughConfig) -> Result<Self> {
        let config = config.validate()?;
        let host = cpal::default_host();
        let input_device = select_input_device(&host, config.input_index)?;
        let output_device = select_output_device(&host, config.output_index)?;
        let input_device_name = device_name(&input_device, "input");
        let output_device_name = device_name(&output_device, "output");
        let input_config = input_device
            .default_input_config()
            .context("failed to read default input stream config")?;
        let output_config = output_device
            .default_output_config()
            .context("failed to read default output stream config")?;
        let stream_shape = validate_stream_shapes(
            StreamShape::from_supported_config(&input_config),
            StreamShape::from_supported_config(&output_config),
        )?;
        let channels = usize::from(stream_shape.channels);
        let capacity_samples = capacity_samples(config.capacity_frames, channels)?;
        let (producer, consumer) = RingBuffer::<f32>::new(capacity_samples);
        let counters = Arc::new(AudioCounters::default());
        let input_stream = build_input_stream(
            &input_device,
            input_config.sample_format(),
            &input_config.config(),
            producer,
            Arc::clone(&counters),
            channels,
        )?;
        let output_stream = build_output_stream(
            &output_device,
            output_config.sample_format(),
            &output_config.config(),
            consumer,
            Arc::clone(&counters),
            channels,
        )?;

        output_stream
            .play()
            .context("failed to start output stream")?;
        input_stream
            .play()
            .context("failed to start input stream")?;

        Ok(Self {
            _input_stream: input_stream,
            _output_stream: output_stream,
            counters,
            stream_info: PassthroughStreamInfo {
                sample_rate_hz: stream_shape.sample_rate,
                channels: stream_shape.channels,
                capacity_frames: config.capacity_frames,
                input_device_name,
                output_device_name,
            },
        })
    }

    #[must_use]
    pub fn metrics(&self) -> AudioMetricsSnapshot {
        self.counters.snapshot()
    }

    #[must_use]
    pub fn stream_info(&self) -> &PassthroughStreamInfo {
        &self.stream_info
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StreamShape {
    sample_rate: u32,
    channels: u16,
}

impl StreamShape {
    fn from_supported_config(config: &cpal::SupportedStreamConfig) -> Self {
        Self {
            sample_rate: config.sample_rate().0,
            channels: config.channels(),
        }
    }
}

fn validate_stream_shapes(input: StreamShape, output: StreamShape) -> Result<StreamShape> {
    ensure!(input.channels > 0, "input channel count must be at least 1");
    ensure!(
        output.channels > 0,
        "output channel count must be at least 1"
    );
    ensure!(
        input.sample_rate == output.sample_rate,
        "input sample rate {} Hz does not match output sample rate {} Hz",
        input.sample_rate,
        output.sample_rate
    );
    ensure!(
        input.channels == output.channels,
        "input channel count {} does not match output channel count {}",
        input.channels,
        output.channels
    );
    Ok(input)
}

fn capacity_samples(capacity_frames: usize, channels: usize) -> Result<usize> {
    let capacity_samples = capacity_frames.checked_mul(channels).ok_or_else(|| {
        anyhow!("capacity-frames {capacity_frames} is too large for {channels} channels")
    })?;
    ensure!(
        capacity_samples <= MAX_CAPACITY_SAMPLES,
        "capacity requires {capacity_samples} samples but Phase 0.1 allows at most {MAX_CAPACITY_SAMPLES}"
    );
    Ok(capacity_samples)
}

fn select_input_device(host: &cpal::Host, index: Option<usize>) -> Result<cpal::Device> {
    match index {
        Some(index) => host
            .input_devices()
            .context("failed to enumerate input devices")?
            .nth(index)
            .ok_or_else(|| anyhow!("input device index {index} was not found")),
        None => host
            .default_input_device()
            .ok_or_else(|| anyhow!("no default input device available")),
    }
}

fn select_output_device(host: &cpal::Host, index: Option<usize>) -> Result<cpal::Device> {
    match index {
        Some(index) => host
            .output_devices()
            .context("failed to enumerate output devices")?
            .nth(index)
            .ok_or_else(|| anyhow!("output device index {index} was not found")),
        None => host
            .default_output_device()
            .ok_or_else(|| anyhow!("no default output device available")),
    }
}

fn device_name(device: &cpal::Device, direction: &str) -> String {
    device
        .name()
        .unwrap_or_else(|error| format!("<unknown {direction} device: {error}>"))
}

fn build_input_stream(
    device: &cpal::Device,
    sample_format: SampleFormat,
    config: &StreamConfig,
    producer: Producer<f32>,
    counters: Arc<AudioCounters>,
    channels: usize,
) -> Result<cpal::Stream> {
    match sample_format {
        SampleFormat::I8 => {
            build_input_stream_for::<i8>(device, config, producer, counters, channels)
        }
        SampleFormat::I16 => {
            build_input_stream_for::<i16>(device, config, producer, counters, channels)
        }
        SampleFormat::I24 => {
            build_input_stream_for::<cpal::I24>(device, config, producer, counters, channels)
        }
        SampleFormat::I32 => {
            build_input_stream_for::<i32>(device, config, producer, counters, channels)
        }
        SampleFormat::I64 => {
            build_input_stream_for::<i64>(device, config, producer, counters, channels)
        }
        SampleFormat::U8 => {
            build_input_stream_for::<u8>(device, config, producer, counters, channels)
        }
        SampleFormat::U16 => {
            build_input_stream_for::<u16>(device, config, producer, counters, channels)
        }
        SampleFormat::U32 => {
            build_input_stream_for::<u32>(device, config, producer, counters, channels)
        }
        SampleFormat::U64 => {
            build_input_stream_for::<u64>(device, config, producer, counters, channels)
        }
        SampleFormat::F32 => {
            build_input_stream_for::<f32>(device, config, producer, counters, channels)
        }
        SampleFormat::F64 => {
            build_input_stream_for::<f64>(device, config, producer, counters, channels)
        }
        _ => bail!("unsupported input sample format: {sample_format}"),
    }
}

fn build_input_stream_for<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    mut producer: Producer<f32>,
    counters: Arc<AudioCounters>,
    channels: usize,
) -> Result<cpal::Stream>
where
    T: SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let error_counters = Arc::clone(&counters);
    device
        .build_input_stream::<T, _, _>(
            config,
            move |input, _| {
                push_input_samples_typed(input, channels, &mut producer, counters.as_ref());
            },
            move |_| {
                error_counters.record_input_stream_error_event();
            },
            None,
        )
        .context("failed to build input stream")
}

fn build_output_stream(
    device: &cpal::Device,
    sample_format: SampleFormat,
    config: &StreamConfig,
    consumer: Consumer<f32>,
    counters: Arc<AudioCounters>,
    channels: usize,
) -> Result<cpal::Stream> {
    match sample_format {
        SampleFormat::I8 => {
            build_output_stream_for::<i8>(device, config, consumer, counters, channels)
        }
        SampleFormat::I16 => {
            build_output_stream_for::<i16>(device, config, consumer, counters, channels)
        }
        SampleFormat::I24 => {
            build_output_stream_for::<cpal::I24>(device, config, consumer, counters, channels)
        }
        SampleFormat::I32 => {
            build_output_stream_for::<i32>(device, config, consumer, counters, channels)
        }
        SampleFormat::I64 => {
            build_output_stream_for::<i64>(device, config, consumer, counters, channels)
        }
        SampleFormat::U8 => {
            build_output_stream_for::<u8>(device, config, consumer, counters, channels)
        }
        SampleFormat::U16 => {
            build_output_stream_for::<u16>(device, config, consumer, counters, channels)
        }
        SampleFormat::U32 => {
            build_output_stream_for::<u32>(device, config, consumer, counters, channels)
        }
        SampleFormat::U64 => {
            build_output_stream_for::<u64>(device, config, consumer, counters, channels)
        }
        SampleFormat::F32 => {
            build_output_stream_for::<f32>(device, config, consumer, counters, channels)
        }
        SampleFormat::F64 => {
            build_output_stream_for::<f64>(device, config, consumer, counters, channels)
        }
        _ => bail!("unsupported output sample format: {sample_format}"),
    }
}

fn build_output_stream_for<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    mut consumer: Consumer<f32>,
    counters: Arc<AudioCounters>,
    channels: usize,
) -> Result<cpal::Stream>
where
    T: SizedSample + FromSample<f32> + Send + 'static,
{
    let error_counters = Arc::clone(&counters);
    device
        .build_output_stream::<T, _, _>(
            config,
            move |output, _| {
                fill_output_samples_typed(output, channels, &mut consumer, counters.as_ref());
            },
            move |_| {
                error_counters.record_output_stream_error_event();
            },
            None,
        )
        .context("failed to build output stream")
}

fn push_input_samples_typed<T>(
    samples: &[T],
    channels: usize,
    producer: &mut Producer<f32>,
    counters: &AudioCounters,
) where
    T: Sample,
    f32: FromSample<T>,
{
    counters.record_input_callback();
    let mut pushed_frames = 0_u64;
    let mut overrun = false;

    for frame in samples.chunks_exact(channels) {
        if producer.slots() < channels {
            overrun = true;
            break;
        }

        let mut pushed_whole_frame = true;
        for sample in frame {
            if producer.push((*sample).to_sample::<f32>()).is_err() {
                pushed_whole_frame = false;
                overrun = true;
                break;
            }
        }

        if pushed_whole_frame {
            pushed_frames += 1;
        } else {
            break;
        }
    }

    if pushed_frames > 0 {
        counters.record_pushed_frames(pushed_frames);
    }
    if overrun {
        counters.record_overrun_event();
    }
}

fn fill_output_samples_typed<T>(
    output: &mut [T],
    channels: usize,
    consumer: &mut Consumer<f32>,
    counters: &AudioCounters,
) where
    T: Sample + FromSample<f32>,
{
    counters.record_output_callback();
    let mut popped_frames = 0_u64;
    let mut underrun = false;

    for frame in output.chunks_exact_mut(channels) {
        if consumer.slots() < channels {
            underrun = true;
            for sample in frame {
                *sample = T::EQUILIBRIUM;
            }
            continue;
        }

        let mut popped_whole_frame = true;
        for sample in frame {
            match consumer.pop() {
                Ok(value) => {
                    *sample = T::from_sample(value);
                }
                Err(_) => {
                    *sample = T::EQUILIBRIUM;
                    popped_whole_frame = false;
                    underrun = true;
                }
            }
        }

        if popped_whole_frame {
            popped_frames += 1;
        }
    }

    if popped_frames > 0 {
        counters.record_popped_frames(popped_frames);
    }
    if underrun {
        counters.record_underrun_event();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rtrb::RingBuffer;
    use vc_core::metrics::AudioCounters;

    #[test]
    fn rejects_zero_capacity_frames() {
        let config = PassthroughConfig {
            capacity_frames: 0,
            ..PassthroughConfig::default()
        };

        let error = config.validate().unwrap_err().to_string();

        assert_eq!(error, "capacity-frames must be at least 1");
    }

    #[test]
    fn rejects_capacity_frames_above_phase_limit() {
        let config = PassthroughConfig {
            capacity_frames: MAX_CAPACITY_FRAMES + 1,
            ..PassthroughConfig::default()
        };

        let error = config.validate().unwrap_err().to_string();

        assert_eq!(
            error,
            format!("capacity-frames must be at most {MAX_CAPACITY_FRAMES}")
        );
    }

    #[test]
    fn rejects_total_capacity_samples_above_phase_limit() {
        let error = capacity_samples(MAX_CAPACITY_FRAMES, 9)
            .unwrap_err()
            .to_string();

        assert_eq!(
            error,
            format!(
                "capacity requires {} samples but Phase 0.1 allows at most {MAX_CAPACITY_SAMPLES}",
                MAX_CAPACITY_FRAMES * 9
            )
        );
    }

    #[test]
    fn rejects_mismatched_stream_rates() {
        let error = validate_stream_shapes(
            StreamShape {
                sample_rate: 48_000,
                channels: 2,
            },
            StreamShape {
                sample_rate: 44_100,
                channels: 2,
            },
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            error,
            "input sample rate 48000 Hz does not match output sample rate 44100 Hz"
        );
    }

    #[test]
    fn rejects_mismatched_channel_counts() {
        let error = validate_stream_shapes(
            StreamShape {
                sample_rate: 48_000,
                channels: 1,
            },
            StreamShape {
                sample_rate: 48_000,
                channels: 2,
            },
        )
        .unwrap_err()
        .to_string();

        assert_eq!(
            error,
            "input channel count 1 does not match output channel count 2"
        );
    }

    #[test]
    fn input_callback_pushes_complete_frames_and_records_metrics() {
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(4);
        let counters = AudioCounters::default();

        push_input_samples_typed(&[0.1, 0.2, 0.3, 0.4], 2, &mut producer, &counters);

        assert_eq!(consumer.pop().unwrap(), 0.1);
        assert_eq!(consumer.pop().unwrap(), 0.2);
        assert_eq!(consumer.pop().unwrap(), 0.3);
        assert_eq!(consumer.pop().unwrap(), 0.4);
        assert_eq!(
            counters.snapshot(),
            vc_core::metrics::AudioMetricsSnapshot {
                input_callbacks: 1,
                pushed_frames: 2,
                ..Default::default()
            }
        );
    }

    #[test]
    fn input_callback_records_one_overrun_event_per_callback() {
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(2);
        let counters = AudioCounters::default();

        push_input_samples_typed(&[0.1, 0.2, 0.3, 0.4], 2, &mut producer, &counters);

        assert_eq!(consumer.pop().unwrap(), 0.1);
        assert_eq!(consumer.pop().unwrap(), 0.2);
        assert_eq!(
            counters.snapshot(),
            vc_core::metrics::AudioMetricsSnapshot {
                input_callbacks: 1,
                pushed_frames: 1,
                overrun_events: 1,
                ..Default::default()
            }
        );
    }

    #[test]
    fn output_callback_pops_frames_and_records_metrics() {
        let (mut producer, mut consumer) = RingBuffer::<f32>::new(4);
        producer.push(0.1).unwrap();
        producer.push(0.2).unwrap();
        producer.push(0.3).unwrap();
        producer.push(0.4).unwrap();
        let counters = AudioCounters::default();
        let mut output = [0.0_f32; 4];

        fill_output_samples_typed(&mut output, 2, &mut consumer, &counters);

        assert_eq!(output, [0.1, 0.2, 0.3, 0.4]);
        assert_eq!(
            counters.snapshot(),
            vc_core::metrics::AudioMetricsSnapshot {
                output_callbacks: 1,
                popped_frames: 2,
                ..Default::default()
            }
        );
    }

    #[test]
    fn output_callback_writes_silence_and_records_one_underrun_event_per_callback() {
        let (_producer, mut consumer) = RingBuffer::<f32>::new(2);
        let counters = AudioCounters::default();
        let mut output = [1.0_f32; 4];

        fill_output_samples_typed(&mut output, 2, &mut consumer, &counters);

        assert_eq!(output, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(
            counters.snapshot(),
            vc_core::metrics::AudioMetricsSnapshot {
                output_callbacks: 1,
                underrun_events: 1,
                ..Default::default()
            }
        );
    }
}
