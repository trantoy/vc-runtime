use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, ValueEnum};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Stage {
    Rms,
}

impl Stage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Rms => "rms",
        }
    }
}

#[derive(Clone, Debug)]
pub struct BenchConfig {
    pub input_path: PathBuf,
    pub chunk_ms: u32,
    pub hop_ms: u32,
    pub stage: Stage,
}

#[derive(Clone, Debug, Serialize)]
pub struct BenchReport {
    pub schema_version: u32,
    pub input_path: String,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub input_frames: u64,
    pub duration_ms: u64,
    pub chunk_ms: u32,
    pub hop_ms: u32,
    pub chunk_frames: u64,
    pub hop_frames: u64,
    pub chunk_count: u64,
    pub stage: String,
    pub total_processing_ms: f64,
    pub realtime_factor: f64,
    pub chunk_processing_p50_us: u64,
    pub chunk_processing_p95_us: u64,
    pub chunk_processing_p99_us: u64,
    pub deadline_miss_events: u64,
    pub accumulated_delay_ms: f64,
    pub checksum: u64,
}

#[derive(Debug)]
struct WavInput {
    sample_rate_hz: u32,
    channels: u16,
    input_frames: usize,
    mono_samples: Vec<f32>,
}

#[derive(Debug, Parser)]
#[command(
    name = "offline-audio-bench",
    about = "Prototype prerecorded-audio benchmark"
)]
struct Cli {
    /// Input WAV file. The prototype currently supports 16-bit PCM WAV.
    #[arg(long)]
    input: PathBuf,

    /// Chunk size in milliseconds.
    #[arg(long, default_value_t = 100)]
    chunk_ms: u32,

    /// Hop size in milliseconds.
    #[arg(long, default_value_t = 50)]
    hop_ms: u32,

    /// Prototype processing stage.
    #[arg(long, value_enum, default_value_t = Stage::Rms)]
    stage: Stage,

    /// Write JSON report to this path instead of stdout.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Pretty-print JSON output.
    #[arg(long, default_value_t = false)]
    pretty: bool,
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    let report = run_benchmark(&BenchConfig {
        input_path: cli.input,
        chunk_ms: cli.chunk_ms,
        hop_ms: cli.hop_ms,
        stage: cli.stage,
    })?;
    let json = if cli.pretty {
        serde_json::to_string_pretty(&report)?
    } else {
        serde_json::to_string(&report)?
    };

    if let Some(output) = cli.output {
        fs::write(output, json).context("failed to write benchmark report")?;
    } else {
        println!("{json}");
    }

    Ok(())
}

pub fn run_benchmark(config: &BenchConfig) -> Result<BenchReport> {
    ensure!(config.chunk_ms > 0, "chunk-ms must be > 0");
    ensure!(config.hop_ms > 0, "hop-ms must be > 0");

    let input = read_wav_input(&config.input_path)?;
    let chunk_frames = frames_for_ms(input.sample_rate_hz, config.chunk_ms)?;
    let hop_frames = frames_for_ms(input.sample_rate_hz, config.hop_ms)?;
    ensure!(
        input.input_frames >= chunk_frames,
        "input WAV is shorter than one chunk"
    );

    let chunk_count = ((input.input_frames - chunk_frames) / hop_frames) + 1;
    let mut chunk_processing_ns = Vec::with_capacity(chunk_count);
    let mut checksum = 0xcbf2_9ce4_8422_2325_u64;

    for index in 0..chunk_count {
        let start = index * hop_frames;
        let end = start + chunk_frames;
        let chunk = &input.mono_samples[start..end];
        let started = Instant::now();
        let stage_value = process_stage(config.stage, chunk);
        let elapsed_ns = started.elapsed().as_nanos();
        chunk_processing_ns.push(elapsed_ns);
        checksum = update_checksum(checksum, stage_value);
    }

    let total_processing_ns: u128 = chunk_processing_ns.iter().copied().sum();
    let deadline_ns = u128::from(config.hop_ms) * 1_000_000;
    let deadline_miss_events = chunk_processing_ns
        .iter()
        .filter(|elapsed| **elapsed > deadline_ns)
        .count() as u64;
    let accumulated_delay_ms = chunk_processing_ns
        .iter()
        .map(|elapsed| elapsed.saturating_sub(deadline_ns) as f64 / 1_000_000.0)
        .sum();
    let duration_ms = ((input.input_frames as u128 * 1_000) / u128::from(input.sample_rate_hz))
        .try_into()
        .context("duration does not fit in u64")?;
    let total_processing_ms = total_processing_ns as f64 / 1_000_000.0;
    let realtime_factor = total_processing_ms / duration_ms as f64;
    let mut sorted = chunk_processing_ns;
    sorted.sort_unstable();

    Ok(BenchReport {
        schema_version: 1,
        input_path: config.input_path.display().to_string(),
        sample_rate_hz: input.sample_rate_hz,
        channels: input.channels,
        input_frames: input.input_frames as u64,
        duration_ms,
        chunk_ms: config.chunk_ms,
        hop_ms: config.hop_ms,
        chunk_frames: chunk_frames as u64,
        hop_frames: hop_frames as u64,
        chunk_count: chunk_count as u64,
        stage: config.stage.as_str().to_owned(),
        total_processing_ms,
        realtime_factor,
        chunk_processing_p50_us: percentile_us(&sorted, 0.50),
        chunk_processing_p95_us: percentile_us(&sorted, 0.95),
        chunk_processing_p99_us: percentile_us(&sorted, 0.99),
        deadline_miss_events,
        accumulated_delay_ms,
        checksum,
    })
}

