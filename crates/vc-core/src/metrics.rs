use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct AudioCounters {
    input_callbacks: AtomicU64,
    output_callbacks: AtomicU64,
    pushed_frames: AtomicU64,
    popped_frames: AtomicU64,
    underrun_events: AtomicU64,
    overrun_events: AtomicU64,
}

impl AudioCounters {
    /// Returns an approximate non-transactional metrics snapshot.
    ///
    /// Each field is loaded independently with relaxed ordering. The result is
    /// suitable for realtime health reporting, not for deriving strict
    /// cross-field invariants.
    #[must_use]
    pub fn snapshot(&self) -> AudioMetricsSnapshot {
        AudioMetricsSnapshot {
            input_callbacks: self.input_callbacks.load(Ordering::Relaxed),
            output_callbacks: self.output_callbacks.load(Ordering::Relaxed),
            pushed_frames: self.pushed_frames.load(Ordering::Relaxed),
            popped_frames: self.popped_frames.load(Ordering::Relaxed),
            underrun_events: self.underrun_events.load(Ordering::Relaxed),
            overrun_events: self.overrun_events.load(Ordering::Relaxed),
        }
    }

    pub fn record_input_callback(&self) {
        self.input_callbacks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_output_callback(&self) {
        self.output_callbacks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_pushed_frames(&self, frames: u64) {
        self.pushed_frames.fetch_add(frames, Ordering::Relaxed);
    }

    pub fn record_popped_frames(&self, frames: u64) {
        self.popped_frames.fetch_add(frames, Ordering::Relaxed);
    }

    pub fn record_underrun_event(&self) {
        self.underrun_events.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_overrun_event(&self) {
        self.overrun_events.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct AudioMetricsSnapshot {
    pub input_callbacks: u64,
    pub output_callbacks: u64,
    pub pushed_frames: u64,
    pub popped_frames: u64,
    pub underrun_events: u64,
    pub overrun_events: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_starts_at_zero() {
        let counters = AudioCounters::default();

        let snapshot = counters.snapshot();

        assert_eq!(snapshot.input_callbacks, 0);
        assert_eq!(snapshot.output_callbacks, 0);
        assert_eq!(snapshot.pushed_frames, 0);
        assert_eq!(snapshot.popped_frames, 0);
        assert_eq!(snapshot.underrun_events, 0);
        assert_eq!(snapshot.overrun_events, 0);
    }

    #[test]
    fn snapshot_reports_recorded_counts() {
        let counters = AudioCounters::default();

        counters.record_input_callback();
        counters.record_input_callback();
        counters.record_output_callback();
        counters.record_pushed_frames(128);
        counters.record_popped_frames(96);
        counters.record_underrun_event();
        counters.record_overrun_event();
        counters.record_overrun_event();

        let snapshot = counters.snapshot();

        assert_eq!(snapshot.input_callbacks, 2);
        assert_eq!(snapshot.output_callbacks, 1);
        assert_eq!(snapshot.pushed_frames, 128);
        assert_eq!(snapshot.popped_frames, 96);
        assert_eq!(snapshot.underrun_events, 1);
        assert_eq!(snapshot.overrun_events, 2);
    }
}
