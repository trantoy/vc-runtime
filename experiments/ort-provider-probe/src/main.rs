//! CLI entrypoint for the ORT provider probe prototype.

#![forbid(unsafe_code)]

use clap::Parser;
use std::path::PathBuf;

use ort_provider_probe::{ProbeRequest, run_probe};

#[derive(Debug, Parser)]
#[command(
    name = "ort-provider-probe",
    about = "Prototype for probing ONNX Runtime provider assignment semantics."
)]
struct Cli {
    /// Provider requested by the caller.
    #[arg(long, default_value = "CPUExecutionProvider")]
    requested_provider: String,

    /// Optional fixture path to simulate provider availability.
    /// Without a fixture, the probe runs in dry-run mode.
    #[arg(long)]
    fixture: Option<PathBuf>,

    /// Pretty-print JSON output.
    #[arg(long)]
    pretty: bool,
}

fn main() {
    let args = Cli::parse();
    let request = ProbeRequest {
        requested_provider: args.requested_provider,
        fixture: args.fixture,
    };

    let report = run_probe(&request);
    let payload = if args.pretty {
        report.to_pretty_json().unwrap_or_else(|error| {
            panic!("failed to serialize probe report to JSON: {error}");
        })
    } else {
        serde_json::to_string(&report).unwrap_or_else(|error| {
            panic!("failed to serialize probe report to JSON: {error}");
        })
    };

    println!("{}", payload);
}
