// mush/crates/mish-shell/src/main.rs
use mish::midi::engine::{MidiEngine, MidiResponse};
use mish_midi_core as mish;

use mush_core::{Response, Shell, run};

struct MishShell(MidiEngine);

impl Shell for MishShell {
    fn prompt(&self) -> &str {
        "mish"
    }

    fn evaluate(&self, input: &str) -> Response {
        match self.0.evaluate(input) {
            MidiResponse::Output(s) => Response::Output(s),
            MidiResponse::Empty => Response::Empty,
            MidiResponse::Exit(s) => Response::Exit(s),
            MidiResponse::Unknown => Response::Output(format!("mish: unknown command: {input}")),
        }
    }

    fn on_start(&self) {
        println!("Welcome to mish — MIDI shell. Type 'help' for commands.");
    }
}

fn main() {
    run(MishShell(MidiEngine::new()));
}
