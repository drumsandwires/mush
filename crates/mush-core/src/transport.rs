/// Implemented by any type that occupies a position on the
/// sample-accurate timeline.
///
/// Used by `TransportState::seek` to recalculate the event
/// cursor without knowing the concrete event type.
pub trait HasSamplePos {
    fn sample_pos(&self) -> u64;
}

/// Non-real-time project metadata.
/// Written once at load time, read for display only.
/// Never accessed from the audio callback.
#[derive(Debug, Default, Clone)]
pub struct ProjectMeta {
    pub title: String,
    pub artist: String,
    pub bpm: f64,
    pub time_sig_num: u8,
    pub time_sig_denom: u8,
}

use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

/// Shared, lock-free transport state.
///
/// Wrapped in `Arc` and cloned into every component that needs it.
/// Fields read by the audio callback use atomics — no locking,
/// no allocation, no blocking on the RT path.
/// `meta` uses `RwLock` but is only written before playback begins.
pub struct TransportState {
    /// Current playhead position in samples from project start.
    /// Written by the audio callback on every buffer.
    /// Read by the EngineMonitor for display.
    pub playhead: Arc<AtomicU64>,

    /// Whether the transport is currently running.
    pub is_playing: Arc<AtomicBool>,

    /// Tempo stored as BPM × 1000 to avoid floating-point atomics.
    /// 120.0 BPM is stored as 120_000u64.
    pub bpm_milli: Arc<AtomicU64>,

    /// Index of the next event to dispatch in the sorted timeline.
    /// Advanced by the audio callback as events are dispatched.
    pub next_event_idx: Arc<AtomicUsize>,

    /// Project metadata — display only, never read by the RT thread.
    pub meta: Arc<RwLock<ProjectMeta>>,
}

impl TransportState {
    pub fn new(bpm: f64) -> Arc<Self> {
        Arc::new(Self {
            playhead: Arc::new(AtomicU64::new(0)),
            is_playing: Arc::new(AtomicBool::new(false)),
            bpm_milli: Arc::new(AtomicU64::new((bpm * 1000.0) as u64)),
            next_event_idx: Arc::new(AtomicUsize::new(0)),
            meta: Arc::new(RwLock::new(ProjectMeta::default())),
        })
    }

    pub fn bpm(&self) -> f64 {
        self.bpm_milli.load(Ordering::Relaxed) as f64 / 1000.0
    }

    pub fn set_bpm(&self, bpm: f64) {
        self.bpm_milli
            .store((bpm * 1000.0) as u64, Ordering::Relaxed);
    }

    pub fn play(&self) {
        self.is_playing.store(true, Ordering::Release);
    }

    pub fn stop(&self) {
        self.is_playing.store(false, Ordering::Release);
    }

    /// Seek to a sample position, recalculating the event cursor.
    ///
    /// # Ordering guarantee
    /// `next_event_idx` is written before `playhead` using
    /// `Ordering::Release` on both stores. The audio callback reads
    /// `playhead` first with `Ordering::Acquire`. This ensures the
    /// callback never observes a playhead position that is ahead of
    /// the event cursor.
    pub fn seek<E: HasSamplePos>(&self, target_sample: u64, timeline: &[E]) {
        let new_idx = timeline.partition_point(|e| e.sample_pos() < target_sample);
        self.next_event_idx.store(new_idx, Ordering::Release);
        self.playhead.store(target_sample, Ordering::Release);
    }
}

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Commands sent from any frontend to the EngineMonitor.
///
/// This is the complete set of things a frontend can ask the
/// engine to do. The EngineMonitor pattern-matches on these
/// and updates TransportState or dispatches work accordingly.
///
/// Serialisable via MessagePack for ZMQ transport.
/// Sent directly via mpsc for Nushell plugins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransportCommand {
    Play,
    Stop,
    Seek(u64),
    SetBpm(f64),
    LoadMidi(PathBuf),
    AudioDevices,
    MidiDevices,
}

#[cfg(test)]
mod tests {
    use super::*;

    // A minimal HasSamplePos implementation for testing seek.
    struct TestEvent(u64);
    impl HasSamplePos for TestEvent {
        fn sample_pos(&self) -> u64 {
            self.0
        }
    }

    #[test]
    fn new_transport_is_stopped() {
        let ts = TransportState::new(120.0);
        assert!(!ts.is_playing.load(Ordering::Acquire));
    }

    #[test]
    fn play_and_stop() {
        let ts = TransportState::new(120.0);
        ts.play();
        assert!(ts.is_playing.load(Ordering::Acquire));
        ts.stop();
        assert!(!ts.is_playing.load(Ordering::Acquire));
    }

    #[test]
    fn bpm_roundtrip() {
        let ts = TransportState::new(120.0);
        assert_eq!(ts.bpm(), 120.0);
        ts.set_bpm(140.5);
        assert_eq!(ts.bpm(), 140.5);
    }

    #[test]
    fn seek_positions_cursor_correctly() {
        let ts = TransportState::new(120.0);
        let timeline = vec![
            TestEvent(0),
            TestEvent(44_100),
            TestEvent(88_200),
            TestEvent(132_300),
        ];

        ts.seek(44_100, &timeline);

        assert_eq!(ts.playhead.load(Ordering::Acquire), 44_100);
        // cursor points at the event AT the seek position
        assert_eq!(ts.next_event_idx.load(Ordering::Acquire), 1);
    }

    #[test]
    fn seek_to_zero_resets_cursor() {
        let ts = TransportState::new(120.0);
        let timeline = vec![TestEvent(0), TestEvent(44_100)];

        ts.seek(44_100, &timeline);
        ts.seek(0, &timeline);

        assert_eq!(ts.playhead.load(Ordering::Acquire), 0);
        assert_eq!(ts.next_event_idx.load(Ordering::Acquire), 0);
    }

    #[test]
    fn seek_past_end_positions_cursor_at_len() {
        let ts = TransportState::new(120.0);
        let timeline = vec![TestEvent(0), TestEvent(44_100)];

        ts.seek(999_999, &timeline);

        assert_eq!(ts.next_event_idx.load(Ordering::Acquire), timeline.len());
    }

    #[test]
    fn bpm_stored_as_milli() {
        let ts = TransportState::new(120.0);
        // Verify the internal representation directly
        assert_eq!(ts.bpm_milli.load(Ordering::Relaxed), 120_000);
        ts.set_bpm(140.5);
        assert_eq!(ts.bpm_milli.load(Ordering::Relaxed), 140_500);
    }
}
