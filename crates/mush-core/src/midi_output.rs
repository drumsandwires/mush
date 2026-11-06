/// Abstraction over a MIDI output connection.
///
/// Implemented by `DesktopMidiOutput` (midir) in mish-midi-core.
/// The trait exists so `MidiEngineRt` does not depend directly on
/// midir — a different backend (e.g. Web MIDI) can be substituted
/// without changing the RT engine.
pub trait MidiOutput: Send + 'static {
    /// Send raw MIDI bytes to the output port.
    /// Called from the MidiEngineRt thread — must not block.
    fn send(&mut self, bytes: &[u8]) -> Result<(), String>;
}
