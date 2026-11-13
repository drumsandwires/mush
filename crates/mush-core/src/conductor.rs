use std::path::PathBuf;
use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use midir::MidiOutput as MidirOutput;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use tracing::{debug, info, warn};

use crate::clock::{Clock, ClockArgs};
use crate::midi::display::list_enumerated_devices;
use crate::midi::engine::DesktopMidiOutput;
use crate::midi::enumerate::list_midi_ports;
use crate::midi::input::{InputEvent, MidiInputHandle};
use crate::midi::preroll::PreRoll;
use crate::midi::timeline::{MidiPayload, load_midi};
use crate::midi_output::MidiOutput;
use crate::subscription::{Subscription, subscription};
use crate::timeline::TimelineEvent;
use crate::transport::{MidiEvent, TransportCommand, TransportState};
use crate::update::{AudioUpdate, EngineUpdate, MidiUpdate};
use crate::zmq_transport::ZmqTransport;

/// Capacity of the audio and MIDI update channels.
/// 64 updates is sufficient for a 100ms reporting interval at
/// any realistic tempo — the conductor drains these on every
/// select! iteration.
const CHANNEL_CAPACITY: usize = 64;

/// Capacity of the lock-free MIDI dispatch queue shared between
/// the Conductor and the Clock's audio callback.
///
/// 1024 slots is sufficient for a dense burst of notes at any
/// realistic tempo. If this fills, the Conductor logs a warning
/// and drops the event rather than blocking the RT thread.
///
/// TODO: make this configurable via `ConductorConfig` post-MVP.
const MIDI_QUEUE_CAPACITY: usize = 1024;

/// Capacity of the MIDI input event channel.
/// The midir callback fires on its own thread; this channel
/// bridges it to the Conductor's async task.
const INPUT_CHANNEL_CAPACITY: usize = 256;

/// Initial BPM used before any `SetBpm` command is received.
///
/// TODO: make this configurable via `ConductorConfig` post-MVP.
const INITIAL_BPM: f64 = 120.0;

/// Tracks the current MIDI input mode.
///
/// Determines whether incoming [`InputEvent`]s are forwarded only
/// (Monitor), also pushed to the pre-roll (Listen), or neither (Off).
#[derive(Debug, Clone, Copy, PartialEq)]
enum InputMode {
    /// No input port open.
    Off,
    /// Input open, events forwarded to Conductor, pre-roll not fed.
    Monitor,
    /// Input open, events forwarded, pre-roll fed.
    Listen,
}

/// The central coordinator for the mush engine.
///
/// Owns the ZMQ transport, receives updates from the Clock and
/// MidiEngineRt via tokio mpsc channels, and fans out EngineUpdates
/// to all registered subscribers.
///
/// # Lifecycle
/// 1. Create with `Conductor::new()`
/// 2. Pass the returned `midi_queue` Arc to `Clock`
/// 3. Pass the returned senders to `Clock` and `MidiEngineRt`
/// 4. Call `Conductor::run().await` — runs until shutdown
///
/// # Example
/// ```rust,ignore
/// let (conductor, _midi_queue, _audio_tx, _midi_tx) = Conductor::new(
///     addrs::CMD_INPROC,
///     addrs::PUB_INPROC,
///     44_100,
/// ).unwrap();
/// conductor.run().await;
/// ```
pub struct Conductor {
    ctx: tmq::Context,
    transport: ZmqTransport,

    audio_rx: mpsc::Receiver<AudioUpdate>,
    /// Sender side of the audio update channel.
    /// Cloned into `ClockArgs` on each `Play` command so the Clock
    /// can report playhead position and errors back to the Conductor.
    audio_tx: mpsc::Sender<AudioUpdate>,

    midi_rx: mpsc::Receiver<MidiUpdate>,

    subs: Vec<Subscription>,

    /// Sample rate passed at construction — used for beat-to-sample
    /// conversion in `QueueEvent` and `LoadMidi`, and passed to
    /// `ClockArgs` on `Play`.
    sample_rate: u32,

    /// Tick resolution preserved from the last LoadMidi / LoadMudl call.
    /// Defaults to 480 if no file has been loaded.
    /// Used by Save to reconstruct ticks from sample positions.
    ticks_per_beat: u16,

    /// Shared transport state — atomics written by `SetBpm`, `Seek`,
    /// `Play`, and `Stop`; read by the Clock's audio callback.
    state: Arc<TransportState>,

    /// The sorted, sample-accurate event timeline.
    ///
    /// Written by `LoadMidi`, `LoadMudl`, and `QueueEvent`.
    /// Read by `Clock` via the shared `Arc`.
    /// Protected by `RwLock` — never held across an audio callback.
    timeline: Arc<std::sync::RwLock<Vec<TimelineEvent<MidiPayload>>>>,

    /// Lock-free queue of raw MIDI bytes for the RT audio callback.
    ///
    /// The Clock pops from this queue on every audio buffer without
    /// locking. Bounded at `MIDI_QUEUE_CAPACITY`.
    ///
    /// Note: `SendMessage` does NOT use this queue — it writes
    /// directly to `midi_connection` if one is open, or returns an
    /// error if not. The queue is solely for the RT dispatch path.
    #[allow(dead_code)]
    midi_queue: Arc<ArrayQueue<[u8; 3]>>,

    /// The currently open MIDI output connection, if any.
    ///
    /// `None`  — no port selected; `SendMessage` returns an error.
    /// `Some`  — port is open; `SendMessage` writes directly.
    ///
    /// Replaced on `MidiPort(Some(name))`. Closed (dropped) on
    /// `MidiPort(None)`.
    midi_connection: Option<Box<dyn MidiOutput>>,

    /// The master clock. `Some` while playing, `None` while stopped.
    ///
    /// Dropping the `Clock` value stops playback — intentional RAII.
    /// `Stop` sets this to `None`.
    clock: Option<Clock>,

    // ── MIDI input ────────────────────────────────────────────────
    /// Active MIDI input connection, if Monitor, Listen, or Record
    /// has opened one. Dropping this closes the midir connection —
    /// intentional RAII.
    midi_input: Option<MidiInputHandle>,

