use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use cpal::StreamConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::queue::ArrayQueue;
use thiserror::Error;
use tracing::{error, info, warn};

use crate::timeline::TimelineEvent;
use crate::update::AudioUpdate;

/// Number of frames per software clock tick.
/// Matches a typical hardware buffer size.
const BUFFER_SIZE: u64 = 512;

/// Minimum samples between playhead reports to the Conductor.
/// 4_410 samples ≈ 100ms at 44.1kHz — avoids flooding the channel.
const REPORT_INTERVAL: u64 = 4_410;

/// Errors that can occur when starting the clock.
#[derive(Debug, Error)]
pub enum ClockError {
    #[error("no default audio output device available")]
    NoDevice,

    #[error("requested audio host '{0}' is not available on this system")]
    UnknownHost(String),

    #[error("requested audio device '{0}' is not available on host '{1}'")]
    UnknownDevice(String, String),

    #[error("cpal error: {0}")]
    Cpal(#[from] cpal::Error),
}

/// Arguments shared between `Clock::start`, `try_cpal`,
/// and `start_software`.
///
/// Generic over the payload type `P` — the clock does not inspect
/// payloads directly. It reads `sample_pos` via the timeline slice
/// and pushes raw bytes to `midi_queue` after the caller extracts
/// them from the payload.
///
/// # Why a struct rather than individual arguments?
/// Seven fields would exceed the conventional argument limit and
/// make call sites unreadable. Grouping them also makes it natural
/// to clone the `Arc` fields before consuming the struct.
pub struct ClockArgs<P>
where
    P: Clone + Send + Sync + 'static,
{
    pub playhead: Arc<AtomicU64>,
    pub is_playing: Arc<AtomicBool>,
    pub next_event_idx: Arc<AtomicUsize>,
    pub timeline: Arc<RwLock<Vec<TimelineEvent<P>>>>,
    pub midi_queue: Arc<ArrayQueue<[u8; 3]>>,
    pub status_tx: tokio::sync::mpsc::Sender<AudioUpdate>,
    pub sample_rate: u32,
    /// Extracts raw MIDI bytes from a payload.
    /// Returns `None` for non-MIDI payloads (e.g. `TempoChange`).
    /// Called on the RT path — must not allocate or block.
    pub extract_midi: fn(&P) -> Option<[u8; 3]>,
    /// CPAL host name to use for playback.
    /// `None` — use `cpal::default_host()`.
    /// `Some(name)` — resolve via `cpal::available_hosts()`.
    pub host_name: Option<String>,
    /// CPAL output device name to use for playback.
    /// `None` — use the host's default output device.
    /// `Some(name)` — resolve via `host.output_devices()`.
    pub device_name: Option<String>,
}

/// The master clock. Drives MIDI dispatch and playhead advancement.
///
/// Must be kept alive for the duration of playback.
/// **Dropping this value stops the clock** — this is intentional RAII.
/// The `Conductor` stores `Option<Clock>` and stops by writing `None`.
///
/// # Variants
/// - `Cpal` — OS audio subsystem drives the callback cadence.
///   Sample-accurate. Preferred.
/// - `Software` — `thread::sleep` drives the tick cadence.
///   Less accurate. Used in CI and headless environments.
pub enum Clock {
    Cpal(cpal::Stream),
    Software {
        running: Arc<AtomicBool>,
        handle: thread::JoinHandle<()>,
    },
}

impl Clock {
    /// Start the master clock.
    ///
    /// Tries to open a cpal output stream first.
    /// Falls back to a software clock if no audio device is available.
    pub fn start<P>(args: ClockArgs<P>) -> Self
    where
        P: Clone + Send + Sync + 'static,
    {
        let fallback = ClockArgs {
            playhead: Arc::clone(&args.playhead),
            is_playing: Arc::clone(&args.is_playing),
            next_event_idx: Arc::clone(&args.next_event_idx),
            timeline: Arc::clone(&args.timeline),
            midi_queue: Arc::clone(&args.midi_queue),
            status_tx: args.status_tx.clone(),
            sample_rate: args.sample_rate,
            extract_midi: args.extract_midi,
            host_name: args.host_name.clone(),
            device_name: args.device_name.clone(),
        };

        match Self::try_cpal(args) {
            Ok(stream) => {
                info!("clock: cpal audio callback (sample-accurate)");
                Clock::Cpal(stream)
            }
            Err(e) => {
                warn!("clock: cpal unavailable ({e}), using software fallback");
                let running = Arc::new(AtomicBool::new(true));
                let handle = Self::start_software(fallback, Arc::clone(&running));
                Clock::Software { running, handle }
            }
        }
    }

