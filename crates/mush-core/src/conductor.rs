use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use midir::MidiOutput as MidirOutput;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::midi::display::list_enumerated_devices;
use crate::midi::engine::DesktopMidiOutput;
use crate::midi::enumerate::list_midi_ports;
use crate::midi::timeline::{MidiPayload, load_midi};
use crate::midi_output::MidiOutput;
use crate::subscription::{Subscription, subscription};
use crate::timeline::TimelineEvent;
use crate::transport::{MidiEvent, TransportCommand};
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
    midi_rx: mpsc::Receiver<MidiUpdate>,
    subs: Vec<Subscription>,

    /// Sample rate passed at construction — used for beat-to-sample
    /// conversion in `QueueEvent` and `LoadMidi`.
    sample_rate: u32,

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

        let midi_queue = Arc::new(ArrayQueue::new(MIDI_QUEUE_CAPACITY));

        let conductor = Self {
            ctx,
            transport,
            audio_rx,
            midi_rx,
            subs: Vec::new(),
            sample_rate,
            timeline: Arc::new(std::sync::RwLock::new(Vec::new())),
            midi_queue: Arc::clone(&midi_queue),
            midi_connection: None,
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
            }
        }
    }

    /// Dispatch a `TransportCommand` to the appropriate handler.
    async fn handle_command(&mut self, cmd: TransportCommand) {
        debug!(?cmd, "handle_command");
        match cmd {
            // ── Transport ─────────────────────────────────────────
            TransportCommand::Play => {
                self.emit_all(EngineUpdate::Audio(AudioUpdate::TransportStarted))
                    .await;
            }

            TransportCommand::Stop => {
                self.emit_all(EngineUpdate::Audio(AudioUpdate::TransportStopped))
                    .await;
            }

            TransportCommand::SetBpm(bpm) => {
                // TODO 8.x: update TransportState::bpm_milli
                let _ = bpm;
            }

            TransportCommand::Seek(pos) => {
                // TODO 8.x: update TransportState::playhead and next_event_idx
                let _ = pos;
            }

            // ── File I/O ──────────────────────────────────────────
            TransportCommand::LoadMidi(path) => {
                let bpm = 120.0; // TODO 8.x: read from TransportState
                let result = load_midi(&path, self.sample_rate, bpm).map_err(|e| e.to_string());
                match result {
                    Ok(events) => {
                        let path_str = path.display().to_string();
                        let count = events.len();
                        *self.timeline.write().unwrap() = events;
                        debug!(path = %path_str, events = count, "timeline loaded");
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
                // Reserved — task 8 (KDL parser)
                let _ = path;
                self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(
                    "LoadMudl not yet implemented".into(),
                )))
                .await;
            }

            TransportCommand::Save(path) => {
                // TODO 8.x: serialise timeline to .mid or .mudl
                let _ = path;
                self.emit_all(EngineUpdate::Midi(MidiUpdate::Error(
                    "Save not yet implemented".into(),
                )))
                .await;
            }

            // ── Event streaming ───────────────────────────────────
            TransportCommand::QueueEvent(event) => {
                self.queue_event(event);
            }

            TransportCommand::SendMessage(bytes) => {
                self.send_raw_message(bytes).await;
            }

            // ── Recording ─────────────────────────────────────────
            TransportCommand::Record { start, path } => {
                // TODO 8.x: start/stop capture, save to path
                let _ = (start, path);
            }

            // ── Device selection ──────────────────────────────────
            TransportCommand::MidiPort(name) => {
                self.select_midi_port(name).await;
            }

            TransportCommand::AudioHost(name) => {
                // TODO 8.x: set or query active_audio_host on TransportState
                let _ = name;
                self.emit_all(EngineUpdate::Audio(AudioUpdate::ActiveHost(None)))
                    .await;
            }

            TransportCommand::AudioDevice(name) => {
                // TODO 8.x: set or query active_audio_device on TransportState
                let _ = name;
                self.emit_all(EngineUpdate::Audio(AudioUpdate::ActiveDevice(None)))
                    .await;
            }

            // ── Enumeration ───────────────────────────────────────
            TransportCommand::AudioDevices => {
                // TODO 8.x: enumerate audio devices, emit DeviceList
            }

            TransportCommand::MidiDevices => {
                self.enumerate_midi_devices().await;
            }

            TransportCommand::AudioHosts => {
                // TODO 8.x: enumerate cpal hosts, emit HostList
                self.emit_all(EngineUpdate::Audio(AudioUpdate::HostList(vec![])))
                    .await;
            }
        }
    }

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
        let bpm = 120.0; // TODO 8.x: read from TransportState
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

    /// Helper — create a Conductor bound on inproc addresses.
    fn make_conductor() -> (
        Conductor,
        Arc<ArrayQueue<[u8; 3]>>,
        mpsc::Sender<AudioUpdate>,
        mpsc::Sender<MidiUpdate>,
    ) {
        Conductor::new(addrs::CMD_INPROC, addrs::PUB_INPROC, SAMPLE_RATE).unwrap()
    }

    // 6.3.1 — Play command emits TransportStarted
    #[tokio::test]
    async fn play_emits_transport_started() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor.handle_command(TransportCommand::Play).await;
        let update = handle.recv().await.unwrap();
        assert_eq!(update, EngineUpdate::Audio(AudioUpdate::TransportStarted));
    }

    // 6.3.2 — Stop command emits TransportStopped
    #[tokio::test]
    async fn stop_emits_transport_stopped() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        let mut handle = conductor.subscribe();
        conductor.handle_command(TransportCommand::Stop).await;
        let update = handle.recv().await.unwrap();
        assert_eq!(update, EngineUpdate::Audio(AudioUpdate::TransportStopped));
    }

    // 6.3.3 — SetBpm does not panic (stub)
    #[tokio::test]
    async fn set_bpm_does_not_panic() {
        let (mut conductor, _q, _audio_tx, _midi_tx) = make_conductor();
        conductor
            .handle_command(TransportCommand::SetBpm(120.0))
            .await;
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
}

