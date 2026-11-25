pub struct TransportState {
    /// Current playhead position in samples from project start.
    /// Written by the audio callback on every buffer.
    /// Read by the Conductor for display.
    pub playhead: Arc<AtomicU64>,

    /// Whether the transport is currently running (playing or recording).
    pub is_playing: Arc<AtomicBool>,

    /// Whether the conductor is currently recording.
    ///
    /// Derived from `record_buffer: Option<_>` on the Conductor — set to
    /// `true` when recording starts, `false` when it stops. Stored here as
    /// an atomic so it can be read by the live view TUI without taking a lock
    /// or holding a reference into the Conductor.
    ///
    /// Invariant: `is_recording` is always `false` when `is_playing` is
    /// `false`. Both may be `true` simultaneously (overdub / punch-in,
    /// post-MVP).
    pub is_recording: Arc<AtomicBool>,

    /// Tempo stored as BPM × 1000 to avoid floating-point atomics.
    /// 120.0 BPM is stored as 120_000u64.
    pub bpm_milli: Arc<AtomicU64>,

    /// Index of the next event to dispatch in the sorted timeline.
    /// Advanced by the audio callback as events are dispatched.
    pub next_event_idx: Arc<AtomicUsize>,

    /// Project metadata — display only, never read by the RT thread.
    pub meta: Arc<RwLock<ProjectMeta>>,

    /// The MIDI output port currently selected by the user.
    pub active_midi_port: Arc<RwLock<Option<String>>>,

    /// The MIDI input port currently selected by the user.
    pub active_midi_input_port: Arc<RwLock<Option<String>>>,

    /// The audio host currently selected by the user.
    pub active_audio_host: Arc<RwLock<Option<String>>>,

    /// The audio output device currently selected by the user.
    pub active_audio_device: Arc<RwLock<Option<String>>>,
}

impl TransportState {
    pub fn new(bpm: f64) -> Arc<Self> {
        Arc::new(Self {
            playhead:            Arc::new(AtomicU64::new(0)),
            is_playing:          Arc::new(AtomicBool::new(false)),
            is_recording:        Arc::new(AtomicBool::new(false)),
            bpm_milli:           Arc::new(AtomicU64::new((bpm * 1000.0) as u64)),
            next_event_idx:      Arc::new(AtomicUsize::new(0)),
            meta:                Arc::new(RwLock::new(ProjectMeta::default())),
            active_midi_port:    Arc::new(RwLock::new(None)),
            active_midi_input_port: Arc::new(RwLock::new(None)),
            active_audio_host:   Arc::new(RwLock::new(None)),
            active_audio_device: Arc::new(RwLock::new(None)),
        })
    }
}