    /// Receiver for events forwarded from the MIDI input callback.
    input_rx: mpsc::Receiver<InputEvent>,

    /// Sender side — cloned into `MidiInputHandle::open` on each
    /// Monitor / Listen / Record command.
    input_tx: mpsc::Sender<InputEvent>,

    /// Pre-roll buffer. Allocated always; fed only when `Listen`
    /// or `Record` is active.
    pre_roll: PreRoll,

    /// Whether the pre-roll is currently being fed.
    /// `true` during Listen and Record; `false` during Monitor-only.
    pre_roll_active: bool,

    /// Active record capture buffer.
    /// `Some` while `Record { start: true }` is in effect.
    record_buffer: Option<Vec<InputEvent>>,

    /// Path to save the completed recording.
    /// Set by `Record { start: true, path: Some(p) }`.
    record_path: Option<PathBuf>,

    /// Current MIDI input mode.
    input_mode: InputMode,
}

impl Conductor {
    /// Create a new `Conductor` and return it alongside the shared
    /// MIDI queue and the senders that `Clock` and `MidiEngineRt`
    /// write into.
    ///
    /// # Arguments
    /// - `cmd_addr`    — address to bind the ROUTER socket
    /// - `pub_addr`    — address to bind the PUB socket
    /// - `sample_rate` — audio device sample rate in Hz (e.g. 44_100)
    ///
    /// Use `addrs::CMD_INPROC` / `addrs::PUB_INPROC` for same-process
    /// deployment (MVP). Use `addrs::CMD_TCP` / `addrs::PUB_TCP` for
    /// standalone daemon deployment (post-MVP).
    ///
    /// # Errors
    /// Returns `Err` if either ZMQ socket fails to bind.
    pub fn new(
        cmd_addr: &str,
        pub_addr: &str,
        sample_rate: u32,
    ) -> Result<
        (
            Self,
            Arc<ArrayQueue<[u8; 3]>>,
            mpsc::Sender<AudioUpdate>,
            mpsc::Sender<MidiUpdate>,
        ),
        tmq::TmqError,
    > {
        let ctx = tmq::Context::new();
        let transport = ZmqTransport::bind(&ctx, cmd_addr, pub_addr)?;

        let (audio_tx, audio_rx) = mpsc::channel(CHANNEL_CAPACITY);
        let (midi_tx, midi_rx) = mpsc::channel(CHANNEL_CAPACITY);
        let (input_tx, input_rx) = mpsc::channel(INPUT_CHANNEL_CAPACITY);

        let midi_queue = Arc::new(ArrayQueue::new(MIDI_QUEUE_CAPACITY));

        let conductor = Self {
            ctx,
            transport,
            audio_tx: audio_tx.clone(),
            audio_rx,
            midi_rx,
            subs: Vec::new(),
            sample_rate,
            ticks_per_beat: 480,
            state: TransportState::new(INITIAL_BPM),
            timeline: Arc::new(std::sync::RwLock::new(Vec::new())),
            midi_queue: Arc::clone(&midi_queue),
            midi_connection: None,
            clock: None,
            midi_input: None,
            input_rx,
            input_tx,
            pre_roll: PreRoll::new(),
            pre_roll_active: false,
            record_buffer: None,
            record_path: None,
            input_mode: InputMode::Off,
        };

        Ok((conductor, midi_queue, audio_tx, midi_tx))
    }

    /// Return a reference to the shared ZMQ context.
    ///
    /// Pass this to `ZmqClient::connect` when deploying in the same
    /// process — inproc transport requires a shared context.
    pub fn context(&self) -> &tmq::Context {
        &self.ctx
    }

    /// Return a clone of the shared timeline `Arc`.
    ///
    /// Pass this to `Clock` so it can read events without copying
    /// the timeline on every buffer.
    pub fn timeline(&self) -> Arc<std::sync::RwLock<Vec<TimelineEvent<MidiPayload>>>> {
        Arc::clone(&self.timeline)
    }

    /// Register an in-process subscriber.
    ///
    /// Returns a `SubscriptionHandle` the caller uses to receive
    /// `EngineUpdate`s. Must be called before `run()`.
    pub fn subscribe(&mut self) -> crate::subscription::SubscriptionHandle {
        let (sub, handle) = subscription(32);
        self.subs.push(sub);
        handle
    }

    /// Run the conductor loop.
    ///
    /// Suspends on `tokio::select!` — wakes only when a command
    /// arrives or an update is ready to fan out. Returns when the
    /// ZMQ transport is closed or all senders are dropped.
    pub async fn run(mut self) {
        let mut drain_interval = time::interval(Duration::from_millis(1));
        drain_interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        loop {
            tokio::select! {
                cmd = self.transport.recv_command() => {
                    match cmd {
                        Some(cmd) => self.handle_command(cmd).await,
                        None      => break,
                    }
                }
                upd = self.audio_rx.recv() => {
                    match upd {
                        Some(upd) => self.emit_all(EngineUpdate::Audio(upd)).await,
                        None      => break,
                    }
                }
                upd = self.midi_rx.recv() => {
                    match upd {
                        Some(upd) => self.emit_all(EngineUpdate::Midi(upd)).await,
                        None      => break,
                    }
                }
                event = self.input_rx.recv() => {
                    match event {
                        Some(event) => self.handle_input_event(event).await,
                        None        => break,
                    }
                }
                _ = drain_interval.tick() => {
                    self.drain_midi_queue();
                }
            }
        }
    }

