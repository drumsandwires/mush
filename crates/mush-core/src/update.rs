use crate::audio::types::AudioSystemInfo;
use crate::midi::types::MidiPortInfo;
use serde::{Deserialize, Serialize};

/// Updates emitted by the audio engine or Conductor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioUpdate {
    /// Playhead advanced to this sample position.
    /// Emitted periodically — not on every buffer.
    PlayheadMoved(u64),

    /// Transport started playing.
    TransportStarted,

    /// Transport stopped.
    TransportStopped,

    /// Response to `TransportCommand::AudioDevices`.
    /// Formatted string at MVP — retire when frontends use `DeviceList`.
    DeviceList(AudioSystemInfo),

    /// Response to `TransportCommand::AudioHosts`.
    /// Structured list of host names available on this platform.
    HostList(Vec<String>),

    /// Response to `TransportCommand::AudioHost(None/Some)`.
    /// `None` — no host selected; platform default is in use.
    /// `Some(name)` — this host is currently active.
    ActiveHost(Option<String>),

    /// Response to `TransportCommand::AudioDevice(None/Some)`.
    /// `None` — no device selected; host default is in use.
    /// `Some(name)` — this device is currently active.
    ActiveDevice(Option<String>),

    /// A non-fatal error from the audio subsystem.
    Error(String),
}

/// Updates emitted by the MIDI engine or Conductor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MidiUpdate {
    /// A MIDI event was dispatched to the output port.
    EventDispatched { channel: u8, note: u8, vel: u8 },

    /// Response to `TransportCommand::MidiDevices`.
    /// Formatted string at MVP — retire when frontends use `ActivePort`.
    PortList(Vec<MidiPortInfo>),

    /// Response to `TransportCommand::MidiPortGet`.
    /// Returns current input and output assignments together.
    /// `None` in either field — no port selected for that direction.
    PortAssignments {
        input: Option<String>,
        output: Option<String>,
    },

    /// Response to `TransportCommand::MidiPort(None/Some)`.
    /// `None` — no port selected; first available is in use.
    /// `Some(name)` — this port is currently active.
    ActivePort(Option<String>),

    /// Response to `TransportCommand::MidiInputPort`.
    /// `None` — input port cleared.
    /// `Some(name)` — this input port is now active.
    ActiveInputPort(Option<String>),

    /// Acknowledgement that a file was saved successfully.
    /// Payload is the path as a display string.
    SaveAck(String),

    /// Acknowledgement that a .mudl file was loaded successfully.
    /// Payload is the path as a display string.
    LoadMudlAck(String),

    /// Capture started successfully — MIDI input is now open and recording.
    RecordStarted,

    /// Capture stopped and the recording was saved successfully.
    /// Payload is the absolute path as a display string.
    RecordSaved(String, usize),

    /// Capture stopped but no events were recorded — nothing written.
    RecordEmpty,

    /// A non-fatal error from the MIDI subsystem.
    Error(String),
}

/// Snapshot of conductor transport state for display purposes.
///
/// Emitted by the conductor whenever transport mode changes —
/// on Play, Stop, Record start, Record stop, and Monitor start/stop.
///
/// Intentionally a snapshot rather than individual field updates so the
/// live view TUI can redraw the header from a single message without
/// assembling state from multiple update streams.
///
/// # Not emitted at audio-callback rate
/// This is a status update, not a per-sample event. It is emitted only
/// on discrete state transitions. The live view TUI polls this to update
/// its header line.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TransportStatus {
    pub is_playing: bool,
    pub is_recording: bool,
    /// Current BPM, derived from `bpm_milli / 1000.0`.
    pub bpm: f64,
}

/// The top-level update type flowing from engine to frontends.
/// Wraps `AudioUpdate` and `MidiUpdate` so frontends receive a
/// single stream of updates regardless of source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EngineUpdate {
    Audio(AudioUpdate),
    Midi(MidiUpdate),
    /// Transport mode changed — playing, recording, stopped, BPM.
    /// See [`TransportStatus`].
    Transport(TransportStatus),
}
