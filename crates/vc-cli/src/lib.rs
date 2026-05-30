#![forbid(unsafe_code)]
//! Command-line interface library for `vc-runtime`.

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::{thread, time::Duration};
use vc_audio::devices::{self, DeviceReport, DeviceSummary};
use vc_audio::passthrough::{
    DEFAULT_CAPACITY_FRAMES, PassthroughConfig, PassthroughSession, PassthroughStreamInfo,
};
use vc_core::metrics::AudioMetricsSnapshot;

#[derive(Debug, Parser)]
#[command(name = "vc")]
#[command(about = "vc-runtime command-line tools")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List input and output audio devices.
    ListDevices,
    /// Run default input to output passthrough.
    Passthrough(PassthroughArgs),
}

#[derive(Debug, Args)]
struct PassthroughArgs {
    /// Run duration in seconds.
    #[arg(long, default_value_t = 10)]
    seconds: u64,

    /// Process-local input device index from list-devices.
    #[arg(
        long,
        help = "Current input enumeration index; may change between runs"
    )]
    input_index: Option<usize>,

    /// Process-local output device index from list-devices.
    #[arg(
        long,
        help = "Current output enumeration index; may change between runs"
    )]
    output_index: Option<usize>,

    /// Ring-buffer capacity measured in audio frames.
    #[arg(long, default_value_t = DEFAULT_CAPACITY_FRAMES)]
    capacity_frames: usize,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::ListDevices => {
            let report = devices::list_devices();
            print!("{}", format_device_report(&report));
            Ok(())
        }
        Command::Passthrough(args) => run_passthrough(args),
    }
}

#[must_use]
pub fn format_device_report(report: &DeviceReport) -> String {
    let mut output = String::new();
    push_device_section(&mut output, "Input devices:", &report.inputs);
    push_device_section(&mut output, "Output devices:", &report.outputs);
    push_warning_section(&mut output, &report.warnings);
    output
}

fn push_device_section(output: &mut String, title: &str, devices: &[DeviceSummary]) {
    output.push_str(title);
    output.push('\n');

    if devices.is_empty() {
        output.push_str("  (none found)\n");
    } else {
        for device in devices {
            output.push_str(&format!("  [{}] {}\n", device.index, device.name));
        }
    }

    output.push('\n');
}

fn push_warning_section(output: &mut String, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }

    output.push_str("Warnings:");
    output.push('\n');

    for warning in warnings {
        output.push_str("  ");
        output.push_str(warning);
        output.push('\n');
    }
}

fn run_passthrough(args: PassthroughArgs) -> Result<()> {
    let session = PassthroughSession::start(PassthroughConfig {
        input_index: args.input_index,
        output_index: args.output_index,
        capacity_frames: args.capacity_frames,
    })?;
    println!("{}", format_passthrough_started(session.stream_info()));

    for elapsed_seconds in 1..=args.seconds {
        thread::sleep(Duration::from_secs(1));
        println!(
            "{}",
            format_passthrough_metrics(elapsed_seconds, session.metrics())
        );
    }

    Ok(())
}

#[must_use]
pub fn format_passthrough_started(stream_info: &PassthroughStreamInfo) -> String {
    format!(
        "Passthrough started: input_device={:?} output_device={:?} sample_rate_hz={} channels={} capacity_frames={}",
        stream_info.input_device_name,
        stream_info.output_device_name,
        stream_info.sample_rate_hz,
        stream_info.channels,
        stream_info.capacity_frames
    )
}

#[must_use]
pub fn format_passthrough_metrics(elapsed_seconds: u64, snapshot: AudioMetricsSnapshot) -> String {
    format!(
        "t={}s input_cb={} output_cb={} pushed_frames={} popped_frames={} underrun_events={} overrun_events={} input_stream_error_events={} output_stream_error_events={}",
        elapsed_seconds,
        snapshot.input_callbacks,
        snapshot.output_callbacks,
        snapshot.pushed_frames,
        snapshot.popped_frames,
        snapshot.underrun_events,
        snapshot.overrun_events,
        snapshot.input_stream_error_events,
        snapshot.output_stream_error_events
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_input_and_output_device_sections() {
        let devices = DeviceReport {
            inputs: vec![DeviceSummary {
                index: 0,
                name: "Mic".to_owned(),
            }],
            outputs: vec![DeviceSummary {
                index: 1,
                name: "Speakers".to_owned(),
            }],
            warnings: Vec::new(),
        };

        let output = format_device_report(&devices);

        assert!(output.contains("Input devices:"));
        assert!(output.contains("  [0] Mic"));
        assert!(output.contains("Output devices:"));
        assert!(output.contains("  [1] Speakers"));
    }

    #[test]
    fn formats_empty_device_sections() {
        let devices = DeviceReport {
            inputs: Vec::new(),
            outputs: Vec::new(),
            warnings: Vec::new(),
        };

        let output = format_device_report(&devices);

        assert!(output.contains("Input devices:"));
        assert!(output.contains("  (none found)"));
        assert!(output.contains("Output devices:"));
    }

    #[test]
    fn formats_listing_warnings() {
        let devices = DeviceReport {
            inputs: Vec::new(),
            outputs: Vec::new(),
            warnings: vec!["failed to list input devices: unavailable".to_owned()],
        };

        let output = format_device_report(&devices);

        assert!(output.contains("Warnings:"));
        assert!(output.contains("  failed to list input devices: unavailable"));
    }

    #[test]
    fn formats_passthrough_metrics_line() {
        let output = format_passthrough_metrics(
            3,
            vc_core::metrics::AudioMetricsSnapshot {
                input_callbacks: 10,
                output_callbacks: 11,
                pushed_frames: 480,
                popped_frames: 448,
                underrun_events: 2,
                overrun_events: 1,
                input_stream_error_events: 4,
                output_stream_error_events: 5,
            },
        );

        assert_eq!(
            output,
            "t=3s input_cb=10 output_cb=11 pushed_frames=480 popped_frames=448 underrun_events=2 overrun_events=1 input_stream_error_events=4 output_stream_error_events=5"
        );
    }

    #[test]
    fn formats_passthrough_started_line_with_selected_devices() {
        let output = format_passthrough_started(&vc_audio::passthrough::PassthroughStreamInfo {
            sample_rate_hz: 48_000,
            channels: 2,
            capacity_frames: 48_000,
            input_device_name: "Mic".to_owned(),
            output_device_name: "Speakers".to_owned(),
        });

        assert_eq!(
            output,
            "Passthrough started: input_device=\"Mic\" output_device=\"Speakers\" sample_rate_hz=48000 channels=2 capacity_frames=48000"
        );
    }
}