    fn try_cpal<P>(args: ClockArgs<P>) -> Result<cpal::Stream, ClockError>
    where
        P: Clone + Send + Sync + 'static,
    {
        let ClockArgs {
            playhead,
            is_playing,
            next_event_idx,
            timeline,
            midi_queue,
            status_tx,
            extract_midi,
            host_name,
            device_name,
            ..
        } = args;

        // ── Host resolution ───────────────────────────────────────────────────
        let host = match host_name {
            None => cpal::default_host(),
            Some(ref name) => cpal::available_hosts()
                .into_iter()
                .find_map(|id| {
                    let h = cpal::host_from_id(id).ok()?;
                    if h.id().name() == name.as_str() {
                        Some(h)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| ClockError::UnknownHost(name.clone()))?,
        };

        // ── Device resolution ─────────────────────────────────────────────────
        let device = match device_name {
            None => host.default_output_device().ok_or(ClockError::NoDevice)?,
            Some(ref name) => host
                .output_devices()
                .map_err(cpal::Error::from)?
                .find(|d| {
                    d.description()
                        .map(|desc| desc.name() == name.as_str())
                        .unwrap_or(false)
                })
                .ok_or_else(|| {
                    ClockError::UnknownDevice(name.clone(), host.id().name().to_string())
                })?,
        };

        let config: StreamConfig = device.default_output_config()?.into();
        let channels = config.channels as usize;

        // Local state owned exclusively by the callback closure.
        // Plain values — not Arc — because only the callback touches them.
        let mut rt_elevated = false;
        let mut last_report = 0u64;

        // The error callback needs its own sender clone.
        // Both clones are moved into their respective closures below.
        let status_tx_err = status_tx.clone();

        let stream = device.build_output_stream(
            config,
            // ── Audio callback ────────────────────────────────────────
            // Called by the OS on every buffer. Must not block or allocate.
            move |out: &mut [f32], _info| {
                // Elevate RT priority on first callback invocation.
                // Done here rather than before `build_output_stream`
                // because some platforms require the thread to be
                // running before priority can be set.
                if !rt_elevated {
                    crate::rt_priority::elevate_current_thread();
                    rt_elevated = true;
                }

                // Output silence — this clock drives MIDI only.
                // A future mash audio engine will fill this buffer.
                out.fill(0.0);

                if !is_playing.load(Ordering::Acquire) {
                    return;
                }

                let buffer_frames = (out.len() / channels) as u64;
                let frame_start = playhead.load(Ordering::Acquire);
                let frame_end = frame_start + buffer_frames;

                if let Ok(tl) = timeline.try_read() {
                    dispatch_events(&tl, &next_event_idx, &midi_queue, frame_end, extract_midi);
                }

                playhead.store(frame_end, Ordering::Release);

                if frame_end - last_report >= REPORT_INTERVAL {
                    let _ = status_tx.try_send(AudioUpdate::PlayheadMoved(frame_end));
                    last_report = frame_end;
                }
            },
            // ── Error callback ────────────────────────────────────────
            move |err| {
                error!("cpal stream error: {err}");
                let _ = status_tx_err.try_send(AudioUpdate::Error(err.to_string()));
            },
            None, // timeout
        )?;

        stream.play()?;
        Ok(stream)
    }

    fn start_software<P>(args: ClockArgs<P>, running: Arc<AtomicBool>) -> thread::JoinHandle<()>
    where
        P: Clone + Send + Sync + 'static,
    {
        // Destructure with `..` — `host_name` and `device_name` are not
        // used by the software clock; it has no audio device to open.
        let ClockArgs {
            playhead,
            is_playing,
            next_event_idx,
            timeline,
            midi_queue,
            status_tx,
            sample_rate,
            extract_midi,
            ..
        } = args;

        let tick_duration = Duration::from_secs_f64(BUFFER_SIZE as f64 / sample_rate as f64);

        thread::Builder::new()
            .name("mush-software-clock".into())
            .spawn(move || {
                let mut last_report = 0u64;

                while running.load(Ordering::Acquire) {
                    let tick_start = Instant::now();

                    if is_playing.load(Ordering::Acquire) {
                        let frame_start = playhead.load(Ordering::Acquire);
                        let frame_end = frame_start + BUFFER_SIZE;

                        if let Ok(tl) = timeline.try_read() {
                            dispatch_events(
                                &tl,
                                &next_event_idx,
                                &midi_queue,
                                frame_end,
                                extract_midi,
                            );
                        }

                        playhead.store(frame_end, Ordering::Release);

                        if frame_end - last_report >= REPORT_INTERVAL {
                            let _ = status_tx.try_send(AudioUpdate::PlayheadMoved(frame_end));
                            last_report = frame_end;
                        }
                    }

                    // Sleep for the remainder of the tick period.
                    // `checked_sub` returns `None` if the tick ran long —
                    // in that case skip the sleep and start the next tick
                    // immediately rather than accumulating drift.
                    if let Some(remaining) = tick_duration.checked_sub(tick_start.elapsed()) {
                        thread::sleep(remaining);
                    }
                }
            })
            .expect("failed to spawn software clock thread")
    }
}

// ── RAII stop ─────────────────────────────────────────────────────────────────

impl Drop for Clock {
    fn drop(&mut self) {
        match self {
            Clock::Software { running, .. } => {
                // Signal the thread to exit on its next tick.
                // We do not join here — `Drop` must not block.
                running.store(false, Ordering::Release);
            }
            Clock::Cpal(_) => {
                // `cpal::Stream` stops the audio subsystem when dropped.
                // The compiler calls drop on the inner value automatically.
            }
        }
    }
}

// ── dispatch_events ───────────────────────────────────────────────────────────

/// Dispatch all timeline events due before `frame_end`.
///
/// Called from both the audio callback and the software clock thread.
/// **Must be lock-free — no allocation, no blocking.**
///
/// Advances `next_event_idx` atomically as events are consumed.
/// If `midi_queue` is full the event is dropped — preferable to
/// blocking the RT thread.
///
/// # Why a function pointer for `extract_midi`?
/// A closure would require a generic bound or a `Box<dyn Fn>`.
/// A function pointer `fn(&P) -> Option<[u8; 3]>` is zero-cost,
/// `Copy`, and sufficient — the extraction logic is always stateless.
pub fn dispatch_events<P>(
    timeline: &[TimelineEvent<P>],
    next_event_idx: &AtomicUsize,
    midi_queue: &ArrayQueue<[u8; 3]>,
    frame_end: u64,
    extract_midi: fn(&P) -> Option<[u8; 3]>,
) where
    P: Clone + Send + Sync + 'static,
{
    loop {
        let idx = next_event_idx.load(Ordering::Acquire);

        match timeline.get(idx) {
            Some(event) if event.sample_pos < frame_end => {
                if let Some(bytes) = extract_midi(&event.payload) {
                    // Prefer dropping to blocking on the RT path.
                    let _ = midi_queue.push(bytes);
                }
                next_event_idx.fetch_add(1, Ordering::Release);
            }
            _ => break,
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timeline::TimelineEvent;
    use crossbeam::queue::ArrayQueue;
    use std::sync::atomic::AtomicUsize;

    /// Minimal payload: either a MIDI note-on or a non-MIDI marker.
    #[derive(Debug, Clone)]
    enum TestPayload {
        Midi([u8; 3]),
        Marker,
    }

    fn extract(p: &TestPayload) -> Option<[u8; 3]> {
        match p {
            TestPayload::Midi(b) => Some(*b),
            TestPayload::Marker => None,
        }
    }

    fn make_timeline(events: Vec<(u64, TestPayload)>) -> Vec<TimelineEvent<TestPayload>> {
        events
            .into_iter()
            .map(|(sample_pos, payload)| TimelineEvent {
                sample_pos,
                payload,
            })
            .collect()
    }

    #[test]
    fn dispatches_events_before_frame_end() {
        let timeline = make_timeline(vec![
            (0, TestPayload::Midi([0x90, 60, 100])),
            (44_100, TestPayload::Midi([0x80, 60, 0])),
        ]);
        let idx = AtomicUsize::new(0);
        let queue = ArrayQueue::new(16);

        dispatch_events(&timeline, &idx, &queue, 44_100, extract);

        // Only the event at sample 0 is before frame_end 44_100.
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop(), Some([0x90, 60, 100]));
        assert_eq!(idx.load(Ordering::Acquire), 1);
    }

    #[test]
    fn skips_non_midi_payloads() {
        let timeline = make_timeline(vec![
            (0, TestPayload::Marker),
            (1, TestPayload::Midi([0x90, 64, 80])),
        ]);
        let idx = AtomicUsize::new(0);
        let queue = ArrayQueue::new(16);

        dispatch_events(&timeline, &idx, &queue, 100, extract);

        // Marker is consumed (idx advances) but not pushed to queue.
        assert_eq!(idx.load(Ordering::Acquire), 2);
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop(), Some([0x90, 64, 80]));
    }

    #[test]
    fn does_not_dispatch_events_at_or_after_frame_end() {
        let timeline = make_timeline(vec![(512, TestPayload::Midi([0x90, 60, 100]))]);
        let idx = AtomicUsize::new(0);
        let queue = ArrayQueue::new(16);

        dispatch_events(&timeline, &idx, &queue, 512, extract);

        assert_eq!(queue.len(), 0);
        assert_eq!(idx.load(Ordering::Acquire), 0);
    }

    #[test]
    fn empty_timeline_does_not_panic() {
        let timeline: Vec<TimelineEvent<TestPayload>> = vec![];
        let idx = AtomicUsize::new(0);
        let queue = ArrayQueue::new(16);

        dispatch_events(&timeline, &idx, &queue, 44_100, extract);

        assert_eq!(queue.len(), 0);
    }
}
