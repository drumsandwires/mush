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
    DeviceList(String),

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
    PortList(String),

    /// Response to `TransportCommand::MidiPort(None/Some)`.
    /// `None` — no port selected; first available is in use.
    /// `Some(name)` — this port is currently active.
    ActivePort(Option<String>),

    /// Acknowledgement that a file was saved successfully.
    /// Payload is the path as a display string.
    SaveAck(String),

    /// Acknowledgement that a .mudl file was loaded successfully.
    /// Payload is the path as a display string.
    LoadMudlAck(String),

    /// Capture stopped and the recording was saved successfully.
    /// Payload is the absolute path as a display string.
    RecordSaved(String),

    /// Capture stopped but no events were recorded — nothing written.
    RecordEmpty,

    /// A non-fatal error from the MIDI subsystem.
    Error(String),
}

/// The top-level update type flowing from engine to frontends.
/// Wraps `AudioUpdate` and `MidiUpdate` so frontends receive a
/// single stream of updates regardless of source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EngineUpdate {
    Audio(AudioUpdate),
    Midi(MidiUpdate),
}

