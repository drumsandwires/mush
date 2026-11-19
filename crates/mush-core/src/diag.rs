
/// A diagnostic event captured on the RT path.
///
/// Must be `Copy` and allocation-free — pushed from audio callback
/// and MIDI RT thread into a lock-free ring buffer.
#[derive(Debug, Clone, Copy)]
pub enum DiagEvent {
    /// `midi_queue.push()` failed — queue was full.
    /// MIDI events are being dropped on the RT path.
    MidiQueueFull { frame: u64 },

    /// `status_tx.try_send()` dropped an update.
    /// Conductor is not draining fast enough.
    StatusChannelFull,

    /// `output.send()` failed in `MidiEngineRt`.
    MidiSendError,

    /// `timeline.try_read()` failed — lock was held.
    /// Timeline was not readable during this buffer period.
    TimelineLockMissed { frame: u64 },
}
