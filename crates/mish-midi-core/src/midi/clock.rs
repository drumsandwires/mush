use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use cpal::StreamConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::queue::ArrayQueue;

use mush_core::update::AudioUpdate;

use crate::midi::timeline::TimelineEvent;

/// Number of frames per software clock tick.
/// Matches a typical hardware buffer size.
const BUFFER_SIZE: u64 = 512;

/// Minimum samples between playhead reports to the EngineMonitor.
/// 4410 samples ≈ 100ms at 44.1kHz — avoids flooding the channel.
const REPORT_INTERVAL: u64 = 4_410;

/// Arguments shared between `Clock::start`, `try_cpal`,
/// and `start_software`. Groups the transport state handles
/// to keep function signatures below the argument limit.
pub struct ClockArgs {
    pub playhead: Arc<AtomicU64>,
    pub is_playing: Arc<AtomicBool>,
    pub next_event_idx: Arc<AtomicUsize>,
    pub timeline: Arc<RwLock<Vec<TimelineEvent>>>,
    pub midi_queue: Arc<ArrayQueue<[u8; 3]>>,
    pub status_tx: tokio::sync::mpsc::Sender<AudioUpdate>,
    pub sample_rate: u32,
}

/// The master clock. Drives MIDI dispatch and playhead advancement.
///
/// Must be kept alive for the duration of playback.
/// Dropping this value stops the clock.
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
    pub fn start(args: ClockArgs) -> Self {
        match Self::try_cpal(ClockArgs {
            playhead: Arc::clone(&args.playhead),
            is_playing: Arc::clone(&args.is_playing),
            next_event_idx: Arc::clone(&args.next_event_idx),
            timeline: Arc::clone(&args.timeline),
            midi_queue: Arc::clone(&args.midi_queue),
            status_tx: args.status_tx.clone(),
            sample_rate: args.sample_rate,
        }) {
            Ok(stream) => {
                eprintln!("clock: cpal audio callback (sample-accurate)");
                Clock::Cpal(stream)
            }
            Err(e) => {
                eprintln!("clock: cpal unavailable ({e}), using software fallback");
                let running = Arc::new(AtomicBool::new(true));
                let handle = Self::start_software(args, Arc::clone(&running));
                Clock::Software { running, handle }
            }
        }
    }

    fn try_cpal(args: ClockArgs) -> Result<cpal::Stream, Box<dyn std::error::Error>> {
        let ClockArgs {
            playhead,
            is_playing,
            next_event_idx,
            timeline,
            midi_queue,
            status_tx,
            ..
        } = args;
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no default audio output device")?;

        let config: StreamConfig = device.default_output_config()?.into();
        let channels = config.channels as usize;

        // Local state owned by the callback.
        // These are plain values — not Arc — because only the
        // callback ever reads or writes them.
        let mut rt_elevated = false;
        let mut last_report = 0u64;
        // Clone before either closure captures status_tx.
        // The audio callback gets the original.
        // The error callback gets the clone.
        let status_tx_err = status_tx.clone();
        let stream = device.build_output_stream(
            config,
            // ── Audio callback ────────────────────────────────────────
            // `move` transfers ownership of all captured variables
            // into the closure. After this point, the variables above
            // belong to the callback — not to try_cpal's stack frame.
            move |out: &mut [f32], _info| {
                // Elevate RT priority once, on the first callback.
                // Must happen here — this is the audio thread.
                if !rt_elevated {
                    mush_core::rt_priority::elevate_current_thread();
                    rt_elevated = true;
                }

                // Fill with silence.
                // TODO Phase 3: replace with AudioClip mixing.
                out.fill(0.0);

                // Early return if not playing — silence is correct.
                if !is_playing.load(Ordering::Acquire) {
                    return;
                }

                let buffer_frames = (out.len() / channels) as u64;
                let frame_start = playhead.load(Ordering::Acquire);
                let frame_end = frame_start + buffer_frames;

                // Dispatch MIDI events due in this buffer window.
                // try_read() never blocks — safe on the RT thread.
                if let Ok(tl) = timeline.try_read() {
                    dispatch_events(&tl, &next_event_idx, &midi_queue, frame_end);
                }

                // Advance the playhead.
                playhead.store(frame_end, Ordering::Release);

                // Report playhead position periodically.
                if frame_end - last_report >= REPORT_INTERVAL {
                    let _ = status_tx.try_send(AudioUpdate::PlayheadMoved(frame_end));
                    last_report = frame_end;
                }
            },
            // ── Error callback ────────────────────────────────────────
            // Separate closure — needs its own clone of status_tx
            // because the audio callback already owns the original.
            {
                move |err| {
                    let _ = status_tx_err.try_send(AudioUpdate::Error(err.to_string()));
                }
            },
            None, // timeout
        )?;

        stream.play()?;
        Ok(stream)
    }

    fn start_software(args: ClockArgs, running: Arc<AtomicBool>) -> thread::JoinHandle<()> {
        let ClockArgs {
            playhead,
            is_playing,
            next_event_idx,
            timeline,
            midi_queue,
            status_tx,
            sample_rate,
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
                            dispatch_events(&tl, &next_event_idx, &midi_queue, frame_end);
                        }

                        playhead.store(frame_end, Ordering::Release);

                        if frame_end - last_report >= REPORT_INTERVAL {
                            let _ = status_tx.try_send(AudioUpdate::PlayheadMoved(frame_end));
                            last_report = frame_end;
                        }
                    }

                    // Sleep for the remainder of the tick period.
                    // checked_sub returns None if elapsed > tick_duration
                    // — i.e. the tick ran long. In that case skip the sleep
                    // and start the next tick immediately.
                    if let Some(remaining) = tick_duration.checked_sub(tick_start.elapsed()) {
                        thread::sleep(remaining);
                    }
                }
            })
            .expect("failed to spawn software clock thread")
    }
}

/// Dispatch all timeline events due before `frame_end`.
///
/// Called from the audio callback and the software clock thread.
/// Must be lock-free — no allocation, no blocking.
///
/// Events whose `sample_pos` falls before `frame_end` are popped
/// from the timeline cursor and pushed to the MIDI queue.
/// If the queue is full the event is dropped — preferable to
/// blocking the RT thread.
pub fn dispatch_events(
    timeline: &[TimelineEvent],
    next_event_idx: &AtomicUsize,
    midi_queue: &ArrayQueue<[u8; 3]>,
    frame_end: u64,
) {
    loop {
        let idx = next_event_idx.load(Ordering::Acquire);

        match timeline.get(idx) {
            Some(event) if event.sample_pos < frame_end => {
                use crate::midi::timeline::EventPayload;
                if let EventPayload::Midi(bytes) = event.payload {
                    let _ = midi_queue.push(bytes);
                }
                next_event_idx.fetch_add(1, Ordering::Release);
            }
            _ => break,
        }
    }
}

impl Drop for Clock {
    fn drop(&mut self) {
        match self {
            Clock::Software { running, .. } => {
                // Signal the thread to exit on its next tick.
                // We do not join here — Drop must not block.
                running.store(false, Ordering::Release);
            }
            Clock::Cpal(_) => {
                // cpal::Stream stops the audio subsystem when dropped.
                // The compiler calls drop on the inner Stream automatically
                // after this match — no explicit action needed.
            }
        }
    }
}
