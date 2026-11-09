use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{
    Arc, RwLock,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};

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
    pub title:         String,
    pub artist:        String,
    pub bpm:           f64,
    pub time_sig_num:  u8,
    pub time_sig_denom: u8,
}

/// A single MIDI event, as received from a Nushell pipeline.
///
/// Represents one row from a MUDL file or structured stream:
///   event beat=1 channel=1 note=60 vel=100 duration=0.5
///
/// `beat` and `duration` are in beats — the Conductor converts
/// to samples using the current BPM and sample rate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MidiEvent {
    pub beat:     f64,
    pub channel:  u8,
    pub note:     u8,
    pub vel:      u8,
    pub duration: f64,
}

/// Shared, lock-free transport state.
///
/// Wrapped in `Arc` and cloned into every component that needs it.
///
/// # Synchronisation strategy
/// - **Atomics** — fields read by the audio callback. Lock-free,
///   never block, single machine-word operations.
/// - **`RwLock`** — fields written rarely (user commands) but read
///   by many components. Multiple readers proceed concurrently.
///   Never held across an audio callback boundary.
pub struct TransportState {
    /// Current playhead position in samples from project start.
    /// Written by the audio callback on every buffer.
    /// Read by the Conductor for display.
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

    /// The MIDI output port currently selected by the user.
    /// `None` — no port selected; Conductor uses the first available.
    /// `Some(name)` — Conductor opens this port by name.
    /// Written by `TransportCommand::MidiPort(Some(name))`.
    /// Read when opening a midir output connection.
    pub active_midi_port: Arc<RwLock<Option<String>>>,

    /// The MIDI input port currently selected by the user.
    /// Independent from `active_midi_port` (output).
    /// `None` — no port selected; first available input is used.
    /// `Some(name)` — Conductor opens this input port by name.
    /// Written by `TransportCommand::MidiInputPort(Some(name))`.
    /// Read when opening a midir input connection.
    pub active_midi_input_port: Arc<RwLock<Option<String>>>,

    /// The audio host currently selected by the user.
    /// `None` — use the platform default host.
    /// `Some(name)` — use this cpal host by name (e.g. "ALSA", "JACK").
    /// Written by `TransportCommand::AudioHost(Some(name))`.
    pub active_audio_host: Arc<RwLock<Option<String>>>,

    /// The audio output device currently selected by the user.
    /// `None` — use the default device for the active host.
    /// `Some(name)` — use this device by name.
    /// Written by `TransportCommand::AudioDevice(Some(name))`.
    pub active_audio_device: Arc<RwLock<Option<String>>>,
}

impl TransportState {
    pub fn new(bpm: f64) -> Arc<Self> {
        Arc::new(Self {
            playhead:               Arc::new(AtomicU64::new(0)),
            is_playing:             Arc::new(AtomicBool::new(false)),
            bpm_milli:              Arc::new(AtomicU64::new((bpm * 1000.0) as u64)),
            next_event_idx:         Arc::new(AtomicUsize::new(0)),
            meta:                   Arc::new(RwLock::new(ProjectMeta::default())),
            active_midi_port:       Arc::new(RwLock::new(None)),
            active_midi_input_port: Arc::new(RwLock::new(None)),
            active_audio_host:      Arc::new(RwLock::new(None)),
            active_audio_device:    Arc::new(RwLock::new(None)),
        })
    }

    pub fn bpm(&self) -> f64 {
        self.bpm_milli.load(Ordering::Relaxed) as f64 / 1000.0
    }

