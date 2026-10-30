// Updates flow from the EngineMonitor outward to all frontends.
// The direction is opposite to TransportCommand:
//
//   TransportCommand  — frontend → engine
//   EngineUpdate      — engine → frontend

use serde::{Deserialize, Serialize};

// For now, device list information is carried as a String
// and will be replaced with a structured type.

/// Updates emitted by the audio engine or EngineMonitor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AudioUpdate {
    /// Playhead advanced to this sample position.
    /// Emitted periodically — not on every buffer.
    PlayheadMoved(u64),

    /// Transport started playing.
    TransportStarted,

    /// Transport stopped.
    TransportStopped,

    /// Response to TransportCommand::AudioDevices.
    /// Payload is a formatted string at MVP — replace with
    /// structured AudioHostInfo.
    DeviceList(String),

    /// A non-fatal error from the audio subsystem.
    Error(String),
}

/// Updates emitted by the MIDI engine or EngineMonitor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MidiUpdate {
    /// A MIDI event was dispatched to the output port.
    EventDispatched { channel: u8, note: u8, vel: u8 },

    /// Response to TransportCommand::MidiDevices.
    /// Payload is a formatted string at MVP — replaced with
    /// structured MidiPortInfo.
    PortList(String),

    /// A non-fatal error from the MIDI subsystem.
    Error(String),
}

/// The top-level update type flowing from engine to frontends.
/// Wraps AudioUpdate and MidiUpdate so frontends receive a single
/// stream of updates regardless of source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EngineUpdate {
    Audio(AudioUpdate),
    Midi(MidiUpdate),
}
