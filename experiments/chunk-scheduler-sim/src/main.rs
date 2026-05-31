use clap::{Parser, ValueEnum};

use chunk_scheduler_sim::{Config, Policy, SimulationSummary, run_simulation};

#[derive(Parser)]
#[command(
    name = "chunk-scheduler-sim",
    version,
    about = "Prototype simulator for chunk scheduling behavior under slow workers"
)]
struct Cli {
    /// Total simulated input duration in ms.
    #[arg(long, default_value_t = 1000)]
    duration_ms: u64,

    /// Nominal chunk duration in ms.
    #[arg(long, default_value_t = 100)]
    chunk_ms: u64,

    /// Chunk hop in ms.
    #[arg(long, default_value_t = 50)]
    hop_ms: u64,

    /// Constant worker time for all chunks if no pattern is provided.
    #[arg(long, default_value_t = 80)]
    worker_ms: u64,

    /// Worker-time pattern in ms, comma-separated. Repeats over chunks.
    #[arg(long)]
    worker_ms_pattern: Option<String>,

    /// Extra deterministic jitter (absolute) around worker_ms/pattern values.
    #[arg(long, default_value_t = 0)]
    worker_ms_jitter: u64,

    /// Seed for deterministic jitter sequence.
    #[arg(long, default_value_t = 0xA5A5_5A5A_1234_5678)]
    worker_ms_seed: u64,

    /// Max number of queued pending chunks.
    #[arg(long, default_value_t = 6)]
    queue_capacity: usize,

    /// Drop policy for deadline misses.
    #[arg(long, value_enum, default_value_t = CliPolicy::DropOldest)]
    policy: CliPolicy,

    /// Print one JSON line per output event.
    #[arg(long, default_value_t = false)]
    trace: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum CliPolicy {
    #[value(name = "drop-oldest")]
    DropOldest,
    #[value(name = "silence-on-underrun")]
    SilenceOnUnderrun,
    #[value(name = "reuse-last")]
    ReuseLast,
}

impl From<CliPolicy> for Policy {
    fn from(value: CliPolicy) -> Self {
        match value {
            CliPolicy::DropOldest => Policy::DropOldest,
            CliPolicy::SilenceOnUnderrun => Policy::SilenceOnUnderrun,
            CliPolicy::ReuseLast => Policy::ReuseLast,
        }
    }
}

fn parse_pattern(pattern: Option<String>) -> Vec<u64> {
    let pattern = pattern.unwrap_or_default();
    if pattern.trim().is_empty() {
        return Vec::new();
    }

    pattern
        .split(',')
        .map(|value| {
            let value = value.trim();
            value.parse::<u64>().unwrap_or_else(|_| {
                panic!("cannot parse worker-ms-pattern element '{value}' as u64")
            })
        })
        .collect()
}

fn main() {
    let cli = Cli::parse();

    validate_input(&cli);
    let worker_pattern_ms = parse_pattern(cli.worker_ms_pattern.clone());

    let cfg = Config {
        duration_ms: cli.duration_ms,
        chunk_ms: cli.chunk_ms,
        hop_ms: cli.hop_ms,
        worker_pattern_ms,
        worker_ms: cli.worker_ms,
        worker_ms_jitter: cli.worker_ms_jitter,
        worker_ms_seed: cli.worker_ms_seed,
        queue_capacity: cli.queue_capacity,
        policy: Policy::from(cli.policy),
    };

    let summary: SimulationSummary = run_simulation(&cfg);
    if cli.trace {
        for event in &summary.events {
            println!("{}", SimulationSummary::event_to_json(event));
        }
    }
    println!("{}", summary.to_summary_json());
}

fn validate_input(cli: &Cli) {
    if cli.duration_ms == 0 {
        panic!("duration-ms must be > 0");
    }
    if cli.chunk_ms == 0 {
        panic!("chunk-ms must be > 0");
    }
    if cli.hop_ms == 0 {
        panic!("hop-ms must be > 0");
    }
    if cli.queue_capacity == 0 {
        panic!("queue-capacity must be > 0");
    }

    if let Some(pattern) = cli.worker_ms_pattern.as_ref() {
        if pattern
            .split(',')
            .any(|value| value.trim().parse::<u64>().unwrap_or(0) == 0 && !value.trim().is_empty())
        {
            panic!("worker-ms-pattern values must all be > 0 (or omitted)");
        }
    }
}
