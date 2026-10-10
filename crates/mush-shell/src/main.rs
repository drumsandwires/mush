mod evaluator;

use mash_audio_core::audio::engine::AudioEngine;
use mish_midi_core::midi::engine::MidiEngine;
use mush_core::{Response, Shell, run};

struct MushShell {
    audio: AudioEngine,
    midi: MidiEngine,
}

impl Shell for MushShell {
    fn prompt(&self) -> &str {
        "mush"
    }

    fn evaluate(&self, input: &str) -> Response {
        evaluator::evaluate(&self.audio, &self.midi, input)
    }

    fn on_start(&self) {
        println!("Welcome to mush — Music shell. Type 'help' for commands.");
        println!("Domains: audio, midi");
    }
}

fn main() {
    run(MushShell {
        audio: AudioEngine::new(),
        midi: MidiEngine::new(),
    });
}