    /// Drain the RT midi_queue into the open MIDI output connection.
    ///
    /// Called on every conductor tick (~1 ms). No-op if no port is
    /// selected or the queue is empty.
    fn drain_midi_queue(&mut self) {
        let Some(conn) = self.midi_connection.as_mut() else {
            // Drain and discard — don't let the queue fill up with
            // stale events if no port is selected.
            while self.midi_queue.pop().is_some() {}
            return;
        };

        while let Some(bytes) = self.midi_queue.pop() {
            if let Err(e) = conn.send(&bytes) {
                warn!(error = %e, "drain_midi_queue: send failed");
            }
        }
    }
    /// Dispatch a `TransportCommand` to the appropriate handler.
    async fn handle_command(&mut self, cmd: TransportCommand) {
        debug!(?cmd, "handle_command");
        match cmd {
            // ── Transport ─────────────────────────────────────────
            TransportCommand::Play => {
                self.start_clock();
                self.emit_all(EngineUpdate::Audio(AudioUpdate::TransportStarted))
                    .await;
            }

            TransportCommand::Stop => {
                self.stop_clock();
                self.emit_all(EngineUpdate::Audio(AudioUpdate::TransportStopped))
                    .await;
            }

            TransportCommand::SetBpm(bpm) => {
                self.state.set_bpm(bpm);
                debug!(bpm, "SetBpm");
            }

            TransportCommand::Seek(pos) => {
                let tl = self.timeline.read().unwrap();
                self.state.seek(pos, &*tl);
                debug!(pos, "Seek");
            }

            // ── File I/O ──────────────────────────────────────────
            TransportCommand::LoadMidi(path) => {
                let bpm = self.state.bpm();
                let result = load_midi(&path, self.sample_rate, bpm).map_err(|e| e.to_string());
                match result {
                    Ok((events, tpb)) => {
                        let path_str = path.display().to_string();
                        let count = events.len();
                        *self.timeline.write().unwrap() = events;
                        self.ticks_per_beat = tpb;
                        self.state
                            .next_event_idx
                            .store(0, std::sync::atomic::Ordering::Release);
                        debug!(path = %path_str, events = count, tpb, "timeline loaded");
                        self.emit_all(EngineUpdate::Midi(MidiUpdate::LoadMudlAck(path_str)))
                            .await;
                    }
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "LoadMidi failed");
                        self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(format!(
                            "LoadMidi failed: {e}"
                        ))))
                        .await;
                    }
                }
            }

            TransportCommand::LoadMudl(path) => {
                // TODO 8.x: parse KDL via mudld::kdl_to_smf, build timeline
                let _ = path;
                self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(
                    "LoadMudl not yet implemented".into(),
                )))
                .await;
            }

            TransportCommand::Save(path) => {
                let timeline = self.timeline.read().unwrap().clone();
                let bpm = self.state.bpm();
                let sr = self.sample_rate;
                let tpb = self.ticks_per_beat;

                match crate::midi::save::save_timeline(&timeline, &path, bpm, sr, tpb) {
                    Ok(()) => {
                        let path_str = path.display().to_string();
                        info!(path = %path_str, "timeline saved");
                        self.emit_all(EngineUpdate::Midi(MidiUpdate::SaveAck(path_str)))
                            .await;
                    }
                    Err(e) => {
                        warn!(path = %path.display(), error = %e, "Save failed");
                        self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(format!(
                            "Save failed: {e}"
                        ))))
                        .await;
                    }
                }
            }

            // ── Event streaming ───────────────────────────────────
            TransportCommand::QueueEvent(event) => {
                self.queue_event(event);
            }

            TransportCommand::SendMessage(bytes) => {
                self.send_raw_message(bytes).await;
            }

            // ── Recording ─────────────────────────────────────────
            TransportCommand::Monitor { path } => {
                self.start_input(InputMode::Monitor, path).await;
            }

            TransportCommand::Listen { path } => {
                self.start_input(InputMode::Listen, path).await;
            }

            TransportCommand::Ignore => {
                self.stop_input().await;
            }

            TransportCommand::SetAutosavePath(path) => {
                self.pre_roll.set_autosave_path(Some(path));
            }

            TransportCommand::Record { start, path } => {
                if start {
                    self.start_record(path).await;
                } else {
                    self.stop_record(path).await;
                }
            }

            // ── Device selection ──────────────────────────────────
            TransportCommand::MidiPort(name) => {
                self.select_midi_port(name).await;
            }

            TransportCommand::MidiInputPort(name) => {
                self.state.set_midi_input_port(name.clone());
                // If input is currently active, reopen on the new port.
                if self.input_mode != InputMode::Off {
                    let mode = self.input_mode;
                    self.start_input(mode, None).await;
                }
            }

            TransportCommand::AudioHost(name) => {
                self.select_audio_host(name).await;
            }

            TransportCommand::AudioDevice(name) => {
                self.select_audio_device(name).await;
            }

            TransportCommand::AudioDevices => {
                self.enumerate_audio_devices().await;
            }

            TransportCommand::AudioHosts => {
                self.enumerate_audio_hosts().await;
            }

            // ── Enumeration ───────────────────────────────────────
            TransportCommand::MidiDevices => {
                self.enumerate_midi_devices().await;
            }
        }
    }

    // ── MIDI input ────────────────────────────────────────────────────────────

    /// Open the MIDI input port and set the input mode.
    ///
    /// If an input is already open it is closed first — switching
    /// from Monitor to Listen (or vice versa) is a single command.
    /// The autosave path is updated if `path` is `Some`.

    async fn start_input(&mut self, mode: InputMode, path: Option<PathBuf>) {
        self.midi_input = None;

        if let Some(p) = path {
            self.pre_roll.set_autosave_path(Some(p));
        }

        // MidiInputHandle::open reports errors via its Result return value.
        // The status_tx parameter is used only for async errors that occur
        // after the connection is established (e.g. callback failures).
        // We provide a disconnected sender — dropped errors are logged by
        // the callback via `warn!` in input.rs.
        let (status_tx, _status_rx) = mpsc::channel::<MidiUpdate>(1);

        let result = match self.state.midi_input_port() {
            Some(ref name) => MidiInputHandle::open(
                name,
                self.input_tx.clone(),
                Arc::clone(&self.state.playhead),
                status_tx,
            )
            .map(|h| (h, name.clone())),
            None => MidiInputHandle::open_first(
                self.input_tx.clone(),
                Arc::clone(&self.state.playhead),
                status_tx,
            ),
        };

        match result {
            Err(e) => {
                warn!(error = %e, "failed to open MIDI input");
                self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(format!(
                    "failed to open MIDI input: {e}"
                ))))
                .await;
            }
            Ok((handle, opened_name)) => {
                self.midi_input = Some(handle);
                self.input_mode = mode;
                self.pre_roll_active = mode == InputMode::Listen;
                info!(port = %opened_name, ?mode, "MIDI input opened");
                self.emit_all(EngineUpdate::Midi(MidiUpdate::ActivePort(Some(
                    opened_name,
                ))))
                .await;
            }
        }
    }

    /// Close the MIDI input port and reset input state.
    ///
    /// Does not affect the record buffer — a recording in progress
    /// is left intact and must be stopped explicitly via
    /// `Record { start: false }`.
    async fn stop_input(&mut self) {
        self.midi_input = None;
        self.input_mode = InputMode::Off;
        self.pre_roll_active = false;
        debug!("MIDI input closed");
        self.emit_all(EngineUpdate::Midi(MidiUpdate::ActivePort(None)))
            .await;
    }

    /// Handle a single event forwarded from the MIDI input callback.
    ///
    /// Pushes to the pre-roll if listening; appends to the record
    /// buffer if recording.
    async fn handle_input_event(&mut self, event: InputEvent) {
        if self.pre_roll_active {
            self.pre_roll.push(event.clone());
        }

        if let Some(ref mut buf) = self.record_buffer {
            buf.push(event);
        }
    }

    /// Begin a bounded record capture.
    ///
    /// Opens the input port if not already active. If a record is
    /// already in progress the command is a no-op — does not reset
    /// the buffer.
    async fn start_record(&mut self, path: Option<PathBuf>) {
        if self.record_buffer.is_some() {
            warn!("Record(start) received but capture already in progress — ignoring");
            return;
        }

        // Open input if not already active.
        if self.midi_input.is_none() {
            self.start_input(InputMode::Listen, None).await;
            if self.midi_input.is_none() {
                // start_input already emitted an error.
                return;
            }
        }

        // Ensure pre-roll is fed during record regardless of prior mode.
        self.pre_roll_active = true;

        self.record_buffer = Some(Vec::new());
        self.record_path = path;
        debug!("record started");
    }

    /// Stop the bounded record capture and save.
    ///
    /// - Emits `RecordSaved(path)` on success.
    /// - Emits `RecordEmpty` if nothing was captured.
    /// - Emits `Error` if no record was in progress.
    async fn stop_record(&mut self, path_override: Option<PathBuf>) {
        let buffer = match self.record_buffer.take() {
            Some(b) => b,
            None => {
                warn!("Record(stop) received but no capture in progress");
                self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(
                    "Record(stop) called but no capture is in progress".into(),
                )))
                .await;
                return;
            }
        };

        if buffer.is_empty() {
            debug!("record stopped — buffer empty");
            self.emit_all(EngineUpdate::Midi(MidiUpdate::RecordEmpty))
                .await;
            return;
        }

        let path = path_override
            .or_else(|| self.record_path.take())
            .unwrap_or_else(|| {
                let ts = std::time::SystemTime::now()
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                PathBuf::from(format!("mush-record-{ts}.mudl"))
            });

        match crate::midi::preroll::save_events(&buffer, &path) {
            Ok(()) => {
                let path_str = path.display().to_string();
                info!(path = %path_str, events = buffer.len(), "record saved");
                self.emit_all(EngineUpdate::Midi(MidiUpdate::RecordSaved(path_str)))
                    .await;
            }
            Err(e) => {
                warn!(error = %e, "record save failed");
                self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(format!(
                    "record save failed: {e}"
                ))))
                .await;
            }
        }
    }

    // ── Clock ─────────────────────────────────────────────────────────────────

    /// Start the master clock.
    ///
    /// Constructs `ClockArgs` from the Conductor's own fields and
    /// calls `Clock::start`. Stores the result in `self.clock`.
    ///
    /// If the clock is already running this is a no-op — a second
    /// `Play` command does not restart playback.
    fn start_clock(&mut self) {
        if self.clock.is_some() {
            debug!("Play received but clock already running — ignoring");
            return;
        }

        self.state.play();

        let args = ClockArgs {
            playhead: Arc::clone(&self.state.playhead),
            is_playing: Arc::clone(&self.state.is_playing),
            next_event_idx: Arc::clone(&self.state.next_event_idx),
            timeline: Arc::clone(&self.timeline),
            midi_queue: Arc::clone(&self.midi_queue),
            status_tx: self.audio_tx.clone(),
            sample_rate: self.sample_rate,
            extract_midi: extract_midi_payload,
            host_name: self.state.audio_host(),
            device_name: self.state.audio_device(),
        };

        self.clock = Some(Clock::start(args));
        debug!(
            host   = ?self.state.audio_host(),
            device = ?self.state.audio_device(),
            "clock started",
        );
    }

    /// Stop the master clock.
    ///
    /// Dropping `Clock` triggers its RAII stop. Sets `is_playing`
    /// to false so the audio callback exits its dispatch loop cleanly
    /// before the stream is dropped.
    fn stop_clock(&mut self) {
        self.state.stop();
        self.clock = None;
        debug!("clock stopped");
    }

    // ── MIDI output ───────────────────────────────────────────────────────────

    /// Enumerate MIDI ports and emit a formatted `PortList`.
    ///
    /// Always enumerates fresh from the OS — no cache.
    async fn enumerate_midi_devices(&mut self) {
        let result = list_midi_ports().map_err(|e| e.to_string());
        let update = match result {
            Err(e) => {
                warn!(error = %e, "MidiDevices enumeration failed");
                MidiUpdate::Error(format!("failed to enumerate MIDI ports: {e}"))
            }
            Ok(ref ports) => {
                let listing = list_enumerated_devices(ports)
                    .unwrap_or_else(|| "no MIDI devices found".into());
                MidiUpdate::PortList(listing)
            }
        };
        self.emit_all(EngineUpdate::Midi(update)).await;
    }

    /// Select a MIDI output port by name, or clear the current selection.
    ///
    /// - `None`       — drop the current connection, emit `ActivePort(None)`
    /// - `Some(name)` — open a connection to the named port, emit
    ///                  `ActivePort(Some(name))` on success or `Error` on failure
    async fn select_midi_port(&mut self, name: Option<String>) {
        match name {
            None => {
                self.midi_connection = None;
                debug!("MIDI output port cleared");
                self.emit_all(EngineUpdate::Midi(MidiUpdate::ActivePort(None)))
                    .await;
            }
            Some(ref port_name) => match open_midi_port_by_name(port_name) {
                Ok(conn) => {
                    self.midi_connection = Some(Box::new(conn));
                    debug!(port = %port_name, "MIDI output port opened");
                    self.emit_all(EngineUpdate::Midi(MidiUpdate::ActivePort(Some(
                        port_name.clone(),
                    ))))
                    .await;
                }
                Err(e) => {
                    warn!(port = %port_name, error = %e, "failed to open MIDI port");
                    self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(format!(
                        "failed to open MIDI port '{port_name}': {e}"
                    ))))
                    .await;
                }
            },
        }
    }

    /// Append a `MidiEvent` (in beats) to the timeline as a note-on /
    /// note-off pair, then re-sort.
    ///
    /// Beat-to-sample conversion uses the current BPM and sample rate.
    fn queue_event(&self, event: MidiEvent) {
        let bpm = self.state.bpm();
        let sr = self.sample_rate as f64;
        let secs_per_beat = 60.0 / bpm;

        let note_on_sample = (event.beat * secs_per_beat * sr) as u64;
        let note_off_sample = ((event.beat + event.duration) * secs_per_beat * sr) as u64;

        let ch = event.channel & 0x0F;

        let note_on = TimelineEvent {
            sample_pos: note_on_sample,
            payload: MidiPayload::Midi([0x90 | ch, event.note, event.vel]),
        };
        let note_off = TimelineEvent {
            sample_pos: note_off_sample,
            payload: MidiPayload::Midi([0x80 | ch, event.note, 0]),
        };

        let mut tl = self.timeline.write().unwrap();
        tl.push(note_on);
        tl.push(note_off);
        tl.sort_by_key(|e| e.sample_pos);
    }

    /// Send raw MIDI bytes directly to the open output connection.
    ///
    /// Emits `Error` if no port is selected — does not fall back to
    ///
    /// Send raw MIDI bytes directly to the open output connection.
    ///
    /// Emits `Error` if no port is selected — does not fall back to
    /// `midi_queue`. The queue is solely for the RT dispatch path.
    async fn send_raw_message(&mut self, bytes: Vec<u8>) {
        if bytes.is_empty() || bytes.len() > 3 {
            warn!(len = bytes.len(), "SendMessage: invalid byte length");
            self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(format!(
                "SendMessage: invalid byte length {}",
                bytes.len()
            ))))
            .await;
            return;
        }

        match self.midi_connection.as_mut() {
            None => {
                warn!("SendMessage: no MIDI output port selected");
                self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(
                    "no MIDI output port selected — use 'midi port <name>' first".into(),
                )))
                .await;
            }
            Some(conn) => match conn.send(&bytes) {
                Ok(()) => {
                    let channel = bytes[0] & 0x0F;
                    let note = bytes.get(1).copied().unwrap_or(0);
                    let vel = bytes.get(2).copied().unwrap_or(0);
                    self.emit_all(EngineUpdate::Midi(MidiUpdate::EventDispatched {
                        channel,
                        note,
                        vel,
                    }))
                    .await;
                }
                Err(e) => {
                    warn!(error = %e, "SendMessage: MIDI send failed");
                    self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(format!(
                        "MIDI send failed: {e}"
                    ))))
                    .await;
                }
            },
        }
    }

    // ── Audio host / device ───────────────────────────────────────────────────

    /// Enumerate available audio hosts and emit `HostList`.
    ///
    /// Always enumerates fresh from cpal — no cache.
    async fn enumerate_audio_hosts(&mut self) {
        match crate::audio::enumerate::enumerate_hosts_and_devices() {
            Err(e) => {
                warn!(error = %e, "AudioHosts enumeration failed");
                self.emit_all(EngineUpdate::Audio(AudioUpdate::Error(format!(
                    "failed to enumerate audio hosts: {e}"
                ))))
                .await;
            }
            Ok(info) => {
                let names = info.hosts.iter().map(|h| h.name.clone()).collect();
                self.emit_all(EngineUpdate::Audio(AudioUpdate::HostList(names)))
                    .await;
            }
        }
    }

    /// Enumerate available audio devices on the active host and emit `DeviceList`.
    ///
    /// If no host has been selected, uses the first available host.
    /// Always enumerates fresh from cpal — no cache.

    async fn enumerate_audio_devices(&mut self) {
        match crate::audio::enumerate::enumerate_hosts_and_devices() {
            Err(e) => {
                warn!(error = %e, "AudioDevices enumeration failed");
                self.emit_all(EngineUpdate::Audio(AudioUpdate::Error(format!(
                    "failed to enumerate audio devices: {e}"
                ))))
                .await;
            }
            Ok(info) => {
                let active_host = self.state.audio_host();

                let host_info = match active_host {
                    Some(ref name) => info.hosts.iter().find(|h| &h.name == name),
                    None => info.hosts.first(),
                };

                match host_info {
                    None => {
                        self.emit_all(EngineUpdate::Audio(AudioUpdate::Error(
                            "no audio hosts available".into(),
                        )))
                        .await;
                    }
                    Some(host) => {
                        // Build a single-host AudioSystemInfo so we can
                        // reuse the existing list_hosts_and_devices formatter.
                        let single = crate::audio::types::AudioSystemInfo {
                            hosts: vec![host.clone()],
                        };
                        let listing = crate::audio::display::list_hosts_and_devices(&single);
                        self.emit_all(EngineUpdate::Audio(AudioUpdate::DeviceList(listing)))
                            .await;
                    }
                }
            }
        }
    }

    /// Query or set the active audio host.
    ///
    /// - `None`       — emit `ActiveHost` with the current selection.
    /// - `Some(name)` — validate the name exists, set it, emit `ActiveHost`.
    ///                  Emits `Error` if the name is not a known host.
    async fn select_audio_host(&mut self, name: Option<String>) {
        match name {
            None => {
                self.emit_all(EngineUpdate::Audio(AudioUpdate::ActiveHost(
                    self.state.audio_host(),
                )))
                .await;
            }
            Some(ref host_name) => match crate::audio::enumerate::enumerate_hosts_and_devices() {
                Err(e) => {
                    warn!(error = %e, "AudioHost enumeration failed during set");
                    self.emit_all(EngineUpdate::Audio(AudioUpdate::Error(format!(
                        "failed to enumerate audio hosts: {e}"
                    ))))
                    .await;
                }
                Ok(info) => {
                    let known = info.hosts.iter().any(|h| &h.name == host_name);
                    if known {
                        self.state.set_audio_host(Some(host_name.clone()));
                        debug!(host = %host_name, "audio host selected");
                        self.emit_all(EngineUpdate::Audio(AudioUpdate::ActiveHost(Some(
                            host_name.clone(),
                        ))))
                        .await;
                    } else {
                        warn!(host = %host_name, "AudioHost: unknown host name");
                        self.emit_all(EngineUpdate::Audio(AudioUpdate::Error(format!(
                            "unknown audio host '{host_name}'"
                        ))))
                        .await;
                    }
                }
            },
        }
    }

    /// Query or set the active audio output device.
    ///
    /// - `None`       — emit `ActiveDevice` with the current selection.
    /// - `Some(name)` — validate the name exists on the active host,
    ///                  set it, emit `ActiveDevice`.
    ///                  Emits `Error` if the name is not found.
    async fn select_audio_device(&mut self, name: Option<String>) {
        match name {
            None => {
                self.emit_all(EngineUpdate::Audio(AudioUpdate::ActiveDevice(
                    self.state.audio_device(),
                )))
                .await;
            }
            Some(ref device_name) => match crate::audio::enumerate::enumerate_hosts_and_devices() {
                Err(e) => {
                    warn!(error = %e, "AudioDevice enumeration failed during set");
                    self.emit_all(EngineUpdate::Audio(AudioUpdate::Error(format!(
                        "failed to enumerate audio devices: {e}"
                    ))))
                    .await;
                }
                Ok(info) => {
                    let active_host = self.state.audio_host();

                    let host_info = match active_host {
                        Some(ref name) => info.hosts.iter().find(|h| &h.name == name),
                        None => info.hosts.first(),
                    };

                    match host_info {
                        None => {
                            self.emit_all(EngineUpdate::Audio(AudioUpdate::Error(
                                "no audio hosts available".into(),
                            )))
                            .await;
                        }
                        Some(host) => {
                            let known = host.devices.iter().any(|d| &d.description == device_name);

                            if known {
                                self.state.set_audio_device(Some(device_name.clone()));
                                debug!(device = %device_name, "audio device selected");
                                self.emit_all(EngineUpdate::Audio(AudioUpdate::ActiveDevice(
                                    Some(device_name.clone()),
                                )))
                                .await;
                            } else {
                                warn!(device = %device_name, "AudioDevice: unknown device");
                                self.emit_all(EngineUpdate::Audio(AudioUpdate::Error(format!(
                                    "unknown audio device '{device_name}'"
                                ))))
                                .await;
                            }
                        }
                    }
                }
            },
        }
    }

    /// Fan out an `EngineUpdate` to all subscribers and the ZMQ PUB socket.
    ///
    /// Removes dead subscriptions (dropped handles) via `retain()`.
    async fn emit_all(&mut self, update: EngineUpdate) {
        debug!(?update, subs = self.subs.len(), "emit_all");
        self.subs.retain(|s| !s.is_closed());
        for sub in &self.subs {
            if let Err(()) = sub.send(update.clone()) {
                warn!(?update, "subscription send failed — channel full or closed");
            }
        }
        let _ = self.transport.send_update(&update).await;
    }
}