    pub fn set_bpm(&self, bpm: f64) {
        self.bpm_milli.store((bpm * 1000.0) as u64, Ordering::Relaxed);
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

    /// Read the active MIDI output port name, if one has been selected.
    pub fn midi_port(&self) -> Option<String> {
        self.active_midi_port.read().unwrap().clone()
    }

    /// Set the active MIDI output port by name.
    /// Pass `None` to clear the selection and revert to first-available.
    pub fn set_midi_port(&self, name: Option<String>) {
        *self.active_midi_port.write().unwrap() = name;
    }

    /// Read the active MIDI input port name, if one has been selected.
    pub fn midi_input_port(&self) -> Option<String> {
        self.active_midi_input_port.read().unwrap().clone()
    }

    /// Set the active MIDI input port by name.
    /// Pass `None` to clear the selection and revert to first-available.
    pub fn set_midi_input_port(&self, name: Option<String>) {
        *self.active_midi_input_port.write().unwrap() = name;
    }

    /// Read the active audio host name, if one has been selected.
    pub fn audio_host(&self) -> Option<String> {
        self.active_audio_host.read().unwrap().clone()
    }

    /// Set the active audio host by name.
    pub fn set_audio_host(&self, name: Option<String>) {
        *self.active_audio_host.write().unwrap() = name;
    }

    /// Read the active audio device name, if one has been selected.
    pub fn audio_device(&self) -> Option<String> {
        self.active_audio_device.read().unwrap().clone()
    }

    /// Set the active audio device by name.
    pub fn set_audio_device(&self, name: Option<String>) {
        *self.active_audio_device.write().unwrap() = name;
    }
}

/// Commands sent from any frontend to the Conductor.
///
/// This is the complete set of things a frontend can ask the
/// engine to do. The Conductor pattern-matches on these and
/// updates `TransportState` or dispatches work accordingly.
///
/// Serialisable via MessagePack for ZMQ transport.
/// Sent directly via mpsc for in-process frontends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransportCommand {
    // ── Transport ─────────────────────────────────────────────────
    Play,
    Stop,
    Seek(u64),
    SetBpm(f64),

    // ── File I/O ──────────────────────────────────────────────────
    LoadMidi(PathBuf),

    /// Load a MUDL file (.mudl) and build the timeline from it.
    /// MUDL is a KDL-based format for mush MIDI event sequences.
    LoadMudl(PathBuf),

    /// Save the current timeline.
    /// File extension determines format: `.mid` or `.mudl`.
    Save(PathBuf),

    // ── Event streaming ───────────────────────────────────────────
    /// Queue a single MIDI event from a Nushell pipeline.
    /// Sent by `midi play` for each record in the input stream:
    ///   open song.mudl | where note > 60 | midi play
    QueueEvent(MidiEvent),

    /// Send raw MIDI bytes immediately, bypassing the timeline.
    /// Used for live input, panic, and test commands.
    /// Payload is 1–3 bytes of standard MIDI wire format.
    SendMessage(Vec<u8>),

    // ── Recording ─────────────────────────────────────────────────
    /// Start or stop MIDI capture.
    ///
    /// `start: true`  — begin capture. Opens the input port if not
    ///   already open. Also enables the pre-roll.
    ///   `path: Some(p)` — pre-specify the save path.
    ///   `path: None`    — path decided at stop time.
    ///
    /// `start: false` — stop capture and save.
    ///   `path: Some(p)` — override the pre-specified path.
    ///   `path: None`    — use pre-specified path, or generate a
    ///                     timestamped default if none was given.
    ///
    /// Conductor responds with `MidiUpdate::RecordSaved(path)`
    /// or `MidiUpdate::RecordEmpty` if nothing was captured.
    /// Emits `MidiUpdate::Error` if stop is called with no active capture.
    Record {
        start: bool,
        path:  Option<PathBuf>,
    },

    /// Open the MIDI input port and forward events to the Conductor.
    /// Does NOT feed the pre-roll buffer.
    /// Autosave path is optional — overrides any previously set path.
    ///
    /// Alias: `midi monitor` / `midi monitor <path>`
    Monitor { path: Option<PathBuf> },

    /// Open the MIDI input port, forward events, and feed the pre-roll.
    /// Autosave path is optional — overrides any previously set path.
    ///
    /// Alias: `midi listen` / `midi listen on` / `midi listen enable`
    Listen { path: Option<PathBuf> },

    /// Stop monitoring or listening. Closes the MIDI input port.
    /// No-op if neither Monitor nor Listen is active.
    ///
    /// Alias: `midi listen off` / `midi listen disable` / `midi ignore`
    Ignore,

    /// Set the pre-roll autosave path independently of Monitor/Listen.
    /// Written to when the pre-roll buffer wraps at capacity.
    SetAutosavePath(PathBuf),

    // ── Device selection ──────────────────────────────────────────
    /// Query or set the active MIDI output port.
    /// `None` — query; engine responds with `MidiUpdate::ActivePort`.
    /// `Some(name)` — select port by name; engine confirms with
    ///   `MidiUpdate::ActivePort(Some(name))`.
    MidiPort(Option<String>),

