use crate::midi_output::MidiOutput;
use std::sync::{Arc, Mutex};

/// A mock MIDI output that records all messages sent to it.
/// The `messages` handle can be cloned before passing the mock
/// to a `Box<dyn MidiOutput>` — both ends share the same buffer.
#[derive(Clone)]
pub struct MockMidiOutput {
    pub messages: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl MockMidiOutput {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl MidiOutput for MockMidiOutput {
    fn send(&mut self, bytes: &[u8]) -> Result<(), String> {
        self.messages.lock().unwrap().push(bytes.to_vec());
        Ok(())
    }
}
