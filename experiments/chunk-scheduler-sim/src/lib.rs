use std::cmp::min;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Policy {
    DropOldest,
    SilenceOnUnderrun,
    ReuseLast,
}

impl Policy {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DropOldest => "drop-oldest",
            Self::SilenceOnUnderrun => "silence-on-underrun",
            Self::ReuseLast => "reuse-last",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub duration_ms: u64,
    pub chunk_ms: u64,
    pub hop_ms: u64,
    pub worker_pattern_ms: Vec<u64>,
    pub worker_ms: u64,
    pub worker_ms_jitter: u64,
    pub worker_ms_seed: u64,
    pub queue_capacity: usize,
    pub policy: Policy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStatus {
    Delivered,
    UnderrunDropped,
    UnderrunSilence,
    UnderrunReused,
}

impl OutputStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Delivered => "delivered",
            Self::UnderrunDropped => "underrun-dropped",
            Self::UnderrunSilence => "underrun-silence",
            Self::UnderrunReused => "underrun-reused-last",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OutputEvent {
    pub chunk_index: usize,
    pub scheduled_time_ms: u64,
    pub status: OutputStatus,
    pub worker_done_time_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct SimulationSummary {
    pub accumulated_delay_ms: u64,
    pub deadline_miss_events: u64,
    pub underrun_events: u64,
    pub dropped_chunks: u64,
    pub max_queue_depth: usize,
    pub policy: Policy,
    pub num_chunks: usize,
    pub events: Vec<OutputEvent>,
}

impl SimulationSummary {
    pub fn to_summary_json(&self) -> String {
        format!(
            "{{\"type\":\"summary\",\"accumulated_delay_ms\":{},\"deadline_miss_events\":{},\"underrun_events\":{},\"dropped_chunks\":{},\"max_queue_depth\":{},\"policy\":\"{}\",\"num_chunks\":{}}}",
            self.accumulated_delay_ms,
            self.deadline_miss_events,
            self.underrun_events,
            self.dropped_chunks,
            self.max_queue_depth,
            self.policy.as_str(),
            self.num_chunks
        )
    }

    pub fn event_to_json(event: &OutputEvent) -> String {
        let done = event
            .worker_done_time_ms
            .map_or_else(|| "null".to_string(), |value| value.to_string());
        format!(
            "{{\"type\":\"event\",\"chunk_index\":{},\"scheduled_time_ms\":{},\"status\":\"{}\",\"worker_done_time_ms\":{}}}",
            event.chunk_index,
            event.scheduled_time_ms,
            event.status.as_str(),
            done
        )
    }
}

struct PendingChunk {
    index: usize,
}

struct DoneChunk {
    index: usize,
    done_time_ms: u64,
}

#[derive(Debug, Clone, Copy)]
struct WorkerState {
    idx: usize,
    done_at_ms: u64,
}

fn deterministic_offset(seed: u64, chunk_index: usize, jitter: u64) -> i64 {
    if jitter == 0 {
        return 0;
    }

    let mut x = seed ^ (chunk_index as u64).wrapping_mul(0x9E3779B97F4A7C15);
    x ^= x >> 12;
    x ^= x << 25;
    x ^= x >> 27;
    x = x.wrapping_mul(0x2545F4914F6CDD1D);
    let span = jitter.saturating_mul(2).saturating_add(1);
    ((x % span) as i64) - (jitter as i64)
}

fn due_time_ms(cfg: &Config, chunk_index: usize) -> u64 {
    cfg.chunk_ms + (chunk_index as u64).saturating_mul(cfg.hop_ms)
}

fn worker_time_ms(cfg: &Config, chunk_index: usize) -> u64 {
    let base = if cfg.worker_pattern_ms.is_empty() {
        cfg.worker_ms
    } else {
        let idx = chunk_index % cfg.worker_pattern_ms.len();
        cfg.worker_pattern_ms[idx]
    };

    let adjusted =
        (base as i64) + deterministic_offset(cfg.worker_ms_seed, chunk_index, cfg.worker_ms_jitter);
    adjusted.max(1) as u64
}

fn drop_stale_completed(completed: &mut VecDeque<DoneChunk>, next_output_index: usize) -> u64 {
    let mut dropped = 0u64;
    while completed
        .front()
        .is_some_and(|done| done.index < next_output_index)
    {
        completed.pop_front();
        dropped = dropped.saturating_add(1);
    }
    dropped
}

fn drop_pending_through(pending: &mut VecDeque<PendingChunk>, chunk_index: usize) -> u64 {
    let mut dropped = 0u64;
    while pending
        .front()
        .is_some_and(|chunk| chunk.index <= chunk_index)
    {
        pending.pop_front();
        dropped = dropped.saturating_add(1);
    }
    dropped
}

fn cancel_worker_through(worker: &mut Option<WorkerState>, chunk_index: usize) -> u64 {
    if worker.as_ref().is_some_and(|work| work.idx <= chunk_index) {
        *worker = None;
        1
    } else {
        0
    }
}

pub fn run_simulation(cfg: &Config) -> SimulationSummary {
    assert!(cfg.duration_ms > 0, "duration_ms must be > 0");
    assert!(cfg.chunk_ms > 0, "chunk_ms must be > 0");
    assert!(cfg.hop_ms > 0, "hop_ms must be > 0");
    assert!(cfg.queue_capacity > 0, "queue_capacity must be > 0");

    let num_chunks = ((cfg.duration_ms.saturating_sub(1)) / cfg.hop_ms) as usize + 1;

    let mut pending: VecDeque<PendingChunk> = VecDeque::new();
    let mut completed: VecDeque<DoneChunk> = VecDeque::new();
    let mut worker: Option<WorkerState> = None;
    let mut next_input_index: usize = 0;
    let mut next_output_index: usize = 0;

    let mut max_queue_depth = 0usize;
    let mut accumulated_delay_ms = 0u64;
    let mut deadline_miss_events = 0u64;
    let mut underrun_events = 0u64;
    let mut dropped_chunks = 0u64;
    let mut events = Vec::with_capacity(num_chunks);
    let mut last_delivered: Option<usize> = None;
    let mut next_output_time = due_time_ms(cfg, 0);

    while next_output_index < num_chunks
        || next_input_index < num_chunks
        || worker.is_some()
        || !pending.is_empty()
        || !completed.is_empty()
    {
        let next_arrival = if next_input_index < num_chunks {
            cfg.hop_ms.saturating_mul(next_input_index as u64)
        } else {
            u64::MAX
        };
        let next_output = if next_output_index < num_chunks {
            next_output_time
        } else {
            u64::MAX
        };
        let next_worker_done = worker.as_ref().map_or(u64::MAX, |state| state.done_at_ms);
        let current_time = min(next_arrival, min(next_worker_done, next_output));

        while next_input_index < num_chunks {
            let arrival_time = cfg.hop_ms.saturating_mul(next_input_index as u64);
            if arrival_time > current_time {
                break;
            }

            if pending.len() >= cfg.queue_capacity {
                pending.pop_front();
                dropped_chunks = dropped_chunks.saturating_add(1);
            }
            pending.push_back(PendingChunk {
                index: next_input_index,
            });
            if pending.len() > max_queue_depth {
                max_queue_depth = pending.len();
            }
            next_input_index += 1;
        }

        if let Some(work) = worker {
            if work.done_at_ms <= current_time {
                worker = None;
                let due = due_time_ms(cfg, work.idx);
                let lateness = work.done_at_ms.saturating_sub(due);
                accumulated_delay_ms = accumulated_delay_ms.saturating_add(lateness);
                if work.idx >= next_output_index {
                    completed.push_back(DoneChunk {
                        index: work.idx,
                        done_time_ms: work.done_at_ms,
                    });
                } else {
                    dropped_chunks = dropped_chunks.saturating_add(1);
                }
            }
        }

        if worker.is_none() {
            if let Some(chunk) = pending.pop_front() {
                let done_at = current_time.saturating_add(worker_time_ms(cfg, chunk.index));
                worker = Some(WorkerState {
                    idx: chunk.index,
                    done_at_ms: done_at,
                });
            }
        }

        if current_time == next_output && next_output_index < num_chunks {
            let scheduled_time_ms = next_output;
            let chunk_index = next_output_index;
            dropped_chunks =
                dropped_chunks.saturating_add(drop_stale_completed(&mut completed, chunk_index));

            if let Some(done) = completed.front() {
                if done.index == chunk_index {
                    let done = completed.pop_front().expect("completed chunk available");
                    last_delivered = Some(chunk_index);
                    events.push(OutputEvent {
                        chunk_index,
                        scheduled_time_ms,
                        status: OutputStatus::Delivered,
                        worker_done_time_ms: Some(done.done_time_ms),
                    });
                } else {
                    deadline_miss_events = deadline_miss_events.saturating_add(1);
                    let status = policy_to_status(cfg.policy, last_delivered.is_some());
                    if status != OutputStatus::UnderrunReused {
                        underrun_events = underrun_events.saturating_add(1);
                    }
                    if cfg.policy == Policy::DropOldest {
                        dropped_chunks = dropped_chunks
                            .saturating_add(drop_pending_through(&mut pending, chunk_index));
                        dropped_chunks = dropped_chunks
                            .saturating_add(cancel_worker_through(&mut worker, chunk_index));
                    }
                    events.push(OutputEvent {
                        chunk_index,
                        scheduled_time_ms,
                        status,
                        worker_done_time_ms: None,
                    });
                }
            } else {
                deadline_miss_events = deadline_miss_events.saturating_add(1);
                let status = policy_to_status(cfg.policy, last_delivered.is_some());
                if status != OutputStatus::UnderrunReused {
                    underrun_events = underrun_events.saturating_add(1);
                }
                if cfg.policy == Policy::DropOldest {
                    dropped_chunks = dropped_chunks
                        .saturating_add(drop_pending_through(&mut pending, chunk_index));
                    dropped_chunks = dropped_chunks
                        .saturating_add(cancel_worker_through(&mut worker, chunk_index));
                }
                events.push(OutputEvent {
                    chunk_index,
                    scheduled_time_ms,
                    status,
                    worker_done_time_ms: None,
                });
            }

            next_output_index = next_output_index.saturating_add(1);
            next_output_time = next_output.saturating_add(cfg.hop_ms);
        }

        if current_time == u64::MAX {
            break;
        }
    }

    SimulationSummary {
        accumulated_delay_ms,
        deadline_miss_events,
        underrun_events,
        dropped_chunks,
        max_queue_depth,
        policy: cfg.policy,
        num_chunks,
        events,
    }
}

fn policy_to_status(policy: Policy, has_last: bool) -> OutputStatus {
    match policy {
        Policy::DropOldest => OutputStatus::UnderrunDropped,
        Policy::SilenceOnUnderrun => OutputStatus::UnderrunSilence,
        Policy::ReuseLast => {
            if has_last {
                OutputStatus::UnderrunReused
            } else {
                OutputStatus::UnderrunSilence
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sim_config(
        duration_ms: u64,
        chunk_ms: u64,
        hop_ms: u64,
        worker_ms: u64,
        queue_capacity: usize,
        policy: Policy,
        worker_pattern_ms: Vec<u64>,
    ) -> Config {
        Config {
            duration_ms,
            chunk_ms,
            hop_ms,
            worker_pattern_ms,
            worker_ms,
            worker_ms_jitter: 0,
            worker_ms_seed: 0,
            queue_capacity,
            policy,
        }
    }

    #[test]
    fn keeps_up_without_misses_with_fast_worker() {
        let cfg = sim_config(200, 40, 50, 30, 4, Policy::DropOldest, vec![]);
        let summary = run_simulation(&cfg);

        assert_eq!(summary.accumulated_delay_ms, 0);
        assert_eq!(summary.deadline_miss_events, 0);
        assert_eq!(summary.underrun_events, 0);
        assert_eq!(summary.dropped_chunks, 0);
        assert_eq!(summary.max_queue_depth, 1);
        assert_eq!(summary.events.len(), 4);
        assert!(
            summary
                .events
                .iter()
                .all(|event| event.status == OutputStatus::Delivered)
        );
    }

    #[test]
    fn slow_model_generates_deadline_misses_and_drops_for_drop_oldest() {
        let cfg = sim_config(400, 100, 100, 300, 10, Policy::DropOldest, vec![]);
        let summary = run_simulation(&cfg);

        assert_eq!(summary.deadline_miss_events, 4);
        assert_eq!(summary.underrun_events, 4);
        assert_eq!(summary.dropped_chunks, 4);
        assert_eq!(summary.accumulated_delay_ms, 0);
        assert_eq!(summary.max_queue_depth, 2);
        assert_eq!(
            summary
                .events
                .iter()
                .map(|event| event.status)
                .collect::<Vec<_>>(),
            vec![
                OutputStatus::UnderrunDropped,
                OutputStatus::UnderrunDropped,
                OutputStatus::UnderrunDropped,
                OutputStatus::UnderrunDropped
            ]
        );
    }

    #[test]
    fn reuse_last_policy_reuses_last_sampled_chunk() {
        let cfg = sim_config(400, 100, 100, 0, 10, Policy::ReuseLast, vec![50, 180]);
        let summary = run_simulation(&cfg);

        assert_eq!(summary.deadline_miss_events, 3);
        assert_eq!(summary.underrun_events, 0);
        assert_eq!(summary.dropped_chunks, 3);
        assert_eq!(summary.accumulated_delay_ms, 220);
        assert_eq!(summary.max_queue_depth, 1);
        assert_eq!(
            summary
                .events
                .iter()
                .map(|event| event.status)
                .collect::<Vec<_>>(),
            vec![
                OutputStatus::Delivered,
                OutputStatus::UnderrunReused,
                OutputStatus::UnderrunReused,
                OutputStatus::UnderrunReused
            ]
        );
    }

    #[test]
    fn drop_oldest_policy_sheds_stale_work_instead_of_preserving_backlog() {
        let drop_cfg = sim_config(800, 100, 100, 260, 10, Policy::DropOldest, vec![]);
        let silence_cfg = sim_config(800, 100, 100, 260, 10, Policy::SilenceOnUnderrun, vec![]);

        let drop_summary = run_simulation(&drop_cfg);
        let silence_summary = run_simulation(&silence_cfg);

        assert!(
            drop_summary.accumulated_delay_ms < silence_summary.accumulated_delay_ms,
            "drop-oldest should cancel stale work instead of letting lateness accumulate"
        );
        assert!(
            drop_summary.max_queue_depth < silence_summary.max_queue_depth,
            "drop-oldest should keep less stale pending work than silence"
        );
    }

    #[test]
    fn reuse_last_policy_distinguishes_deadline_misses_from_output_underruns() {
        let cfg = sim_config(400, 100, 100, 0, 10, Policy::ReuseLast, vec![50, 180]);
        let summary = run_simulation(&cfg);

        assert!(
            summary.deadline_miss_events > summary.underrun_events,
            "reuse-last should record compute misses without counting reused slots as silence underruns"
        );
    }
}