    /// Query or set the active MIDI input port.
    /// Independent from `MidiPort` (output).
    /// `None` — query; engine responds with `MidiUpdate::ActiveInputPort`.
    /// `Some(name)` — select input port by name.
    MidiInputPort(Option<String>),

    /// Query or set the active audio host.
    /// `None` — query; engine responds with `AudioUpdate::ActiveHost`.
    /// `Some(name)` — select host by name (e.g. "ALSA", "JACK").
    AudioHost(Option<String>),

    /// Query or set the active audio output device.
    /// `None` — query; engine responds with `AudioUpdate::ActiveDevice`.
    /// `Some(name)` — select device by name.
    AudioDevice(Option<String>),

    // ── Enumeration ───────────────────────────────────────────────
    /// Enumerate available MIDI output ports.
    /// Engine responds with `MidiUpdate::PortList`.
    MidiDevices,

    /// Enumerate available audio output devices on the active host.
    /// Engine responds with `AudioUpdate::DeviceList`.
    AudioDevices,

    /// Enumerate available audio hosts on this platform.
    /// Engine responds with `AudioUpdate::HostList`.
    AudioHosts,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let ts       = TransportState::new(120.0);
        let timeline = vec![
            TestEvent(0),
            TestEvent(44_100),
            TestEvent(88_200),
            TestEvent(132_300),
        ];
        ts.seek(44_100, &timeline);
        assert_eq!(ts.playhead.load(Ordering::Acquire),       44_100);
        assert_eq!(ts.next_event_idx.load(Ordering::Acquire), 1);
    }

    #[test]
    fn seek_to_zero_resets_cursor() {
        let ts       = TransportState::new(120.0);
        let timeline = vec![TestEvent(0), TestEvent(44_100)];
        ts.seek(44_100, &timeline);
        ts.seek(0, &timeline);
        assert_eq!(ts.playhead.load(Ordering::Acquire),       0);
        assert_eq!(ts.next_event_idx.load(Ordering::Acquire), 0);
    }

    #[test]
    fn seek_past_end_positions_cursor_at_len() {
        let ts       = TransportState::new(120.0);
        let timeline = vec![TestEvent(0), TestEvent(44_100)];
        ts.seek(999_999, &timeline);
        assert_eq!(ts.next_event_idx.load(Ordering::Acquire), timeline.len());
    }

    #[test]
    fn bpm_stored_as_milli() {
        let ts = TransportState::new(120.0);
        assert_eq!(ts.bpm_milli.load(Ordering::Relaxed), 120_000);
        ts.set_bpm(140.5);
        assert_eq!(ts.bpm_milli.load(Ordering::Relaxed), 140_500);
    }

    #[test]
    fn active_midi_port_defaults_to_none() {
        let ts = TransportState::new(120.0);
        assert_eq!(ts.midi_port(), None);
    }

    #[test]
    fn set_and_clear_midi_port() {
        let ts = TransportState::new(120.0);
        ts.set_midi_port(Some("IAC Driver Bus 1".into()));
        assert_eq!(ts.midi_port(), Some("IAC Driver Bus 1".into()));
        ts.set_midi_port(None);
        assert_eq!(ts.midi_port(), None);
    }

    #[test]
    fn active_midi_input_port_defaults_to_none() {
        let ts = TransportState::new(120.0);
        assert_eq!(ts.midi_input_port(), None);
    }

    #[test]
    fn set_and_clear_midi_input_port() {
        let ts = TransportState::new(120.0);
        ts.set_midi_input_port(Some("IAC Driver Bus 1".into()));
        assert_eq!(ts.midi_input_port(), Some("IAC Driver Bus 1".into()));
        ts.set_midi_input_port(None);
        assert_eq!(ts.midi_input_port(), None);
    }

    #[test]
    fn set_and_clear_audio_host() {
        let ts = TransportState::new(120.0);
        ts.set_audio_host(Some("JACK".into()));
        assert_eq!(ts.audio_host(), Some("JACK".into()));
        ts.set_audio_host(None);
        assert_eq!(ts.audio_host(), None);
    }

    #[test]
    fn set_and_clear_audio_device() {
        let ts = TransportState::new(120.0);
        ts.set_audio_device(Some("hw:0,0".into()));
        assert_eq!(ts.audio_device(), Some("hw:0,0".into()));
        ts.set_audio_device(None);
        assert_eq!(ts.audio_device(), None);
    }
}