// ── free functions ────────────────────────────────────────────────────────────

/// Extract raw MIDI bytes from a `MidiPayload`.
///
/// Passed as a function pointer to `ClockArgs::extract_midi`.
/// Returns `None` for non-MIDI payloads (e.g. `TempoChange`).
/// Zero-cost, stateless, safe to call from the RT path.
fn extract_midi_payload(p: &MidiPayload) -> Option<[u8; 3]> {
    match p {
        MidiPayload::Midi(bytes) => Some(*bytes),
        MidiPayload::TempoChange(_) => None,
    }
}

/// Open a MIDI output connection to the port matching `name`.
///
/// Creates a fresh `MidiOutput` handle for enumeration — midir
/// consumes the handle on `connect()` so it cannot be reused.
///
/// Returns `Err` if no port with that name exists or the connection fails.
fn open_midi_port_by_name(name: &str) -> Result<DesktopMidiOutput, String> {
    let output = MidirOutput::new("mush-midi-out").map_err(|e| e.to_string())?;
    let ports = output.ports();

    let port = ports
        .iter()
        .find(|p| output.port_name(p).as_deref() == Ok(name))
        .ok_or_else(|| format!("MIDI port '{name}' not found"))?;

    let connection = output.connect(port, name).map_err(|e| e.to_string())?;

    Ok(DesktopMidiOutput::from_connection(connection))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::addrs;
    use crate::update::{AudioUpdate, EngineUpdate, MidiUpdate};

    const SAMPLE_RATE: u32 = 44_100;

    fn make_conductor() -> (
        Conductor,
        Arc<ArrayQueue<[u8; 3]>>,
        mpsc::Sender<AudioUpdate>,
        mpsc::Sender<MidiUpdate>,
    ) {
        Conductor::new(addrs::CMD_INPROC, addrs::PUB_INPROC, SAMPLE_RATE).unwrap()
    }

    // 6.3.1 — Play command starts clock and emits TransportStarted
    #[tokio::test]
    async fn play_emits_transport_started() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor.handle_command(TransportCommand::Play).await;
        let update = handle.recv().await.unwrap();
        assert_eq!(update, EngineUpdate::Audio(AudioUpdate::TransportStarted));
        assert!(conductor.clock.is_some());
        assert!(
            conductor
                .state
                .is_playing
                .load(std::sync::atomic::Ordering::Acquire)
        );
    }

    // 6.3.2 — Stop command drops clock and emits TransportStopped
    #[tokio::test]
    async fn stop_emits_transport_stopped() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor.handle_command(TransportCommand::Play).await;
        let _ = handle.recv().await.unwrap();
        conductor.handle_command(TransportCommand::Stop).await;
        let update = handle.recv().await.unwrap();
        assert_eq!(update, EngineUpdate::Audio(AudioUpdate::TransportStopped));
        assert!(conductor.clock.is_none());
        assert!(
            !conductor
                .state
                .is_playing
                .load(std::sync::atomic::Ordering::Acquire)
        );
    }

    // 6.3.3 — Second Play is a no-op (clock already running)
    #[tokio::test]
    async fn double_play_is_noop() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor.handle_command(TransportCommand::Play).await;
        let clock_ptr = conductor.clock.as_ref().map(|c| c as *const _);
        conductor.handle_command(TransportCommand::Play).await;
        assert_eq!(clock_ptr, conductor.clock.as_ref().map(|c| c as *const _));
    }

    // 8.3.7 — SetBpm updates TransportState
    #[tokio::test]
    async fn set_bpm_updates_state() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor
            .handle_command(TransportCommand::SetBpm(140.0))
            .await;
        assert!((conductor.state.bpm() - 140.0).abs() < 0.001);
    }

    // 8.3.7 — Seek updates playhead and next_event_idx
    #[tokio::test]
    async fn seek_updates_playhead() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor
            .handle_command(TransportCommand::Seek(88_200))
            .await;
        assert_eq!(
            conductor
                .state
                .playhead
                .load(std::sync::atomic::Ordering::Acquire),
            88_200
        );
    }

    // 8.3.7 — QueueEvent uses live BPM from TransportState
    #[tokio::test]
    async fn queue_event_uses_live_bpm() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor
            .handle_command(TransportCommand::SetBpm(60.0))
            .await;
        conductor.queue_event(MidiEvent {
            beat: 1.0,
            channel: 1,
            note: 60,
            vel: 100,
            duration: 1.0,
        });
        let tl = conductor.timeline.read().unwrap();
        assert_eq!(tl[0].sample_pos, 44_100);
    }

    // 6.3.5 — Update reaches SubscriptionHandle
    #[tokio::test]
    async fn update_reaches_subscription_handle() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor
            .emit_all(EngineUpdate::Audio(AudioUpdate::PlayheadMoved(44_100)))
            .await;
        let update = handle.recv().await.unwrap();
        assert_eq!(
            update,
            EngineUpdate::Audio(AudioUpdate::PlayheadMoved(44_100))
        );
    }

    // 6.3.6 — Closed subscription is removed cleanly
    #[tokio::test]
    async fn closed_subscription_removed_cleanly() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let handle = conductor.subscribe();
        drop(handle);
        conductor
            .emit_all(EngineUpdate::Audio(AudioUpdate::TransportStopped))
            .await;
        assert_eq!(conductor.subs.len(), 0);
    }

    // 6.3.10 — Multiple subscriptions all receive update
    #[tokio::test]
    async fn multiple_subscriptions_all_receive_update() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle_a = conductor.subscribe();
        let mut handle_b = conductor.subscribe();
        let mut handle_c = conductor.subscribe();
        conductor
            .emit_all(EngineUpdate::Audio(AudioUpdate::TransportStarted))
            .await;
        let expected = EngineUpdate::Audio(AudioUpdate::TransportStarted);
        assert_eq!(handle_a.recv().await.unwrap(), expected);
        assert_eq!(handle_b.recv().await.unwrap(), expected);
        assert_eq!(handle_c.recv().await.unwrap(), expected);
    }

    // 8.3.5 — Timeline is empty on construction
    #[tokio::test]
    async fn timeline_empty_on_construction() {
        let (conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        assert!(conductor.timeline.read().unwrap().is_empty());
    }

    // 8.3.5 — QueueEvent appends note-on and note-off to timeline
    #[tokio::test]
    async fn queue_event_appends_note_pair() {
        let (conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor.queue_event(MidiEvent {
            beat: 1.0,
            channel: 1,
            note: 60,
            vel: 100,
            duration: 0.5,
        });
        let tl = conductor.timeline.read().unwrap();
        assert_eq!(tl.len(), 2);
        assert!(matches!(tl[0].payload, MidiPayload::Midi([0x91, 60, 100])));
        assert!(matches!(tl[1].payload, MidiPayload::Midi([0x81, 60, 0])));
    }

    // 8.3.5 — Timeline remains sorted after multiple QueueEvent calls
    #[tokio::test]
    async fn queue_event_timeline_stays_sorted() {
        let (conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor.queue_event(MidiEvent {
            beat: 3.0,
            channel: 1,
            note: 64,
            vel: 80,
            duration: 0.25,
        });
        conductor.queue_event(MidiEvent {
            beat: 1.0,
            channel: 1,
            note: 60,
            vel: 100,
            duration: 0.5,
        });
        conductor.queue_event(MidiEvent {
            beat: 2.0,
            channel: 1,
            note: 62,
            vel: 90,
            duration: 0.5,
        });
        let tl = conductor.timeline.read().unwrap();
        assert!(
            tl.windows(2).all(|w| w[0].sample_pos <= w[1].sample_pos),
            "timeline is not sorted after multiple QueueEvent calls"
        );
    }

    // 8.3.5 — LoadMidi with a valid fixture populates the timeline
    #[tokio::test]
    async fn load_midi_populates_timeline() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        let path = std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/test.mid"
        ));
        conductor
            .handle_command(TransportCommand::LoadMidi(path))
            .await;
        assert!(!conductor.timeline.read().unwrap().is_empty());
        let update = handle.recv().await.unwrap();
        assert!(matches!(
            update,
            EngineUpdate::Midi(MidiUpdate::LoadMudlAck(_))
        ));
    }

    // 8.3.6 — MidiDevices emits PortList
    #[tokio::test]
    async fn midi_devices_emits_port_list() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor
            .handle_command(TransportCommand::MidiDevices)
            .await;
        let update = handle.recv().await.unwrap();
        assert!(matches!(
            update,
            EngineUpdate::Midi(MidiUpdate::PortList(_))
        ));
    }

    // 8.3.6 — MidiPort(None) emits ActivePort(None) and clears connection
    #[tokio::test]
    async fn midi_port_none_clears_connection() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor
            .handle_command(TransportCommand::MidiPort(None))
            .await;
        let update = handle.recv().await.unwrap();
        assert_eq!(update, EngineUpdate::Midi(MidiUpdate::ActivePort(None)));
        assert!(conductor.midi_connection.is_none());
    }

    // 8.3.6 — MidiPort(Some(invalid)) emits Error
    #[tokio::test]
    async fn midi_port_invalid_name_emits_error() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor
            .handle_command(TransportCommand::MidiPort(Some(
                "nonexistent-port-xyz".into(),
            )))
            .await;
        let update = handle.recv().await.unwrap();
        assert!(matches!(update, EngineUpdate::Midi(MidiUpdate::Error(_))));
        assert!(conductor.midi_connection.is_none());
    }

    // 8.3.6 — SendMessage with no port selected emits Error
    #[tokio::test]
    async fn send_message_no_port_emits_error() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor
            .handle_command(TransportCommand::SendMessage(vec![0x90, 60, 100]))
            .await;
        let update = handle.recv().await.unwrap();
        assert!(matches!(update, EngineUpdate::Midi(MidiUpdate::Error(_))));
    }

    // 8.3.6 — SendMessage with invalid byte length emits Error
    #[tokio::test]
    async fn send_message_invalid_length_emits_error() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor
            .handle_command(TransportCommand::SendMessage(vec![0x90, 60, 100, 0]))
            .await;
        let update = handle.recv().await.unwrap();
        assert!(matches!(update, EngineUpdate::Midi(MidiUpdate::Error(_))));
    }

    // 8.3.8 — Conductor starts with input mode Off
    #[tokio::test]
    async fn input_mode_off_on_construction() {
        let (conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        assert_eq!(conductor.input_mode, InputMode::Off);
        assert!(conductor.midi_input.is_none());
        assert!(!conductor.pre_roll_active);
        assert!(conductor.record_buffer.is_none());
    }

    // 8.3.8 — Ignore with no active input is a no-op (no panic)
    #[tokio::test]
    async fn ignore_with_no_input_is_noop() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor.handle_command(TransportCommand::Ignore).await;
        // Emits ActivePort(None) even when already off.
        let update = handle.recv().await.unwrap();
        assert_eq!(update, EngineUpdate::Midi(MidiUpdate::ActivePort(None)));
        assert_eq!(conductor.input_mode, InputMode::Off);
    }

    // 8.3.8 — Record(stop) with no active capture emits Error
    #[tokio::test]
    async fn record_stop_without_start_emits_error() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor
            .handle_command(TransportCommand::Record {
                start: false,
                path: None,
            })
            .await;
        let update = handle.recv().await.unwrap();
        assert!(matches!(update, EngineUpdate::Midi(MidiUpdate::Error(_))));
    }

    // 8.3.8 — SetAutosavePath updates pre-roll path
    #[tokio::test]
    async fn set_autosave_path_updates_preroll() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let path = PathBuf::from("/tmp/test-autosave.mudl");
        conductor
            .handle_command(TransportCommand::SetAutosavePath(path.clone()))
            .await;
        assert_eq!(conductor.pre_roll.next_autosave_path(), path);
    }

    // 8.3.8 — MidiInputPort updates TransportState
    #[tokio::test]
    async fn midi_input_port_updates_state() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor
            .handle_command(TransportCommand::MidiInputPort(Some("Test Port".into())))
            .await;
        assert_eq!(conductor.state.midi_input_port(), Some("Test Port".into()));
    }

    // 8.3.8 — pre-roll is empty on construction
    #[tokio::test]
    async fn pre_roll_empty_on_construction() {
        let (conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        assert!(conductor.pre_roll.is_empty());
    }

    // 8.3.8 — handle_input_event feeds pre-roll when pre_roll_active
    #[tokio::test]
    async fn handle_input_event_feeds_preroll_when_active() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor.pre_roll_active = true;
        let event = InputEvent {
            wall_time: std::time::Instant::now(),
            sample_pos: 0,
            bytes: [0x90, 60, 100],
        };
        conductor.handle_input_event(event).await;
        assert_eq!(conductor.pre_roll.len(), 1);
    }

    // 8.3.8 — handle_input_event does not feed pre-roll when inactive
    #[tokio::test]
    async fn handle_input_event_skips_preroll_when_inactive() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor.pre_roll_active = false;
        let event = InputEvent {
            wall_time: std::time::Instant::now(),
            sample_pos: 0,
            bytes: [0x90, 60, 100],
        };
        conductor.handle_input_event(event).await;
        assert!(conductor.pre_roll.is_empty());
    }

    // 8.3.8 — handle_input_event feeds record buffer when recording
    #[tokio::test]
    async fn handle_input_event_feeds_record_buffer() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor.record_buffer = Some(Vec::new());
        let event = InputEvent {
            wall_time: std::time::Instant::now(),
            sample_pos: 0,
            bytes: [0x90, 60, 100],
        };
        conductor.handle_input_event(event).await;
        assert_eq!(conductor.record_buffer.as_ref().unwrap().len(), 1);
    }
}