fn read_wav_input(path: &Path) -> Result<WavInput> {
    let mut reader = hound::WavReader::open(path).context("failed to open WAV input")?;
    let spec = reader.spec();
    ensure!(spec.channels > 0, "WAV channel count must be > 0");
    ensure!(
        spec.sample_format == hound::SampleFormat::Int && spec.bits_per_sample == 16,
        "only 16-bit PCM WAV is supported by this prototype"
    );

    let channels = usize::from(spec.channels);
    let raw_samples = reader
        .samples::<i16>()
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("failed to read WAV samples")?;
    ensure!(
        raw_samples.len() % channels == 0,
        "WAV sample count is not divisible by channel count"
    );

    let input_frames = raw_samples.len() / channels;
    let mut mono_samples = Vec::with_capacity(input_frames);
    for frame in raw_samples.chunks_exact(channels) {
        let sum: f32 = frame
            .iter()
            .map(|sample| f32::from(*sample) / f32::from(i16::MAX))
            .sum();
        mono_samples.push(sum / channels as f32);
    }

    Ok(WavInput {
        sample_rate_hz: spec.sample_rate,
        channels: spec.channels,
        input_frames,
        mono_samples,
    })
}

fn frames_for_ms(sample_rate_hz: u32, ms: u32) -> Result<usize> {
    let frames = (u64::from(sample_rate_hz) * u64::from(ms)) / 1_000;
    if frames == 0 {
        bail!("chunk/hop duration is too small for input sample rate")
    }
    frames
        .try_into()
        .context("frame count does not fit in usize")
}

fn process_stage(stage: Stage, chunk: &[f32]) -> f32 {
    match stage {
        Stage::Rms => {
            let square_sum: f32 = chunk.iter().map(|sample| sample * sample).sum();
            (square_sum / chunk.len() as f32).sqrt()
        }
    }
}

fn update_checksum(current: u64, value: f32) -> u64 {
    let mut checksum = current;
    for byte in value.to_bits().to_le_bytes() {
        checksum ^= u64::from(byte);
        checksum = checksum.wrapping_mul(0x100_0000_01b3);
    }
    checksum
}

fn percentile_us(sorted_ns: &[u128], quantile: f64) -> u64 {
    if sorted_ns.is_empty() {
        return 0;
    }
    let max_index = sorted_ns.len() - 1;
    let index = (max_index as f64 * quantile).ceil() as usize;
    (sorted_ns[index] / 1_000).try_into().unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{SampleFormat, WavSpec, WavWriter};
    use tempfile::tempdir;

    fn write_test_wav(path: &std::path::Path) {
        let spec = WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(path, spec).expect("create wav");
        for i in 0..16_000 {
            let sample = (((i % 128) as i16) - 64) * 128;
            writer.write_sample(sample).expect("write sample");
        }
        writer.finalize().expect("finalize wav");
    }

    #[test]
    fn reports_deterministic_chunking_for_prerecorded_wav() {
        let dir = tempdir().expect("tempdir");
        let input = dir.path().join("speech.wav");
        write_test_wav(&input);

        let report = run_benchmark(&BenchConfig {
            input_path: input,
            chunk_ms: 100,
            hop_ms: 50,
            stage: Stage::Rms,
        })
        .expect("benchmark");

        assert_eq!(report.schema_version, 1);
        assert_eq!(report.sample_rate_hz, 16_000);
        assert_eq!(report.channels, 1);
        assert_eq!(report.input_frames, 16_000);
        assert_eq!(report.duration_ms, 1_000);
        assert_eq!(report.chunk_count, 19);
        assert_eq!(report.deadline_miss_events, 0);
        assert_ne!(report.checksum, 0);
    }

    #[test]
    fn rejects_unsupported_chunk_shape() {
        let dir = tempdir().expect("tempdir");
        let input = dir.path().join("speech.wav");
        write_test_wav(&input);

        let error = run_benchmark(&BenchConfig {
            input_path: input,
            chunk_ms: 0,
            hop_ms: 50,
            stage: Stage::Rms,
        })
        .unwrap_err()
        .to_string();

        assert_eq!(error, "chunk-ms must be > 0");
    }
}
