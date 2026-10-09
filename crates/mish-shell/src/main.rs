mod evaluator;

use mish_midi_core::midi::engine::MidiEngine;
use mush_core::{Response, Shell, run};

struct MishShell(MidiEngine);

impl Shell for MishShell {
    fn prompt(&self) -> &str {
        "mish"
    }

    fn evaluate(&self, input: &str) -> Response {
        evaluator::evaluate(&self.0, input)
    }

    fn on_start(&self) {
        println!("Welcome to mish — MIDI shell. Type 'help' for commands.");
    }
}

fn main() {
    run(MishShell(MidiEngine::new()));
}
