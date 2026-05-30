#![forbid(unsafe_code)]
//! Command-line interface library for `vc-runtime`.

use anyhow::Result;
use clap::{Parser, Subcommand};
use vc_audio::devices::{self, DeviceReport, DeviceSummary};

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
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::ListDevices => {
            let report = devices::list_devices();
            print!("{}", format_device_report(&report));
            Ok(())
        }
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
}
