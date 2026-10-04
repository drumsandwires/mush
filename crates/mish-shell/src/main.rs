// mush/crates/mish-shell/src/main.rs
use mish_core::{MidiEngine, MidiResponse};
use mush_core::{run, Response, Shell};

struct MishShell(MidiEngine);

impl Shell for MishShell {
    fn prompt(&self) -> &str {
        "mish> "
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
