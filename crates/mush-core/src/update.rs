// mush/crates/mush-core/src/update.rs
//
// Updates flow from the EngineMonitor outward to all frontends.
// The direction is opposite to TransportCommand:
//
//   TransportCommand  — frontend → engine
//   EngineUpdate      — engine → frontend

use serde::{Deserialize, Serialize};

// Forward declaration — AudioHostInfo is defined in mash-audio-core.
// We use a placeholder here and will replace it in Slice 1c when
// mash-audio-core gains its serde types.
//
// For now, device list information is carried as a String
// and will be replaced with the structured type in Slice 1c.

/// Updates emitted by the audio engine or EngineMonitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioUpdate {
    /// Playhead advanced to this sample position.
    /// Emitted periodically — not on every buffer.
    PlayheadMoved(u64),

    /// Transport started playing.
    TransportStarted,

    /// Transport stopped.
    TransportStopped,

    /// Response to TransportCommand::AudioDevices.
    /// Payload is a formatted string at MVP — replaced with
    /// structured AudioHostInfo in Slice 1c.
    DeviceList(String),

    /// A non-fatal error from the audio subsystem.
    Error(String),
}

/// Updates emitted by the MIDI engine or EngineMonitor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MidiUpdate {
    /// A MIDI event was dispatched to the output port.
    EventDispatched { channel: u8, note: u8, vel: u8 },

    /// Response to TransportCommand::MidiDevices.
    /// Payload is a formatted string at MVP — replaced with
    /// structured MidiPortInfo in Slice 1b.
    PortList(String),

    /// A non-fatal error from the MIDI subsystem.
    Error(String),
}

/// The top-level update type flowing from engine to frontends.
/// Wraps AudioUpdate and MidiUpdate so frontends receive a single
/// stream of updates regardless of source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EngineUpdate {
    Audio(AudioUpdate),
    Midi(MidiUpdate),
}
